# MetaMask Setup Guide

This guide walks you through configuring MetaMask to use Penum Private RPC.

## Prerequisites

- MetaMask browser extension installed
- `penum-rpc-client` and `penum-rpc-gateway` running

## Step-by-Step Instructions

### 1. Verify Penum RPC is Running

Before configuring MetaMask, ensure both components are running:

```bash
# Terminal 1: Start gateway
cd penum-rpc-gateway
cargo run --release

# Terminal 2: Start client
cd penum-rpc-client
cargo run --release
```

You should see:

```
🔒 Penum RPC Server listening on http://127.0.0.1:8545
🎨 Penum UI available at http://127.0.0.1:8546
```

### 2. Open MetaMask Settings

1. Click the MetaMask extension icon
2. Click your account icon (top-right)
3. Select **Settings**
4. Click **Networks** in the sidebar

### 3. Add Custom Network

Click **Add Network** (or **Add a network manually** on newer versions)

### 4. Enter Network Details

#### For Ethereum Mainnet via Penum:

- **Network Name**: `Ethereum via Penum`
- **New RPC URL**: `http://127.0.0.1:8545`
- **Chain ID**: `1`
- **Currency Symbol**: `ETH`
- **Block Explorer URL**: `https://etherscan.io` (optional)

#### For Sepolia Testnet via Penum:

- **Network Name**: `Sepolia via Penum`
- **New RPC URL**: `http://127.0.0.1:8545`
- **Chain ID**: `11155111`
- **Currency Symbol**: `SepoliaETH`
- **Block Explorer URL**: `https://sepolia.etherscan.io` (optional)

### 5. Save and Switch

1. Click **Save**
2. MetaMask will automatically switch to the new network
3. You should see "Ethereum via Penum" as the selected network

### 6. Verify Connection

Open the Penum UI at `http://127.0.0.1:8546` and verify:

- ✅ Status shows "Penum RPC Running"
- ✅ Connection health is "Connected"

### 7. Test Transaction (Testnet Recommended)

**⚠️ For your first test, use Sepolia testnet!**

1. Make sure you have test ETH (get from [Sepolia faucet](https://sepoliafaucet.com/))
2. Send a small test transaction to yourself
3. Wait for confirmation
4. Verify on [Sepolia Etherscan](https://sepolia.etherscan.io)

## Troubleshooting

### "Network request failed" error

**Solution**: Ensure `penum-rpc-client` is running:

```bash
cd penum-rpc-client
cargo run --release
```

### Transactions stuck pending

**Possible causes**:

1. Gateway cannot reach RPC provider
   - Check `penum-rpc-gateway` configuration
   - Verify your Alchemy/Infura API key is valid
2. Network congestion (use testnet for testing)

**Check logs**:

```bash
# In the terminal running penum-rpc-client
# Look for connection errors (packet contents are never logged)
```

### Wrong chain ID error

**Solution**: Double-check the Chain ID matches your intended network:

- Mainnet: `1`
- Sepolia: `11155111`
- Goerli: `5`

### Cannot connect to localhost

**Solution**:

1. Verify port 8545 is not already in use:
   ```bash
   netstat -an | findstr 8545
   ```
2. If another process is using 8545, either stop it or change `rpc_port` in config

## Privacy Verification

After setup, verify privacy is working:

### 1. Check RPC Provider Logs

If using Alchemy:

1. Go to [Alchemy Dashboard](https://dashboard.alchemy.com/)
2. View request logs
3. Verify IP address shown is the **gateway IP**, not your personal IP

### 2. Check UI

Visit `http://127.0.0.1:8546`:

- You should see connection health indicator
- NO wallet addresses should be displayed
- NO transaction details should be shown

### 3. Wireshark Verification (Advanced)

Capture traffic between client and gateway:

```bash
# All packets should be exactly 1024 bytes
# No plaintext JSON-RPC should be visible
```

## Switching Back to Default RPC

To stop using Penum:

1. Open MetaMask
2. Click the network dropdown
3. Select "Ethereum Mainnet" or your preferred default network

Your original MetaMask configuration is preserved.

## Best Practices

### ✅ Do:

- Use testnet (Sepolia) for initial testing
- Keep `penum-rpc-client` running while using MetaMask
- Verify connection health in UI before important transactions

### ❌ Don't:

- Use for high-value transactions without thorough testing
- Assume full anonymity (on-chain data is still public)
- Share your RPC provider API key

## Multi-Network Setup

You can add multiple networks through Penum:

1. Add each network as a separate MetaMask network
2. All use the same RPC URL: `http://127.0.0.1:8545`
3. Only the Chain ID differs

Example:

- Ethereum Mainnet (Chain ID: 1)
- Sepolia Testnet (Chain ID: 11155111)
- Arbitrum (Chain ID: 42161)

All traffic goes through Penum, regardless of which network you select.

## Need Help?

- Check `penum-rpc-client` terminal for connection errors
- Verify gateway configuration (`config.example.json`)
- Visit UI at `http://127.0.0.1:8546` for status

---

**Next Steps**: [Security Verification Guide](verification.md)
