use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct RpcForwarder {
    client: Client,
    provider_url: String,
}

impl RpcForwarder {
    pub fn new(provider_url: String) -> Self {
        Self {
            client: Client::new(),
            provider_url,
        }
    }

    pub async fn forward_request(&self, json_rpc: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Parse request to ensure it's valid JSON
        let request: Value = serde_json::from_slice(json_rpc)?;
    
        // Forward to RPC provider
        let response = self
            .client
            .post(&self.provider_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
    
        // Check if the response status is successful
        if !response.status().is_success() {
            eprintln!("RPC provider returned error status: {}", response.status());
            return Err(anyhow::anyhow!("RPC provider returned error status: {}", response.status()));
        }
    
        // Get response body
        let response_text = response.text().await?;
            
        // Debug: print the response
        eprintln!("Gateway received response from RPC provider: {}", response_text);
            
        // Parse the response to ensure it's valid JSON-RPC
        let _response_value: Value = serde_json::from_str(&response_text)?;
            
        Ok(response_text.as_bytes().to_vec())
    }
}
