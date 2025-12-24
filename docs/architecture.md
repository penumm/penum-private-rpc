# Penum RPC Architecture

This document provides a deep-dive into the technical architecture of Penum Private RPC.

## System Overview

```
┌─────────────┐      ┌──────────────────┐      ┌──────────────────┐      ┌─────────────┐
│  MetaMask   │─────▶│ penum-rpc-client │─────▶│ penum-rpc-gateway│─────▶│ RPC Provider│
│             │      │  (localhost:8545)│      │   (encrypted)    │      │   (Alchemy) │
└─────────────┘      └──────────────────┘      └──────────────────┘      └─────────────┘
                             │                         │
                             ▼                         ▼
                    ┌──────────────────┐      ┌─────────────────┐
                    │   UI Server      │      │  RPC Forwarder  │
                    │ (localhost:8546) │      │   (HTTPS)       │
                    └──────────────────┘      └─────────────────┘
```

## Component Details

### 1. penum-rpc-client

**Purpose**: Local HTTP server that acts as an Ethereum RPC endpoint for MetaMask.

**Key Modules**:

#### `rpc_server.rs`

- Listens on `127.0.0.1:8545`
- Accepts JSON-RPC 2.0 requests via HTTP POST
- Validates supported methods:
  - `eth_call`
  - `eth_getBalance`
  - `eth_blockNumber`
  - `eth_sendRawTransaction`
  - `eth_getTransactionReceipt`
- Returns JSON-RPC errors for unsupported methods

#### `penum_client.rs`

- Wraps JSON-RPC request into Penum packet
- Performs encryption handshake with gateway
- Encrypts request with ChaCha20-Poly1305
- Sends 1024-byte fixed packet
- Receives and decrypts response

#### `crypto.rs`

- X25519 ephemeral key generation
- Diffie-Hellman key exchange
- HKDF session key derivation (salt: `"penum-v1"`)
- ChaCha20-Poly1305 AEAD encryption/decryption

#### `packet.rs`

- Fixed 1024-byte packet structure
- Random padding generation (cryptographically secure)

#### `ui.rs`

- Simple web UI on `127.0.0.1:8546`
- Shows RPC endpoint URL
- Shows connection status
- **NO** wallet addresses
- **NO** transaction details

### 2. penum-rpc-gateway

**Purpose**: Final hop that decrypts Penum packets and forwards to real RPC provider.

**Key Modules**:

#### `gateway.rs`

- TCP listener for incoming Penum connections
- Ephemeral key exchange per connection
- Decrypts incoming request packet
- Extracts JSON-RPC from padded payload
- Re-encrypts response packet
- **Fail-silent**: Drops connections on errors without logging packet contents

#### `rpc_forwarder.rs`

- HTTPS client using `reqwest`
- Forwards JSON-RPC to configured provider (Alchemy/Infura)
- Receives JSON-RPC response
- Returns response bytes

#### `crypto.rs`

- Same crypto primitives as client
- Session key derived using matching HKDF parameters

## Data Flow

### Request Flow

```
1. MetaMask sends JSON-RPC request to http://127.0.0.1:8545
   ↓
2. rpc_server.rs receives request, validates method
   ↓
3. penum_client.rs wraps request:
   - Create random 1024-byte buffer
   - Place JSON at specific offset
   - Connect to gateway
   - Perform X25519 key exchange
   - Encrypt with session key
   - Send encrypted packet
   ↓
4. gateway.rs receives packet:
   - Perform key exchange
   - Decrypt packet
   - Extract JSON-RPC (skip padding)
   ↓
5. rpc_forwarder.rs:
   - Forward JSON-RPC to Alchemy/Infura via HTTPS
   - Receive JSON response
   ↓
6. gateway.rs wraps response:
   - Create random 1024-byte buffer
   - Place JSON response
   - Encrypt with SAME session key
   - Send back to client
   ↓
7. penum_client.rs:
   - Decrypt response
   - Extract JSON-RPC response
   ↓
8. rpc_server.rs returns response to MetaMask
```

### Response Flow

Responses follow the same path in reverse, using the **same session key** established during the handshake.

