use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use anyhow::Result;

pub struct Relay {
    next_hop: SocketAddr,
}

impl Relay {
    pub fn new(next_hop: SocketAddr) -> Self {
        Self { next_hop }
    }

    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        // Connect to the next hop in the relay chain
        let next_stream = TcpStream::connect(self.next_hop).await?;
        
        // Forward data bidirectionally between current connection and next hop
        let (stream_reader, stream_writer) = tokio::io::split(stream);
        let (next_reader, next_writer) = tokio::io::split(next_stream);
        
        // Forward data in both directions
        let forward_task = tokio::spawn(async move {
            let mut reader = stream_reader;
            let mut writer = next_writer;
            let mut buffer = [0u8; 1024]; // Fixed packet size
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        if let Err(_) = writer.write_all(&buffer[..n]).await {
                            break; // Forward failed
                        }
                        if let Err(_) = writer.flush().await {
                            break; // Flush failed
                        }
                    }
                    Err(_) => break, // Read failed
                }
            }
        });
        
        let backward_task = tokio::spawn(async move {
            let mut reader = next_reader;
            let mut writer = stream_writer;
            let mut buffer = [0u8; 1024]; // Fixed packet size
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        if let Err(_) = writer.write_all(&buffer[..n]).await {
                            break; // Forward failed
                        }
                        if let Err(_) = writer.flush().await {
                            break; // Flush failed
                        }
                    }
                    Err(_) => break, // Read failed
                }
            }
        });
        
        // Wait for either task to complete
        tokio::select! {
            _ = forward_task => {},
            _ = backward_task => {},
        };
        
        Ok(())
    }
}

pub async fn start_relay(listen_addr: &str, listen_port: u16, next_hop: SocketAddr) -> Result<()> {
    let relay = Relay::new(next_hop);
    let listener = TcpListener::bind(format!("{}:{}", listen_addr, listen_port)).await?;
    
    println!("🔗 Relay listening on {}:{} forwarding to {}", listen_addr, listen_port, next_hop);
    
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let relay_clone = relay.clone();
                tokio::spawn(async move {
                    let _ = relay_clone.handle_connection(stream).await;
                });
            }
            Err(_) => {
                continue; // Continue accepting connections on error
            }
        }
    }
}

// Clone implementation for Relay
impl Clone for Relay {
    fn clone(&self) -> Self {
        Self {
            next_hop: self.next_hop,
        }
    }
}