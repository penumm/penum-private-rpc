use crate::crypto::{decrypt_in_place, encrypt_in_place, derive_session_key, EphemeralKeys};
use crate::rpc_forwarder::RpcForwarder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use x25519_dalek::PublicKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const PACKET_SIZE: usize = 1024;
const HEADER_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const PAYLOAD_SIZE: usize = PACKET_SIZE - HEADER_SIZE - TAG_SIZE;

#[derive(Deserialize, Serialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: Value,
    id: Value,
}

pub struct Gateway {
    rpc_forwarder: RpcForwarder,
}

impl Clone for Gateway {
    fn clone(&self) -> Self {
        Gateway {
            rpc_forwarder: self.rpc_forwarder.clone(),
        }
    }
}

impl Gateway {
    pub fn new(rpc_forwarder: RpcForwarder) -> Self {
        Self { rpc_forwarder }
    }

    pub async fn handle_connection(&self, mut stream: TcpStream) -> anyhow::Result<()> {
        // Generate ephemeral keys for this connection
        let server_keys = EphemeralKeys::generate();
        let server_pub = server_keys.public;

        // Receive client public key (32 bytes)
        let mut client_pub_bytes = [0u8; 32];
        if let Err(_) = stream.read_exact(&mut client_pub_bytes).await {
            return Ok(()); // Fail silently
        }
        let client_pub = PublicKey::from(client_pub_bytes);

        // Send server public key (32 bytes)
        if let Err(_) = stream.write_all(server_pub.as_bytes()).await {
            return Ok(()); // Fail silently
        }

        // Derive session key using HKDF with salt "penum-v1"
        let shared_secret = server_keys.diffie_hellman(&client_pub);
        let session_key = derive_session_key(shared_secret);

        // Receive encrypted packet (exactly 1024 bytes)
        let mut encrypted_packet = [0u8; PACKET_SIZE];
        if let Err(_) = stream.read_exact(&mut encrypted_packet).await {
            return Ok(()); // Fail silently
        }

        // Decrypt packet (this is a request)
        let (header, payload_and_tag) = encrypted_packet.split_at(HEADER_SIZE);
        let (payload, tag) = payload_and_tag.split_at(PAYLOAD_SIZE);
        let mut payload = payload.to_vec();
        
        let tag_array: [u8; TAG_SIZE] = match tag.try_into() {
            Ok(tag) => tag,
            Err(_) => return Ok(()), // Fail silently
        };
        if let Err(_) = decrypt_in_place(&session_key, header, &mut payload, &tag_array, true) {
            return Ok(()); // Fail silently
        }

        // Extract JSON-RPC request from padding
        // The client places the JSON at the end of the payload with random padding at the start
        
        // Find the JSON by looking for the first '{' and the last '}' in the payload
        // But also validate that the extracted content is valid JSON to avoid false matches
        let json_start = match payload.iter().position(|&b| b == b'{') {
            Some(pos) => pos,
            None => return Ok(()), // Fail silently
        };
        
        // Find end of JSON (last '}' after the start)
        let json_end = match payload[json_start..].iter().rposition(|&b| b == b'}') {
            Some(pos) => pos,
            None => return Ok(()), // Fail silently
        };
        
        let json_rpc_candidate = &payload[json_start..json_start + json_end + 1];
        
        // Validate that the extracted data is valid UTF-8 before parsing
        let json_str = match std::str::from_utf8(json_rpc_candidate) {
            Ok(s) => s,
            Err(_) => return Ok(()), // Fail silently
        };
        
        // Validate that it's proper JSON and is a valid JSON-RPC request
        // This adds extra validation to make the parsing more robust
        let parsed_json: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => return Ok(()), // Fail silently
        };
        
