# REQUIREMENTS.md - Bitcoin Wallet Rust Modernization

## Overview

This document specifies the complete functional and non-functional requirements for modernizing the Armory Bitcoin wallet from C++/Python to Rust, ensuring feature parity with modern Bitcoin protocol improvements.

**Foundation Document**: [MODERNIZATION_ANALYSIS.md](MODERNIZATION_ANALYSIS.md) - Comprehensive legacy codebase analysis

**Target**: Rust 1.78+ with modern Bitcoin ecosystem libraries

**Platform Support**: macOS (arm64/x86_64), Fedora Linux (x86_64)

## Functional Requirements

### Core Wallet Operations

| Feature | Legacy Implementation | Modern Rust Target | Priority | Status |
|---------|----------------------|-------------------|----------|---------|
| **Key Generation** | Custom chaincode + ECDSA DH | BIP-32 HD wallets + Taproot descriptors | HIGH | ✅ STARTED |
| **Address Types** | P2PKH, P2SH | P2PKH, P2SH, P2WPKH, P2WSH, P2TR (Taproot) | HIGH | PLANNED |
| **Transaction Creation** | Custom builder | PSBT v2 (BIP-370) + RBF support | HIGH | PLANNED |
| **Multi-signature** | Custom M-of-N | Descriptor-based multisig + miniscript | HIGH | PLANNED |
| **Offline Signing** | Custom format | PSBT-based offline workflow | HIGH | PLANNED |
| **Wallet Encryption** | AES + custom ROMIX KDF | ChaCha20Poly1305 + Argon2id KDF | HIGH | ✅ IMPLEMENTED |
| **Backup/Recovery** | Fragmented paper backup | BIP-39 mnemonic seeds | HIGH | PLANNED |

#### Key Generation Specifications

**Legacy System (PyBtcWallet.py:64-150)**:
- Custom deterministic wallet using ECDSA Diffie-Hellman
- Non-standard chaincode derivation
- Encrypted storage with ROMIX KDF

**Modern Requirements**:
- **BIP-32** Hierarchical Deterministic Wallets
- **BIP-39** Mnemonic seed phrases (12/24 words)
- **BIP-44/49/84** Derivation paths for different address types
- **BIP-341** Taproot key derivation
- **Descriptor-based** address generation

#### Address Type Support

**Implementation Matrix**:
```
Legacy Support:
- P2PKH (1...) - Pay to Public Key Hash
- P2SH (3...) - Pay to Script Hash (basic multisig)

Modern Requirements:
- P2PKH (1...) - Backward compatibility
- P2SH (3...) - Enhanced multisig support
- P2WPKH (bc1q...) - Native SegWit v0
- P2WSH (bc1q...) - Native SegWit v0 scripts
- P2TR (bc1p...) - Taproot (SegWit v1)
```

#### Transaction Processing

**PSBT v2 Implementation (BIP-370)**:
- Replace legacy custom transaction format
- Support for RBF (Replace-by-Fee) by default
- Hardware wallet compatibility
- Multi-party transaction construction
- Fee estimation and coin selection optimization

**Legacy Limitations (Transaction.py)**:
- Pre-SegWit transaction handling only
- No PSBT support
- Limited script engine (not consensus-safe)
- Manual fee calculation

### Network Operations

| Feature | Legacy Implementation | Modern Rust Target | Priority | Implementation Notes |
|---------|----------------------|-------------------|----------|---------------------|
| **P2P Protocol** | Custom Twisted implementation | BIP-324 encrypted transport | MEDIUM | Opportunistic encryption |
| **Bootstrap** | BitTorrent blockchain download | Standard P2P sync + optional Tor | MEDIUM | Remove BitTorrent dependency |
| **Broadcasting** | Direct P2P + fallback RPC | Multiple broadcast methods | LOW | Add redundancy |
| **Network Security** | Unencrypted P2P | BIP-324 + Tor v3 support | HIGH | Privacy enhancement |

