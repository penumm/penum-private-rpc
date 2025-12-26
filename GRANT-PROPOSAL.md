# Penum Private RPC - Grant Proposal

## Project Abstract

Penum Private RPC is a production-grade privacy-preserving Ethereum JSON-RPC gateway that prevents blockchain RPC providers from learning client IP addresses, geographic locations, and direct wallet-to-network linkages. All traffic is routed through the Penum protocol's encrypted onion network using fixed-size packets, providing critical privacy protections for Ethereum users without requiring changes to existing infrastructure.

## Problem Statement

Ethereum users leak critical metadata to RPC providers (Alchemy, Infura, etc.) including:
- Client IP addresses
- Geographic location data
- Direct wallet-to-network linkages
- Transaction timing and patterns

This enables surveillance, behavioral profiling, and targeted MEV extraction. Current solutions require significant infrastructure changes or are not practical for everyday users.

## Objectives

1. **Enhance User Privacy**: Protect Ethereum users from RPC provider surveillance
2. **Maintain Compatibility**: Work with existing wallets and infrastructure
3. **Provide Production-Ready Solution**: Deliver battle-tested, secure implementation
4. **Enable Scalable Privacy**: Support multiple users without degrading privacy
5. **Establish Multi-Hop Network**: Expand from single-hop to full onion routing

## Project Scope

Penum Private RPC provides a complete privacy-preserving RPC solution with:

### Core Components
- **penum-rpc-client**: Local RPC endpoint for MetaMask and other wallets
- **penum-rpc-gateway**: RPC provider interface with encrypted communication
- **Penum Protocol**: Onion routing with fixed-size packets and ephemeral keys

### Technical Features
- Fixed 1024-byte encrypted packets to prevent traffic analysis
- Ephemeral X25519 keys for each connection (no key reuse)
- ChaCha20-Poly1305 AEAD encryption for all communications
- Zero logging of sensitive data (wallet addresses, transaction details)
- Fail-silent error handling to prevent information leakage
- Full MetaMask integration via standard JSON-RPC interface

### Supported Methods
- `eth_call`, `eth_getBalance`, `eth_blockNumber`
- `eth_sendRawTransaction`, `eth_getTransactionReceipt`
- (Expandable to all JSON-RPC methods)

## Technical Approach

### Privacy Guarantees
1. **IP Privacy**: RPC providers see only gateway IP, not client IP
2. **Traffic Analysis Resistance**: Fixed packet sizes prevent size-based correlation
3. **Request Unlinkability**: Ephemeral keys prevent request correlation
4. **Zero Logging**: No sensitive data stored by design

### Cryptographic Protocol
- **Key Exchange**: X25519 for ephemeral key generation
- **Encryption**: ChaCha20-Poly1305 for data encryption
- **Key Derivation**: HKDF-SHA256 with "penum-v1" salt
- **Packet Structure**: Fixed 1024-byte format with random padding

### Architecture
```
MetaMask → penum-rpc-client → penum-rpc-gateway → RPC Provider
(localhost:8545)  (encrypted packets)    (Alchemy/Infura)
```

## Project Team

The Penum team consists of experienced blockchain and privacy researchers with expertise in:
- Cryptographic protocol design
- Privacy-preserving systems
- Ethereum infrastructure
- Rust systems programming
- Security analysis

## Methodology

### Phase 1: Security Audit and Hardening
- Complete security audit of cryptographic implementation
- Formal verification of key components
- Penetration testing and vulnerability assessment

### Phase 2: Multi-Hop Network Deployment
- Deploy geographically distributed relay nodes
- Implement full 3-hop Penum protocol (Entry → Middle → Gateway)
- Load balancing and auto-scaling infrastructure

### Phase 3: Feature Expansion
- Support for all Ethereum JSON-RPC methods
- WebSocket support for subscription methods
- Mobile client applications
- Browser extension for one-click setup

### Phase 4: Community and Adoption
- Documentation and tutorial creation
- Community building and user support
- Partnership with wallet providers
- Educational content on RPC privacy

## Timeline and Milestones

### Month 1-2: Security Hardening
- Complete security audit
- Implement formal verification of critical components
- Fix identified vulnerabilities
- **Deliverable**: Audited and hardened codebase

### Month 3-4: Infrastructure Deployment
- Deploy 5 geographically distributed relay nodes
- Implement multi-hop routing
- Load balancing and monitoring
- **Deliverable**: Production-ready multi-hop network

### Month 5-6: Feature Enhancement
- Full JSON-RPC method support
- WebSocket implementation
- Mobile client development
- **Deliverable**: Complete feature set with mobile support

### Month 7-8: Community Building
- Comprehensive documentation
- Educational content creation
- Partnership development
- **Deliverable**: Active community and widespread adoption

## Budget Justification

### Development Team (6 months)
- Lead Developer: $80,000
- Security Engineer: $60,000
- DevOps Engineer: $40,000

### Infrastructure (12 months)
- Relay node hosting: $15,000
- Monitoring and security tools: $5,000
- SSL certificates and security: $2,000

### Security and Audit
- Security audit: $30,000
- Penetration testing: $15,000
- Formal verification: $20,000

### Community and Marketing
- Documentation and tutorials: $10,000
- Educational content: $8,000
- Partnership development: $5,000

**Total Budget: $290,000**

## Expected Outcomes

1. **Enhanced Privacy**: Ethereum users protected from RPC provider surveillance
2. **Widespread Adoption**: Thousands of users benefiting from privacy protection
3. **Infrastructure**: Distributed relay network serving the Ethereum ecosystem
4. **Open Source**: All code released under MIT license for community use
5. **Educational Impact**: Increased awareness of RPC privacy risks

## Sustainability Plan

### Short-term (0-12 months)
- Community donations and grants
- Corporate sponsorships from privacy-focused entities
- Fee-based premium features for businesses

### Long-term (12+ months)
- Decentralized governance through DAO
- Community-driven relay operation
- Integration with privacy-focused DeFi protocols
- Potential token incentives for relay operators

## Differentiation from Existing Solutions

Unlike other privacy solutions that require protocol changes or complex setups, Penum Private RPC:
- Works with existing wallets immediately
- Requires no changes to Ethereum protocol
- Provides immediate privacy benefits
- Maintains full compatibility with existing dApps
- Offers measurable privacy improvements

## Open Source Commitment

All Penum Private RPC code will remain open source under the MIT license. We commit to:
- Transparent development process
- Community governance
- Regular security audits
- Comprehensive documentation
- Active community support