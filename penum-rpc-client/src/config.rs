use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfig {
    pub addr: SocketAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcClientConfig {
    pub entry_relay: SocketAddr,
    pub middle_relay: SocketAddr,
    pub gateway: SocketAddr,
    pub rpc_port: u16,
    pub ui_port: u16,
    pub protocol_version: u8,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            entry_relay: "127.0.0.1:9001".parse().unwrap(),
            middle_relay: "127.0.0.1:9002".parse().unwrap(),
            gateway: "127.0.0.1:9003".parse().unwrap(),
            rpc_port: 8545,
            ui_port: 8546,
            protocol_version: 1,
        }
    }
}

impl RpcClientConfig {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config: RpcClientConfig = serde_json::from_str(json)?;
        Ok(config)
    }
}
