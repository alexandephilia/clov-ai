//! MCP tool response filters for token optimization.
//!
//! Pure functions that strip bloat from MCP tool responses before they
//! reach the LLM context window. Each filter targets a specific MCP tool's
//! response format and removes navigation elements, boilerplate, and
//! redundant content.
//!
//! # Supported Tools
//!
//! - Exa: `web_search_exa`, `web_search_advanced_exa`, `crawling_exa`, `get_code_context_exa`

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

/// Default max characters per search result text block.
const SEARCH_RESULT_MAX_CHARS: usize = 1500;

/// Default max characters for crawled page content.
const CRAWL_MAX_CHARS: usize = 3000;

lazy_static! {
    // Navigation/chrome patterns (case-insensitive line matching)
    static ref NAV_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)^\s*(skip to (main |)content|menu|navigation|breadcrumb|sidebar)\s*$").unwrap(),
        Regex::new(r"(?i)^\s*(cookie|privacy|terms of (use|service)|accept all|reject all|manage preferences)\s*$").unwrap(),
        Regex::new(r"(?i)^\s*(sign in|sign up|log in|log out|register|subscribe|newsletter)\s*$").unwrap(),
        Regex::new(r"(?i)^\s*(share|tweet|facebook|linkedin|reddit|copy link|print)\s*$").unwrap(),
        Regex::new(r"(?i)^\s*(©|copyright|all rights reserved|powered by)\b").unwrap(),
        Regex::new(r"(?i)^\s*(previous|next|related (posts|articles)|you (may|might) also like)\s*$").unwrap(),
        Regex::new(r"(?i)^\s*(advertisement|sponsored|promoted|ad)\s*$").unwrap(),
        Regex::new(r"(?i)^\s*(loading|please wait|fetching)\.*\s*$").unwrap(),
        Regex::new(r"(?i)^\s*\[?(home|about|contact|blog|faq|help|support)\]?\s*$").unwrap(),
    ];

    // Footer patterns (multi-word lines that look like footer links)
    static ref FOOTER_LINK_PATTERN: Regex = Regex::new(
        r"(?i)^\s*[|·•–—]\s*(privacy|terms|contact|about|sitemap|careers|press)"
    ).unwrap();

    // Repeated whitespace/blank lines
    static ref MULTIPLE_BLANKS: Regex = Regex::new(r"\n{4,}").unwrap();
    static ref MULTIPLE_SPACES: Regex = Regex::new(r"[ \t]{3,}").unwrap();

    // URL tracking params
    static ref URL_TRACKING: Regex = Regex::new(
        r"[?&](utm_[a-z]+|ref|source|campaign|medium|content|fbclid|gclid|mc_[a-z]+|_ga|_gl)=[^&\s]*"
    ).unwrap();

    // Markdown link-heavy lines (navigation bars rendered as markdown links)
    static ref NAV_LINK_LINE: Regex = Regex::new(
        r"^\s*(\[.{1,30}\]\(.+?\)\s*){3,}\s*$"
    ).unwrap();
}

/// Filter Exa web search results (`web_search_exa`, `web_search_advanced_exa`).
///
/// Strips navigation chrome, cookie notices, footer boilerplate, and truncates
/// overly long result text blocks. Designed for search result content that
/// includes extracted page text.
pub fn filter_exa_search(text: &str) -> String {
    if let Ok(mut json) = serde_json::from_str::<Value>(text) {
        if let Some(results) = json.get_mut("results").and_then(|r| r.as_array_mut()) {
            for result in results {
                if let Some(text_val) = result.get_mut("text") {
                    if let Some(inner_text) = text_val.as_str() {
                        let cleaned = strip_web_chrome(inner_text);
                        let cleaned = collapse_whitespace(&cleaned);
                        let cleaned = clean_urls(&cleaned);
                        *text_val = Value::String(truncate_content(&cleaned, SEARCH_RESULT_MAX_CHARS));
                    }
                }
            }
            return serde_json::to_string(&json).unwrap_or_else(|_| text.to_string());
        }
    }

    // Fallback for non-JSON or unexpected structure
    let cleaned = strip_web_chrome(text);
    let cleaned = collapse_whitespace(&cleaned);
    let cleaned = clean_urls(&cleaned);
    truncate_content(&cleaned, SEARCH_RESULT_MAX_CHARS)
}

