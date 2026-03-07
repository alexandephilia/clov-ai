//! MCP stdio proxy with response filtering.
//!
//! Sits between Claude Code and any MCP server, forwarding JSON-RPC messages
//! bidirectionally over stdio. Intercepts `tools/call` responses and routes
//! them through tool-specific filters to strip context-polluting bloat.
//!
//! # Architecture
//!
//! ```text
//! Claude Code ──stdin──▶ clov mcp proxy ──stdin──▶ Real MCP Server
//! Claude Code ◀──stdout── clov mcp proxy ◀──stdout── Real MCP Server
//!                               │
//!                         [filter engine]
//!                   strips bloat from tool responses
//! ```
//!
//! # Protocol
//!
//! MCP uses JSON-RPC 2.0 over stdio. Messages are newline-delimited JSON.
//! The proxy tracks request IDs from `tools/call` requests to know which
//! responses need filtering and which tool's filter to apply.

use crate::mcp_filters;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Run the MCP proxy.
///
/// Spawns the real MCP server as a child process and forwards JSON-RPC
/// messages bidirectionally. Tool call responses are filtered through
/// tool-specific filters before being passed back to Claude Code.
///
/// # Arguments
///
/// - `server_cmd`: Command to launch the real MCP server (e.g., "npx")
/// - `server_args`: Arguments for the server command
/// - `no_filter`: If true, disable filtering (pure passthrough for debugging)
/// - `verbose`: Verbosity level for debug output
pub fn run_proxy(
    server_cmd: &str,
    server_args: &[String],
    no_filter: bool,
    verbose: u8,
) -> Result<()> {
    if verbose > 0 {
        eprintln!("[clov-mcp] Starting proxy for: {} {:?}", server_cmd, server_args);
    }

    // Spawn the real MCP server
    let mut child = Command::new(server_cmd)
        .args(server_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // pass server logs through
        .spawn()
        .with_context(|| format!("Failed to spawn MCP server: {} {:?}", server_cmd, server_args))?;

    // Request ID → tool name mapping for response routing
    let pending_requests: Arc<Mutex<HashMap<Value, String>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Take ownership of child's stdin/stdout
    let child_stdin = child
        .stdin
        .take()
        .context("Failed to capture MCP server stdin")?;
    let child_stdout = child
        .stdout
        .take()
        .context("Failed to capture MCP server stdout")?;

    // Thread 1: Client stdin → Child stdin (forward requests)
    let pending_clone = Arc::clone(&pending_requests);
    let stdin_thread = thread::spawn(move || {
        forward_client_to_server(child_stdin, pending_clone, verbose)
    });

    // Main thread: Child stdout → Client stdout (filter responses)
    let result = forward_server_to_client(child_stdout, &pending_requests, no_filter, verbose);

    // Wait for child process
    let _ = child.wait();

    // Wait for stdin thread
    let _ = stdin_thread.join();

    result
}

/// Forward messages from client (our stdin) to the MCP server (child stdin).
///
/// Also tracks `tools/call` request IDs so we know which responses to filter.
fn forward_client_to_server(
    mut child_stdin: std::process::ChildStdin,
    pending_requests: Arc<Mutex<HashMap<Value, String>>>,
    verbose: u8,
) -> Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break, // stdin closed
        };

        if line.trim().is_empty() {
            continue;
        }

        // Try to parse and track tool call requests
        if let Ok(msg) = serde_json::from_str::<Value>(&line) {
            track_tool_call_request(&msg, &pending_requests, verbose);
        }

        // Forward to child regardless of parse success
        if writeln!(child_stdin, "{}", line).is_err() {
            break; // child stdin closed
        }
    }

    Ok(())
}

/// Forward messages from the MCP server (child stdout) to client (our stdout).
///
/// Intercepts `tools/call` responses and applies filters.
fn forward_server_to_client(
    child_stdout: std::process::ChildStdout,
    pending_requests: &Arc<Mutex<HashMap<Value, String>>>,
    no_filter: bool,
    verbose: u8,
) -> Result<()> {
    let reader = BufReader::new(child_stdout);
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let output = if no_filter {
            line
        } else {
            match serde_json::from_str::<Value>(&line) {
                Ok(msg) => {
                    let filtered = filter_tool_response(msg, pending_requests, verbose);
                    serde_json::to_string(&filtered).unwrap_or(line)
                }
                Err(_) => line, // not valid JSON, pass through
            }
        };

        if writeln!(stdout_lock, "{}", output).is_err() {
            break; // stdout closed
        }
        let _ = stdout_lock.flush();
    }

    Ok(())
}

/// Track `tools/call` requests so we can route responses to the right filter.
///
/// When we see a request like:
/// ```json
/// {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"web_search_exa",...}}
/// ```
/// We store `5 → "web_search_exa"` so when response ID=5 arrives, we know
/// which filter to apply.
fn track_tool_call_request(
    msg: &Value,
    pending: &Arc<Mutex<HashMap<Value, String>>>,
    verbose: u8,
) {
    let method = msg.get("method").and_then(|m| m.as_str());
    let id = msg.get("id");
    let tool_name = msg
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str());

    if let (Some("tools/call"), Some(id), Some(name)) = (method, id, tool_name) {
        if verbose > 0 {
            eprintln!("[clov-mcp] Tracking request id={} tool={}", id, name);
        }
        if let Ok(mut map) = pending.lock() {
            map.insert(id.clone(), name.to_string());
        }
    }
}