### User Interface

| Feature | Legacy Implementation | Modern Rust Target | Priority | Migration Path |
|---------|----------------------|-------------------|----------|---------------|
| **GUI Framework** | PyQt4 (EOL) | Modern Rust GUI (egui/tauri) or CLI-first | MEDIUM | CLI → GUI phased approach |
| **RPC Interface** | JSON-RPC daemon | Modern REST API + JSON-RPC compatibility | HIGH | Backward compatibility required |
| **Configuration** | Custom config files | TOML-based configuration | LOW | Modern format adoption |

### Advanced Features

#### Multi-signature Support

**Legacy System**:
- Custom M-of-N implementation
- Manual key coordination
- Limited script types

**Modern Requirements**:
- **Miniscript** policy language for complex spending conditions
- **Descriptor-based** multisig wallets
- **PSBT** collaborative signing workflow
- **Hardware wallet** integration for secure key storage

#### Hardware Wallet Integration

**New Capability Requirements**:
- **HWI** (Hardware Wallet Interface) support
- **PSBT** signing workflow
- Device enumeration and management
- **Ledger, Trezor, Coldcard** compatibility

### Legacy Compatibility

#### Wallet Import Requirements

**Source**: `/Users/grimm/coding/gits/armory/armoryengine/PyBtcWallet.py:108-150`

**Legacy Format Specifications**:
```
File Structure:
- Header (352 bytes): File ID, version, network, metadata
- Crypto Section (512 bytes): KDF parameters, salt, IV
- Key Generator (237 bytes): Root key and chaincode
- Entries: Address data, comments, transaction metadata
```

**Import Capabilities**:
- Decrypt ROMIX-encrypted wallets
- Convert to modern descriptor format
- Preserve all address history and metadata
- Maintain watching-only wallet functionality

## Non-Functional Requirements

### Security Requirements

#### Memory Safety
- **Rust Ownership System**: Eliminate buffer overflows and use-after-free
- **Secure Memory Handling**: Automatic zeroization of sensitive data
- **Type Safety**: Compile-time prevention of common vulnerabilities

#### Cryptographic Standards
- **BIP-340** Schnorr signatures for Taproot
- **BIP-341** Taproot script validation
- **BIP-324** Encrypted P2P transport
- **Argon2id** memory-hard key derivation
- **ChaCha20Poly1305** AEAD encryption

#### Key Management
- **Hardware Security**: HSM and hardware wallet support
- **Air-gapped** operation capability
- **Secure key generation** with proper entropy
- **Key rotation** and backup procedures

### Performance Requirements

#### Signing Latency
- **Target**: <50ms for standard transactions
- **Hardware wallet**: <5 seconds including user confirmation
- **Batch signing**: Linear scaling for multiple inputs

#### Synchronization Performance
- **Initial sync**: Parallel UTXO processing
- **Incremental sync**: <10 seconds for new blocks
- **Memory usage**: <500MB peak during full node interaction

#### Storage Efficiency
- **Database**: Indexed UTXO set for fast lookup
- **Pruning**: Optional transaction history pruning
- **Compression**: Efficient descriptor storage

### Compatibility Requirements

#### Cross-platform Support
- **macOS**: arm64 and x86_64 architectures
- **Linux**: x86_64 (Fedora 38+, Ubuntu 22.04+)
- **Build system**: `cargo build` on all platforms
- **Dependencies**: Minimal system requirements

#### Network Compatibility
- **Bitcoin Core RPC**: Full compatibility with Bitcoin Core 25.0+
- **Electrum servers**: Optional lightweight mode
- **P2P protocol**: Bitcoin Core peer compatibility

#### Standards Compliance
- **BIP compliance**: Implementation of relevant Bitcoin Improvement Proposals
- **Interoperability**: Standard wallet file formats where applicable
- **API consistency**: Familiar interfaces for existing users

### Scalability Requirements

