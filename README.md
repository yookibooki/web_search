# web search MCP for AI agents — free and unlimited

Adds ~18 tokens to your system prompt.

Get your free API key at [langsearch.com](https://langsearch.com), set it as `LANGSEARCH_API_KEY`, and install:

**Linux / macOS**
```sh
curl -fsSL https://raw.githubusercontent.com/yookibooki/web_search/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
powershell -c "irm https://raw.githubusercontent.com/yookibooki/web_search/main/install.ps1 | iex"
```

Add to your AI tool config:
```json
{
  "mcpServers": {
    "web_search": {
      "command": "web_search",
      "env": { "LANGSEARCH_API_KEY": "<your-api-key>" }
    }
  }
}
```