## Packet Structure

### Encrypted Packet Format

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

### Payload Content (after decryption)

```
┌────────────────────────────────┐
│  Random Padding                │ ← Cryptographically secure random
├────────────────────────────────┤
│  JSON-RPC Request/Response     │ ← Actual data
├────────────────────────────────┤
│  Random Padding                │ ← More random data
└────────────────────────────────┘
```

The JSON-RPC data is placed at a variable offset within the payload, surrounded by random padding.

## Cryptographic Protocol

### Handshake Sequence

```
Client                           Gateway
  │                                 │
  ├──── Client Public Key (32B) ───▶│
  │                                 │
  │◀─── Server Public Key (32B) ────┤
  │                                 │
 [Derive Session Key]          [Derive Session Key]
  │                                 │
  ├──── Encrypted Packet (1024B) ──▶│
  │                                 │
  │◀─── Encrypted Response (1024B) ─┤
  │                                 │
[Drop all keys]                [Drop all keys]
  │                                 │
  └─────────────────────────────────┘
```

### Key Derivation

```rust
// Both client and gateway perform:
shared_secret = X25519(my_private, peer_public)
session_key = HKDF-SHA256(
    ikm = shared_secret,
    salt = "penum-v1",
    info = "",
    output_len = 32
)
```

### Encryption

```rust
// ChaCha20-Poly1305
nonce = [0u8; 12]  // All zeros (acceptable for ephemeral keys)
aad = header[0..32]
ciphertext, tag = encrypt(session_key, nonce, aad, plaintext)
```

## Security Properties

### Privacy Guarantees

| Property                    | Guaranteed | Why                                  |
| --------------------------- | ---------- | ------------------------------------ |
| IP Privacy                  | ✅ Yes     | RPC provider only sees gateway IP    |
| Wallet Unlinkability        | ✅ Yes     | Each request uses new session key    |
| Traffic Analysis Resistance | ✅ Yes     | Fixed 1024-byte packets              |
| Request Unlinkability       | ✅ Yes     | No persistent state between requests |

### What's NOT Guaranteed

| Property                            | Guaranteed | Why                                |
| ----------------------------------- | ---------- | ---------------------------------- |
| Full Anonymity                      | ❌ No      | On-chain data is public            |
| End-to-End Timing Attack Resistance | ❌ No      | Advanced adversaries can correlate |
| Metadata Privacy                    | ⚠️ Partial | RPC method visible to gateway      |

## Threat Model

### Assumptions

**Trusted**:

- Your local machine running `penum-rpc-client`
- The `penum-rpc-gateway` operator

**Untrusted**:

- RPC provider (Alchemy, Infura)
- Network observers between client and gateway
- Network observers between gateway and RPC provider

### Attack Scenarios

#### 1. Malicious RPC Provider

**Attack**: RPC provider tries to learn client IP

**Defense**:

- ✅ RPC provider only sees gateway IP
- ✅ Session keys prevent request correlation

#### 2. Network Observer (Client ↔ Gateway)

**Attack**: Passive observer tries to identify wallet activity

**Defense**:

- ✅ Fixed packet sizes prevent traffic analysis
- ✅ Encryption prevents content inspection
- ⚠️ Timing analysis still possible

#### 3. Compromised Gateway

**Attack**: Gateway tries to learn wallet addresses

**Defense**:

- ⚠️ Gateway sees plaintext JSON-RPC
- ✅ Gateway does NOT see client IP (in multi-hop setup)
- ⚠️ Trust gateway operator

## Performance Characteristics

### Latency

```
Total Latency = Network_Latency + Crypto_Overhead + Provider_Latency

Network_Latency: ~20-50ms (client ↔ gateway)
Crypto_Overhead: ~1-5ms (key exchange + encryption)
Provider_Latency: ~50-200ms (gateway ↔ RPC provider)

Total: ~70-250ms per request
```

### Throughput

- **Theoretical Max**: ~5000 req/s (limited by encryption)
- **Practical**: ~1000 req/s (with connection overhead)
- **MetaMask Usage**: ~1-10 req/s (typical user)

### Resource Usage

