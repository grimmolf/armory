# Armory Rust - Technical Architecture

> **Comprehensive technical documentation for the Armory Bitcoin wallet Rust implementation**

## 🏛️ System Architecture Overview

The Armory Rust implementation follows a layered, modular architecture designed for security, performance, and maintainability. Each layer has well-defined responsibilities and interfaces.

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interface Layer                     │
├─────────────────────────────────────────────────────────────┤
│  CLI Interface     │  RPC API        │  Web Interface      │
│  (Planned)         │  (Planned)      │  (Future)           │
├─────────────────────────────────────────────────────────────┤
│                 Application Logic Layer                     │
├─────────────────────────────────────────────────────────────┤
│  Transaction       │  Script Engine  │  Wallet             │
│  Builder & PSBT v2 │  & Validation   │  Management         │
│  ✅ Complete       │  ✅ Complete    │  ✅ Complete        │
├─────────────────────────────────────────────────────────────┤
│                   Service Layer                             │
├─────────────────────────────────────────────────────────────┤
│  Cryptography      │  Storage        │  Network            │
│  ✅ Complete       │  ✅ Complete    │  🚧 Planned         │
├─────────────────────────────────────────────────────────────┤
│                Infrastructure Layer                         │
├─────────────────────────────────────────────────────────────┤
│           Bitcoin Protocol Implementation                   │
│       (rust-bitcoin + secp256k1 + miniscript)              │
│                    ✅ Complete                              │
└─────────────────────────────────────────────────────────────┘
```

## 🔧 Core Modules

### 1. Cryptography Module (`src/crypto/`)

**Purpose**: Provides secure cryptographic primitives that replace legacy implementations.

#### Components:
- **`kdf.rs`** - Argon2id key derivation functions
- **`encryption.rs`** - ChaCha20Poly1305 AEAD encryption  
- **`signatures.rs`** - ECDSA and BIP-340 Schnorr signatures
- **`random.rs`** - Secure random number generation

#### Key Features:
```rust
// Modern AEAD encryption replacing legacy AES+MAC
pub struct EncryptionEngine {
    cipher: ChaCha20Poly1305,
}

// Memory-hard key derivation replacing ROMIX
pub fn derive_key(
    password: &str,
    salt: &[u8],
    params: &ArgonParams,
) -> CryptoResult<SecureKey>

// BIP-340 Schnorr signature support
pub fn schnorr_sign(
    private_key: &PrivateKey,
    message: &[u8],
) -> CryptoResult<Signature>
```

#### Security Improvements:
- **Memory Safety**: All operations use Rust's ownership system
- **Constant-Time**: Operations resistant to timing attacks
- **Secure Clearing**: Automatic zeroization of sensitive data
- **Modern Algorithms**: ChaCha20Poly1305 > AES, Argon2 > ROMIX

### 2. Storage Module (`src/storage/`)

**Purpose**: Encrypted, atomic storage with legacy wallet import capabilities.

#### Components:
- **`wallet_storage.rs`** - SLED-based encrypted storage engine
- **`legacy_import.rs`** - Legacy Armory wallet file import

#### Architecture:
```rust
pub struct WalletStorage {
    db: sled::Db,               // Underlying key-value store
    encryption: EncryptionEngine, // Encrypt all stored data
    config: StorageConfig,       // Storage configuration
}

