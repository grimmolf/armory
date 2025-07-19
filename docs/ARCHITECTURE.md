# Architecture Guide

## System Overview

The Armory Rust modernization follows a layered architecture designed for security, maintainability, and extensibility. The system separates concerns into distinct modules while maintaining clear interfaces between components.

**🧹 RECENT MODERNIZATION**: As of v0.4.1, the project has completed a massive cleanup, eliminating all legacy Python/C++ components and achieving a **pure Rust architecture**. This results in enhanced security, simplified maintenance, and elimination of all end-of-life dependencies.

## 🏆 **Pure Rust Architecture Benefits**

### ✅ **Eliminated Complexity**
- **Single Language**: No more Python/C++ multi-language complexity
- **Unified Build System**: Single Cargo-based build process
- **Memory Safety**: Rust ownership system prevents entire classes of vulnerabilities
- **Cross-Platform**: Native Rust compilation without platform-specific build chains

### ✅ **Security Improvements**
- **No EOL Dependencies**: Eliminated Python 2.7, PyQt4, Crypto++ 5.6.1
- **Modern Cryptography**: ChaCha20Poly1305, Argon2id, BIP-340 Schnorr signatures
- **Reduced Attack Surface**: Single-language implementation
- **Memory Protection**: Automatic zeroization of sensitive data

### ✅ **Development Experience**
- **Simplified Setup**: `cargo build` and you're ready to develop
- **Unified Testing**: Single test suite with 127/127 tests passing (100%)
- **Modern Tooling**: cargo fmt, clippy, documentation generation
- **Type Safety**: Compile-time error detection and prevention

```
┌─────────────────────────────────────────────────────┐
│                 User Interface Layer                │
├─────────────────────────────────────────────────────┤
│ ✅ CLI Interface │  JSON-RPC API  │  Future GUI      │
├─────────────────────────────────────────────────────┤
│                Application Layer                     │
├─────────────────────────────────────────────────────┤
│ Wallet Management │ Transaction │  Network         │
│                   │  Processing │  Communication    │
├─────────────────────────────────────────────────────┤
│                 Core Services Layer                 │
├─────────────────────────────────────────────────────┤
│  Cryptography   │   Storage     │   Error          │
│   Services      │   Services    │   Handling       │
├─────────────────────────────────────────────────────┤
│                Infrastructure Layer                  │
├─────────────────────────────────────────────────────┤
│ Bitcoin Protocol │   Database    │   System        │
│   Libraries      │   Engine      │   Integration   │
└─────────────────────────────────────────────────────┘
```

## Module Architecture

### 1. Cryptography Module (`src/crypto/`)

**Purpose**: Provides secure cryptographic operations with automatic memory management.

**Components**:
- **Key Derivation Functions (KDF)**: Argon2id implementation for secure password-based key derivation
- **Encryption**: ChaCha20Poly1305 AEAD encryption with automatic nonce generation
- **Digital Signatures**: BIP-340 Schnorr and ECDSA signature operations
- **Random Number Generation**: Cryptographically secure entropy collection

**Design Principles**:
- Memory safety through automatic zeroization
- Constant-time operations where applicable
- Strong default parameters for all cryptographic operations
- Clear separation between public and private key operations

```rust
// Crypto module interface
pub mod crypto {
    // Key derivation with memory-hard properties
    pub fn derive_key_from_password(password: &str, salt: &[u8], params: &KdfParams) -> CryptoResult<Vec<u8>>;
    
    // AEAD encryption with authentication
    pub fn encrypt_data(key: &SecureKey, plaintext: &[u8]) -> CryptoResult<EncryptedData>;
    
    // BIP-340 Schnorr signatures for Taproot
    pub fn sign_schnorr(message: &[u8], private_key: &PrivateKey) -> CryptoResult<[u8; 64]>;
    
    // Secure random generation
    pub fn generate_random_bytes(len: usize) -> CryptoResult<Vec<u8>>;
}
```

### 2. Storage Module (`src/storage/`)

**Purpose**: Manages persistent data storage with encryption and backup capabilities.

**Components**:
- **Wallet Storage**: SLED-based encrypted storage with atomic operations
- **Legacy Import**: Conversion from legacy Armory wallet formats
- **Backup Management**: Automatic backup creation and rotation