**Client**:

- CPU: Negligible (<1% for typical usage)
- Memory: ~10 MB
- Network: ~1024 bytes per request/response

**Gateway**:

- CPU: ~2-5% per 100 req/s
- Memory: Minimal (stateless)
- Network: ~2048 bytes per request/response

## Future Enhancements

### Planned Features

1. **Multi-Hop Routing**: Implement full 3-hop Penum path (Entry → Middle → Gateway)
2. **Load Balancing**: Support multiple gateway operators
3. **Method Expansion**: Support all Ethereum JSON-RPC methods
4. **WebSocket Support**: For subscription-based methods (`eth_subscribe`)

### Research Directions

1. **Differential Privacy**: Add noise to request patterns
2. **Batch Requests**: Optimize multiple concurrent requests
3. **Caching**: Cache common read-only queries (careful with privacy)

## Scaling Architecture

### Infrastructure Scaling

#### Relay Network Expansion

- Deploy geographically distributed relay nodes to reduce latency
- Implement relay discovery/selection algorithms to choose optimal paths
- Create incentive mechanisms (tokens, fees) to encourage relay operation
- Use cloud providers (AWS, GCP, Azure) for reliable infrastructure

#### Gateway Scaling

- Run multiple gateway instances behind load balancers
- Implement auto-scaling based on request volume
- Use containerization (Docker/Kubernetes) for easy deployment
- Consider edge computing for global distribution

#### Performance Optimization

- Implement connection pooling to reduce handshake overhead
- Use more efficient encryption algorithms where possible
- Add caching layers for frequently requested data
- Optimize packet sizes and routing algorithms

### User Adoption Strategy

#### User Experience

- Create one-click installation scripts
- Build browser extensions that automatically configure Metamask
- Develop mobile apps for easy setup
- Provide hosted solutions for non-technical users

#### Integration

- Create SDKs for popular languages (JavaScript, Python, Go)
- Build Metamask/WalletConnect-compatible integrations
- Offer API gateways that convert standard RPC to Penum protocol
- Partner with existing privacy tools and wallets

#### Education

- Create simple setup guides with visual instructions
- Build a community around the project
- Provide documentation and tutorials
- Demonstrate clear privacy benefits

### Privacy Protection at Scale

#### Traffic Analysis Countermeasures

- Implement constant-rate padding (not just fixed-size packets)
- Add random timing delays to obscure request patterns
- Use mix networks or DC-nets for additional anonymity
- Implement cover traffic (fake requests) to obscure real activity

#### Network-Level Privacy

- Ensure relay operators can't correlate traffic between hops
- Implement proper forward secrecy
- Use onion routing principles with multiple encryption layers
- Consider integration with Tor or other anonymity networks

#### Protocol Improvements

- Implement shared relays used by many users simultaneously
- Use zero-knowledge proofs where possible to reduce information leakage
- Implement batch processing to obscure individual request timing
- Add decoy requests to confuse traffic analysis

### Sustainability Model

#### Funding

- Freemium model: Basic privacy free, premium features paid
- Transaction fee model: Small percentage of DeFi transactions
- Corporate subscriptions for exchanges/businesses
- Grant funding from privacy-focused organizations

#### Governance

- Decentralized autonomous organization (DAO) for protocol governance
- Community-driven relay operation
- Open-source development with multiple contributors
- Regular security audits and transparency reports

### Critical Success Factors

#### Maintain Privacy While Scaling

- More users should increase anonymity (network effect)
- Never compromise privacy for performance
- Regular privacy analysis and improvements
- Resistance to centralization pressures

#### Security

- Regular security audits as the system grows
- Bug bounty programs
- Formal verification of critical components
- Defense against DoS and other attacks

#### Compliance

- Ensure legal compliance in different jurisdictions
- Build in compliance mechanisms for regulated entities
- Maintain good relationships with Ethereum ecosystem

---

**Related Documents**:

- [Penum Protocol Specification](../../penum-spec/)
- [MetaMask Setup Guide](metamask-setup.md)
- [Security Verification](verification.md)
- [Scaling and Deployment Guide](scaling-deployment.md)