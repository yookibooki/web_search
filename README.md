# web_search

MCP server that exposes web search as a tool. Calls LangSearch API via `curl` (pre-installed on macOS and Windows 10+).

## Prerequisites

- [LangSearch API key](https://langsearch.com) — set as `LANGSEARCH_API_KEY` environment variable

## Install

**Linux / macOS**
```sh
curl -fsSL https://raw.githubusercontent.com/yookibooki/web_search/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
powershell -c "irm https://raw.githubusercontent.com/yookibooki/web_search/main/install.ps1 | iex"
```

Installs to `~/.local/bin/web_search`. Make sure `~/.local/bin` is in your `$PATH`.

## MCP Client Setup

Add this entry to your MCP client config:

**Claude Desktop / VS Code / Cline / any MCP client:**

```json
{
  "mcpServers": {
    "web_search": {
      "command": "web_search",
      "env": {
        "LANGSEARCH_API_KEY": "<your-api-key>"
      }
    }
  }
}
```

## Usage

The server exposes one tool: `web_search`.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | — | Search query |
| `freshness` | string | no | `noLimit` | `noLimit`, `oneDay`, `oneWeek`, `oneMonth`, `oneYear` |

## Build from Source

```sh
cargo build --release
# binary: target/release/web_search
```

## Supported Platforms

| OS | Architecture |
|---|---|
| Linux | x86_64, ARM64 |
| macOS (Apple Silicon) | ARM64 |
| macOS (Intel) | x86_64 |
| Windows | x86_64, ARM64 |
