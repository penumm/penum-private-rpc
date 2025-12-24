mod config;
mod crypto;
mod gateway;
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
    println!("   RPC Provider: {}", config.rpc_provider_url);
    println!();

    let rpc_forwarder = RpcForwarder::new(config.rpc_provider_url);

    gateway::start_gateway(&config.listen_addr, config.listen_port, rpc_forwarder).await?;

    Ok(())
}
