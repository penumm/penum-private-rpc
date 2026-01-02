use crate::penum_client::PenumRpcClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub async fn start_rpc_server(
    port: u16,
    penum_client: Arc<PenumRpcClient>,
) -> anyhow::Result<()> {
    let penum_client = warp::any().map(move || penum_client.clone());

    let rpc_route = warp::post()
        .and(warp::path::end())
        .and(warp::body::json())
        .and(penum_client)
        .and_then(handle_rpc_request);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    println!("🔒 Penum RPC Server listening on http://127.0.0.1:{}", port);
    println!("📋 Supported methods: eth_call, eth_getBalance, eth_blockNumber, eth_sendRawTransaction, eth_getTransactionReceipt");

    warp::serve(rpc_route.with(cors))
        .run(([127, 0, 0, 1], port))
        .await;

    Ok(())
}

async fn handle_rpc_request(
    request: JsonRpcRequest,
    penum_client: Arc<PenumRpcClient>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate supported methods
    let supported_methods = [
        "eth_call",
        "eth_getBalance",
        "eth_blockNumber",
        "eth_sendRawTransaction",
        "eth_getTransactionReceipt",
    ];

    if !supported_methods.contains(&request.method.as_str()) {
        let error_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not supported: {}", request.method),
            }),
            id: request.id,
        };
        return Ok(warp::reply::json(&error_response));
    }

    // Serialize request to JSON
    let request_json = match serde_json::to_vec(&request) {
        Ok(json) => json,
        Err(_) => {
            let error_response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error: Failed to serialize request".to_string(),
                }),
                id: request.id,
            };
            return Ok(warp::reply::json(&error_response));
        }
    };

    // Send through Penum
    match penum_client.send_rpc_request(&request_json).await {
        Ok(response_data) => {
            // Parse response - the gateway returns the raw JSON-RPC response from the provider
            // Convert the response bytes to string and parse as JSON
            match std::str::from_utf8(&response_data) {
                Ok(response_str) => {
                    match serde_json::from_str::<JsonRpcResponse>(response_str) {
                        Ok(response) => Ok(warp::reply::json(&response)),
                        Err(_) => {
                            // If parsing as structured response fails, try to parse as raw JSON and wrap in proper response
                            match serde_json::from_str::<serde_json::Value>(response_str) {
                                Ok(raw_response) => {
                                    // Try to extract a valid response from the raw JSON
                                    if let Some(obj) = raw_response.as_object() {
                                        if obj.contains_key("result") || obj.contains_key("error") {
                                            // This looks like a valid JSON-RPC response, return as-is
                                            Ok(warp::reply::json(&raw_response))
                                        } else {
                                            // This doesn't look like a valid response, return error
                                            let error_response = JsonRpcResponse {
                                                jsonrpc: "2.0".to_string(),
                                                result: None,
                                                error: Some(JsonRpcError {
                                                    code: -32603,
                                                    message: "Invalid response format from gateway".to_string(),
                                                }),
                                                id: request.id,
                                            };
                                            Ok(warp::reply::json(&error_response))
                                        }
                                    } else {
                                        let error_response = JsonRpcResponse {
                                            jsonrpc: "2.0".to_string(),
                                            result: None,
                                            error: Some(JsonRpcError {
                                                code: -32603,
                                                message: "Invalid response from gateway".to_string(),
                                            }),
                                            id: request.id,
                                        };
                                        Ok(warp::reply::json(&error_response))
                                    }
                                }
                                Err(_) => {
                                    let error_response = JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code: -32603,
                                            message: "Invalid response from gateway".to_string(),
                                        }),
                                        id: request.id,
                                    };
                                    Ok(warp::reply::json(&error_response))
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: "Invalid UTF-8 response from gateway".to_string(),
                        }),
                        id: request.id,
                    };
                    Ok(warp::reply::json(&error_response))
                }
            }
        }
        Err(_e) => {
            // Fail silently - never log internal errors to prevent information leakage
            let error_response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                }),
                id: request.id,
            };
            Ok(warp::reply::json(&error_response))
        }
    }
}
