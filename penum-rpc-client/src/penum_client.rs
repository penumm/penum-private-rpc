use crate::config::RpcClientConfig;
use crate::crypto::{encrypt_in_place, decrypt_in_place, EphemeralKeys, derive_session_key};
use crate::packet::{Packet, PACKET_SIZE, AEAD_TAG_LEN};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use x25519_dalek::PublicKey;
use serde_json::Value;

pub struct PenumRpcClient {
    config: RpcClientConfig,
}

impl PenumRpcClient {
    pub fn new(config: RpcClientConfig) -> Self {
        Self { config }
    }

    pub async fn send_rpc_request(&self, json_rpc: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Validate request size
        if json_rpc.len() > 960 {
            return Err(anyhow::anyhow!("Request too large: {} bytes (max 960)", json_rpc.len()));
        }

        // Create random packet for privacy (cryptographically secure random padding)
        let mut buffer = Packet::new_random();

        // Place JSON-RPC payload in packet
        // We place it near the end to leave room for different packet layouts
        let payload_len = json_rpc.len();
        let payload_start = PACKET_SIZE - AEAD_TAG_LEN - payload_len - 32;
        buffer[payload_start..payload_start + payload_len].copy_from_slice(json_rpc);

        // Debug: print what we're sending
        if let Ok(json_str) = std::str::from_utf8(json_rpc) {
            eprintln!("Sending JSON request: {}", json_str);
        } else {
            eprintln!("Sending request is not valid UTF-8");
        }

        // Generate ephemeral keys for this request
        let client_keys = EphemeralKeys::generate();
        let client_pub = client_keys.public;

        // Connect to gateway
        let mut stream = TcpStream::connect(self.config.gateway).await?;

        // Penum handshake: send client public key (32 bytes)
        stream.write_all(client_pub.as_bytes()).await?;

        // Receive gateway public key (32 bytes)
        let mut server_pub_bytes = [0u8; 32];
        stream.read_exact(&mut server_pub_bytes).await?;
        let server_pub = PublicKey::from(server_pub_bytes);

        // Derive session key using HKDF with salt "penum-v1"
        let shared_secret = client_keys.diffie_hellman(&server_pub);
        let session_key = derive_session_key(shared_secret);

        // Encrypt packet: header (32 bytes) + payload (976 bytes) + tag (16 bytes)
        let mut encrypted_packet = buffer;
        let (header, payload_and_tag) = encrypted_packet.split_at_mut(32);
        let (payload, tag_space) = payload_and_tag.split_at_mut(PACKET_SIZE - 32 - AEAD_TAG_LEN);
        
        // Encrypt payload with header as AAD
        let tag = encrypt_in_place(&session_key, header, payload)?;
        tag_space.copy_from_slice(&tag);

        // Send encrypted packet (exactly 1024 bytes)
        assert_eq!(encrypted_packet.len(), PACKET_SIZE);
        stream.write_all(&encrypted_packet).await?;

        // Receive encrypted response (exactly 1024 bytes)
        let mut response = [0u8; PACKET_SIZE];
        stream.read_exact(&mut response).await?;

        // Decrypt response using same session key
        let (resp_header, resp_data_and_tag) = response.split_at(32);
        let (resp_data, resp_tag) = resp_data_and_tag.split_at(PACKET_SIZE - 32 - AEAD_TAG_LEN);
        let mut resp_data = resp_data.to_vec();
        
        let tag_array: [u8; 16] = resp_tag.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid tag length"))?;
        decrypt_in_place(&session_key, resp_header, &mut resp_data, &tag_array)?;

        // Extract JSON-RPC response from padding
        // The gateway places the response at the end of the payload section with random padding at the start
        // So we should look for JSON at the end of the payload first (similar to how gateway handles requests)
        
        // Try to find JSON at the end of the payload (where gateway places it)
        let mut json_response: &[u8] = &[];
        let mut found_json = false;
        
        // Look for JSON from the end backwards, as the gateway places it at the end
        // Start from near the end and work backwards to find a valid JSON
        for end in (0..resp_data.len()).rev() {
            if resp_data[end] == b'}' {
                // Look for matching opening bracket from this end position backwards
                let mut brace_count = 1;
                for start in (0..end).rev() {
                    if resp_data[start] == b'}' {
                        brace_count += 1;
                    } else if resp_data[start] == b'{' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            // Found potential JSON object
                            let candidate = &resp_data[start..end + 1];
                            if std::str::from_utf8(candidate).is_ok() {
                                // Try to parse as JSON to make sure it's valid
                                if serde_json::from_slice::<Value>(candidate).is_ok() {
                                    json_response = candidate;
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
            let json_start = resp_data.iter().position(|&b| b == b'{')
                .ok_or_else(|| anyhow::anyhow!("No JSON response found"))?;
            
            // Find end of JSON (last '}' after the start)
            let json_end = resp_data[json_start..].iter().rposition(|&b| b == b'}')
                .ok_or_else(|| anyhow::anyhow!("Incomplete JSON response"))?;
            
            let mut json_response_candidate = &resp_data[json_start..json_start + json_end + 1];
            
            // Validate that the extracted data is valid UTF-8 before parsing
            if std::str::from_utf8(json_response_candidate).is_ok() {
                // Try to parse as JSON to make sure it's valid
                if serde_json::from_slice::<Value>(json_response_candidate).is_ok() {
                    json_response = json_response_candidate;
                    found_json = true;
                }
            }
            
            // If still not found, try to find a valid JSON substring
            if !found_json {
                for start in json_start..resp_data.len() {
                    if resp_data[start] == b'{' {
                        for end in (start + 1)..resp_data.len() {
                            if resp_data[end] == b'}' {
                                let candidate = &resp_data[start..end + 1];
                                if std::str::from_utf8(candidate).is_ok() {
                                    // Try to parse as JSON to make sure it's valid
                                    if serde_json::from_slice::<Value>(candidate).is_ok() {
                                        json_response = candidate;
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
            return Err(anyhow::anyhow!("Could not extract valid JSON from response payload"));
        }
        
        Ok(json_response.to_vec())
    }
}