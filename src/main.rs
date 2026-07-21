use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Deserialize)]
struct RpcRequest { id: Option<Value>, method: String, #[serde(default)] params: Option<Value> }

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")] id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")] result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")] error: Option<RpcError>,
}

#[derive(Serialize)]
struct RpcError { code: i32, message: String }

#[derive(Deserialize)]
struct ToolCallParams { name: String, #[serde(default)] arguments: Option<Value> }

#[derive(Deserialize)]
struct SearchArgs { query: String, #[serde(default)] freshness: Option<String> }

#[derive(Deserialize)]
struct FetchArgs { url: String }

#[derive(Deserialize)]
struct ApiResponse { data: ApiData }

#[derive(Deserialize)]
struct ApiData { #[serde(rename = "webPages")] web_pages: WebPages }

#[derive(Deserialize)]
struct WebPages { value: Vec<WebPageItem> }

#[derive(Deserialize)]
struct WebPageItem { name: String, url: String }

fn input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "query": { "type": "string" },
            "freshness": {
                "type": "string",
                "enum": ["noLimit", "oneDay", "oneWeek", "oneMonth", "oneYear"]
            }
        },
        "required": ["query"]
    })
}

fn fetch_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "url": { "type": "string" }
        },
        "required": ["url"]
    })
}

fn clean_url(url: &str) -> String {
    url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url)
        .trim_end_matches('/').to_string()
}

fn respond(id: Value, result: Value) -> RpcResponse {
    RpcResponse { jsonrpc: "2.0", id: Some(id), result: Some(result), error: None }
}

fn respond_err(id: Value, code: i32, msg: String) -> RpcResponse {
    RpcResponse { jsonrpc: "2.0", id: Some(id), result: None, error: Some(RpcError { code, message: msg }) }
}

fn http_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .timeout_global(Some(Duration::from_secs(30)))
                .build(),
        )
    })
}

fn handle_call_tool(params: Option<&Value>) -> Value {
    let call: ToolCallParams = match params.and_then(|p| serde_json::from_value(p.clone()).ok()) {
        Some(p) => p,
        None => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": "invalid params"}]}),
    };
    match call.name.as_str() {
        "WebSearch" => handle_search(call.arguments),
        "WebFetch" => handle_fetch(call.arguments),
        _ => {
            let msg = format!("unknown tool: {}", call.name);
            serde_json::json!({"isError": true, "content": [{"type": "text", "text": msg}]})
        },
    }
}

fn handle_search(arguments: Option<Value>) -> Value {
    let search: SearchArgs = arguments.and_then(|a| serde_json::from_value(a).ok())
        .unwrap_or(SearchArgs { query: String::new(), freshness: None });
    let freshness = search.freshness.unwrap_or_else(|| "noLimit".to_string());
    let api_key = std::env::var("LANGSEARCH_API_KEY").unwrap_or_default();
    let body = serde_json::json!({ "query": search.query, "freshness": freshness });

    let response = match http_agent()
        .post("https://api.langsearch.com/v1/web-search")
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send_json(&body)
    {
        Ok(r) => r,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };
    let api: ApiResponse = match response.into_body().read_json() {
        Ok(r) => r,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };
    let mut lines: Vec<String> = Vec::new();
    for item in &api.data.web_pages.value {
        lines.push(format!("{} {}", item.name, clean_url(&item.url)));
    }
    serde_json::json!({ "content": [{"type": "text", "text": lines.join("\n")}] })
}

fn handle_fetch(arguments: Option<Value>) -> Value {
    let fetch: FetchArgs = match arguments.and_then(|a| serde_json::from_value(a).ok()) {
        Some(f) => f,
        None => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": "invalid params"}]}),
    };
    let url = fetch.url;

    // Fetch HTML with ureq (in-process, no curl subprocess)
    let html = match http_agent().get(&url).call() {
        Ok(r) => match r.into_body().read_to_string() {
            Ok(s) => s,
            Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
        },
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };

    // Pipe through html2markdown
    let mut child = match Command::new("html2markdown")
        .args([
            "--domain", &url,
            "--plugin-table",
            "--opt-table-header-promotion",
            "--opt-table-cell-padding-behavior", "minimal",
            "--opt-table-skip-empty-rows",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };

    // Write HTML to html2markdown's stdin and close
    if let Err(e) = child.stdin.take().unwrap().write_all(html.as_bytes()) {
        return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]});
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return serde_json::json!({"isError": true, "content": [{"type": "text", "text": stderr}]});
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let cleaned = clean_markdown(&text);
    serde_json::json!({ "content": [{"type": "text", "text": cleaned}] })
}

fn clean_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_blank = false;

    for line in text.lines() {
        let line = strip_noise_links(line);
        let trimmed = line.trim();

        // Collapse consecutive blank lines
        if trimmed.is_empty() {
            if prev_blank {
                continue;
            }
            prev_blank = true;
        } else {
            prev_blank = false;
        }

        result.push_str(&line);
        result.push('\n');
    }

    result
}

fn strip_noise_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while !rest.is_empty() {
        // Strip empty link []()
        if rest.starts_with("[](") {
            if let Some(end) = rest[3..].find(')') {
                rest = &rest[3 + end + 1..];
                continue;
            }
        }

        // Strip [hide]() link
        if rest.starts_with("[hide](") {
            if let Some(end) = rest[7..].find(')') {
                rest = &rest[7 + end + 1..];
                continue;
            }
        }

        let ch = rest.chars().next().unwrap();
        result.push(ch);
        rest = &rest[ch.len_utf8()..];
    }

    result
}

fn handle(req: RpcRequest) -> RpcResponse {
    let id = req.id.unwrap_or(Value::Null);
    match req.method.as_str() {
        "initialize" => respond(id, serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "webhands", "version": env!("CARGO_PKG_VERSION") }
        })),
        "notifications/initialized" => RpcResponse { jsonrpc: "2.0", id: None, result: None, error: None },
        "tools/list" => respond(id, serde_json::json!({
            "tools": [
                { "name": "WebSearch", "inputSchema": input_schema() },
                { "name": "WebFetch", "inputSchema": fetch_input_schema() }
            ]
        })),
        "tools/call" => respond(id, handle_call_tool(req.params.as_ref())),
        other => respond_err(id, -32601, format!("method not found: {other}")),
    }
}

fn main() {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    for line in stdin.lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        if line.trim().is_empty() { continue; }
        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                let _ = writeln!(stdout, r#"{{"jsonrpc":"2.0","error":{{"code":-32700,"message":"parse error"}}}}"#);
                continue;
            }
        };
        let res = handle(req);
        if res.id.is_none() { continue; }
        let _ = writeln!(stdout, "{}", serde_json::to_string(&res).unwrap());
    }
}
