package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"runtime"
	"runtime/debug"
	"strings"
)

type Freshness string

const (
	FreshnessNoLimit  Freshness = "noLimit"
	FreshnessOneDay   Freshness = "oneDay"
	FreshnessOneWeek  Freshness = "oneWeek"
	FreshnessOneMonth Freshness = "oneMonth"
	FreshnessOneYear  Freshness = "oneYear"
)

type rpcReq struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id,omitempty"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params,omitempty"`
}

type rpcRes struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id,omitempty"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *rpcErr         `json:"error,omitempty"`
}

type rpcErr struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

var inputSchema = map[string]any{
	"type": "object",
	"properties": map[string]any{
		"query":     map[string]any{"type": "string"},
		"freshness": map[string]any{"type": "string", "enum": []string{"noLimit", "oneDay", "oneWeek", "oneMonth", "oneYear"}, "default": "noLimit"},
	},
	"required":             []string{"query"},
	"additionalProperties": false,
}

func respond(id json.RawMessage, v any) rpcRes {
	b, _ := json.Marshal(v)
	return rpcRes{JSONRPC: "2.0", ID: id, Result: json.RawMessage(b)}
}

func respondErr(id json.RawMessage, code int, msg string) rpcRes {
	return rpcRes{JSONRPC: "2.0", ID: id, Error: &rpcErr{Code: code, Message: msg}}
}

func callTool(args json.RawMessage) map[string]any {
	var in struct {
		Query     string    `json:"query"`
		Freshness Freshness `json:"freshness,omitempty"`
	}
	json.Unmarshal(args, &in)
	if in.Freshness == "" {
		in.Freshness = FreshnessNoLimit
	}

	body, _ := json.Marshal(map[string]any{"query": in.Query, "freshness": in.Freshness})
	apiKey := os.Getenv("LANGSEARCH_API_KEY")

	// Run curl in a subprocess so all HTTP/TLS/DNS memory lives outside
	// the Go process and is reclaimed when curl exits.
	cmd := exec.Command("curl", "-s", "-X", "POST",
		"https://api.langsearch.com/v1/web-search",
		"-H", "Authorization: Bearer "+apiKey,
		"-H", "Content-Type: application/json",
		"-d", string(body),
		"--max-time", "30",
	)
	raw, err := cmd.Output()
	if err != nil {
		return map[string]any{
			"isError": true,
			"content": []map[string]any{{"type": "text", "text": err.Error()}},
		}
	}

	var apiResp struct {
		Data struct {
			WebPages struct {
				Value []struct {
					Name string `json:"name"`
					URL  string `json:"url"`
				} `json:"value"`
			} `json:"webPages"`
		} `json:"data"`
	}
	if err := json.Unmarshal(raw, &apiResp); err != nil {
		return map[string]any{
			"isError": true,
			"content": []map[string]any{{"type": "text", "text": err.Error()}},
		}
	}

	var out strings.Builder
	for i, item := range apiResp.Data.WebPages.Value {
		if i > 0 {
			out.WriteByte('\n')
		}
		u := strings.TrimPrefix(strings.TrimPrefix(item.URL, "https://"), "http://")
		u = strings.TrimRight(u, "/")
		out.WriteString(item.Name)
		out.WriteByte(' ')
		out.WriteString(u)
	}

	return map[string]any{
		"content": []map[string]any{{"type": "text", "text": out.String()}},
	}
}

func handle(req rpcReq) rpcRes {
	switch req.Method {
	case "initialize":
		return respond(req.ID, map[string]any{
			"protocolVersion": "2025-03-26",
			"capabilities":    map[string]any{"tools": map[string]any{}},
			"serverInfo":      map[string]any{"name": "web_search", "version": "1.0.0"},
		})

	case "notifications/initialized":
		return rpcRes{}

	case "tools/list":
		return respond(req.ID, map[string]any{
			"tools": []map[string]any{{"name": "web_search", "inputSchema": inputSchema}},
		})

	case "tools/call":
		var p struct {
			Name      string          `json:"name"`
			Arguments json.RawMessage `json:"arguments,omitempty"`
		}
		if err := json.Unmarshal(req.Params, &p); err != nil {
			return respondErr(req.ID, -32700, "invalid params")
		}
		if p.Name != "web_search" {
			return respondErr(req.ID, -32602, fmt.Sprintf("unknown tool: %s", p.Name))
		}
		return respond(req.ID, callTool(p.Arguments))

	default:
		return respondErr(req.ID, -32601, fmt.Sprintf("method not found: %s", req.Method))
	}
}

func main() {
	log.SetOutput(os.Stderr)
	log.SetFlags(0)
	runtime.GOMAXPROCS(1)

	dec := json.NewDecoder(os.Stdin)
	enc := json.NewEncoder(os.Stdout)
	enc.SetEscapeHTML(false)

	for {
		var raw json.RawMessage
		if err := dec.Decode(&raw); err != nil {
			if err == io.EOF {
				return
			}
			log.Printf("stdin: %v", err)
			return
		}

		var req rpcReq
		if err := json.Unmarshal(raw, &req); err != nil {
			enc.Encode(respondErr(nil, -32700, "parse error"))
			continue
		}

		res := handle(req)
		if len(req.ID) > 0 {
			enc.Encode(res)
		}
		if req.Method == "tools/call" {
			runtime.GC()
			debug.FreeOSMemory()
		}
	}
}
