# Penum Private RPC - Quick Start

## 🚀 Quick Start (3 Steps)

### 1. Build

```powershell
cd D:\penum\penum-private-rpc
.\build.ps1
```

**⚠️ Windows File Locking Issue?**

- Close Visual Studio / IDEs
- Temporarily disable antivirus
- Reboot if needed
- Try again

### 2. Configure

Edit `penum-rpc-gateway\config.json`:

```json
{
  "rpc_provider_url": "https://eth-mainnet.g.alchemy.com/v2/YOUR_ACTUAL_API_KEY"
}
```

Get API key: [Alchemy](https://www.alchemy.com/) or [Infura](https://www.infura.io/)

### 3. Run

```powershell
.\run.ps1
```

This starts:

- Gateway on `127.0.0.1:9003`
- Client on `127.0.0.1:8545` (RPC) and `127.0.0.1:8546` (UI)

## ✅ Test

```powershell
.\test.ps1
```

Expected output:

```
🧪 Testing Penum Private RPC...
 ✅ UI server running
 ✅ eth_blockNumber: Block #19234567
 ✅ eth_getBalance: 12.34 ETH
 🎉 All tests passed!
```

## 🦊 Configure MetaMask

1. Open MetaMask
2. Settings → Networks → Add Network
3. Enter:

   - **Network Name**: Ethereum via Penum
   - **RPC URL**: `http://127.0.0.1:8545`
   - **Chain ID**: `1` (Mainnet) or `11155111` (Sepolia)
   - **Currency**: ETH

4. Send test transaction

## 🔒 Privacy Verification

### Check Your IP is Hidden

1. Go to [Alchemy Dashboard](https://dashboard.alchemy.com/)
2. View request logs
3. **Verify**: IP shown is gateway IP, NOT your personal IP ✅

### Check Packet Encryption

1. Install [Wireshark](https://www.wireshark.org/)
2. Capture on loopback: filter `tcp.port == 9003`
3. Send RPC request (use test.ps1)
4. **Verify**:
   - All data packets exactly 1024 bytes ✅
   - No plaintext JSON visible ✅

## 📚 Full Documentation

- [Full README](README.md) - Complete guide
- [Architecture](docs/architecture.md) - Technical deep-dive
- [Build Guide](BUILD.md) - Troubleshooting
- [Verification](docs/verification.md) - Security verification
- [Walkthrough](walkthrough.md) - Implementation details

## ⚠️ Troubleshooting

### Build fails with "file in use"

**Solution**: Windows antivirus is locking files

1. Disable antivirus temporarily
2. Run `.\build.ps1` again

### "Connection refused" when testing

**Solution**: Services not running

1. Run `.\run.ps1`
2. Wait 5 seconds
3. Run `.\test.ps1`

### MetaMask shows "Invalid RPC"

**Solution**: Wrong config or services not running

1. Ensure `.\run.ps1` shows "Penum RPC Server listening"
2. Check RPC URL is exactly `http://127.0.0.1:8545`
3. Try curl test first (see README.md)

## 🎯 Success Criteria

Your setup is working if:

✅ `.\build.ps1` completes without errors  
✅ `.\test.ps1` shows all tests passing  
✅ MetaMask connects to `http://127.0.0.1:8545`  
✅ Transactions send successfully  
✅ Alchemy logs show gateway IP (not your IP)  
✅ Wireshark shows 1024-byte encrypted packets

## 📞 Need Help?

See full documentation in:

- `README.md` - Main documentation
- `BUILD.md` - Build troubleshooting
- `docs/` - Detailed guides
