use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

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
                "enum": ["noLimit", "oneDay", "oneWeek", "oneMonth", "oneYear"],
                "default": "noLimit"
            }
        },
        "required": ["query"],
        "additionalProperties": false
    })
}

fn web_fetch_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "url": { "type": "string" }
        },
        "required": ["url"],
        "additionalProperties": false
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

fn handle_call_tool(params: Option<&Value>) -> Value {
    let call: ToolCallParams = match params.and_then(|p| serde_json::from_value(p.clone()).ok()) {
        Some(p) => p,
        None => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": "invalid params"}]}),
    };
    match call.name.as_str() {
        "web_search" => handle_web_search(call.arguments),
        "web_fetch" => handle_web_fetch(call.arguments),
        _ => {
            let msg = format!("unknown tool: {}", call.name);
            serde_json::json!({"isError": true, "content": [{"type": "text", "text": msg}]})
        },
    }
}

fn handle_web_search(arguments: Option<Value>) -> Value {
    let search: SearchArgs = arguments.and_then(|a| serde_json::from_value(a).ok())
        .unwrap_or(SearchArgs { query: String::new(), freshness: None });
    let freshness = search.freshness.unwrap_or_else(|| "noLimit".to_string());
    let api_key = std::env::var("LANGSEARCH_API_KEY").unwrap_or_default();
    let body = serde_json::json!({ "query": search.query, "freshness": freshness });

    let output = match Command::new("curl").arg("-s").arg("-X").arg("POST")
        .arg("https://api.langsearch.com/v1/web-search")
        .arg("-H").arg(format!("Authorization: Bearer {}", api_key))
        .arg("-H").arg("Content-Type: application/json")
        .arg("-d").arg(body.to_string())
        .arg("--max-time").arg("30").output()
    {
        Ok(o) => o,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };
    if !output.status.success() {
        return serde_json::json!({"isError": true, "content": [{"type": "text", "text": String::from_utf8_lossy(&output.stderr).into_owned()}]});
    }
    let api: ApiResponse = match serde_json::from_slice(&output.stdout) {
        Ok(r) => r,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": format!("{e}: {}", String::from_utf8_lossy(&output.stdout))}]}),
    };
    let mut lines: Vec<String> = Vec::new();
    for item in &api.data.web_pages.value {
        lines.push(format!("{} {}", item.name, clean_url(&item.url)));
    }
    serde_json::json!({ "content": [{"type": "text", "text": lines.join("\n")}] })
}

fn handle_web_fetch(arguments: Option<Value>) -> Value {
    let fetch: FetchArgs = match arguments.and_then(|a| serde_json::from_value(a).ok()) {
        Some(f) => f,
        None => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": "invalid params"}]}),
    };
    let url = fetch.url;

    let mut curl = match Command::new("curl")
        .args(["--no-progress-meter", "-L", "--max-time", "30", &url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };

    let curl_stdout = curl.stdout.take().unwrap();

    let html2md = match Command::new("html2markdown")
        .args([
            "--domain", &url,
            "--plugin-table",
            "--opt-table-header-promotion",
            "--opt-table-cell-padding-behavior", "minimal",
            "--opt-table-skip-empty-rows",
        ])
        .stdin(curl_stdout)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(h) => h,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };

    let output = match html2md.wait_with_output() {
        Ok(o) => o,
        Err(e) => return serde_json::json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
    };

    let curl_ok = match curl.wait() {
        Ok(s) => s.success(),
        Err(_) => false,
    };
    if !curl_ok {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let msg = if stderr.is_empty() { "curl request failed".to_string() } else { stderr };
        return serde_json::json!({"isError": true, "content": [{"type": "text", "text": msg}]});
    }

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
            "serverInfo": { "name": "web_search", "version": "1.0.0" }
        })),
        "notifications/initialized" => RpcResponse { jsonrpc: "2.0", id: None, result: None, error: None },
        "tools/list" => respond(id, serde_json::json!({
            "tools": [
                { "name": "web_search", "inputSchema": input_schema() },
                { "name": "web_fetch", "inputSchema": web_fetch_input_schema() }
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
