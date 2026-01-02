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


        // Generate ephemeral keys for this request
        let client_keys = EphemeralKeys::generate();
        let client_pub = client_keys.public;

        // Connect to the entry relay which will forward to middle relay and then to gateway
        let mut stream = TcpStream::connect(self.config.entry_relay).await?;

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
        
        // Encrypt payload with header as AAD (this is a request)
        let tag = encrypt_in_place(&session_key, header, payload, true)?;
        tag_space.copy_from_slice(&tag);

        // Send encrypted packet (exactly 1024 bytes)
        assert_eq!(encrypted_packet.len(), PACKET_SIZE);
        stream.write_all(&encrypted_packet).await?;

        // Receive encrypted response (exactly 1024 bytes)
        let mut response = [0u8; PACKET_SIZE];
        stream.read_exact(&mut response).await?;

        // Decrypt response using same session key (this is a response)
        let (resp_header, resp_data_and_tag) = response.split_at(32);
        let (resp_data, resp_tag) = resp_data_and_tag.split_at(PACKET_SIZE - 32 - AEAD_TAG_LEN);
        let mut resp_data = resp_data.to_vec();
        
        let tag_array: [u8; 16] = resp_tag.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid tag length"))?;
        decrypt_in_place(&session_key, resp_header, &mut resp_data, &tag_array, false)?;

        // Extract JSON-RPC response from padding
        // Find the JSON by looking for the first '{' and the last '}' in the response data
        let json_start = resp_data.iter().position(|&b| b == b'{')
            .ok_or_else(|| anyhow::anyhow!("No JSON response found"))?;
        
        // Find end of JSON (last '}' after the start)
        let json_end = resp_data[json_start..].iter().rposition(|&b| b == b'}')
            .ok_or_else(|| anyhow::anyhow!("Incomplete JSON response"))?;
        
        let json_response_candidate = &resp_data[json_start..json_start + json_end + 1];
        
        // Validate that the extracted data is valid UTF-8 before parsing
        let json_str = std::str::from_utf8(json_response_candidate)
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in JSON response"))?;
        
        // Validate that it's proper JSON and is a valid JSON-RPC response
        let parsed_json: Value = serde_json::from_str(json_str)
            .map_err(|_| anyhow::anyhow!("Invalid JSON format"))?;
        
        // Verify it has the required JSON-RPC response fields
        if parsed_json.get("jsonrpc").is_none() && !parsed_json.get("result").is_some() && !parsed_json.get("error").is_some() {
            return Err(anyhow::anyhow!("Invalid JSON-RPC response format"));
        }
        
        let json_response = json_response_candidate;
        
        Ok(json_response.to_vec())
    }
}