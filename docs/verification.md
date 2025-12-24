# Security Verification Guide

This guide helps you verify that Penum Private RPC is correctly protecting your privacy.

## Verification Checklist

Use this checklist to confirm all privacy guarantees are working:

- [ ] Packet size invariant (all packets exactly 1024 bytes)
- [ ] IP privacy (RPC provider sees only gateway IP)
- [ ] No logging of sensitive data
- [ ] Ephemeral key lifecycle
- [ ] Encryption working end-to-end
- [ ] Fail-silent behavior
- [ ] JSON extraction working for both requests and responses
- [ ] Multi-user scalability (traffic analysis resistance)
- [ ] Performance under load

## 1. Packet Size Verification

### Goal

Confirm ALL network packets are exactly 1024 bytes.

### Method: Wireshark Capture

#### Setup Wireshark

1. Install [Wireshark](https://www.wireshark.org/download.html)
2. Start capture on loopback interface (`Adapter for loopback traffic capture`)
3. Apply filter: `tcp.port == 9003`

#### Capture Traffic

```bash
# Terminal 1: Start gateway
cd penum-rpc-gateway
cargo run --release

# Terminal 2: Start client
cd penum-rpc-client
cargo run --release

# Terminal 3: Send test request
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

#### Verify Results

In Wireshark, find TCP packets containing application data:

1. Look for packets with `Len: 1024` in the "Length" column
2. Click packet → Expand "TCP" → Check "TCP Segment Len: 1024"
3. **All data packets should be exactly 1024 bytes**

**Expected**:

```
Frame 1: 1024 bytes on wire
Frame 2: 1024 bytes on wire
```

**Failure**: If you see packets of varying sizes, the implementation is broken.

### Method: tcpdump (Command-Line)

```bash
# Capture traffic
tcpdump -i lo -w penum-capture.pcap tcp port 9003

# Analyze capture
tcpdump -r penum-capture.pcap -n | grep "length 1024"
```

All data packets should show `length 1024`.

## 2. IP Privacy Verification

### Goal

Confirm RPC provider sees only the gateway IP, not your client IP.

### Method: RPC Provider Dashboard

#### Using Alchemy

1. Configure gateway with Alchemy API key
2. Send test request through Penum:
   ```bash
   curl -X POST http://127.0.0.1:8545 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
   ```
3. Go to [Alchemy Dashboard](https://dashboard.alchemy.com/)
4. Click "Requests" → View recent logs
5. Check the "IP Address" column

**Expected**: IP address shown is your **gateway's public IP**, not your local machine's IP.

**Failure**: If you see your personal/home IP, traffic is leaking.

#### Using Infura

1. Configure gateway with Infura API key
2. Send test request
3. Go to [Infura Dashboard](https://infura.io/dashboard)
4. View request logs
5. Check source IP

Same verification as Alchemy.

### Method: Direct vs Penum Comparison

**Direct request** (WITHOUT Penum):

```bash
curl https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

Check Alchemy dashboard → Note your IP address (e.g., `203.0.113.5`)

**Penum request**:

```bash
curl http://127.0.0.1:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

Check Alchemy dashboard → IP should be **different** (gateway IP)

## 3. No Logging Verification

### Goal

Confirm no sensitive data is logged.

### Method: Log Inspection

#### Start Components with Output Capture

```bash
# Gateway
cd penum-rpc-gateway
cargo run --release 2>&1 | tee gateway.log

# Client
cd penum-rpc-client
cargo run --release 2>&1 | tee client.log
```

#### Send Test Request with Identifiable Data

```bash
# Use a specific wallet address
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb","latest"],"id":1}'
```

#### Search Logs for Sensitive Data

```bash
# Search for wallet address
grep -i "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" gateway.log
grep -i "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb" client.log

# Search for JSON-RPC method with params
grep -i "eth_getBalance" gateway.log
grep -i "eth_getBalance" client.log
```

**Expected**: No matches (or only in generic startup messages)

**Failure**: If wallet addresses or method parameters appear in logs, logging is NOT properly disabled.

### Allowed Logging

✅ **Acceptable**:

- "Penum Gateway listening on 127.0.0.1:9003"
- "Connection established"
- "Connection closed"

❌ **NOT Acceptable**:

- Wallet addresses (e.g., "0x742d35...")
- RPC method parameters
- Packet contents
- Session keys

## 4. Ephemeral Key Lifecycle

### Goal

Confirm new keys are generated for every connection.

### Method: Code Inspection

Check `crypto.rs` in both client and gateway:

```rust
// Find EphemeralKeys::generate() calls
// Should be called ONCE per connection, never reused
```

Look for key storage:

- ✅ Good: `let keys = EphemeralKeys::generate();` inside connection handler
- ❌ Bad: `static KEYS: ...` (global/reused keys)

### Method: Memory Inspection (Advanced)

Use a memory profiler to confirm keys are dropped:

```bash
# Run with Valgrind (Linux)
valgrind --leak-check=full cargo run --release
```

Keys should be deallocated when connection closes.

## 5. Encryption End-to-End Test

### Goal

Confirm packets are encrypted (no plaintext JSON-RPC on wire).

### Method: Wireshark Content Inspection

1. Capture traffic as in Section 1
2. Click on a 1024-byte packet
3. Expand "TCP" → "TCP Segment Data"
4. Look at the hex dump

**Expected**: Random-looking bytes, no recognizable patterns

**Examples of what you should NOT see**:

- `{"jsonrpc":"2.0"` (plaintext JSON)
- `eth_blockNumber` (method name)
- ASCII strings in payload

**Example of correct encrypted data**:

```
0000: a7 3f 9e 2d 10 f4 88 c3 d1 05 3b 7a 9f 21 04 6e
0010: 3c e8 92 73 aa 45 67 8d 11 f0 9a 2b 5e 84 7f 1a
...
```

### Method: String Search

```bash
# Capture to file
tcpdump -i lo -w capture.pcap tcp port 9003

# Search for plaintext strings
strings capture.pcap | grep -E "(jsonrpc|eth_)"
```

**Expected**: No matches (or only in HTTP layer to localhost:8545)

**Failure**: If you find JSON-RPC strings in gateway traffic, encryption is broken.

## 6. Fail-Silent Behavior

### Goal

Confirm connections fail silently without revealing error details.

### Method: Introduce Errors

#### Test 1: Invalid Packet

Send malformed data to gateway:

```bash
echo "INVALID_DATA" | nc 127.0.0.1 9003
```

**Expected**: Connection closes immediately, no error message sent back

#### Test 2: Wrong Key

Modify client to use incorrect key derivation:

```rust
// In crypto.rs, change salt
let hk = Hkdf::<Sha256>::new(Some(b"WRONG_SALT"), secret.as_bytes());
```

**Expected**: Gateway decryption fails, connection closes silently

#### Test 3: Gateway Unavailable

Stop gateway, send RPC request:

```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

**Expected**: Generic JSON-RPC error, no detailed error message:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Internal error"
  },
  "id": 1
}
```

**Failure**: If specific error details are returned (e.g., "Decryption failed at byte 123"), fail-silent is broken.

## 7. JSON Extraction Verification

### Goal

Confirm that JSON extraction works correctly for both requests and responses.

### Method: Debug Output Verification

#### Check Gateway Request Extraction

1. Start gateway with debug output
2. Send test request: `curl -X POST http://127.0.0.1:8545 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'`
3. Verify gateway shows: `Extracted JSON request: {"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}`