#### Wallet Capacity
- **Addresses**: Support for 100,000+ addresses per wallet
- **Transactions**: Efficient handling of transaction history
- **UTXO management**: Fast coin selection algorithms

#### Multi-wallet Support
- **Concurrent access**: Multiple wallets simultaneously
- **Resource isolation**: Per-wallet resource management
- **Backup coordination**: Atomic backup operations

## Quality Requirements

### Reliability
- **Uptime**: 99.9% availability for daemon mode
- **Data integrity**: Atomic file operations and checksums
- **Error recovery**: Graceful handling of corruption

### Maintainability
- **Code coverage**: >90% test coverage for core modules
- **Documentation**: Comprehensive API documentation
- **Modular design**: Clear separation of concerns

### Usability
- **CLI interface**: Intuitive command structure
- **Error messages**: Clear, actionable error reporting
- **Migration tools**: Seamless upgrade path from legacy

## Testing Requirements

### Unit Testing
- **Cryptographic functions**: Comprehensive test vectors
- **Wallet operations**: All core functionality tested
- **Network protocols**: Mock testing for network operations

### Integration Testing
- **End-to-end workflows**: Complete transaction lifecycle
- **Legacy import**: Test with real legacy wallet files
- **Hardware wallets**: Testing with supported devices

### Security Testing
- **Penetration testing**: Third-party security assessment
- **Fuzzing**: Input validation robustness
- **Side-channel analysis**: Timing attack resistance

## Compliance Requirements

### Regulatory Considerations
- **Privacy**: Enhanced privacy through Tor integration
- **Auditability**: Transaction logging and reporting capabilities
- **Open source**: AGPL v3 license compliance

### Industry Standards
- **OWASP**: Secure coding practices
- **NIST**: Cryptographic standards compliance
- **Bitcoin**: BIP implementation requirements

## Migration Strategy

### Phase 1: Core Infrastructure (Months 1-4)
- ✅ Project setup with modern Rust libraries
- ✅ Cryptographic foundation implementation
- ✅ Storage layer with legacy import capability
- Basic wallet operations

### Phase 2: Transaction Processing (Months 5-8)
- PSBT v2 implementation
- Transaction builder with RBF support
- Fee estimation and coin selection
- Hardware wallet integration

### Phase 3: Network Layer (Months 9-12)
- BIP-324 encrypted transport implementation
- P2P networking with modern protocols
- RPC interface modernization
- Tor integration

### Phase 4: User Interface (Months 13-16)
- CLI interface completion
- Legacy wallet migration tools
- Documentation and testing
- Security audit and final validation

## Success Metrics

### Technical Metrics
- **Build Success**: `cargo build` succeeds on all target platforms
- **Test Coverage**: >90% line coverage for core modules
- **Performance**: <50ms transaction signing latency
- **Security**: Zero clippy warnings, clean cargo audit results
- **Compatibility**: Import 100% of legacy Armory wallet files

### User Metrics
- **Migration Success**: Seamless upgrade path from legacy
- **Feature Parity**: All legacy features available in modern format
- **Usability**: Reduced complexity for common operations
- **Documentation**: Complete user and developer guides

## Risk Mitigation

### Technical Risks
- **Cryptographic compatibility**: Maintain interoperability during transition
- **Performance regression**: Continuous benchmarking during development
- **Security vulnerabilities**: Regular security audits and reviews

### Timeline Risks
- **Dependency issues**: Fallback plans for library incompatibilities
- **Resource constraints**: Phased delivery with incremental value
- **Integration complexity**: Extensive testing at each phase

---

**References:**
- **MODERNIZATION_ANALYSIS.md**: Comprehensive legacy assessment
- **Legacy Implementation**: `/Users/grimm/coding/gits/armory/armoryengine/PyBtcWallet.py:64-150`
- **Security Findings**: Critical vulnerabilities in Python 2.7, PyQt4, Crypto++ 5.6.1
- **Build Dependencies**: Make-based legacy system with platform-specific requirements