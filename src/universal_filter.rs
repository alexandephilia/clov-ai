//! Universal content filtering for MCP responses.
//!
//! Detects content type and structure automatically without hardcoded
//! tool-specific logic. Applies heuristic-based chrome stripping,
//! structured-data reduction, and adaptive truncation.

use crate::tracking::estimate_tokens;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{Map, Value};

const DEFAULT_MAX_ARRAY_ITEMS: usize = 8;
const DEFAULT_MAX_OBJECT_KEYS: usize = 12;
pub struct FilterContext {
    pub max_tokens: usize,
    pub preserve_code: bool,
    pub aggressive_chrome_strip: bool,
    pub max_array_items: usize,
    pub max_object_keys: usize,
}

impl Default for FilterContext {
    fn default() -> Self {
        Self {
            max_tokens: 2000,
            preserve_code: true,
            aggressive_chrome_strip: true,
            max_array_items: DEFAULT_MAX_ARRAY_ITEMS,
            max_object_keys: DEFAULT_MAX_OBJECT_KEYS,
        }
    }
}

pub fn filter_response(text: &str, context: &FilterContext) -> String {
    if let Ok(mut json) = serde_json::from_str::<Value>(text) {
        filter_json_value(&mut json, context);
        return serde_json::to_string(&json).unwrap_or_else(|_| text.to_string());
    }

    filter_text_content(text, context)
}

pub fn filter_json_value(value: &mut Value, context: &FilterContext) {
    match value {
        Value::Object(map) => filter_object(map, context),
        Value::Array(arr) => filter_array(arr, context, None),
        Value::String(text) => {
            *text = filter_text_content(text, context);
        }
        _ => {}
    }
}

fn filter_object(map: &mut Map<String, Value>, context: &FilterContext) {
    let content_type = detect_content_type(map);

    for key in text_like_keys() {
        if let Some(Value::String(text)) = map.get_mut(*key) {
            *text = filter_text_for_key(text, context, key, &content_type);
        }
    }

    match content_type {
        ContentType::WebSearch => apply_search_filters(map, context),
        ContentType::StructuredData => apply_data_filters(map, context),
        ContentType::Code => apply_code_filters(map, context),
        ContentType::PlainText | ContentType::Unknown => {}
    }

    for (key, value) in map.iter_mut() {
        if text_like_keys().contains(&key.as_str()) {
            continue;
        }

        match value {
            Value::Object(_) | Value::Array(_) => filter_json_value(value, context),
            _ => {}
        }
    }
}

