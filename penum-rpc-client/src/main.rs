mod config;
mod crypto;
mod packet;
mod penum_client;
mod rpc_server;
mod ui;

use config::RpcClientConfig;
use penum_client::PenumRpcClient;
use std::sync::Arc;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try to load from config.json, fall back to default if not found
    let config = match fs::read_to_string("config.json") {
        Ok(config_str) => RpcClientConfig::from_json(&config_str)?,
        Err(_) => {
            println!("⚠️  config.json not found, using default configuration");
            RpcClientConfig::default()
        }
    };

    println!("🚀 Starting Penum RPC Client");
    println!("   Entry Relay:  {}", config.entry_relay);
    println!("   Middle Relay: {}", config.middle_relay);
    println!("   Gateway:      {}", config.gateway);
    println!();

    // Create Penum client
    let penum_client = Arc::new(PenumRpcClient::new(config.clone()));

    // Start RPC server and UI server concurrently
    let rpc_server = tokio::spawn(rpc_server::start_rpc_server(
        config.rpc_port,
        penum_client.clone(),
    ));

    let ui_server = tokio::spawn(ui::start_ui_server(config.ui_port, config.rpc_port));

    // Wait for both servers
    tokio::select! {
        _ = rpc_server => {},
        _ = ui_server => {},
    }

    Ok(())
}
