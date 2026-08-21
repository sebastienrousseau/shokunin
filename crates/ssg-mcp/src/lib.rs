// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Model Context Protocol server over ssg's existing RPC registry.
//!
//! MCP is line-delimited JSON-RPC 2.0 on stdio. The protocol surface an
//! editor actually needs is small — `initialize`, `tools/list`,
//! `tools/call` — so this is hand-rolled rather than pulling in an SDK for
//! three message shapes.
//!
//! Nothing here defines tools. `#[ssg_rpc]` already registers every callable
//! with its input and output JSON Schema, and that registry is walked at
//! runtime, so a tool added to ssg appears over MCP without a second
//! declaration. A separate list would be a copy, and copies drift — the
//! failure this codebase has hit repeatedly.
//!
//! Transport is deliberately separate from protocol: [`handle_line`] maps
//! one request string to an optional response string and touches no I/O, so
//! the whole surface is testable without spawning a process or a pipe.

use serde_json::{json, Value};

/// MCP revision this server implements.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// JSON-RPC 2.0 error codes used here.
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;

fn error(id: Value, code: i64, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
    .to_string()
}

fn result(id: Value, payload: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": payload }).to_string()
}

/// Every registered RPC, as an MCP tool definition.
///
/// The input schema is passed through verbatim: `schemars` already emits
/// JSON Schema, which is what MCP's `inputSchema` is.
#[must_use]
pub fn tools() -> Vec<Value> {
    ssg_rpc::dispatch::iter_descriptors()
        .map(|d| {
            let schema = (d.schema)();
            json!({
                "name": d.name,
                "description": format!(
                    "ssg RPC `{}`. Input and output schemas are generated \
                     from the Rust types, not hand-written.",
                    d.name
                ),
                "inputSchema": schema.input,
            })
        })
        .collect()
}

/// Handle one JSON-RPC line.
///
/// Returns `None` for notifications, which by JSON-RPC 2.0 take no reply —
/// answering one would desynchronise a client that is not expecting a
/// message.
#[must_use]
pub fn handle_line(line: &str) -> Option<String> {
    let req: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return Some(error(Value::Null, PARSE_ERROR, "invalid JSON")),
    };

    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    // Absent `id` means notification. `null` is a real id, so the two are
    // distinguished by presence, not by value.
    let id = req.get("id").cloned();

    if method.is_empty() {
        let reply_id = id.unwrap_or(Value::Null);
        return Some(error(reply_id, INVALID_REQUEST, "missing method"));
    }

    let Some(id) = id else {
        // Notifications: `notifications/initialized` is the expected one.
        return None;
    };

    match method {
        "initialize" => Some(result(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "ssg-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                },
            }),
        )),
        "ping" => Some(result(id, json!({}))),
        "tools/list" => Some(result(id, json!({ "tools": tools() }))),
        "tools/call" => Some(handle_call(id, &req)),
        _ => Some(error(id, METHOD_NOT_FOUND, method)),
    }
}

fn handle_call(id: Value, req: &Value) -> String {
    let params = req.get("params");
    let Some(name) = params.and_then(|p| p.get("name")).and_then(Value::as_str)
    else {
        return error(id, INVALID_REQUEST, "params.name is required");
    };

    // MCP omits `arguments` when a tool takes none; the RPC trampolines
    // expect a JSON object either way.
    let args = params
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    match ssg_rpc::dispatch(name, &args.to_string()) {
        Ok(payload) => result(
            id,
            json!({
                "content": [{ "type": "text", "text": payload }],
                "isError": false,
            }),
        ),
        // A tool that fails is not a protocol failure. MCP wants the error
        // reported *inside* a successful result so the model can read it and
        // adapt; a JSON-RPC error would surface as a broken server instead.
        Err(e) => result(
            id,
            json!({
                "content": [{ "type": "text", "text": e.to_string() }],
                "isError": true,
            }),
        ),
    }
}