fn filter_array(arr: &mut Vec<Value>, context: &FilterContext, key_hint: Option<&str>) {
    if arr.is_empty() {
        return;
    }

    for item in arr.iter_mut().take(context.max_array_items) {
        filter_json_value(item, context);
    }

    if arr.len() > context.max_array_items {
        let original_len = arr.len();
        arr.truncate(context.max_array_items);
        let mut summary = Map::new();
        summary.insert(
            "_clov_summary".to_string(),
            Value::String(format!(
                "truncated {} items from {}{}",
                original_len - context.max_array_items,
                original_len,
                key_hint
                    .map(|key| format!(" in `{}`", key))
                    .unwrap_or_default()
            )),
        );
        arr.push(Value::Object(summary));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContentType {
    WebSearch,
    Code,
    StructuredData,
    PlainText,
    Unknown,
}

fn detect_content_type(map: &Map<String, Value>) -> ContentType {
    if search_like_keys().iter().any(|key| map.contains_key(*key)) {
        return ContentType::WebSearch;
    }

    if (map.contains_key("url") || map.contains_key("title"))
        && text_like_keys()
            .iter()
            .any(|key| map.get(*key).and_then(Value::as_str).is_some())
    {
        return ContentType::WebSearch;
    }

    if data_like_keys().iter().any(|key| map.contains_key(*key)) {
        return ContentType::StructuredData;
    }

    if let Some(text) = text_like_keys()
        .iter()
        .find_map(|key| map.get(*key).and_then(Value::as_str))
    {
        if looks_like_code(text) {
            return ContentType::Code;
        }
        return ContentType::PlainText;
    }

    if map.values().any(|value| matches!(value, Value::Array(_))) {
        return ContentType::StructuredData;
    }

    ContentType::Unknown
}

fn apply_search_filters(map: &mut Map<String, Value>, context: &FilterContext) {
    // Strip top-level noise keys injected by search providers (exa, serper, etc.)
    for key in search_noise_keys() {
        map.remove(*key);
    }

    for key in search_like_keys() {
        if let Some(Value::Array(items)) = map.get_mut(*key) {
            for item in items.iter_mut().take(context.max_array_items) {
                if let Value::Object(obj) = item {
                    retain_priority_keys(
                        obj,
                        &[
                            "title",
                            "url",
                            "text",
                            "snippet",
                            "summary",
                            "content",
                            "highlights",
                            "score",
                        ],
                    );

                    for text_key in ["text", "snippet", "summary", "content", "highlights"] {
                        if let Some(Value::String(text)) = obj.get_mut(text_key) {
                            *text =
                                filter_text_for_key(text, context, text_key, &ContentType::WebSearch);
                        }
                    }
                }
                filter_json_value(item, context);
            }
            filter_array(items, context, Some(key));
        }
    }
}

fn apply_code_filters(map: &mut Map<String, Value>, context: &FilterContext) {
    for key in text_like_keys() {
        if let Some(Value::String(text)) = map.get_mut(*key) {
            *text = filter_text_for_key(text, context, key, &ContentType::Code);
        }
    }
}

fn apply_data_filters(map: &mut Map<String, Value>, context: &FilterContext) {
    for key in data_like_keys() {
        if let Some(value) = map.get_mut(*key) {
            summarize_data_value(value, context, Some(key));
        }
    }

    for (key, value) in map.iter_mut() {
        if data_like_keys().contains(&key.as_str()) {
            continue;
        }

        if matches!(value, Value::Array(_) | Value::Object(_)) {
            summarize_data_value(value, context, Some(key));
        }
    }
}

fn summarize_data_value(value: &mut Value, context: &FilterContext, key_hint: Option<&str>) {
    match value {
        Value::Array(items) => {
            let sample_keys = collect_common_keys(items);
            for item in items.iter_mut().take(context.max_array_items) {
                if let Value::Object(obj) = item {
                    if !sample_keys.is_empty() {
                        retain_priority_keys(obj, &sample_keys);
                    }
                    trim_object(obj, context);
                } else {
                    filter_json_value(item, context);
                }
            }
            filter_array(items, context, key_hint);
        }
        Value::Object(map) => trim_object(map, context),
        _ => {}
    }
}

fn trim_object(map: &mut Map<String, Value>, context: &FilterContext) {
    let ordered_keys: Vec<String> = map.keys().cloned().collect();
    let priority = prioritized_keys_from(&ordered_keys);

    if map.len() > context.max_object_keys {
        let mut keep = priority;
        if keep.len() < context.max_object_keys {
            for key in ordered_keys {
                if !keep.contains(&key) {
                    keep.push(key);
                }
                if keep.len() >= context.max_object_keys {
                    break;
                }
            }
        }

        let dropped = map.len().saturating_sub(keep.len());
        map.retain(|key, _| keep.contains(key));
        map.insert(
            "_clov_summary".to_string(),
            Value::String(format!("dropped {} low-priority fields", dropped)),
        );
    }

    for value in map.values_mut() {
        filter_json_value(value, context);
    }
}

fn retain_priority_keys(map: &mut Map<String, Value>, priority: &[&str]) {
    let owned_priority: Vec<String> = priority.iter().map(|key| (*key).to_string()).collect();
    let has_non_priority = map
        .keys()
        .any(|key| !owned_priority.iter().any(|candidate| candidate == key));
    if !has_non_priority {
        return;
    }

    let mut kept = 0usize;
    map.retain(|key, _| {
        let should_keep = owned_priority.iter().any(|candidate| candidate == key);
        if should_keep {
            kept += 1;
        }
        should_keep
    });

    if kept == 0 {
        return;
    }

    map.insert(
        "_clov_summary".to_string(),
        Value::String("retained priority fields only".to_string()),
    );
}

fn collect_common_keys(items: &[Value]) -> Vec<&'static str> {
    let key_priority = [
        "id",
        "name",
        "title",
        "type",
        "path",
        "file",
        "url",
        "status",
        "score",
        "summary",
        "snippet",
        "text",
        "content",
        "created_at",
        "updated_at",
    ];

    key_priority
        .iter()
        .copied()
        .filter(|candidate| {
            items.iter().take(DEFAULT_MAX_ARRAY_ITEMS).any(|item| {
                item.as_object()
                    .map(|obj| obj.contains_key(*candidate))
                    .unwrap_or(false)
            })
        })
        .collect()
}

fn prioritized_keys_from(keys: &[String]) -> Vec<String> {
    let key_priority = [
        "id",
        "name",
        "title",
        "type",
        "path",
        "file",
        "url",
        "status",
        "score",
        "summary",
        "snippet",
        "text",
        "content",
        "created_at",
        "updated_at",
        "metadata",
    ];

    key_priority
        .iter()
        .filter(|candidate| keys.iter().any(|key| key == *candidate))
        .map(|candidate| (*candidate).to_string())
        .collect()
}

fn filter_text_for_key(
    text: &str,
    context: &FilterContext,
    key: &str,
    content_type: &ContentType,
) -> String {
    if *content_type == ContentType::WebSearch
        && matches!(key, "text" | "snippet" | "summary" | "content" | "highlights")
    {
        let cleaned = if context.aggressive_chrome_strip {
            strip_universal_chrome(text)
        } else {
            text.to_string()
        };
        let normalized = normalize_escaped_line_breaks(&cleaned);
        let article = cleanup_article_text(&normalized);
        let article_budget = context.max_tokens.max(3000);
        return filter_by_token_budget(&article, article_budget);
    }

    if *content_type == ContentType::Code
        || (context.preserve_code
            && *content_type != ContentType::WebSearch
            && looks_like_primary_code_blob(text))
    {
        return filter_code_content(text, context);
    }

    if matches!(key, "html" | "body" | "content") && looks_like_html(text) {
        let cleaned = strip_universal_chrome(text);
        return filter_long_form_text(&cleaned, context);
    }

    filter_text_content(text, context)
}

fn filter_text_content(text: &str, context: &FilterContext) -> String {
    let cleaned = if context.aggressive_chrome_strip {
        strip_universal_chrome(text)
    } else {
        text.to_string()
    };

    filter_long_form_text(&cleaned, context)
}

fn filter_long_form_text(text: &str, context: &FilterContext) -> String {
    let normalized_article = normalize_escaped_line_breaks(text);

    if looks_like_article(&normalized_article) {
        let cleaned = cleanup_article_text(&normalized_article);
        let article_budget = context.max_tokens.max(3000);
        return filter_by_token_budget(&cleaned, article_budget);
    }

    let limit = adaptive_truncation_limit(text);
    filter_by_token_budget(text, context.max_tokens.min(limit))
}

fn filter_code_content(text: &str, context: &FilterContext) -> String {
    let normalized = collapse_whitespace_preserving_indentation(text);
    let limit = context
        .max_tokens
        .min(adaptive_truncation_limit(&normalized).max(1200));
    filter_by_token_budget(&normalized, limit)
}

pub fn strip_universal_chrome(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut skip_block = false;
    let mut blank_count = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_count += 1;
            skip_block = false;
            if blank_count <= 2 {
                result.push('\n');
            }
            continue;
        }
        blank_count = 0;

        if is_navigation_chrome(trimmed) || is_footer_garbage(trimmed) || is_advertisement(trimmed)
        {
            skip_block = true;
            continue;
        }

        if skip_block && seems_like_meaningful_content(trimmed) {
            skip_block = false;
        }

        if skip_block {
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    collapse_whitespace(&result)
}

fn is_navigation_chrome(line: &str) -> bool {
    lazy_static! {
        static ref NAV: Vec<Regex> = vec![
            Regex::new(r"(?i)^\s*(skip to (main |)content|menu|navigation|breadcrumb|sidebar)\s*$")
                .unwrap(),
            Regex::new(r"(?i)^\s*(sign in|log in|register|subscribe|newsletter)\s*$").unwrap(),
            Regex::new(r"(?i)^\s*\[?(home|about|contact|blog|faq|help|support)\]?\s*$").unwrap(),
            Regex::new(
                r"(?i)^\s*(previous|next|related (posts|articles)|you (may|might) also like)\s*$"
            )
            .unwrap(),
        ];
    }
    NAV.iter().any(|p| p.is_match(line.trim()))
}

fn is_footer_garbage(line: &str) -> bool {
    lazy_static! {
        static ref FOOTER: Vec<Regex> = vec![
            Regex::new(r"(?i)^\s*(cookie|privacy|terms of (use|service)|accept all|reject all|manage preferences)\s*$").unwrap(),
            Regex::new(r"(?i)^\s*(©|copyright|all rights reserved|powered by)\b").unwrap(),
            Regex::new(r"(?i)^\s*[|·•–—]\s*(privacy|terms|contact|about|sitemap|careers|press)").unwrap(),
        ];
    }
    FOOTER.iter().any(|p| p.is_match(line.trim()))
}

fn is_advertisement(line: &str) -> bool {
    lazy_static! {
        static ref ADS: Vec<Regex> =
            vec![Regex::new(r"(?i)^\s*(advertisement|sponsored|promoted|ad)\s*$").unwrap(),];
    }
    ADS.iter().any(|p| p.is_match(line.trim()))
}

fn seems_like_meaningful_content(line: &str) -> bool {
    line.len() > 24
        || line.contains(". ")
        || line.contains(": ")
        || line.contains(" - ")
        || line.contains('|')
}

fn collapse_whitespace(text: &str) -> String {
    let result = MULTIPLE_BLANKS.replace_all(text, "\n\n");
    let result = MULTIPLE_SPACES.replace_all(&result, " ");
    result.trim().to_string()
}

fn cleanup_article_text(text: &str) -> String {
    let normalized = normalize_escaped_line_breaks(text);
    let mut cleaned_lines = Vec::new();
    let mut blank_run = 0usize;
    let mut in_fenced_block = false;

    for raw_line in normalized.lines() {
        let trimmed = raw_line.trim();

        if is_fence_marker(trimmed) {
            in_fenced_block = !in_fenced_block;
            continue;
        }

        let unquoted = strip_markdown_quote_prefix(trimmed);

        if unquoted.is_empty() {
            blank_run += 1;
            if blank_run <= 2 {
                cleaned_lines.push(String::new());
            }
            continue;
        }

        blank_run = 0;

        if in_fenced_block {
            continue;
        }

        if is_markdown_table_separator(&unquoted) {
            continue;
        }

        let without_heading = ARTICLE_HEADING.replace(&unquoted, "").trim().to_string();
        if should_drop_article_boilerplate(&without_heading) {
            continue;
        }
        let without_hard_breaks = without_heading.trim_end_matches('\\').trim().to_string();
        let normalized_line = if looks_like_markdown_table_row(&without_hard_breaks) {
            normalize_markdown_table_row(&without_hard_breaks)
        } else {
            without_hard_breaks
        };

        if normalized_line.is_empty() {
            continue;
        }

        cleaned_lines.push(collapse_inline_spacing(&normalized_line));
    }

    collapse_whitespace(&cleaned_lines.join("\n"))
}

fn collapse_inline_spacing(text: &str) -> String {
    MULTIPLE_SPACES.replace_all(text, " ").trim().to_string()
}

fn normalize_escaped_line_breaks(text: &str) -> String {
    text.replace("\\r\\n", "\n")
        .replace("\\n", "\n")
        .replace("\\r", "\n")
}

fn collapse_whitespace_preserving_indentation(text: &str) -> String {
    let mut collapsed = Vec::new();
    let mut blank_run = 0usize;

    for line in text.lines() {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= 2 {
                collapsed.push(String::new());
            }
            continue;
        }

        blank_run = 0;
        collapsed.push(line.trim_end().to_string());
    }

    collapsed.join("\n").trim().to_string()
}

