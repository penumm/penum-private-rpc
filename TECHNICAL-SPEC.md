# Penum Private RPC - Technical Specification

## Overview
Penum Private RPC is a privacy-preserving Ethereum JSON-RPC gateway that prevents blockchain RPC providers from learning client IP addresses, geographic locations, and direct wallet-to-network linkages. All traffic is routed through encrypted onion network using fixed-size packets.

## Architecture

### Components
1. **penum-rpc-client**: Local RPC endpoint for MetaMask
2. **penum-rpc-gateway**: RPC provider interface
3. **Penum Protocol**: Encryption and routing layer

### Data Flow
```
MetaMask → penum-rpc-client → penum-rpc-gateway → RPC Provider
           (localhost:8545)     (encrypted packets)    (Alchemy/Infura)
```

## Cryptographic Protocol

### Key Exchange
- Algorithm: X25519 (ECDH over Curve25519)
- Purpose: Ephemeral key exchange per connection
- Implementation: `x25519-dalek` crate

### Encryption
- Algorithm: ChaCha20-Poly1305 (AEAD)
- Purpose: Encrypt all communications
- Implementation: `chacha20poly1305` crate

### Key Derivation
- Algorithm: HKDF-SHA256
- Salt: "penum-v1"
- Info: ""
- Output: 32-byte session key

### Packet Structure
```
┌────────────────────────────────┐
│      Header (32 bytes)         │ ← AAD for AEAD
├────────────────────────────────┤
│                                │
│    Encrypted Payload           │
│    (976 bytes)                 │ ← Encrypted with ChaCha20-Poly1305
│                                │
├────────────────────────────────┤
│   AEAD Tag (16 bytes)          │ ← Authentication tag
└────────────────────────────────┘
Total: 1024 bytes (fixed)
```

## Security Properties

### Privacy Guarantees
- IP Privacy: RPC provider only sees gateway IP
- Wallet Unlinkability: Each request uses new session key
- Traffic Analysis Resistance: Fixed 1024-byte packets
- Request Unlinkability: No persistent state between requests

### Threat Model
- Trusted: Local client machine, gateway operator
- Untrusted: RPC providers, network observers

### Attack Resistance
- Malicious RPC Provider: Only sees gateway IP
- Network Observer: Fixed packet sizes prevent traffic analysis
- Timing Analysis: Possible but difficult with multiple users

## Implementation Details

### Client Configuration
```json
{
  "entry_relay": "127.0.0.1:9001",
  "middle_relay": "127.0.0.1:9002", 
  "gateway": "127.0.0.1:9003",
  "rpc_port": 8545,
  "ui_port": 8546
}
```

### Gateway Configuration
```json
{
  "listen_addr": "127.0.0.1",
  "listen_port": 9003,
  "rpc_provider_url": "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
}
```

### Supported Methods
- `eth_call`
- `eth_getBalance`
- `eth_blockNumber`
- `eth_sendRawTransaction`
- `eth_getTransactionReceipt`

## Performance Characteristics

### Latency
- Network: ~20-50ms (client ↔ gateway)
- Crypto: ~1-5ms (key exchange + encryption)
- Provider: ~50-200ms (gateway ↔ RPC provider)
- Total: ~70-250ms per request

### Throughput
- Theoretical Max: ~5000 req/s
- Practical: ~1000 req/s
- Typical User: ~1-10 req/s

### Resource Usage
- Client: ~10MB memory, <1% CPU
- Gateway: Minimal state, scalable processing

## Security Features

### Fixed-Size Packets
- All network traffic uses exactly 1024-byte packets
- Prevents traffic analysis based on packet sizes

### Ephemeral Keys
- New X25519 keypair generated for every connection
- Keys are never reused or stored

### Zero Logging
- No wallet addresses logged
- No transaction parameters logged
- No IP addresses stored
- Only connection-level errors logged (without packet contents)

### Fail-Silent
- On any error, connections close silently
- No error details sent back to client

## Future Enhancements

### Multi-Hop Routing
- Full 3-hop Penum path (Entry → Middle → Gateway)
- Enhanced privacy through multiple relays

### Performance Optimization
- Connection pooling to reduce handshake overhead
- More efficient encryption algorithms
- Caching for frequently requested data

### Protocol Improvements
- Shared relays used by many users simultaneously
- Batch processing to obscure individual request timing
- Decoy requests to confuse traffic analysis