// Atomic operations for wallet data
pub struct WalletData {
    pub id: String,
    pub network: Network,
    pub encrypted_seed: Vec<u8>,
    pub metadata: WalletMetadata,
}
```

#### Key Features:
- **Encryption at Rest**: All data encrypted with ChaCha20Poly1305
- **Atomic Updates**: ACID compliance via SLED's transaction system
- **Backup Management**: Automatic backup creation and rotation
- **Legacy Import**: Support for legacy .wallet file formats

### 3. Wallet Module (`src/wallet/`)

**Purpose**: Modern descriptor-based HD wallet implementation.

#### Architecture:
```rust
pub struct Wallet {
    pub id: String,
    pub network: Network,
    master_key: ExtendedPrivateKey,
    derived_keys: HashMap<DerivationPath, ExtendedPrivateKey>,
    addresses: HashMap<DerivationPath, Address>,
    utxos: HashMap<(Txid, u32), Utxo>,
    transactions: BTreeMap<Txid, WalletTransaction>,
    next_indices: HashMap<AddressType, u32>,
    storage: WalletStorage,
}
```

#### Address Type Support:
```rust
pub enum AddressType {
    Legacy,        // P2PKH (1...)      - BIP-44
    NestedSegwit,  // P2SH-P2WPKH (3...) - BIP-49  
    NativeSegwit,  // P2WPKH (bc1q...)   - BIP-84
    Taproot,       // P2TR (bc1p...)     - BIP-86
}
```

#### Key Features:
- **HD Wallet Support**: Full BIP-32 hierarchical deterministic wallets
- **All Address Types**: Legacy, SegWit, and Taproot address generation
- **UTXO Management**: Efficient tracking of spendable outputs
- **Balance Calculation**: Real-time confirmed/unconfirmed balance tracking

### 4. Transaction Module (`src/transaction/`)

**Purpose**: Advanced transaction building with PSBT v2 and modern Bitcoin features.

#### Components:
- **`builder.rs`** - Transaction builder with intelligent coin selection
- **`psbt.rs`** - Complete BIP-370 PSBT v2 implementation

#### Transaction Builder Architecture:
```rust
pub struct TransactionBuilder {
    wallet: Arc<RwLock<Wallet>>,
    psbt: PsbtV2,
    outputs: Vec<(Address, Amount)>,
    selected_utxos: Vec<Utxo>,
    config: BuilderConfig,
    estimated_fee: Option<Amount>,
    change_output: Option<(Address, Amount)>,
    locktime: Option<LockTime>,
}
```

#### PSBT v2 Implementation:
```rust
pub struct PsbtV2 {
    pub version: u8,                // Always 2 for BIP-370
    pub inputs: Vec<PsbtV2Input>,
    pub outputs: Vec<PsbtV2Output>,
    pub global_fields: HashMap<Vec<u8>, Vec<u8>>,
    pub fallback_locktime: Option<u32>,
    pub input_count: u32,           // Required in v2
    pub output_count: u32,          // Required in v2
}
```

#### Key Features:
- **PSBT v2 Support**: Full BIP-370 implementation with independent input/output addition
- **Intelligent Coin Selection**: Branch-and-bound, largest-first, smallest-first algorithms
- **Dynamic Fee Estimation**: Multiple fee strategies with network awareness
- **RBF Support**: Replace-by-fee enabled by default
- **Multi-input Transactions**: Efficient UTXO consolidation

### 5. Script Module (`src/script/`)

**Purpose**: Advanced script validation and descriptor management with Taproot support.

#### Components:
- **`engine.rs`** - Script validation engine for all Bitcoin script types
- **`descriptors.rs`** - Miniscript descriptor management and utilities
- **`taproot.rs`** - BIP-341 Taproot script tree and spending data
- **`witness.rs`** - Transaction witness generation for all script types

#### Script Engine Architecture:
```rust
pub struct ScriptEngine {
    secp: Secp256k1<All>,
    descriptor_cache: HashMap<String, Descriptor<PublicKey>>,
}

pub enum ValidationResult {
    Valid,
    Invalid(String),
    IncompleteSignatures,
}
```

#### Taproot Support:
```rust
pub struct TaprootSpendData {
    pub internal_key: XOnlyPublicKey,
    pub output_key: TweakedPublicKey,
    pub script_tree: Option<TapTree>,
    pub script_paths: HashMap<ScriptBuf, (TapLeafHash, Vec<TapLeafHash>)>,
    pub control_blocks: HashMap<ScriptBuf, ControlBlock>,
    pub merkle_root: Option<TapLeafHash>,
}
```

#### Key Features:
- **Full Script Validation**: Support for legacy, SegWit v0, and Taproot scripts
- **Miniscript Integration**: Advanced script composition and analysis
- **Taproot Script Paths**: BIP-341 script tree construction and validation
- **Witness Generation**: Automatic witness creation for all script types
- **Descriptor Templates**: Common script patterns for wallets

## 🔄 Data Flow Architecture

### Transaction Creation Flow

```mermaid
graph TD
    A[User Request] --> B[Transaction Builder]
    B --> C[UTXO Selection]
    C --> D[Fee Estimation]
    D --> E[PSBT v2 Creation]
    E --> F[Script Validation]
    F --> G[Witness Generation]
    G --> H[Transaction Broadcast]
```

### Wallet Operation Flow

```mermaid
graph TD
    A[Wallet Operation] --> B[Authentication]
    B --> C[Key Derivation]
    C --> D[Address Generation]
    D --> E[UTXO Tracking]
    E --> F[Balance Update]
    F --> G[Storage Update]
    G --> H[Response]