/// Filter Exa crawled page content (`crawling_exa`).
///
/// More aggressive filtering — strips sidebars, footers, and navigation,
/// keeping only the main content body. Larger truncation limit since
/// crawl results are typically single-page extractions.
pub fn filter_exa_crawl(text: &str) -> String {
    if let Ok(mut json) = serde_json::from_str::<Value>(text) {
        if let Some(results) = json.get_mut("results").and_then(|r| r.as_array_mut()) {
            for result in results {
                if let Some(text_val) = result.get_mut("text") {
                    if let Some(inner_text) = text_val.as_str() {
                        let cleaned = strip_web_chrome(inner_text);
                        let cleaned = strip_markdown_nav(&cleaned);
                        let cleaned = collapse_whitespace(&cleaned);
                        let cleaned = clean_urls(&cleaned);
                        *text_val = Value::String(truncate_content(&cleaned, CRAWL_MAX_CHARS));
                    }
                }
            }
            return serde_json::to_string(&json).unwrap_or_else(|_| text.to_string());
        }
    }

    // Fallback
    let cleaned = strip_web_chrome(text);
    let cleaned = strip_markdown_nav(&cleaned);
    let cleaned = collapse_whitespace(&cleaned);
    let cleaned = clean_urls(&cleaned);
    truncate_content(&cleaned, CRAWL_MAX_CHARS)
}

/// Filter Exa code context results (`get_code_context_exa`).
///
/// Light filtering only — code context is high-signal. Strips site chrome
/// but preserves code blocks and explanatory text.
pub fn filter_exa_code(text: &str) -> String {
    let cleaned = strip_web_chrome(text);
    collapse_whitespace(&cleaned)
}

/// Route a tool response to the appropriate filter.
///
/// Returns `None` if no filter exists for the tool (passthrough).
pub fn filter_for_tool(tool_name: &str, text: &str) -> Option<String> {
    match tool_name {
        "web_search_exa" | "web_search_advanced_exa" => Some(filter_exa_search(text)),
        "crawling_exa" => Some(filter_exa_crawl(text)),
        "get_code_context_exa" => Some(filter_exa_code(text)),
        // These tools have structured/short output, no filtering needed
        "company_research_exa" | "people_search_exa"
        | "deep_researcher_start" | "deep_researcher_check" => None,
        _ => None,
    }
}

// ── Internal helpers ──

