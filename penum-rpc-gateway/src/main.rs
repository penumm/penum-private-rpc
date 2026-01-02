mod config;
mod crypto;
mod gateway;
mod relay;
mod rpc_forwarder;

use config::GatewayConfig;
use rpc_forwarder::RpcForwarder;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try to load from config.json, fall back to default if not found
    let config = match fs::read_to_string("config.json") {
        Ok(config_str) => GatewayConfig::from_json(&config_str)?,
        Err(_) => {
            println!("⚠️  config.json not found, using default configuration");
            GatewayConfig::default()
        }
    };

    println!("🚀 Starting Penum RPC Gateway");
    println!("   Listen:       {}:{}", config.listen_addr, config.listen_port);
    println!("   RPC Provider: <configured>"); // Don't log actual provider URL for privacy
    println!();

    // Check if we're running as a relay or as a gateway
    if let Some(ref next_hop_str) = std::env::var("PENUM_NEXT_HOP").ok() {
        // Running as a relay - forward to next hop
        let next_hop: std::net::SocketAddr = next_hop_str.parse()
            .map_err(|_| anyhow::anyhow!("Invalid next hop address: {}", next_hop_str))?;
        relay::start_relay(&config.listen_addr, config.listen_port, next_hop).await?;
    } else {
        // Running as a gateway - process RPC requests
        let rpc_forwarder = RpcForwarder::new(config.rpc_provider_url, config.allow_public_mempool, config.mev_blocker_url);
        gateway::start_gateway(&config.listen_addr, config.listen_port, rpc_forwarder, config.allow_public_mempool).await?;
    }

    Ok(())
}
