# WebHands — free search and fetch MCP for AI agents

Adds ~18 tokens to your system prompt.

Get your free API key at [langsearch.com](https://langsearch.com), set it as `LANGSEARCH_API_KEY`, and install:

**Linux / macOS**
```sh
curl -fsSL https://raw.githubusercontent.com/yookibooki/webhands/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
powershell -c "irm https://raw.githubusercontent.com/yookibooki/webhands/main/install.ps1 | iex"
```

Installs the `webhands` binary and its companion `html2markdown` converter (used by `WebFetch`) to:

- **Linux** → `~/.local/bin/webhands` (+ `~/.local/bin/html2markdown`)
- **macOS** → `/usr/local/bin/webhands` (+ `/usr/local/bin/html2markdown`)
- **Windows** → `%LOCALAPPDATA%\Microsoft\WindowsApps\webhands.exe` (+ `html2markdown.exe`)

If the command isn't found after install, make sure the install directory is on your `PATH`
(e.g. on Linux add `~/.local/bin` to your shell's `PATH`).

Add to your AI tool config:
```json
{
  "mcpServers": {
    "webhands": {
      "command": "webhands",
      "env": { "LANGSEARCH_API_KEY": "<your-api-key>" }
    }
  }
}
```

## Usage

Two tools are exposed over MCP:

- **`WebSearch`** — searches the web. Takes a required `query` string and an optional
  `freshness` (`noLimit`, `oneDay`, `oneWeek`, `oneMonth`, `oneYear`).
- **`WebFetch`** — fetches a URL and returns it as cleaned markdown. Takes a required `url` string.

## Install

Both installers download the **latest release** of `webhands` and `html2markdown` via
GitHub's `releases/latest/download` endpoint — no version is pinned and no API
call is made.
