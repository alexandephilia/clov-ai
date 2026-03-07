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
const DEFAULT_MAX_STRING_CHARS: usize = 8_000;

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
            *text = filter_text_for_key(text, context, *key, &content_type);
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
    if *content_type == ContentType::Code || (context.preserve_code && looks_like_code(text)) {
        return filter_code_content(text, context);
    }

    if matches!(key, "html" | "body" | "content") && looks_like_html(text) {
        let cleaned = strip_universal_chrome(text);
        return filter_by_token_budget(&cleaned, context.max_tokens);
    }

    filter_text_content(text, context)
}

fn filter_text_content(text: &str, context: &FilterContext) -> String {
    let cleaned = if context.aggressive_chrome_strip {
        strip_universal_chrome(text)
    } else {
        text.to_string()
    };
    let limit = adaptive_truncation_limit(&cleaned);
    filter_by_token_budget(&cleaned, context.max_tokens.min(limit))
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
    lazy_static! {
        static ref MULTIPLE_BLANKS: Regex = Regex::new(r"\n{4,}").unwrap();
        static ref MULTIPLE_SPACES: Regex = Regex::new(r"[ \t]{3,}").unwrap();
    }
    let result = MULTIPLE_BLANKS.replace_all(text, "\n\n");
    let result = MULTIPLE_SPACES.replace_all(&result, " ");
    result.trim().to_string()
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
    let limited = if text.len() > DEFAULT_MAX_STRING_CHARS {
        &text[..safe_truncation_boundary(text, DEFAULT_MAX_STRING_CHARS)]
    } else {
        text
    };

    let estimated = estimate_tokens(limited);

    if estimated <= max_tokens {
        return limited.to_string();
    }

    let mut low = 0;
    let mut high = limited.len();

    while low < high {
        let mid = (low + high) / 2;
        let adj_mid = safe_truncation_boundary(limited, mid);
        let chunk = &limited[..adj_mid];

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
        &limited[..low],
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
}