/// Strip web chrome: navigation, cookie notices, footer links, ads.
fn strip_web_chrome(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut skip_block = false;
    let mut blank_count = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip empty lines but track them
        if trimmed.is_empty() {
            blank_count += 1;
            // Blank line ends any nav skip block (section boundary)
            skip_block = false;
            if blank_count <= 2 {
                result.push('\n');
            }
            continue;
        }
        blank_count = 0;

        // Skip navigation/chrome lines
        if NAV_PATTERNS.iter().any(|p| p.is_match(trimmed)) {
            skip_block = true;
            continue;
        }

        // Skip footer link patterns
        if FOOTER_LINK_PATTERN.is_match(trimmed) {
            continue;
        }

        // End skip block on substantial content (even without blank line)
        if skip_block && trimmed.len() > 40 {
            skip_block = false;
        }

        if skip_block {
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Strip markdown navigation link bars (3+ links on one line).
fn strip_markdown_nav(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        if NAV_LINK_LINE.is_match(line) {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Collapse excessive whitespace.
fn collapse_whitespace(text: &str) -> String {
    let result = MULTIPLE_BLANKS.replace_all(text, "\n\n");
    let result = MULTIPLE_SPACES.replace_all(&result, " ");
    result.trim().to_string()
}

/// Strip tracking parameters from URLs.
fn clean_urls(text: &str) -> String {
    URL_TRACKING.replace_all(text, "").to_string()
}

/// Truncate content to max chars with a marker.
fn truncate_content(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    // Find a good break point (end of line near limit)
    let truncated = &text[..max_chars];
    let break_point = truncated.rfind('\n').unwrap_or(max_chars);
    let mut result = text[..break_point].to_string();
    let remaining = text.len() - break_point;
    result.push_str(&format!(
        "\n\n[... {} chars truncated by clov]",
        remaining
    ));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_exa_search_strips_nav() {
        let input = "Skip to main content\nMenu\nNavigation\n\nThis is the actual article content that we want to keep.\n\nCookie\nPrivacy\n© 2025 All Rights Reserved";
        let result = filter_exa_search(input);
        assert!(result.contains("actual article content"));
        assert!(!result.contains("Skip to main content"));
        assert!(!result.contains("Cookie"));
        assert!(!result.contains("© 2025"));
    }

    #[test]
    fn test_filter_exa_search_truncates_long_content() {
        let long_content = "Important info. ".repeat(200); // ~3200 chars
        let result = filter_exa_search(&long_content);
        assert!(result.len() < long_content.len());
        assert!(result.contains("[..."));
        assert!(result.contains("truncated by clov]"));
    }

    #[test]
    fn test_filter_exa_search_passthrough_short() {
        let short = "Rust is a systems programming language focused on safety.";
        let result = filter_exa_search(short);
        assert_eq!(result, short);
    }

    #[test]
    fn test_filter_exa_crawl_strips_boilerplate() {
        let input = "Skip to content\n\nSign in\nSubscribe\n\nThe main article body goes here with important technical details.\n\nShare\nTweet\nFacebook\n\n| Privacy | Terms | Contact | About | Sitemap";
        let result = filter_exa_crawl(input);
        assert!(result.contains("main article body"));
        assert!(!result.contains("Skip to content"));
        assert!(!result.contains("Sign in"));
        assert!(!result.contains("Subscribe"));
    }

    #[test]
    fn test_filter_exa_code_preserves_code() {
        let input = "Menu\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\nThis function prints hello.\n\nCookie";
        let result = filter_exa_code(input);
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!"));
        assert!(result.contains("This function prints hello."));
        assert!(!result.contains("Cookie"));
    }

    #[test]
    fn test_filter_for_tool_routing() {
        let text = "Skip to content\n\nActual content here.\n\nCookie";
        assert!(filter_for_tool("web_search_exa", text).is_some());
        assert!(filter_for_tool("web_search_advanced_exa", text).is_some());
        assert!(filter_for_tool("crawling_exa", text).is_some());
        assert!(filter_for_tool("get_code_context_exa", text).is_some());
        assert!(filter_for_tool("company_research_exa", text).is_none());
        assert!(filter_for_tool("unknown_tool", text).is_none());
    }

    #[test]
    fn test_clean_urls_strips_tracking() {
        let url = "https://example.com/page?utm_source=twitter&utm_medium=social&id=42";
        let cleaned = clean_urls(url);
        assert!(cleaned.contains("id=42"));
        assert!(!cleaned.contains("utm_source"));
        assert!(!cleaned.contains("utm_medium"));
    }

    #[test]
    fn test_collapse_whitespace() {
        let input = "line1\n\n\n\n\n\nline2\n\n\n\n\nline3";
        let result = collapse_whitespace(input);
        assert_eq!(result, "line1\n\nline2\n\nline3");
    }

    #[test]
    fn test_strip_markdown_nav() {
        let input = "[Home](/) [About](/about) [Blog](/blog) [Contact](/contact)\n\nActual content here.";
        let result = strip_markdown_nav(input);
        assert!(!result.contains("[Home]"));
        assert!(result.contains("Actual content here."));
    }
}
