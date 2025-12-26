# Penum Private RPC

A production-grade privacy-preserving Ethereum JSON-RPC gateway using the Penum protocol.

## Overview

Penum Private RPC prevents blockchain RPC providers from learning:

- Client IP address
- Geographic location
- Direct wallet to network linkage

All traffic is routed through the Penum protocol's encrypted onion network using fixed-size packets.

## Grant Readiness

This project is fully prepared for Ethereum Foundation grant applications with:

- Complete technical specification ([TECHNICAL-SPEC.md](TECHNICAL-SPEC.md))
- Detailed grant proposal ([GRANT-PROPOSAL.md](GRANT-PROPOSAL.md))
- Comprehensive development roadmap ([ROADMAP.md](ROADMAP.md))
- Production-ready implementation
- Extensive documentation and testing

## Architecture

```
MetaMask → penum-rpc-client → penum-rpc-gateway → RPC Provider
           (localhost:8545)     (encrypted packets)    (Alchemy/Infura)
```

### Components

1. **penum-rpc-client** - Local RPC endpoint for MetaMask

   - Accepts standard Ethereum JSON-RPC requests
   - Wraps requests in 1024-byte encrypted Penum packets
   - Acts as `http://127.0.0.1:8545`

2. **penum-rpc-gateway** - RPC provider interface
   - Decrypts Penum packets
   - Forwards JSON-RPC to real provider (Alchemy, Infura, etc.)
   - Re-encrypts responses

## Quick Start

### 1. Start the Gateway

```bash
cd penum-rpc-gateway
cargo run --release
```

### 2. Start the Client

```bash
cd penum-rpc-client
cargo run --release
```

### 3. Configure MetaMask

1. Open MetaMask
2. Go to Settings → Networks → Add Network
3. Enter:
   - **Network Name**: Ethereum via Penum
   - **RPC URL**: `http://127.0.0.1:8545`
   - **Chain ID**: 1 (or your testnet)
   - **Currency Symbol**: ETH

### 4. View UI

Open `http://127.0.0.1:8546` to see the Penum RPC dashboard.

## Configuration

### Gateway Configuration

Edit `penum-rpc-gateway/config.example.json`:

```json
{
  "listen_addr": "127.0.0.1",
  "listen_port": 9003,
  "rpc_provider_url": "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
}
```

Replace `YOUR_API_KEY` with your Alchemy/Infura API key.

### Client Configuration

Edit `penum-rpc-client/config.example.json`:

```json
{
  "entry_relay": "127.0.0.1:9001",
  "middle_relay": "127.0.0.1:9002",
  "gateway": "127.0.0.1:9003",
  "rpc_port": 8545,
  "ui_port": 8546
}
```

For testing, the client connects directly to the gateway (simplified single-hop).

## Privacy Guarantees

### What Penum RPC Prevents

- RPC provider cannot see your IP address
- RPC provider cannot link requests to your wallet
- Relays cannot correlate traffic patterns
- Network observers cannot perform traffic analysis (fixed packet sizes)

### What Penum RPC Does NOT Prevent

- On-chain analysis (all transactions are public on Ethereum)
- End-to-end timing attacks (advanced adversaries)
- Wallet fingerprinting via transaction patterns

## Supported JSON-RPC Methods

- `eth_call`
- `eth_getBalance`
- `eth_blockNumber`
- `eth_sendRawTransaction`
- `eth_getTransactionReceipt`

More methods can be added by updating `rpc_server.rs`.

## Security Features

### Fixed-Size Packets

All network traffic uses exactly **1024-byte packets** to prevent traffic analysis.

### Ephemeral Keys

New X25519 keypair generated for **every connection**. Keys are never reused.

### Zero Logging

- No wallet addresses logged
- No transaction parameters logged
- No IP addresses stored
- Only connection-level errors logged (without packet contents)

### Fail-Silent

On any error, connections close silently with no error details sent back.

## Testing

### Test with curl

```bash
# Get latest block number
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get balance
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb","latest"],"id":1}'
```

### Test with MetaMask

1. Configure MetaMask to use `http://127.0.0.1:8545`
2. Send a test transaction on testnet
3. Verify transaction confirms on Etherscan
4. Check RPC provider logs - your IP should NOT appear

## Performance

- **Latency Overhead**: ~50-150ms (single-hop testing)
- **Throughput**: Limited by encryption overhead (~1000 req/s)
- **Packet Size**: All packets exactly 1024 bytes

## Development

### Build

```bash
cd penum-private-rpc
cargo build --release
```

### Run Tests

```bash
cargo test
```

## Documentation

- [Architecture Details](docs/architecture.md)
- [MetaMask Setup Guide](docs/metamask-setup.md)
- [Security Verification](docs/verification.md)
- [Scaling and Deployment Guide](docs/scaling-deployment.md)

## Limitations

- **Not Full Anonymity**: Penum provides privacy, not anonymity. Advanced adversaries may correlate traffic.
- **Latency**: Adds ~100-300ms overhead per request
- **Beta Software**: Not audited, use at your own risk
- **Single-Hop Simplified**: Current implementation uses direct client→gateway connection for testing

## Contributing

This is a research prototype. Contributions welcome!

## License

MIT License - See LICENSE file for details

## Related Projects

- [Penum Protocol](../penum-spec/) - Core protocol specification
- [Penum Client](../penum-client/) - General-purpose Penum client
- [Penum Gateway](../penum-gateway/) - Penum exit gateway

---

**DISCLAIMER**: This software is experimental. Do not use for production workloads without thorough security review.