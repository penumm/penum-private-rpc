# Penum Private RPC - Build and Run Guide

## Prerequisites

- Rust toolchain (1.70+)
- Windows with MSVC build tools OR Linux/Mac
- Alchemy or Infura API key

## Build Instructions

### Option 1: Build All (Recommended after fixing Windows file locking)

```bash
cd penum-private-rpc
cargo build --release
```

### Option 2: Build Individual Components

If workspace build fails due to Windows file locking:

```bash
# Build gateway
cd penum-rpc-gateway
cargo build --release

# Build client
cd ../penum-rpc-client
cargo build --release
```

### Troubleshooting Build Issues

#### Windows File Locking Error

**Symptom**: `The process cannot access the file because it is being used by another process`

**Solutions**:

1. Close all running cargo/rust processes
2. Temporarily disable antivirus
3. Run: `cargo clean` then rebuild
4. Build components separately (Option 2 above)

#### Missing MSVC Tools

**Symptom**: `linker 'link.exe' not found`

**Solution**: Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ development tools

## Configuration

### Gateway Configuration

1. Create `penum-rpc-gateway/config.json`:

```json
{
  "listen_addr": "127.0.0.1",
  "listen_port": 9003,
  "rpc_provider_url": "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY_HERE"
}
```

2. Replace `YOUR_API_KEY_HERE` with your actual Alchemy/Infura API key

### Client Configuration

1. Create `penum-rpc-client/config.json`:

```json
{
  "entry_relay": "127.0.0.1:9001",
  "middle_relay": "127.0.0.1:9002",
  "gateway": "127.0.0.1:9003",
  "rpc_port": 8545,
  "ui_port": 8546,
  "protocol_version": 1
}
```

## Running the System

### Terminal 1: Start Gateway

```bash
cd penum-rpc-gateway
cargo run --release
```

Expected output:

```
🚀 Starting Penum RPC Gateway
   Listen:       127.0.0.1:9003
   RPC Provider: https://eth-mainnet..g.alchemy.com/v2/...

🌐 Penum Gateway listening on 127.0.0.1:9003
```

### Terminal 2: Start Client

```bash
cd penum-rpc-client
cargo run --release
```

Expected output:

```
🚀 Starting Penum RPC Client
   Entry Relay:  127.0.0.1:9001
   Middle Relay: 127.0.0.1:9002
   Gateway:      127.0.0.1:9003

🔒 Penum RPC Server listening on http://127.0.0.1:8545
📋 Supported methods: eth_call, eth_getBalance, eth_blockNumber...
🎨 Penum UI available at http://127.0.0.1:8546
```

## Testing

### 1. View UI

Open browser: `http://127.0.0.1:8546`

You should see the Penum RPC dashboard.

### 2. Test with curl

```bash
# Get latest block number
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

Expected response:

```json
{ "jsonrpc": "2.0", "result": "0x...", "id": 1 }
```

### 3. Test with MetaMask

See [docs/metamask-setup.md](docs/metamask-setup.md) for full instructions.

## Development Build (with debug info)

```bash
cargo build
cargo run
```

## Verification

See [docs/verification.md](docs/verification.md) for security verification steps.

## Common Issues

### "Connection refused" error

**Cause**: Gateway not running

**Solution**: Start gateway first (Terminal 1), then client (Terminal 2)

### "Invalid API key" error

**Cause**: Wrong or missing Alchemy/Infura key

**Solution**: Check `config.json` and verify API key is valid

### Port already in use

**Cause**: Another process using 8545 or 9003

**Solution**:

- Find process: `netstat -an | findstr 8545`
- Kill process or change port in config

## Next Steps

- Read [Architecture Documentation](docs/architecture.md)
- Review [Security Verification Guide](docs/verification.md)
- Configure MetaMask following [metamask-setup.md](docs/metamask-setup.md)