        // Verify it has the required JSON-RPC fields
        if parsed_json.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
            return Ok(()); // Fail silently
        }
        
        if parsed_json.get("method").and_then(|v| v.as_str()).is_none() {
            return Ok(()); // Fail silently
        }
        
        
        // Now convert to our specific request struct
        let request: JsonRpcRequest = match serde_json::from_value(parsed_json) {
            Ok(req) => req,
            Err(_) => return Ok(()), // Fail silently
        };
        
        let json_rpc = json_rpc_candidate;
        



        
        // Validate method name format
        if request.method.is_empty() {
            return Ok(()); // Fail silently
        }
        
        // Validate that method name is a valid string (no control characters, reasonable length)
        if request.method.len() > 100 {
            return Ok(()); // Fail silently
        }
        
        // Check for potentially dangerous methods
        if request.method.starts_with("_") {
            return Ok(()); // Fail silently
        }
        
        // MEV safety check: validate transaction privacy parameters
        if request.method == "eth_sendRawTransaction" {
            // Check for MEV protection parameters in the transaction
            if let Some(params) = request.params.as_array() {
                if let Some(tx_data) = params.get(0) {
                    if let Some(tx_str) = tx_data.as_str() {
                        // Validate transaction format
                        if !tx_str.starts_with("0x") {
                            return Ok(()); // Fail silently
                        }
                        
                        // Check for privacy-enhancing transaction metadata
                        // This is a hook for future privacy features
                        if tx_str.len() < 10 { // Minimum transaction length check
                            return Ok(()); // Fail silently
                        }
                    }
                }
            }
        }
        
        // Forward to RPC provider
        let response = match self.rpc_forwarder.forward_request(json_rpc).await {
            Ok(resp) => resp,
            Err(_) => return Ok(()), // Fail silently
        };

        // Create response packet with random padding
        let mut response_packet = [0u8; PACKET_SIZE];
        rand::thread_rng().fill_bytes(&mut response_packet);

        // Place response in packet (near end, leaving random padding at start)
        // If response is too large, truncate it to fit in the payload size
        let response_len = std::cmp::min(response.len(), PAYLOAD_SIZE);
        let response_start = PAYLOAD_SIZE - response_len;
        response_packet[HEADER_SIZE + response_start..HEADER_SIZE + response_start + response_len]
            .copy_from_slice(&response[..response_len]);

        // Encrypt response with same session key (this is a response)
        let (resp_header, resp_payload_and_tag) = response_packet.split_at_mut(HEADER_SIZE);
        let (resp_payload, resp_tag_space) = resp_payload_and_tag.split_at_mut(PAYLOAD_SIZE);
        
        let tag = match encrypt_in_place(&session_key, resp_header, resp_payload, false) {
            Ok(t) => t,
            Err(_) => return Ok(()), // Fail silently
        };
        resp_tag_space.copy_from_slice(&tag);

        // Send encrypted response (exactly 1024 bytes)
        assert_eq!(response_packet.len(), PACKET_SIZE);
        if let Err(_) = stream.write_all(&response_packet).await {
            return Ok(()); // Fail silently
        }

        Ok(())
    }
}

pub async fn start_gateway(
    listen_addr: &str,
    listen_port: u16,
    rpc_forwarder: RpcForwarder,
    _allow_public_mempool: bool,
) -> anyhow::Result<()> {
    let gateway = Gateway::new(rpc_forwarder);
    let listener = TcpListener::bind(format!("{}:{}", listen_addr, listen_port)).await?;

    println!("🌐 Penum Gateway listening on {}:{}", listen_addr, listen_port);
    println!("   Privacy mode: ON (no logging of request contents)");

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                // Clone gateway to handle each connection concurrently
                let gateway_clone = gateway.clone();
                
                // Spawn a task to handle each connection concurrently
                tokio::spawn(async move {
                    let _ = gateway_clone.handle_connection(stream).await;
                    // Fail silently - never log connection errors to prevent information leakage
                });
            }
            Err(_) => {
                // Continue loop on accept error to maintain service availability
                continue;
            }
        }
    }
}