/// Drive the protocol over a reader and writer.
///
/// Split out from `main` so the loop itself is exercised by tests with
/// in-memory buffers rather than a spawned process.
///
/// # Errors
///
/// Returns any I/O error from the underlying reader or writer.
pub fn serve<R: std::io::BufRead, W: std::io::Write>(
    input: R,
    mut output: W,
) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = handle_line(&line) {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use ssg_rpc::{ssg_rpc, RpcError};

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct EchoIn {
        text: String,
    }

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct EchoOut {
        text: String,
    }

    // The host binary decides what is registered; `inventory` only sees what
    // is linked. ssg itself currently registers no production RPC, so a test
    // that asserted "the registry is non-empty" would be asserting a fact
    // about someone else's crate — and would pass or fail for reasons
    // unrelated to this code. Registering a fixture here exercises the
    // mapping itself, which is what this crate is responsible for.
    #[ssg_rpc]
    #[doc = "Test fixture: echoes its input back."]
    fn ssg_mcp_test_echo(input: EchoIn) -> Result<EchoOut, RpcError> {
        Ok(EchoOut { text: input.text })
    }

    fn call(line: &str) -> Value {
        let out = handle_line(line).expect("expected a response");
        serde_json::from_str(&out).expect("response must be JSON")
    }

    #[test]
    fn initialize_reports_protocol_and_server() {
        let v = call(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#);
        assert_eq!(v["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(v["result"]["serverInfo"]["name"], "ssg-mcp");
        assert_eq!(v["id"], 1);
    }

    #[test]
    fn tools_list_is_generated_from_the_rpc_registry() {
        let v = call(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#);
        let tools = v["result"]["tools"].as_array().expect("tools array");
        let names: Vec<&str> =
            tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(
            names.contains(&"ssg_mcp_test_echo"),
            "the fixture RPC should surface as a tool: {names:?}"
        );
        // Every tool must carry a schema: a tool an agent cannot call
        // correctly is worse than one that is absent.
        for t in tools {
            assert!(t["name"].is_string(), "tool without a name: {t}");
            assert!(
                t["inputSchema"].is_object(),
                "tool without an inputSchema: {t}"
            );
        }
        assert_eq!(tools.len(), ssg_rpc::registered_names().len());
    }

    /// The path that matters: a client lists tools, calls one, and reads a
    /// result. Everything else is framing.
    #[test]
    fn tools_call_round_trips_through_the_rpc_dispatcher() {
        let v = call(
            r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"ssg_mcp_test_echo","arguments":{"text":"hello"}}}"#,
        );
        assert_eq!(v["result"]["isError"], false, "call failed: {v}");
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .expect("text content");
        let payload: Value =
            serde_json::from_str(text).expect("tool output must be JSON");
        assert_eq!(payload["text"], "hello");
    }

    #[test]
    fn notifications_get_no_reply() {
        // JSON-RPC 2.0: a request without `id` is a notification. Replying
        // would desynchronise a client that is not reading one.
        assert!(handle_line(
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
        )
        .is_none());
    }

    #[test]
    fn null_id_is_an_id_not_a_notification() {
        let v = call(r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#);
        assert!(v.get("result").is_some(), "null id must still be answered");
    }

    #[test]
    fn unknown_method_is_method_not_found() {
        let v = call(r#"{"jsonrpc":"2.0","id":3,"method":"nope"}"#);
        assert_eq!(v["error"]["code"], METHOD_NOT_FOUND);
    }

    #[test]
    fn malformed_json_is_a_parse_error() {
        let v = call("{not json");
        assert_eq!(v["error"]["code"], PARSE_ERROR);
    }

    #[test]
    fn missing_method_is_an_invalid_request() {
        let v = call(r#"{"jsonrpc":"2.0","id":4}"#);
        assert_eq!(v["error"]["code"], INVALID_REQUEST);
    }

    #[test]
    fn call_without_a_name_is_rejected() {
        let v = call(
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{}}"#,
        );
        assert_eq!(v["error"]["code"], INVALID_REQUEST);
    }

    #[test]
    fn unknown_tool_reports_inside_the_result() {
        // Not a JSON-RPC error: the model should see the failure and adapt,
        // rather than the client treating the server as broken.
        let v = call(
            r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"nope","arguments":{}}}"#,
        );
        assert!(v.get("error").is_none(), "should not be a protocol error");
        assert_eq!(v["result"]["isError"], true);
    }

    /// A reader that fails mid-stream must surface the error, not truncate
    /// the session silently. A server that stops answering looks identical
    /// to one that finished.
    #[test]
    fn serve_propagates_a_reader_error() {
        struct Failing;
        impl std::io::Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("reader exploded"))
            }
        }
        let reader = std::io::BufReader::new(Failing);
        let mut out = Vec::new();
        let err = serve(reader, &mut out).expect_err("should propagate");
        assert!(err.to_string().contains("reader exploded"));
    }

    /// Likewise a writer: if the client's pipe closes, the loop must stop
    /// with an error rather than spin producing replies nobody reads.
    #[test]
    fn serve_propagates_a_writer_error() {
        // Accepts the bytes, then fails the flush — so both trait methods
        // are exercised, and the failure lands on the path `serve` actually
        // takes after writing a reply.
        struct Failing;
        impl std::io::Write for Failing {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("writer exploded"))
            }
        }
        let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
        let err = serve(std::io::Cursor::new(input), Failing)
            .expect_err("should propagate");
        assert!(err.to_string().contains("writer exploded"));
    }

    #[test]
    fn serve_answers_requests_and_skips_notifications() {
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n",
            "\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n",
        );
        let mut out = Vec::new();
        serve(std::io::Cursor::new(input), &mut out).expect("serve");
        let text = String::from_utf8(out).expect("utf8");
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2, "one reply each, none for the notification");
        assert!(lines[0].contains("protocolVersion"));
        assert!(lines[1].contains("\"result\""));
    }
}
