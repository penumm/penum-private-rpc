use crate::crypto::{decrypt_in_place, encrypt_in_place, derive_session_key, EphemeralKeys};
use crate::rpc_forwarder::RpcForwarder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use x25519_dalek::PublicKey;
use rand::RngCore;
use serde_json::Value;

const PACKET_SIZE: usize = 1024;
const HEADER_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const PAYLOAD_SIZE: usize = PACKET_SIZE - HEADER_SIZE - TAG_SIZE;

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
        stream.read_exact(&mut client_pub_bytes).await?;
        let client_pub = PublicKey::from(client_pub_bytes);

        // Send server public key (32 bytes)
        stream.write_all(server_pub.as_bytes()).await?;

        // Derive session key using HKDF with salt "penum-v1"
        let shared_secret = server_keys.diffie_hellman(&client_pub);
        let session_key = derive_session_key(shared_secret);

        // Receive encrypted packet (exactly 1024 bytes)
        let mut encrypted_packet = [0u8; PACKET_SIZE];
        stream.read_exact(&mut encrypted_packet).await?;

        // Decrypt packet
        let (header, payload_and_tag) = encrypted_packet.split_at(HEADER_SIZE);
        let (payload, tag) = payload_and_tag.split_at(PAYLOAD_SIZE);
        let mut payload = payload.to_vec();
        
        let tag_array: [u8; TAG_SIZE] = tag.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid tag size"))?;
        decrypt_in_place(&session_key, header, &mut payload, &tag_array)?;

        // Extract JSON-RPC request from padding
        // The client places the JSON at the end of the payload with random padding at the start
        // So we should look for JSON at the end of the payload first
        
        // Try to find JSON at the end of the payload (where client places it)
        let mut json_rpc: &[u8] = &[];
        let mut found_json = false;
        
        // Look for JSON from the end backwards, as the client places it at the end
        // Start from near the end and work backwards to find a valid JSON
        for end in (0..payload.len()).rev() {
            if payload[end] == b'}' {
                // Look for matching opening bracket from this end position backwards
                let mut brace_count = 1;
                for start in (0..end).rev() {
                    if payload[start] == b'}' {
                        brace_count += 1;
                    } else if payload[start] == b'{' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            // Found potential JSON object
                            let candidate = &payload[start..end + 1];
                            if std::str::from_utf8(candidate).is_ok() {
                                // Try to parse as JSON to make sure it's valid
                                if serde_json::from_slice::<Value>(candidate).is_ok() {
                                    json_rpc = candidate;
                                    found_json = true;
                                    break;
                                }
                            }
                        }
                    }
                    if found_json {
                        break;
                    }
                }
                if found_json {
                    break;
                }
            }
        }
        
        // If we didn't find JSON from the end, try the original method
        if !found_json {
            // Find start of JSON (first '{')
            let json_start = payload.iter().position(|&b| b == b'{')
                .ok_or_else(|| anyhow::anyhow!("No JSON request found"))?;
            
            // Find end of JSON (last '}' after the start)
            let json_end = payload[json_start..].iter().rposition(|&b| b == b'}')
                .ok_or_else(|| anyhow::anyhow!("Incomplete JSON request"))?;
            
            let mut json_rpc_candidate = &payload[json_start..json_start + json_end + 1];
            
            // Validate that the extracted data is valid UTF-8 before parsing
            if std::str::from_utf8(json_rpc_candidate).is_ok() {
                // Try to parse as JSON to make sure it's valid
                if serde_json::from_slice::<Value>(json_rpc_candidate).is_ok() {
                    json_rpc = json_rpc_candidate;
                    found_json = true;
                }
            }
            
            // If still not found, try to find a valid JSON substring
            if !found_json {
                for start in json_start..payload.len() {
                    if payload[start] == b'{' {
                        for end in (start + 1)..payload.len() {
                            if payload[end] == b'}' {
                                let candidate = &payload[start..end + 1];
                                if std::str::from_utf8(candidate).is_ok() {
                                    // Try to parse as JSON to make sure it's valid
                                    if serde_json::from_slice::<Value>(candidate).is_ok() {
                                        json_rpc = candidate;
                                        found_json = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if found_json {
                            break;
                        }
                    }
                }
            }
        }
        
        if !found_json {
            return Err(anyhow::anyhow!("Could not extract valid JSON from payload"));
        }
        
        // Debug: print the extracted JSON (first 200 chars)
        if let Ok(json_str) = std::str::from_utf8(json_rpc) {
            eprintln!("Extracted JSON request: {}", json_str.get(0..std::cmp::min(200, json_str.len())).unwrap_or("<too short to show>"));
        } else {
            eprintln!("Extracted request is not valid UTF-8");
            return Err(anyhow::anyhow!("Invalid UTF-8 in JSON request"));
        }

        // Forward to RPC provider
        let response = self.rpc_forwarder.forward_request(json_rpc).await?;

        // Create response packet with random padding
        let mut response_packet = [0u8; PACKET_SIZE];
        rand::thread_rng().fill_bytes(&mut response_packet);

        // Place response in packet (near end, leaving random padding at start)
        // If response is too large, truncate it to fit in the payload size
        let response_len = std::cmp::min(response.len(), PAYLOAD_SIZE);
        let response_start = PAYLOAD_SIZE - response_len;
        response_packet[HEADER_SIZE + response_start..HEADER_SIZE + response_start + response_len]
            .copy_from_slice(&response[..response_len]);

        // Encrypt response with same session key
        let (resp_header, resp_payload_and_tag) = response_packet.split_at_mut(HEADER_SIZE);
        let (resp_payload, resp_tag_space) = resp_payload_and_tag.split_at_mut(PAYLOAD_SIZE);
        
        let tag = encrypt_in_place(&session_key, resp_header, resp_payload)?;
        resp_tag_space.copy_from_slice(&tag);

        // Send encrypted response (exactly 1024 bytes)
        assert_eq!(response_packet.len(), PACKET_SIZE);
        stream.write_all(&response_packet).await?;

        Ok(())
    }
}

pub async fn start_gateway(
    listen_addr: &str,
    listen_port: u16,
    rpc_forwarder: RpcForwarder,
) -> anyhow::Result<()> {
    let gateway = Gateway::new(rpc_forwarder);
    let listener = TcpListener::bind(format!("{}:{}", listen_addr, listen_port)).await?;

    println!("🌐 Penum Gateway listening on {}:{}", listen_addr, listen_port);
    println!("   Privacy mode: ON (no logging of request contents)");

    loop {
        let (stream, addr) = listener.accept().await?;
        
        // Clone gateway to handle each connection concurrently
        let gateway_clone = gateway.clone();
        
        // Spawn a task to handle each connection concurrently
        tokio::spawn(async move {
            let result = gateway_clone.handle_connection(stream).await;
            if let Err(e) = result {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}