#### Check Gateway Response Handling

1. Verify gateway shows: `Gateway received response from RPC provider: {"jsonrpc":"2.0","id":1,"result":"0x..."}`

#### Check Client Response Extraction

1. Verify the client successfully receives and processes the response
2. Check that the correct JSON is returned to the local RPC endpoint

## 8. UI Privacy Verification

### Goal

Confirm UI displays NO sensitive data.

### Method: UI Inspection

1. Open `http://127.0.0.1:8546`
2. Send several transactions through MetaMask
3. Inspect UI content

**Should Display**:

- ✅ "Penum RPC Running"
- ✅ RPC endpoint URL
- ✅ Connection health indicator

**Should NOT Display**:

- ❌ Wallet addresses
- ❌ Transaction hashes
- ❌ RPC method names with parameters
- ❌ Request/response contents

### Method: HTML Source Inspection

View page source (`Ctrl+U`):

```html
<!-- Search for sensitive patterns -->
```

No wallet addresses or transaction data should be in the HTML.

## 9. Scaling and Performance Verification

### Goal

Verify the system can handle multiple users and maintain privacy.

### Method: Load Testing

#### Concurrent Connections Test

```bash
# Send multiple requests simultaneously
for i in {1..10}; do
  curl -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":'"$i"'}' &
done
wait
```

