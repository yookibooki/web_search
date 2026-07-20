You are starting with a blank context. Carries no memory of prior sessions.

The user wants you to rewrite their MCP web_search server in Rust, targeting minimal memory usage (~2 MB RSS) and a small binary.

## Directory

Working directory: `/home/dev/workspace/web_search`
There is an existing `python_mcp_server.py` file in the directory — read it to understand the exact behavior.

## Requirements

### Protocol
- Implement MCP protocol over stdio using raw JSON-RPC 2.0 (no MCP SDK crate).
- Read newline-delimited JSON from stdin, parse each message, dispatch, write responses to stdout.
- Handle these methods:
  - `initialize` → respond with server capabilities and `"protocolVersion": "2025-03-26"`
  - `notifications/initialized` → no response
  - `tools/list` → respond with the `web_search` tool definition (schema below)
  - `tools/call` → validate tool name, call the HTTP endpoint, return results
  - anything else → JSON-RPC method-not-found error

### Tool schema
The tool is named `web_search` with input schema:
```json
{
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
}
```

No `description` fields anywhere.

### HTTP call
- Do NOT use `reqwest` or any Rust HTTP crate.
- Use `std::process::Command` to spawn `curl` with these args:
  `-s -X POST https://api.langsearch.com/v1/web-search`
  `-H "Authorization: Bearer $LANGSEARCH_API_KEY"`
  `-H "Content-Type: application/json"`
  `-d '{"query":"...","freshness":"..."}'`
  `--max-time 30`
- The API key comes from the `LANGSEARCH_API_KEY` environment variable.
- If `freshness` is not provided by the client, default to `"noLimit"`.
- If `curl` fails or returns non-JSON, return `isError: true` with the error text.
- Parse the API response JSON using `serde` + `serde_json`.

### Output format
Preserve the exact same formatting as the Python version, which joins results separated by newlines, each line being: `name url` where:
- `name` is the `name` field from the API response item
- `url` is the `url` field with `https://` prefix removed, then `http://` prefix removed, then trailing `/` characters stripped
- If the API returns zero results, return an empty string

### Memory strategy
- No GC, no runtime — Rust handles this naturally.
- No memory-tuning crates or hacks needed.
- Use `std::process::Command` for HTTP so all TLS/HTTP/DNS memory lives in the `curl` subprocess and is reclaimed when it exits.
- The Rust process itself should stay lean: minimal allocations, no leaking, no extra threads.

### Dependencies
Only external crates: `serde` (with `derive`) and `serde_json`. Everything else from stdlib.

### Code style
- Aim for under 200 lines of readable, well-structured code.
- Keep it simple: no macros (beyond `#[derive(Deserialize)]`), no async, no threads.
- Read one JSON-RPC message at a time from stdin, process synchronously, write response.

### Build and config
- Build with `cargo build --release`
- Strip the binary: `strip target/release/web_search`
- Update `~/.kimi-code/mcp.json` to point to the compiled binary at `/home/dev/workspace/web_search/target/release/web_search`
- Keep the existing `LANGSEARCH_API_KEY`, `GODEBUG=madvdontneed=1`, and `GOMEMLIMIT=5242880` env vars in mcp.json (the env vars don't hurt Rust and may be useful)

### Testing
- No external test harness needed. Test the server with a quick end-to-end by piping JSON-RPC messages through it and checking responses.
