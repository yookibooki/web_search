# WebHands — AGENTS.md

Hard rules and constraints for AI agents working on this project. Read before making any changes.

## Architecture

- Rust MCP server over stdio using **raw JSON-RPC 2.0** — no MCP SDK crate, no framework (avoids dependency bloat, minimizes binary size and memory consumption).
- Read newline-delimited JSON from stdin, parse, dispatch, write responses to stdout.
- **Single-threaded synchronous loop** — no async, no threads, no rayon, no tokio.

## Dependencies

Only ever add these crates: `serde` (with `derive` feature), `serde_json`, and `ureq`.
Everything else must come from `std` — no `reqwest`, no `anyhow`, no `thiserror`, no `clap`, no nothing.

## HTTP calls

- **Must** use `ureq` crate with a shared `Agent` (configured with 30s global timeout via `OnceLock`). Never use `std::process::Command` for HTTP.
- **`WebSearch` tool**: `POST https://api.langsearch.com/v1/web-search` with `Authorization: Bearer $LANGSEARCH_API_KEY` and `Content-Type: application/json`, body `{"query": "...", "freshness": "..."}`. Use `.send_json()` and `.into_json()`.
- **`WebFetch` tool**: `GET <url>` via `ureq`, then pipe the HTML string into `html2markdown --domain=<url> --plugin-table --opt-table-header-promotion --opt-table-cell-padding-behavior minimal --opt-table-skip-empty-rows` via child process stdin, then post-process in Rust (strip `[]()` empty links, `[hide]()` links, collapse blank lines).
- API key from `LANGSEARCH_API_KEY` environment variable.
- `html2markdown` is a system binary (Go-based HTML-to-Markdown converter) installed alongside `webhands`. Not a Rust crate.
- If `ureq` fails or returns non-JSON → `isError: true` with error text.

## MCP protocol

Handle exactly these methods:

| Method | Response |
|---|---|
| `initialize` | `{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"webhands","version":"1.0.0"}}` |
| `notifications/initialized` | No response (skip if `id` is `None`) |
| `tools/list` | Two tools `WebSearch` and `WebFetch` with `inputSchema` — no `description` fields anywhere |
| `tools/call` | Validate tool name, dispatch to search or fetch, return results |
| anything else | JSON-RPC error `-32601` "method not found" |

### Tool schema (`WebSearch`)

```json
{
  "type": "object",
  "properties": {
    "query": { "type": "string" },
    "freshness": {
      "type": "string",
      "enum": ["noLimit", "oneDay", "oneWeek", "oneMonth", "oneYear"]
    }
  },
  "required": ["query"]
}
```

No `description` field on any property or on the tool itself — minimizes token usage in the system prompt.

### Tool schema (`WebFetch`)

```json
{
  "type": "object",
  "properties": {
    "url": { "type": "string" }
  },
  "required": ["url"]
}
```

No `description` fields. Same rule applies.

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
- HTTP/TLS/DNS is handled in-process by `ureq` (pure Rust, no subprocess overhead).

## Build & config

- Build: `cargo build --release` (profile in `Cargo.toml` already sets `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`).
- Binary: `target/release/webhands`.
- Always strip the binary after build (already done by profile).

## Testing

- No test framework. Test by piping JSON-RPC messages through the binary and checking responses.
- Cover: initialize, tools/list, notifications/initialized (no output), tools/call (real API), unknown method, unknown tool, parse error.

## Install scripts

- `install.sh` — POSIX sh, detects Linux/macOS + arch, downloads from GitHub releases.
  - Linux: installs to `~/.local/bin/webhands`
  - macOS: installs to `/usr/local/bin/webhands`
  - Supports x86_64, aarch64/arm64, i686/i386 on Linux; x86_64 and arm64 on macOS.
- `install.ps1` — PowerShell, detects Windows arch, downloads from GitHub releases, installs to `%LOCALAPPDATA%\Microsoft\WindowsApps\webhands.exe` (already in PATH).
  - Supports x86_64, ARM64, i686/i386.
- Both scripts also download and extract the `html2markdown` binary from `JohannesKaufmann/html-to-markdown` alongside `webhands`.
- Both download from the GitHub `releases/latest/download` endpoint — no version is hardcoded and no API call is made.
- Download URL patterns:
  - `https://github.com/yookibooki/webhands/releases/latest/download/webhands-${TARGET}`
  - `https://github.com/JohannesKaufmann/html-to-markdown/releases/latest/download/html-to-markdown_${HTML_TARGET}.tar.gz` (`.zip` on Windows)

## Release workflow (`.github/workflows/releaser.yml`)

- Trigger: push tag `v*`.
- **3 build jobs** — `build-linux` (ubuntu-24.04), `build-macos` (macos-latest), `build-windows` (windows-latest) — each runs in parallel.
- Each job builds its **native** target and **cross-compiles** two additional targets:
  - Linux: `x86_64-unknown-linux-gnu` (native) + `aarch64-unknown-linux-gnu` + `i686-unknown-linux-gnu` (cross)
  - macOS: `aarch64-apple-darwin` (native) + `x86_64-apple-darwin` (cross)
  - Windows: `x86_64-pc-windows-msvc` (native) + `aarch64-pc-windows-msvc` + `i686-pc-windows-msvc` (cross)
- Uses `Swatinem/rust-cache@v2` for dependency caching.
- Binary renamed to `webhands-{target}{.ext}` before upload.
- Separate `release` job collects all artifacts and publishes via `softprops/action-gh-release@v3`.

## Prohibited changes

- Do not add async runtime, threads, or tokio.
- Do not add MCP SDK or any framework.
- Do not add `description` fields to the tool schema.
- Do not change the URL cleaning logic.
- Do not switch to a different API endpoint.
- Do not add optional dependencies or feature flags.
- Do not replace `html2markdown` with a Rust crate — it must remain a system binary invoked via `Command`.
- Do not use `std::process::Command` for HTTP — use `ureq` instead.