**Expected**: All requests complete successfully with correct responses

#### Traffic Analysis Resistance

1. Monitor packet timing patterns
2. Verify that request patterns cannot be correlated between users
3. Confirm that packet sizes remain constant under load

### Method: Performance Under Load

#### Latency Test

```bash
# Test average response time
time for i in {1..100}; do
  curl -s -X POST http://127.0.0.1:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' > /dev/null
done
```

**Expected**: Response times remain reasonable under load (typically <500ms per request)

## 10. MetaMask Integration Test

### End-to-End Workflow

1. Configure MetaMask to use `http://127.0.0.1:8545`
2. Send test transaction on Sepolia testnet
3. Wait for confirmation
4. Verify on Etherscan

**Check**:

- ✅ Transaction confirms successfully
- ✅ MetaMask shows correct balance
- ✅ No errors in MetaMask console

## Common Failures

### ❌ Packet Size Violation

**Symptom**: Wireshark shows varying packet sizes

**Cause**: Padding logic broken

**Fix**: Ensure all packets are exactly 1024 bytes

### ❌ IP Leakage

**Symptom**: RPC provider logs show your personal IP

**Cause**: Direct connection to provider (bypassing gateway)

**Fix**: Verify client connects to gateway, not directly to Alchemy/Infura

### ❌ Plaintext Leakage

**Symptom**: Wireshark shows JSON-RPC strings

**Cause**: Encryption not applied

**Fix**: Check `encrypt_in_place` is called before sending

### ❌ Verbose Logging

**Symptom**: Wallet addresses in logs

**Cause**: Debug logging not removed

**Fix**: Remove all `println!` with sensitive data

### ❌ JSON Extraction Failure

**Symptom**: "Invalid UTF-8 in JSON request" or "Invalid UTF-8 response from gateway"

**Cause**: Improper JSON extraction from padded packets

**Fix**: Verify enhanced JSON extraction logic is working in both gateway and client

## Automated Verification Script

```bash
#!/bin/bash
# verify.sh - Automated privacy verification

echo "🔍 Penum RPC Privacy Verification"
echo ""

# 1. Check packet sizes
echo "[1/5] Checking packet sizes..."
# TODO: Add tcpdump analysis

# 2. Check logs for sensitive data
echo "[2/5] Checking logs for wallet addresses..."
if grep -r "0x[a-fA-F0-9]\{40\}" *.log 2>/dev/null; then
    echo "❌ FAIL: Wallet addresses found in logs"
else
    echo "✅ PASS: No wallet addresses in logs"
fi

# 3. Check for plaintext JSON-RPC
echo "[3/5] Checking for plaintext JSON-RPC..."
# TODO: Add packet capture analysis

# 4. Check UI privacy
echo "[4/5] Checking UI..."
if curl -s http://127.0.0.1:8546 | grep -E "0x[a-fA-F0-9]{40}"; then
    echo "❌ FAIL: UI displays wallet addresses"
else
    echo "✅ PASS: UI privacy OK"
fi

# 5. Check JSON extraction
echo "[5/5] Checking JSON extraction..."
if curl -s -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | grep -q "error"; then
  echo "❌ FAIL: JSON extraction failed"
else
  echo "✅ PASS: JSON extraction working"
fi

echo ""
echo "Verification complete"
```

## Reporting Issues

If verification fails:

1. Document exact steps to reproduce
2. Include relevant logs (with sensitive data redacted)
3. Note which verification test failed
4. Open GitHub issue with details

---

**Next Steps**: After verification passes, you can confidently use Penum RPC for privacy-preserving Ethereum access.

**Related**: [Architecture Documentation](architecture.md) | [MetaMask Setup](metamask-setup.md)