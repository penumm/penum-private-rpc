use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub listen_port: u16,
    pub rpc_provider_url: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9003,
            rpc_provider_url: "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY".to_string(),
        }
    }
}

impl GatewayConfig {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config: GatewayConfig = serde_json::from_str(json)?;
        Ok(config)
    }
}