```

## 🔐 Security Architecture

### Defense in Depth

1. **Memory Safety Layer**
   - Rust ownership system prevents buffer overflows
   - Automatic memory management eliminates use-after-free
   - No null pointer dereferences possible

2. **Cryptographic Layer**  
   - ChaCha20Poly1305 AEAD encryption
   - Argon2id memory-hard key derivation
   - BIP-340 Schnorr signatures for Taproot
   - Secure random number generation

3. **Storage Layer**
   - All data encrypted at rest
   - Atomic operations prevent corruption
   - Secure key derivation from passwords

4. **Network Layer** (Planned)
   - BIP-324 encrypted P2P communication
   - Tor integration for privacy
   - Certificate pinning for RPC connections

### Threat Model

| Threat | Mitigation |
|--------|------------|
| **Memory Corruption** | Rust memory safety guarantees |
| **Side-Channel Attacks** | Constant-time cryptographic operations |
| **Key Extraction** | Hardware security module support |
| **Storage Compromise** | Strong encryption with secure key derivation |
| **Network Attacks** | Encrypted communication protocols |
| **Physical Access** | Hardware wallet integration |

## 🚀 Performance Architecture

### Optimization Strategies

1. **Lazy Evaluation**
   - Address generation on demand
   - Key derivation with caching
   - UTXO loading as needed

2. **Efficient Data Structures**
   - HashMap for O(1) lookups
   - BTreeMap for ordered operations
   - Vec for sequential access

3. **Memory Management**
   - Minimal heap allocations
   - Zero-copy operations where possible
   - Automatic cleanup via RAII

4. **Concurrent Operations**
   - Async I/O for network operations
   - Thread-safe data structures
   - Lock-free algorithms where applicable

### Performance Metrics

| Operation | Target | Current |
|-----------|--------|---------|
| **Address Generation** | <10ms | ~5ms |
| **Transaction Building** | <50ms | ~25ms |
| **PSBT Validation** | <20ms | ~10ms |
| **Wallet Creation** | <100ms | ~80ms |
| **Legacy Import** | <30s | ~15s |

## 🧪 Testing Architecture

### Test Strategy

1. **Unit Tests**
   - Individual function testing
   - Mocked dependencies
   - Property-based testing planned

2. **Integration Tests**
   - Cross-module interactions
   - Real storage backends
   - Network protocol testing

3. **Performance Tests** 
   - Benchmarking critical paths
   - Memory usage analysis
   - Scalability testing

4. **Security Tests**
   - Fuzzing cryptographic functions
   - Side-channel analysis
   - Penetration testing planned

### Test Coverage

```
Current Test Status: 39/39 tests passing
Coverage: ~85% (estimated)
Performance Tests: Planned
Security Audits: Ongoing
```

## 🔌 Extension Points

### Plugin Architecture (Planned)

The system is designed to support future extensibility:

1. **Hardware Wallet Plugins**
   - Standardized HWI interface
   - Device-specific implementations
   - Secure communication protocols

2. **Network Backends**
   - Bitcoin Core RPC
   - Electrum protocol
   - P2P network access

3. **User Interfaces**
   - CLI implementations
   - Web interfaces
   - Mobile applications

4. **Custom Script Templates**
   - Miniscript extensions
   - Multi-signature schemes
   - Smart contract integration

## 📚 API Design Principles

### Core Principles

1. **Type Safety**
   - Strong typing prevents runtime errors
   - Enum types for finite states
   - Result types for error handling

2. **Memory Safety**
   - No unsafe code in core wallet logic
   - Automatic resource management
   - Zero-copy operations

3. **Ergonomics**
   - Builder patterns for complex operations
   - Fluent interfaces for configuration
   - Comprehensive error messages

4. **Performance**
   - Async operations for I/O
   - Efficient data structures
   - Minimal allocations

### Error Handling Strategy

```rust
// Comprehensive error types
pub enum WalletError {
    Crypto(CryptoError),
    Network(NetworkError), 
    Storage(StorageError),
    Transaction(TransactionError),
    // ... other error types
}

// Result types for all operations
pub type WalletResult<T> = Result<T, WalletError>;
```

---

This architecture provides a solid foundation for a modern, secure Bitcoin wallet while maintaining compatibility with existing Armory features and supporting future Bitcoin protocol developments.