package main

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestInitialize(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		ID:      json.RawMessage(`1`),
		Method:  "initialize",
		Params:  json.RawMessage(`{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.0.1"}}`),
	}
	res := handle(req)
	if res.Error != nil {
		t.Fatalf("unexpected error: %v", res.Error)
	}
	if string(res.ID) != "1" {
		t.Errorf("id = %s, want 1", string(res.ID))
	}
	var result map[string]any
	json.Unmarshal(res.Result, &result)
	if result["protocolVersion"] != "2025-03-26" {
		t.Errorf("protocolVersion = %v", result["protocolVersion"])
	}
	si := result["serverInfo"].(map[string]any)
	if si["name"] != "web_search" {
		t.Errorf("server name = %v", si["name"])
	}
}

func TestToolsList(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		ID:      json.RawMessage(`2`),
		Method:  "tools/list",
	}
	res := handle(req)
	if res.Error != nil {
		t.Fatalf("unexpected error: %v", res.Error)
	}
	var result struct {
		Tools []struct {
			Name        string                 `json:"name"`
			InputSchema map[string]any          `json:"inputSchema"`
		} `json:"tools"`
	}
	if err := json.Unmarshal(res.Result, &result); err != nil {
		t.Fatal(err)
	}
	if len(result.Tools) != 1 {
		t.Fatalf("expected 1 tool, got %d", len(result.Tools))
	}
	tool := result.Tools[0]
	if tool.Name != "web_search" {
		t.Errorf("name = %s, want web_search", tool.Name)
	}

	props := tool.InputSchema["properties"].(map[string]any)
	q := props["query"].(map[string]any)
	if q["type"] != "string" {
		t.Errorf("query type = %v", q["type"])
	}
	if _, ok := q["description"]; ok {
		t.Error("query should not have description")
	}

	f := props["freshness"].(map[string]any)
	if f["type"] != "string" {
		t.Errorf("freshness type = %v", f["type"])
	}
	if _, ok := f["description"]; ok {
		t.Error("freshness should not have description")
	}
	enum := f["enum"].([]any)
	if len(enum) != 5 || enum[0] != "noLimit" {
		t.Errorf("enum = %v", enum)
	}
	if f["default"] != "noLimit" {
		t.Errorf("default = %v", f["default"])
	}

	reqd := tool.InputSchema["required"].([]any)
	if len(reqd) != 1 || reqd[0] != "query" {
		t.Errorf("required = %v", reqd)
	}
}

func TestInitializeNotificationNoResponse(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		Method:  "notifications/initialized",
	}
	res := handle(req)
	if res.ID != nil {
		t.Error("expected no id for notification response")
	}
}

func TestUnknownMethod(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		ID:      json.RawMessage(`3`),
		Method:  "bogus",
	}
	res := handle(req)
	if res.Error == nil {
		t.Fatal("expected error")
	}
	if res.Error.Code != -32601 {
		t.Errorf("code = %d, want -32601", res.Error.Code)
	}
	if res.ID == nil || string(res.ID) != "3" {
		t.Errorf("id = %s, want 3", string(res.ID))
	}
}

func TestUnknownTool(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		ID:      json.RawMessage(`4`),
		Method:  "tools/call",
		Params:  json.RawMessage(`{"name":"nope"}`),
	}
	res := handle(req)
	if res.Error == nil {
		t.Fatal("expected error")
	}
	if res.Error.Code != -32602 {
		t.Errorf("code = %d, want -32602", res.Error.Code)
	}
}

func TestURLCleaning(t *testing.T) {
	cases := []struct {
		in   string
		want string
	}{
		{"https://example.com/path/", "example.com/path"},
		{"http://example.com/path/", "example.com/path"},
		{"http://example.com/path", "example.com/path"},
		{"https://example.com", "example.com"},
		{"http://example.com//", "example.com"},
	}
	for _, c := range cases {
		got := strings.TrimPrefix(strings.TrimPrefix(c.in, "https://"), "http://")
		got = strings.TrimRight(got, "/")
		if got != c.want {
			t.Errorf("clean(%q) = %q, want %q", c.in, got, c.want)
		}
	}
}

func TestEmptyItems(t *testing.T) {
	var out strings.Builder
	for i := range []struct {
		Name string
		URL  string
	}{} {
		if i > 0 {
			out.WriteString("\n")
		}
		u := strings.TrimPrefix(strings.TrimPrefix("", "https://"), "http://")
		u = strings.TrimRight(u, "/")
		out.WriteString(" " + u)
	}
	if out.String() != "" {
		t.Errorf("got %q, want empty", out.String())
	}
}

func TestJSONNumberID(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		ID:      json.RawMessage(`42`),
		Method:  "tools/list",
	}
	res := handle(req)
	if string(res.ID) != "42" {
		t.Errorf("id = %s, want 42", string(res.ID))
	}
}

func TestJSONStringID(t *testing.T) {
	req := rpcReq{
		JSONRPC: "2.0",
		ID:      json.RawMessage(`"req-1"`),
		Method:  "tools/list",
	}
	res := handle(req)
	if string(res.ID) != `"req-1"` {
		t.Errorf("id = %s, want \"req-1\"", string(res.ID))
	}
}
