use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub listen_port: u16,
    pub rpc_provider_url: String,
    pub allow_public_mempool: bool,  // Privacy guard setting
    pub mev_blocker_url: Option<String>, // MEV safety hook
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9003,
            rpc_provider_url: "https://cloudflare-eth.com".to_string(),
            allow_public_mempool: false,  // Default to privacy-safe
            mev_blocker_url: None,
        }
    }
}

impl GatewayConfig {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