/// Filter a tool response if it matches a tracked request.
///
/// Looks up the response ID in our pending requests map. If found,
/// routes the response content through the appropriate tool filter.
fn filter_tool_response(
    mut msg: Value,
    pending: &Arc<Mutex<HashMap<Value, String>>>,
    verbose: u8,
) -> Value {
    // Only process responses (have "result" or "error", no "method")
    if msg.get("method").is_some() || msg.get("result").is_none() {
        return msg;
    }

    let id = match msg.get("id") {
        Some(id) => id.clone(),
        None => return msg,
    };

    // Look up which tool this response belongs to
    let tool_name = {
        let mut map = match pending.lock() {
            Ok(m) => m,
            Err(_) => return msg,
        };
        map.remove(&id)
    };

    let tool_name = match tool_name {
        Some(name) => name,
        None => return msg, // not a tracked tool call response
    };

    if verbose > 0 {
        eprintln!("[clov-mcp] Filtering response for tool: {}", tool_name);
    }

    // Filter the content array
    if let Some(result) = msg.get_mut("result") {
        if let Some(content) = result.get_mut("content") {
            if let Some(arr) = content.as_array_mut() {
                let mut total_input: usize = 0;
                let mut total_output: usize = 0;

                for item in arr.iter_mut() {
                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            total_input += text.len();

                            if let Some(filtered) =
                                mcp_filters::filter_for_tool(&tool_name, text)
                            {
                                total_output += filtered.len();
                                item["text"] = Value::String(filtered);
                            } else {
                                total_output += text.len();
                            }
                        }
                    }
                }

                if verbose > 0 && total_input > 0 {
                    let saved = total_input.saturating_sub(total_output);
                    let pct = if total_input > 0 {
                        (saved as f64 / total_input as f64) * 100.0
                    } else {
                        0.0
                    };
                    eprintln!(
                        "[clov-mcp] {} → {} chars ({:.0}% saved)",
                        total_input, total_output, pct
                    );
                }
            }
        }
    }

    msg
}

/// Check if a JSON-RPC message is a tool call response.
///
/// A tool call response has:
/// - "jsonrpc": "2.0"
/// - "id": <some value>
/// - "result" field (success) or "error" field (failure)
/// - NO "method" field (that would make it a request)
pub fn is_tool_call_response(msg: &Value) -> bool {
    msg.get("id").is_some()
        && (msg.get("result").is_some() || msg.get("error").is_some())
        && msg.get("method").is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_tool_call_response_valid() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "result": {
                "content": [{"type": "text", "text": "hello"}]
            }
        });
        assert!(is_tool_call_response(&msg));
    }

    #[test]
    fn test_is_tool_call_response_request_not_response() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {"name": "web_search_exa"}
        });
        assert!(!is_tool_call_response(&msg));
    }

    #[test]
    fn test_is_tool_call_response_notification_not_response() {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": "notifications/tools/list_changed"
        });
        assert!(!is_tool_call_response(&msg));
    }

    #[test]
    fn test_request_id_tracking() {
        let pending: Arc<Mutex<HashMap<Value, String>>> = Arc::new(Mutex::new(HashMap::new()));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "web_search_exa",
                "arguments": {"query": "rust programming"}
            }
        });

        track_tool_call_request(&request, &pending, 0);

        let map = pending.lock().unwrap();
        assert_eq!(map.get(&json!(42)), Some(&"web_search_exa".to_string()));
    }

    #[test]
    fn test_filter_tool_response_filters_tracked() {
        let pending: Arc<Mutex<HashMap<Value, String>>> = Arc::new(Mutex::new(HashMap::new()));

        // Track a request
        pending
            .lock()
            .unwrap()
            .insert(json!(7), "web_search_exa".to_string());

        // Create a response with nav chrome
        let response = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "result": {
                "content": [{
                    "type": "text",
                    "text": "Skip to main content\nMenu\n\nActual search result content here.\n\nCookie\n© 2025"
                }]
            }
        });

        let filtered = filter_tool_response(response, &pending, 0);
        let text = filtered["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("Actual search result content"));
        assert!(!text.contains("Skip to main content"));
        assert!(!text.contains("Cookie"));

        // Request should be removed from pending
        assert!(pending.lock().unwrap().is_empty());
    }

    #[test]
    fn test_passthrough_non_tool_messages() {
        let pending: Arc<Mutex<HashMap<Value, String>>> = Arc::new(Mutex::new(HashMap::new()));

        // A tools/list response — has no tracked ID, should pass through
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": [{"name": "web_search_exa", "description": "Search the web"}]
            }
        });

        let result = filter_tool_response(msg.clone(), &pending, 0);
        assert_eq!(result, msg); // unchanged
    }
}