**Storage Layout**:
```
~/.armory-rust/
├── wallets.db/           # SLED database directory
│   ├── conf             # Database configuration
│   ├── db               # Main wallet data
│   └── snap.*/          # Snapshots
├── backups/             # Automatic backups
│   ├── wallet_20240101_120000.backup
│   └── wallet_20240101_130000.backup
└── logs/                # Application logs
    └── armory.log
```

**Security Features**:
- All sensitive data encrypted at rest using ChaCha20Poly1305
- Keys derived from user passphrase using Argon2id
- Atomic file operations prevent corruption
- Secure backup with integrity verification

### 3. Wallet Module (`src/wallet/`)

**Purpose**: Implements hierarchical deterministic (HD) wallets with modern Bitcoin standards.

**Components**:
- **Descriptor Wallets**: Output script descriptors for all address types
- **HD Key Management**: BIP-32 hierarchical key derivation
- **Address Generation**: Support for Legacy, SegWit, and Taproot addresses
- **UTXO Management**: Transaction output tracking and balance calculation

**Wallet Architecture**:
```rust
pub struct Wallet {
    id: String,                    // Unique wallet identifier
    network: Network,              // Bitcoin network (mainnet/testnet/regtest)
    master_key: ExtendedPrivateKey, // BIP-32 master key
    derived_keys: HashMap<DerivationPath, ExtendedPrivateKey>, // Cached keys
    addresses: HashMap<DerivationPath, Address>,               // Generated addresses
    utxos: HashMap<(Txid, u32), Utxo>,                       // Unspent outputs
    transactions: BTreeMap<Txid, WalletTransaction>,          // Transaction history
    next_indices: HashMap<AddressType, u32>,                  // Address derivation counters
    storage: WalletStorage,        // Persistent storage backend
}
```

**Address Type Support**:
| Type | Standard | Format | Derivation Path |
|------|----------|--------|-----------------|
| Legacy | P2PKH | `1...` | `m/44'/0'/0'/0/n` |
| Nested SegWit | P2SH-P2WPKH | `3...` | `m/49'/0'/0'/0/n` |
| Native SegWit | P2WPKH | `bc1q...` | `m/84'/0'/0'/0/n` |
| Taproot | P2TR | `bc1p...` | `m/86'/0'/0'/0/n` |

### 4. Transaction Module (`src/transaction/`)

**Purpose**: Handles transaction creation, signing, and broadcast with PSBT v2 support.

**Components**:
- **Transaction Builder**: Constructs transactions with optimal fee estimation
- **PSBT Implementation**: Partially Signed Bitcoin Transaction version 2
- **Fee Estimation**: Dynamic fee calculation based on mempool conditions
- **Coin Selection**: Algorithms for optimal UTXO selection

**Transaction Workflow**:
```rust
// 1. Create transaction builder
let mut builder = TransactionBuilder::new(wallet, network);

// 2. Add recipients
builder.add_recipient(recipient_address, amount)?;

// 3. Select UTXOs and calculate fees
builder.select_utxos_and_fees(fee_rate)?;

// 4. Build PSBT
let psbt = builder.build_psbt()?;

// 5. Sign transaction
let signed_psbt = wallet.sign_psbt(psbt)?;

// 6. Finalize and broadcast
let final_tx = signed_psbt.finalize()?;
network.broadcast_transaction(final_tx)?;
```

### 5. Network Module (`src/network/`)

**Purpose**: Provides Bitcoin network communication with privacy and security enhancements.

**Components**:
- **P2P Protocol**: Direct peer-to-peer communication with Bitcoin nodes
- **RPC Client**: Bitcoin Core JSON-RPC interface
- **BIP-324 Support**: Encrypted transport foundation implemented (ready for enhancement)
- **Tor Integration**: SOCKS5 proxy support implemented for privacy-preserving access

**Network Architecture**:
```rust
pub enum NetworkBackend {
    BitcoinCore {
        rpc_client: RpcClient,
        url: String,
        auth: AuthCredentials,
    },
    P2P {
        peers: Vec<PeerConnection>,
        protocol_version: u32,
    },
    Electrum {
        servers: Vec<ElectrumServer>,
        ssl_context: SslContext,
    },
}
```

### 6. CLI Module (`src/cli/`)

**Purpose**: Provides command-line interface for wallet operations.

**Components**:
- **Command Parser**: Clap-based argument parsing with subcommands
- **Configuration Management**: TOML-based configuration files
- **Interactive Mode**: REPL-style wallet interaction
- **Output Formatting**: JSON, table, and human-readable formats

