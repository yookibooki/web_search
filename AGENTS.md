# web_search — AGENTS.md

Hard rules and constraints for AI agents working on this project. Read before making any changes.

## Architecture

- Rust MCP server over stdio using **raw JSON-RPC 2.0** — no MCP SDK crate, no framework (avoids dependency bloat, minimizes binary size and memory consumption).
- Read newline-delimited JSON from stdin, parse, dispatch, write responses to stdout.
- **Single-threaded synchronous loop** — no async, no threads, no rayon, no tokio.

## Dependencies

Only ever add these crates: `serde` (with `derive` feature) and `serde_json`.
Everything else must come from `std` — no `reqwest`, no `anyhow`, no `thiserror`, no `clap`, no nothing.

## HTTP calls

- **Must** use `std::process::Command` to spawn `curl`. Never add an HTTP crate.
- Args: `-s -X POST https://api.langsearch.com/v1/web-search -H "Authorization: Bearer $LANGSEARCH_API_KEY" -H "Content-Type: application/json" -d '{...}' --max-time 30`
- API key from `LANGSEARCH_API_KEY` environment variable.
- If `curl` fails or returns non-JSON → `isError: true` with error text.

## MCP protocol

Handle exactly these methods:

| Method | Response |
|---|---|
| `initialize` | `{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"web_search","version":"1.0.0"}}` |
| `notifications/initialized` | No response (skip if `id` is `None`) |
| `tools/list` | Single tool `web_search` with `inputSchema` — no `description` fields anywhere |
| `tools/call` | Validate tool name, call HTTP endpoint, return results |
| anything else | JSON-RPC error `-32601` "method not found" |

### Tool schema (`web_search`)

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

No `description` field on any property or on the tool itself — minimizes token usage in the system prompt.

## Output formatting

- Results joined by newlines, each line: `name url`
- `name` → API item's `name` field
- `url` → remove `https://` prefix, then `http://` prefix, then strip trailing `/`
- Zero results → return empty string `""`

## Code style

- Permitted macros: `#[derive(Deserialize)]`, `#[derive(Serialize)]`, `serde_json::json!`.
  No custom `macro_rules!` macros, no proc macros beyond serde derives.

- No `unsafe`, no `extern crate`, no feature gates.

## Memory & performance

- No GC, no runtime — Rust handles this.
- Keep the process lean: minimal allocations, no leaking, no extra threads.
- HTTP/TLS/DNS memory lives in the `curl` subprocess and is reclaimed when it exits.

## Build & config

- Build: `cargo build --release` (profile in `Cargo.toml` already sets `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`).
- Binary: `target/release/web_search`.
- Always strip the binary after build (already done by profile).

## Testing

- No test framework. Test by piping JSON-RPC messages through the binary and checking responses.
- Cover: initialize, tools/list, notifications/initialized (no output), tools/call (real API), unknown method, unknown tool, parse error.

## Install scripts

- `install.sh` — POSIX sh, detects Linux/macOS + arch, downloads from GitHub releases.
  - Linux: installs to `~/.local/bin/web_search`
  - macOS: installs to `/usr/local/bin/web_search`
- `install.ps1` — PowerShell, detects Windows arch, downloads from GitHub releases, installs to `%LOCALAPPDATA%\Microsoft\WindowsApps\web_search.exe` (already in PATH).
- Both accept `VERSION` env var (defaults to `v0.1.0`).
- Download URL pattern: `https://github.com/yookibooki/web_search/releases/download/${VERSION}/web_search-${TARGET}`.

## Release workflow (`.github/workflows/releaser.yml`)

- Trigger: push tag `v*`.
- **3 build jobs** — `build-linux` (ubuntu-24.04), `build-macos` (macos-latest), `build-windows` (windows-latest) — each runs in parallel.
- Each job builds its **native** target and **cross-compiles** one additional target:
  - Linux: `x86_64-unknown-linux-gnu` (native) + `aarch64-unknown-linux-gnu` (cross)
  - macOS: `aarch64-apple-darwin` (native) + `x86_64-apple-darwin` (cross)
  - Windows: `x86_64-pc-windows-msvc` (native) + `aarch64-pc-windows-msvc` (cross)
- Uses `Swatinem/rust-cache@v2` for dependency caching.
- Binary renamed to `web_search-{target}{.ext}` before upload.
- Separate `release` job collects all artifacts and publishes via `softprops/action-gh-release@v3`.

## Prohibited changes

- Do not add any HTTP client crate (reqwest, ureq, etc.).
- Do not add async runtime, threads, or tokio.
- Do not add MCP SDK or any framework.
- Do not add `description` fields to the tool schema.
- Do not change the URL cleaning logic.
- Do not switch to a different API endpoint.
- Do not add optional dependencies or feature flags.
