use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct RpcForwarder {
    client: Client,
    provider_url: String,
    allow_public_mempool: bool,
    mev_blocker_url: Option<String>,
}

impl RpcForwarder {
    pub fn new(provider_url: String, allow_public_mempool: bool, mev_blocker_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            provider_url,
            allow_public_mempool,
            mev_blocker_url,
        }
    }

    pub async fn forward_request(&self, json_rpc: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Parse request to ensure it's valid JSON
        let request: Value = serde_json::from_slice(json_rpc)?;
        
        // Extract method name
        let method = request.get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid JSON-RPC: missing method field"))?;
        
        // Transaction privacy guard: prevent eth_sendRawTransaction from going to public RPC
        // if allow_public_mempool is false
        if method == "eth_sendRawTransaction" {
            // Additional privacy validation: check transaction format and privacy indicators
            if let Some(params) = request.get("params").and_then(|p| p.as_array()) {
                if let Some(tx_data) = params.first().and_then(|t| t.as_str()) {
                    // Validate transaction format
                    if !tx_data.starts_with("0x") || tx_data.len() < 10 {
                        return Err(anyhow::anyhow!("Invalid transaction format"));
                    }
                    
                    // Check for privacy-enhancing transaction indicators
                    // Look for specific patterns that indicate privacy-conscious transactions
                    let _tx_lowercase = tx_data.to_lowercase(); // Used for future privacy checks
                    
                    // Check if transaction has privacy/ordering intent flags
                    // Look for special parameters that indicate MEV protection needs
                    if params.len() > 1 {
                        if let Some(options) = params.get(1) {
                            // Check for privacy/ordering options in the second parameter
                            if let Some(obj) = options.as_object() {
                                // Look for privacy-related fields like 'privacy', 'mevBlocker', 'flashbots', etc.
                                if obj.contains_key("privacy") || obj.contains_key("mevBlocker") || obj.contains_key("flashbots") {
                                    // This transaction has privacy/ordering intent
                                    // Ensure it's routed through appropriate privacy-preserving backends
                                    if self.mev_blocker_url.is_none() && !self.allow_public_mempool {
                                        return Err(anyhow::anyhow!("Privacy-intent transaction requires MEV protection configuration"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Check if transaction privacy is enforced
            if !self.allow_public_mempool {
                // Check if we have a MEV blocker URL to route transactions through
                if let Some(ref mev_url) = self.mev_blocker_url {
                    // Route through MEV blocker instead of public RPC
                    let response = self
                        .client
                        .post(mev_url)
                        .header("Content-Type", "application/json")
                        .json(&request)
                        .send()
                        .await
                        .map_err(|_| anyhow::anyhow!("Failed to send transaction to MEV blocker"))?;
                
                    // Check if the response status is successful
                    if !response.status().is_success() {
                        return Err(anyhow::anyhow!("MEV blocker returned error status"));
                    }
                
                    // Get response body
                    let response_text = response.text().await
                        .map_err(|_| anyhow::anyhow!("Failed to read response from MEV blocker"))?;
                        
                    // Parse the response to ensure it's valid JSON-RPC
                    let _response_value: Value = serde_json::from_str(&response_text)
                        .map_err(|_| anyhow::anyhow!("Invalid JSON-RPC response from MEV blocker"))?;
                        
                    return Ok(response_text.as_bytes().to_vec());
                } else {
                    // If no MEV blocker is configured but public mempool is not allowed,
                    // reject the transaction
                    return Err(anyhow::anyhow!("Transaction privacy guard: No private relay configured for transaction submission"));
                }
            } else {
                // For additional privacy, log if a transaction is being sent to public RPC when privacy is expected
                // In a production system, this could be an alert
                eprintln!("⚠️  Transaction being sent to public RPC - privacy may be compromised");
            }
        }
        
        // Forward other methods to the configured RPC provider
        let response = self
            .client
            .post(&self.provider_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send request to RPC provider"))?;
    
        // Check if the response status is successful
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("RPC provider returned error status"));
        }
    
        // Get response body
        let response_text = response.text().await
            .map_err(|_| anyhow::anyhow!("Failed to read response from RPC provider"))?;
            
        // Parse the response to ensure it's valid JSON-RPC
        let _response_value: Value = serde_json::from_str(&response_text)
            .map_err(|_| anyhow::anyhow!("Invalid JSON-RPC response from provider"))?;
            
        Ok(response_text.as_bytes().to_vec())
    }
}