**Command Structure**:
```
armory-rust
├── create <name>         # Create new wallet
├── list                  # List all wallets
├── address <wallet>      # Generate new address
├── balance <wallet>      # Check wallet balance
├── send <wallet>         # Send transaction
├── import <path>         # Import legacy wallet
├── backup <wallet>       # Create wallet backup
└── restore <path>        # Restore from backup
```

## Security Architecture

### Defense in Depth

The system implements multiple layers of security:

1. **Memory Safety**: Rust's ownership system prevents buffer overflows and use-after-free
2. **Cryptographic Security**: Modern algorithms with secure defaults
3. **Storage Security**: Encryption at rest with strong key derivation
4. **Network Security**: Encrypted transport foundation and Tor SOCKS5 support implemented
5. **Process Isolation**: Minimal privileges and secure defaults

### Key Management

```
Master Seed (256 bits)
       ↓
BIP-32 Master Key
       ↓
Account Keys (m/purpose'/coin'/account')
       ↓
Chain Keys (m/purpose'/coin'/account'/change)
       ↓
Address Keys (m/purpose'/coin'/account'/change/index)
```

### Threat Model

**Protected Against**:
- Memory corruption vulnerabilities
- Cryptographic weaknesses in legacy algorithms
- Storage attacks against encrypted data
- Network surveillance and traffic analysis
- Dependency vulnerabilities through modern tooling

**Assumptions**:
- System is not compromised at the kernel level
- Hardware random number generator is trustworthy
- User maintains control of storage device
- Network connections may be monitored

## Performance Architecture

### Optimization Strategies

1. **Lazy Loading**: Objects created only when needed
2. **Caching**: Frequently accessed data cached in memory
3. **Batch Operations**: Multiple storage operations combined
4. **Async I/O**: Non-blocking network and storage operations
5. **Zero-Copy**: Minimal data copying in critical paths

### Performance Targets

| Operation | Target | Current |
|-----------|--------|---------|
| Key Derivation | <10ms | ~5ms |
| Address Generation | <5ms | ~2ms |
| Transaction Signing | <50ms | ~25ms |
| Storage Read | <10ms | ~5ms |
| Storage Write | <50ms | ~20ms |

### Memory Management

- **Automatic Cleanup**: Rust RAII ensures resource deallocation
- **Secure Zeroization**: Sensitive data cleared from memory automatically
- **Bounded Memory**: Maximum memory usage limits prevent DoS
- **Efficient Serialization**: Minimal overhead for data conversion

## Integration Points

### External Dependencies

```rust
// Bitcoin protocol implementation
bitcoin = "0.32"          // Core Bitcoin types and operations
secp256k1 = "0.29"        // Elliptic curve cryptography

// Modern cryptography
chacha20poly1305 = "0.10" // AEAD encryption
argon2 = "0.5"            // Memory-hard key derivation

// Storage and networking
sled = "0.34"             // Embedded database
tokio = "1.35"            // Async runtime
reqwest = "0.11"          // HTTP client

// Utilities
serde = "1.0"             // Serialization
clap = "4.4"              // CLI parsing
tracing = "0.1"           // Structured logging
```

### API Interfaces

**Internal APIs**:
- Module boundaries use strongly-typed Rust interfaces
- Error handling through Result<T, E> types
- Async operations use Tokio futures

**External APIs**:
- JSON-RPC compatible with Bitcoin Core
- REST API for web integration (future consideration)
- Plugin system for extensibility (future consideration)

## Deployment Architecture

### Single Binary Deployment

```
armory-rust (executable)
├── Built-in wallet engine
├── CLI interface
├── Storage encryption
├── Network communication
└── Legacy import tools
```

### Configuration Management

```toml
# ~/.armory-rust/config.toml
[wallet]
default_network = "bitcoin"
data_directory = "~/.armory-rust"
auto_backup = true
backup_count = 5

[network]
bitcoin_rpc_url = "http://localhost:8332"
bitcoin_rpc_user = "user"
bitcoin_rpc_password = "password"
use_tor = false

[security]
session_timeout = 3600
require_password_confirmation = true
memory_lock = true
```

### Scalability Considerations

- **Multiple Wallets**: Concurrent access to different wallets
- **Large Transaction History**: Efficient indexing and pagination
- **High-Frequency Operations**: Batching and caching optimizations
- **Resource Limits**: Configurable memory and storage bounds

This architecture provides a solid foundation for secure, maintainable, and extensible Bitcoin wallet functionality while maintaining compatibility with existing Bitcoin infrastructure.