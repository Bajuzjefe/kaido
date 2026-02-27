mod tools;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                write_response(&stdout, &resp);
                continue;
            }
        };

        let _ = request.jsonrpc; // acknowledge
        // JSON-RPC notifications (no id) must not produce a response.
        if request.id.is_none() {
            if request.method == "notifications/initialized" {
                continue;
            }
            continue;
        }

        let id = request.id.clone().unwrap_or(Value::Null);
        let response = handle_request(&request.method, &request.params, id);
        write_response(&stdout, &response);
    }
}

fn write_response(stdout: &io::Stdout, response: &JsonRpcResponse) {
    let json = serde_json::to_string(response).unwrap();
    let mut out = stdout.lock();
    let _ = writeln!(out, "{}", json);
    let _ = out.flush();
}

fn handle_request(method: &str, params: &Value, id: Value) -> JsonRpcResponse {
    match method {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "kaido",
                    "version": "0.1.0"
                }
            })),
            error: None,
        },

        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "tools": tools::tool_definitions()
            })),
            error: None,
        },

        "tools/call" => {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

            match tools::call_tool(name, &arguments) {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    })),
                    error: None,
                },
                Err(msg) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": msg
                        }],
                        "isError": true
                    })),
                    error: None,
                },
            }
        }

        "notifications/initialized" | "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(Value::Object(Default::default())),
            error: None,
        },

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", method),
            }),
        },
    }
}