fn looks_like_article(text: &str) -> bool {
    if text.len() < 300 {
        return false;
    }

    if looks_like_primary_code_blob(text) {
        return false;
    }

    let sentence_markers = text.matches(". ").count()
        + text.matches("! ").count()
        + text.matches("? ").count()
        + text
            .lines()
            .filter(|line| line.trim_end().ends_with('.'))
            .count();
    let substantial_lines = text.lines().filter(|line| line.trim().len() >= 24).count();
    let markdown_signals = text.lines().any(|line| {
        let trimmed = line.trim();
        ARTICLE_HEADING.is_match(trimmed)
            || looks_like_markdown_table_row(trimmed)
            || trimmed.ends_with('\\')
    }) || text.contains("\\n");
    let escaped_article_shape = text.contains("\\n")
        && (text.contains("> ")
            || text.contains("# ")
            || text.contains("## ")
            || text.contains("| --- |")
            || text.contains("```"));

    if escaped_article_shape {
        return true;
    }

    sentence_markers >= 3 && (substantial_lines >= 4 || markdown_signals)
}

fn looks_like_primary_code_blob(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let codeish_lines = lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            is_fence_marker(trimmed)
                || trimmed.ends_with(';')
                || trimmed.ends_with('{')
                || trimmed.ends_with('}')
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("const ")
                || trimmed.starts_with("let ")
                || trimmed.starts_with("import ")
                || trimmed.starts_with("export ")
                || trimmed.starts_with("interface ")
        })
        .count();

    let prose_lines = lines
        .iter()
        .filter(|line| line.trim().len() >= 32 && line.contains(' '))
        .count();

    codeish_lines >= 8 && codeish_lines > prose_lines
}

fn adaptive_truncation_limit(text: &str) -> usize {
    let line_count = text.lines().count();
    let avg_line_length = text.len() / line_count.max(1);

    if avg_line_length > 120 {
        return 6000;
    }

    if avg_line_length > 80 {
        return 5000;
    }

    if avg_line_length > 40 {
        return 2500;
    }

    1500
}

fn filter_by_token_budget(text: &str, max_tokens: usize) -> String {
    let estimated = estimate_tokens(text);

    if estimated <= max_tokens {
        return text.to_string();
    }

    let mut low = 0;
    let mut high = text.len();

    while low < high {
        let mid = (low + high) / 2;
        let adj_mid = safe_truncation_boundary(text, mid);
        let chunk = &text[..adj_mid];

        if estimate_tokens(chunk) <= max_tokens {
            if low == adj_mid {
                break;
            }
            low = adj_mid;
        } else {
            high = adj_mid;
        }
    }

    format!(
        "{}\n[... truncated to {} tokens by clov]",
        &text[..low],
        max_tokens
    )
}

fn safe_truncation_boundary(text: &str, mut offset: usize) -> usize {
    offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn looks_like_code(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.contains("```")
        || trimmed.contains("fn ")
        || trimmed.contains("class ")
        || trimmed.contains("const ")
        || trimmed.contains("let ")
        || trimmed.contains("import ")
        || trimmed.contains("#include")
        || trimmed.contains("package ")
        || trimmed.contains("interface ")
}

fn looks_like_html(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    lowered.contains("<html")
        || lowered.contains("<body")
        || lowered.contains("<main")
        || lowered.contains("<div")
        || lowered.contains("</")
}

fn looks_like_markdown_table_row(line: &str) -> bool {
    let pipe_count = line.matches('|').count();
    pipe_count >= 2 && (line.starts_with('|') || line.ends_with('|'))
}

fn is_markdown_table_separator(line: &str) -> bool {
    if !looks_like_markdown_table_row(line) {
        return false;
    }

    let cells: Vec<&str> = line
        .split('|')
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .collect();

    !cells.is_empty()
        && cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

fn normalize_markdown_table_row(line: &str) -> String {
    let cells: Vec<&str> = line
        .split('|')
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .collect();

    match cells.as_slice() {
        [] => String::new(),
        [single] => (*single).to_string(),
        [left, right] => format!("{}: {}", left, right),
        _ => cells.join(" — "),
    }
}

fn strip_markdown_quote_prefix(line: &str) -> String {
    let mut current = line.trim();

    while let Some(rest) = current.strip_prefix('>') {
        current = rest.trim_start();
    }

    current.to_string()
}

fn is_fence_marker(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("```") || trimmed == "~~~"
}

fn should_drop_article_boilerplate(line: &str) -> bool {
    let lowered = line.trim().to_ascii_lowercase();
    lowered.starts_with("fetch the complete documentation index at:")
        || lowered.starts_with("use this file to discover all available pages")
        || lowered == "documentation index"
}

fn search_like_keys() -> &'static [&'static str] {
    &["results", "items", "documents", "matches"]
}

fn search_noise_keys() -> &'static [&'static str] {
    &[
        // exa-specific internal metadata
        "requestTags",
        "effectiveFilters",
        "requestId",
        "costDollars",
        "searchTime",
        "requestTime",
        "processingTime",
        // generic search provider noise
        "statuses",
        "debugInfo",
        "_metadata",
        "rateLimit",
        "credits",
    ]
}

fn data_like_keys() -> &'static [&'static str] {
    &["data", "rows", "records", "nodes", "entries", "documents"]
}

fn text_like_keys() -> &'static [&'static str] {
    &[
        "text", "content", "summary", "snippet", "body", "html", "source",
    ]
}

lazy_static! {
    static ref MULTIPLE_BLANKS: Regex = Regex::new(r"\n{4,}").unwrap();
    static ref MULTIPLE_SPACES: Regex = Regex::new(r"[ \t]{3,}").unwrap();
    static ref ARTICLE_HEADING: Regex = Regex::new(r"^#{1,6}\s+").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strips_navigation_chrome_from_plain_text() {
        let input = "Home\nAbout\nMain article starts here and contains real content.\nPrivacy\n";
        let output = strip_universal_chrome(input);
        assert!(output.contains("Main article starts here"));
        assert!(!output.contains("Home"));
        assert!(!output.contains("Privacy"));
    }

    #[test]
    fn preserves_code_shape_when_filtering_code() {
        let input = "fn main() {\n    println!(\"hi\");\n}\n";
        let output = filter_response(input, &FilterContext::default());
        assert!(output.contains("fn main()"));
        assert!(output.contains("println!"));
    }

    #[test]
    fn summarizes_large_structured_arrays() {
        let mut value = json!({
            "rows": [
                {"id": 1, "name": "a", "email": "a@example.com", "metadata": {"a": 1}},
                {"id": 2, "name": "b", "email": "b@example.com", "metadata": {"b": 2}},
                {"id": 3, "name": "c", "email": "c@example.com", "metadata": {"c": 3}},
                {"id": 4, "name": "d", "email": "d@example.com", "metadata": {"d": 4}},
                {"id": 5, "name": "e", "email": "e@example.com", "metadata": {"e": 5}},
                {"id": 6, "name": "f", "email": "f@example.com", "metadata": {"f": 6}},
                {"id": 7, "name": "g", "email": "g@example.com", "metadata": {"g": 7}},
                {"id": 8, "name": "h", "email": "h@example.com", "metadata": {"h": 8}},
                {"id": 9, "name": "i", "email": "i@example.com", "metadata": {"i": 9}}
            ]
        });

        filter_json_value(&mut value, &FilterContext::default());

        let rows = value["rows"].as_array().unwrap();
        assert_eq!(rows.len(), DEFAULT_MAX_ARRAY_ITEMS + 1);
        assert!(rows.last().unwrap()["_clov_summary"].is_string());
        assert!(rows[0].get("id").is_some());
        assert!(rows[0].get("name").is_some());
    }

    #[test]
    fn filters_search_results_to_priority_fields() {
        let mut value = json!({
            "results": [
                {
                    "title": "Example",
                    "url": "https://example.com",
                    "text": "Home\nResult body with useful content\nPrivacy",
                    "extra": "drop me"
                }
            ]
        });

        filter_json_value(&mut value, &FilterContext::default());

        let result = value["results"][0].as_object().unwrap();
        assert!(result.contains_key("title"));
        assert!(result.contains_key("url"));
        assert!(result.contains_key("text"));
        assert!(!result.contains_key("extra"));
    }

    #[test]
    fn cleans_article_readability_artifacts() {
        let input = concat!(
            "## Overview\\n",
            "This article explains the fix in plain language for readers.\\n",
            "It preserves useful context while removing noisy formatting.\\n\\n",
            "| Signal | Meaning |\\n",
            "| --- | --- |\\n",
            "| latency | low |\\n",
            "| noise | reduced |\\n\\n",
            "The crawler also emitted markdown hard breaks for layout.\\n",
            "That should read like normal prose instead.\\\\\n",
            "Another useful sentence closes the example."
        );

        let output = filter_response(input, &FilterContext::default());

        assert!(output.contains("Overview"));
        assert!(output.contains("latency: low"));
        assert!(output.contains("noise: reduced"));
        assert!(!output.contains("##"));
        assert!(!output.contains("| --- |"));
        assert!(!output.contains("\\n"));
        assert!(!output.contains("\\\\\n"));
    }

    #[test]
    fn cleans_exa_quoted_article_noise() {
        let input = concat!(
            "> Fetch the complete documentation index at: https://code.claude.com/docs/llms.txt\\n",
            "> Use this file to discover all available pages before exploring further.\\n",
            ">\\n",
            "> ## Documentation Index\\n\\n",
            "# Orchestrate teams of Claude Code sessions\\n\\n",
            "> Coordinate multiple Claude Code instances working together as a team.\\n\\n",
            "## Compare with subagents\\n\\n",
            "| | Subagents | Agent teams |\\n",
            "| --- | --- | --- |\\n",
            "| Context | Own context window | Fully independent |\\n\\n",
            "```json\\n",
            "{\\n",
            "  \\\"env\\\": {\\n",
            "    \\\"CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS\\\": \\\"1\\\"\\n",
            "  }\\n",
            "}\\n",
            "```\\n\\n",
            "This final paragraph explains the feature clearly in normal prose.\\n"
        );

        let output = filter_response(input, &FilterContext::default());

        assert!(output.contains("Orchestrate teams of Claude Code sessions"));
        assert!(output.contains("Coordinate multiple Claude Code instances working together as a team."));
        assert!(output.contains("Subagents: Agent teams"));
        assert!(output.contains("Context — Own context window — Fully independent"));
        assert!(output.contains("This final paragraph explains the feature clearly in normal prose."));
        assert!(!output.contains("Fetch the complete documentation index"));
        assert!(!output.contains("Documentation Index"));
        assert!(!output.contains("##"));
        assert!(!output.contains("> "));
        assert!(!output.contains("| --- |"));
        assert!(!output.contains("```"));
        assert!(!output.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"));
        assert!(!output.contains("\\n"));
    }

    #[test]
    fn treats_crawl_result_rows_as_web_articles_not_code() {
        let raw_text = concat!(
            "> Fetch the complete documentation index at: https://code.claude.com/docs/llms.txt\\n",
            "> ## Documentation Index\\n\\n",
            "# Orchestrate teams of Claude Code sessions\\n\\n",
            "| | Subagents | Agent teams |\\n",
            "| --- | --- | --- |\\n",
            "| Context | Own context window | Fully independent |\\n\\n",
            "```json\\n",
            "{\\n",
            "  \\\"env\\\": {\\n",
            "    \\\"CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS\\\": \\\"1\\\"\\n",
            "  }\\n",
            "}\\n",
            "```\\n\\n",
            "This paragraph should remain readable after cleanup.\\n"
        );

        let mut value = json!({
            "results": [
                {
                    "title": "Orchestrate teams of Claude Code sessions - Claude Code Docs",
                    "url": "https://code.claude.com/docs/en/agent-teams",
                    "text": raw_text
                }
            ]
        });

        filter_json_value(&mut value, &FilterContext::default());

        let text = value["results"][0]["text"].as_str().unwrap();
        assert!(text.contains("Orchestrate teams of Claude Code sessions"));
        assert!(text.contains("This paragraph should remain readable after cleanup."));
        assert!(!text.contains("Fetch the complete documentation index"));
        assert!(!text.contains("Documentation Index"));
        assert!(!text.contains("##"));
        assert!(!text.contains("| --- |"));
        assert!(!text.contains("```"));
        assert!(!text.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"));
    }
}
