# API Reference - Armory Rust

> **Complete API documentation for the Armory Bitcoin wallet Rust implementation**

## 📋 Table of Contents

- [Core Types](#core-types)
- [Error Handling](#error-handling)  
- [Cryptography Module](#cryptography-module)
- [Storage Module](#storage-module)
- [Wallet Module](#wallet-module)
- [Transaction Module](#transaction-module)
- [Script Module](#script-module)
- [Examples](#examples)

## 🔧 Core Types

### Network

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Bitcoin mainnet
    Bitcoin,
    /// Bitcoin testnet
    Testnet,
    /// Bitcoin signet
    Signet,
    /// Bitcoin regtest
    Regtest,
}
```

**Usage**:
```rust
let network = Network::Bitcoin;
let testnet = Network::Testnet;
```

### AddressType

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressType {
    /// Legacy Pay-to-Public-Key-Hash (P2PKH) - 1...
    Legacy,
    /// Pay-to-Script-Hash wrapped SegWit (P2SH-P2WPKH) - 3...
    NestedSegwit,
    /// Native SegWit Pay-to-Witness-Public-Key-Hash (P2WPKH) - bc1q...
    NativeSegwit,
    /// Taproot Pay-to-Taproot (P2TR) - bc1p...
    Taproot,
}
```

**Methods**:
```rust
impl AddressType {
    /// Get the BIP derivation path for this address type
    pub fn derivation_path(&self, account: u32, change: u32, index: u32) -> DerivationPath
}
```

## ❌ Error Handling

### Main Error Types

```rust
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Cryptographic operation failed: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Network communication error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Storage operation failed: {0}")]  
    Storage(#[from] StorageError),
    
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    
    #[error("Legacy wallet import failed: {0}")]
    LegacyImport(String),
    
    #[error("Hardware wallet error: {0}")]
    Hardware(String),
    
    #[error("Key generation failed")]
    KeyGeneration,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### Result Types

```rust
/// Result type for all wallet operations
pub type WalletResult<T> = Result<T, WalletError>;

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Result type for storage operations  
pub type StorageResult<T> = Result<T, StorageError>;

/// Result type for transaction operations
pub type TransactionResult<T> = Result<T, TransactionError>;
```

## 🔐 Cryptography Module

### Key Derivation Functions

```rust
/// Argon2id configuration parameters
#[derive(Debug, Clone)]
pub struct ArgonParams {
    pub memory_cost: u32,    // Memory usage in KiB
    pub time_cost: u32,      // Number of iterations
    pub parallelism: u32,    // Degree of parallelism
    pub output_len: usize,   // Output length in bytes
}

/// Derive a key using Argon2id
pub fn derive_key(
    password: &str,
    salt: &[u8],
    params: &ArgonParams,
) -> CryptoResult<SecureKey>

/// Default Argon2id parameters for wallet encryption
pub fn default_argon_params() -> ArgonParams
```

### Encryption

```rust
/// ChaCha20Poly1305 encryption engine
pub struct EncryptionEngine {
    cipher: ChaCha20Poly1305,
}

impl EncryptionEngine {
    /// Create new encryption engine with given key
    pub fn new(key: &SecureKey) -> CryptoResult<Self>
    
    /// Encrypt data with additional authenticated data
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> CryptoResult<Vec<u8>>
    
    /// Decrypt data with additional authenticated data
    pub fn decrypt(&self, ciphertext: &[u8], aad: &[u8]) -> CryptoResult<Vec<u8>>
}
```

### Digital Signatures

```rust
/// Private key for signing operations
#[derive(ZeroizeOnDrop)]
pub struct PrivateKey {
    key_data: [u8; 32],
}

impl PrivateKey {
    /// Generate a new random private key
    pub fn generate(network: Network) -> Self
    
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self>
    
    /// Get the corresponding public key
    pub fn public_key(&self, secp: &Secp256k1<All>) -> PublicKey
    
    /// Sign a message with ECDSA
    pub fn sign_ecdsa(&self, message: &[u8]) -> CryptoResult<Signature>
    
    /// Sign a message with Schnorr (BIP-340)
    pub fn sign_schnorr(&self, message: &[u8]) -> CryptoResult<Signature>
}
```

## 💾 Storage Module

### WalletStorage

```rust
/// Configuration for wallet storage
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Path to the storage directory
    pub storage_path: PathBuf,
    /// Enable automatic backup creation
    pub auto_backup: bool,
    /// Number of backup files to retain
    pub backup_count: usize,
}

/// Encrypted wallet storage engine
pub struct WalletStorage {
    db: sled::Db,
    encryption: EncryptionEngine,
    config: StorageConfig,
}

impl WalletStorage {
    /// Create new storage instance
    pub fn new(config: StorageConfig) -> StorageResult<Self>
    
    /// Save wallet data to storage
    pub fn save_wallet_data(&self, data: &WalletData) -> StorageResult<()>
    
    /// Load wallet data from storage
    pub fn load_wallet_data(&self, id: &str) -> StorageResult<WalletData>
    
    /// List all wallet IDs
    pub fn list_wallet_ids(&self) -> StorageResult<Vec<String>>
    
    /// Delete wallet from storage
    pub fn delete_wallet(&self, id: &str) -> StorageResult<()>
    
    /// Create backup of wallet
    pub fn backup_wallet(&self, id: &str) -> StorageResult<PathBuf>
}
```

### WalletData

```rust
/// Encrypted wallet data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    /// Unique wallet identifier
    pub id: String,
    /// Bitcoin network
    pub network: Network,
    /// Encrypted master seed
    pub encrypted_seed: Vec<u8>,
    /// Wallet metadata
    pub metadata: WalletMetadata,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
}
```

### Legacy Import

```rust
/// Import legacy Armory wallet file
pub fn import_armory_wallet(
    file_path: &Path,
    passphrase: Option<&str>,
    storage: &WalletStorage,
) -> StorageResult<ImportResult>

/// Result of legacy wallet import
#[derive(Debug)]
pub struct ImportResult {
    pub wallet_id: String,
    pub addresses_imported: usize,
    pub transactions_imported: usize,
    pub warnings: Vec<String>,
}
```

## 🏦 Wallet Module

### Wallet

```rust
/// Modern descriptor-based HD wallet
pub struct Wallet {
    pub id: String,
    pub network: Network,
    // Private fields...
}

impl Wallet {
    /// Create a new wallet with random master key
    pub fn create_new(
        id: String, 
        network: Network, 
        storage: WalletStorage
    ) -> WalletResult<Self>
    
    /// Load existing wallet from storage
    pub fn load(id: String, storage: WalletStorage) -> WalletResult<Self>
    
    /// Save wallet to storage
    pub fn save(&self) -> WalletResult<()>
    
    /// Generate a new receiving address
    pub fn get_new_address(&mut self, address_type: AddressType) -> WalletResult<Address>
    
    /// Generate a change address
    pub fn get_change_address(&mut self, address_type: AddressType) -> WalletResult<Address>
    
    /// Get all addresses for a specific type
    pub fn get_addresses(&self, address_type: AddressType, change: bool) -> Vec<Address>
    
    /// Get total balance in satoshis
    pub fn balance(&self) -> u64
    
    /// Get confirmed balance
    pub fn confirmed_balance(&self) -> u64
    
    /// Get unconfirmed balance
    pub fn unconfirmed_balance(&self) -> u64
    
    /// Get spendable UTXOs
    pub fn spendable_utxos(&self, min_confirmations: u32, current_height: u32) -> Vec<&Utxo>
    
    /// Add a UTXO to the wallet
    pub fn add_utxo(&mut self, utxo: Utxo)
    
    /// Remove a spent UTXO
    pub fn remove_utxo(&mut self, txid: Txid, vout: u32) -> Option<Utxo>
    
    /// Check if an address belongs to this wallet
    pub fn owns_address(&self, address: &Address) -> Option<DerivationPath>
}
```

### UTXO

```rust
/// Unspent Transaction Output information
#[derive(Debug, Clone)]
pub struct Utxo {
    /// Transaction ID containing this output
    pub txid: Txid,
    /// Output index within the transaction
    pub vout: u32,
    /// Value in satoshis
    pub value: u64,
    /// Script pubkey
    pub script_pubkey: ScriptBuf,
    /// Address that owns this UTXO
    pub address: Address,
    /// Derivation path for this address
    pub derivation_path: DerivationPath,
    /// Block height when confirmed (None if unconfirmed)
    pub block_height: Option<u32>,
}
```

## 💳 Transaction Module

### TransactionBuilder

```rust
/// Transaction builder with PSBT v2 support
pub struct TransactionBuilder {
    // Private fields...
}

impl TransactionBuilder {
    /// Create new transaction builder
    pub fn new(wallet: Arc<RwLock<Wallet>>) -> TransactionResult<Self>
    
    /// Create transaction builder with custom configuration
    pub fn with_config(
        wallet: Arc<RwLock<Wallet>>, 
        config: BuilderConfig
    ) -> TransactionResult<Self>
    
    /// Add recipient output
    pub fn add_recipient(
        &mut self, 
        address: Address, 
        amount: Amount
    ) -> TransactionResult<&mut Self>
    
    /// Set fee strategy
    pub fn fee_strategy(&mut self, strategy: FeeStrategy) -> &mut Self
    
    /// Set coin selection strategy
    pub fn coin_selection(&mut self, strategy: CoinSelectionStrategy) -> &mut Self
    
    /// Enable or disable RBF
    pub fn rbf(&mut self, enable: bool) -> &mut Self
    
    /// Set explicit locktime
    pub fn locktime(&mut self, locktime: LockTime) -> &mut Self
    
    /// Estimate transaction fee
    pub fn estimate_fee(&mut self) -> TransactionResult<Amount>
    
    /// Select UTXOs for the transaction
    pub fn select_utxos(&mut self) -> TransactionResult<&mut Self>
    
    /// Build the PSBT v2
    pub fn build_psbt(&mut self) -> TransactionResult<PsbtV2>
    
    /// Get total input value
    pub fn total_input_value(&self) -> Amount
    
    /// Get total output value
    pub fn total_output_value(&self) -> Amount
    
    /// Get current fee estimate
    pub fn fee(&self) -> Option<Amount>
    
    /// Get change amount
    pub fn change_amount(&self) -> Option<Amount>
}
```

### BuilderConfig

```rust
/// Transaction builder configuration
#[derive(Debug, Clone)]
pub struct BuilderConfig {
    /// Fee estimation strategy
    pub fee_strategy: FeeStrategy,
    /// Coin selection strategy
    pub coin_selection: CoinSelectionStrategy,
    /// Enable RBF by default
    pub enable_rbf: bool,
    /// Minimum number of confirmations for UTXOs
    pub min_confirmations: u32,
    /// Target number of confirmations
    pub target_confirmations: u32,
    /// Maximum fee rate to prevent fee overpayment
    pub max_fee_rate: FeeRate,
}
```

### Fee Strategies

```rust
/// Fee estimation strategy
#[derive(Debug, Clone, Copy)]
pub enum FeeStrategy {
    /// Use specified fee rate (sat/vB)
    FeeRate(FeeRate),
    /// Target confirmation within N blocks
    ConfirmationTarget(u32),
    /// Low priority (slower confirmation, lower fee)
    LowPriority,
    /// Normal priority (moderate confirmation, moderate fee)
    Normal,
    /// High priority (fast confirmation, higher fee)
    HighPriority,
}

/// Coin selection strategy
#[derive(Debug, Clone, Copy)]
pub enum CoinSelectionStrategy {
    /// Largest first (minimizes change)
    LargestFirst,
    /// Smallest first (minimizes fees)
    SmallestFirst,
    /// Branch and bound (optimal selection)
    BranchAndBound,
    /// Random selection (enhances privacy)
    Random,
}
```

### PSBT v2

```rust
/// PSBT v2 structure implementing BIP-370
#[derive(Debug, Clone)]
pub struct PsbtV2 {
    /// PSBT version (2 for BIP-370)
    pub version: u8,
    /// Transaction inputs with PSBT input data
    pub inputs: Vec<PsbtV2Input>,
    /// Transaction outputs with PSBT output data
    pub outputs: Vec<PsbtV2Output>,
    /// Global PSBT fields
    pub global_fields: HashMap<Vec<u8>, Vec<u8>>,
    /// Fallback locktime for the transaction
    pub fallback_locktime: Option<u32>,
    /// Input count (required in v2)
    pub input_count: u32,
    /// Output count (required in v2)
    pub output_count: u32,
}

impl PsbtV2 {
    /// Create new empty PSBT v2
    pub fn new() -> TransactionResult<Self>
    
    /// Create PSBT v2 from transaction template
    pub fn from_tx_template(
        inputs: Vec<(Txid, u32)>,
        outputs: Vec<(ScriptBuf, Amount)>,
        locktime: Option<u32>,
    ) -> TransactionResult<Self>
    
    /// Add input to PSBT v2
    pub fn add_input(
        &mut self,
        previous_txid: Txid,
        previous_output_index: u32,
        sequence: Option<u32>,
    ) -> TransactionResult<()>
    
    /// Add output to PSBT v2
    pub fn add_output(&mut self, amount: Amount, script: ScriptBuf) -> TransactionResult<()>
    
    /// Set witness UTXO for input
    pub fn set_witness_utxo(&mut self, input_index: usize, utxo: TxOut) -> TransactionResult<()>
    
    /// Calculate transaction fee
    pub fn fee(&self) -> TransactionResult<Amount>
    
    /// Check if PSBT is ready for finalization
    pub fn is_ready_for_finalization(&self) -> bool
    
    /// Finalize PSBT to create final transaction
    pub fn finalize(&self) -> TransactionResult<Transaction>
}
```

## 📜 Script Module

### ScriptEngine

```rust
/// Script execution engine with modern Bitcoin features
pub struct ScriptEngine {
    // Private fields...
}

impl ScriptEngine {
    /// Create new script engine
    pub fn new() -> Self
    
    /// Validate a script for a transaction input
    pub fn validate_script(
        &mut self,
        context: &ScriptContext,
        script_pubkey: &ScriptBuf,
        script_sig: Option<&ScriptBuf>,
        witness: Option<&bitcoin::Witness>,
    ) -> TransactionResult<ValidationResult>
    
    /// Parse and cache a miniscript descriptor
    pub fn parse_descriptor(&mut self, descriptor: &str) -> TransactionResult<Descriptor<PublicKey>>
    
    /// Generate script from miniscript descriptor
    pub fn descriptor_to_script(&mut self, descriptor: &str) -> TransactionResult<ScriptBuf>
    
    /// Check if a descriptor is satisfiable
    pub fn is_satisfiable(&mut self, descriptor: &str) -> TransactionResult<bool>
}
```

### ScriptDescriptor

```rust
/// Script descriptor wrapper with additional metadata
#[derive(Debug, Clone)]
pub struct ScriptDescriptor {
    /// The descriptor type
    pub descriptor_type: DescriptorType,
    /// Raw descriptor string
    pub descriptor: String,
    /// Network for address generation
    pub network: Network,
}

impl ScriptDescriptor {
    /// Create a new script descriptor
    pub fn new(
        descriptor_type: DescriptorType, 
        descriptor: String, 
        network: Network
    ) -> TransactionResult<Self>
    
    /// Create a Legacy P2PKH descriptor
    pub fn legacy(pubkey: &PublicKey, network: Network) -> TransactionResult<Self>
    
    /// Create a SegWit P2WPKH descriptor
    pub fn wpkh(pubkey: &PublicKey, network: Network) -> TransactionResult<Self>
    
    /// Create a Taproot descriptor with key-only spending
    pub fn taproot_key_only(pubkey: &PublicKey, network: Network) -> TransactionResult<Self>
    
    /// Generate the script pubkey for this descriptor
    pub fn script_pubkey(&mut self) -> TransactionResult<ScriptBuf>
    
    /// Generate an address for this descriptor
    pub fn address(&mut self) -> TransactionResult<Address>
    
    /// Get maximum witness size for fee estimation
    pub fn max_witness_size(&mut self) -> TransactionResult<usize>
}
```

### TaprootSpendData

```rust
/// Taproot spending data container
#[derive(Debug, Clone)]
pub struct TaprootSpendData {
    /// The internal key used for key-path spending
    pub internal_key: XOnlyPublicKey,
    /// The tweaked output key
    pub output_key: TweakedPublicKey,
    /// Script tree information
    pub script_tree: Option<TapTree>,
    /// Control blocks for script path spending
    pub control_blocks: HashMap<ScriptBuf, ControlBlock>,
}

impl TaprootSpendData {
    /// Create new Taproot spend data for key-only spending
    pub fn key_only(internal_key: XOnlyPublicKey) -> TransactionResult<Self>
    
    /// Create Taproot spend data with script tree
    pub fn with_script_tree(
        internal_key: XOnlyPublicKey,
        script_tree: TapTree,
    ) -> TransactionResult<Self>
    
    /// Get control block for a specific script
    pub fn control_block(&self, script: &ScriptBuf) -> Option<&ControlBlock>
    
    /// Check if this can be spent via key path
    pub fn supports_key_path(&self) -> bool
    
    /// Check if this can be spent via script path
    pub fn supports_script_path(&self) -> bool
}
```

### WitnessGenerator

```rust
/// Witness generator for transaction inputs
pub struct WitnessGenerator {
    // Private fields...
}

impl WitnessGenerator {
    /// Create new witness generator
    pub fn new() -> Self
    
    /// Generate witness for a transaction input
    pub fn generate_witness(
        &self,
        transaction: &Transaction,
        input_index: usize,
        prevout: &TxOut,
        witness_type: &WitnessType,
        signing_data: &SigningData,
    ) -> TransactionResult<Witness>
    
    /// Estimate witness size for fee calculation
    pub fn estimate_witness_size(witness_type: &WitnessType) -> usize
}
```

## 📝 Examples

### Creating a New Wallet

```rust
use armory_rust::{Wallet, Network, StorageConfig, WalletStorage};
use std::path::PathBuf;

// Set up storage
let config = StorageConfig {
    storage_path: PathBuf::from("./wallets"),
    auto_backup: true,
    backup_count: 3,
};
let storage = WalletStorage::new(config)?;

// Create new wallet
let wallet = Wallet::create_new(
    "my-wallet".to_string(),
    Network::Bitcoin,
    storage
)?;

println!("Created wallet: {}", wallet.id);
```

### Generating Addresses

```rust
use armory_rust::AddressType;

// Generate different address types
let legacy_addr = wallet.get_new_address(AddressType::Legacy)?;
let segwit_addr = wallet.get_new_address(AddressType::NativeSegwit)?;
let taproot_addr = wallet.get_new_address(AddressType::Taproot)?;

println!("Legacy: {}", legacy_addr);
println!("SegWit: {}", segwit_addr);
println!("Taproot: {}", taproot_addr);
```

### Building a Transaction

```rust
use armory_rust::{TransactionBuilder, FeeStrategy, CoinSelectionStrategy};
use bitcoin::{Address, Amount};
use std::sync::{Arc, RwLock};

// Set up transaction builder
let wallet_ref = Arc::new(RwLock::new(wallet));
let mut builder = TransactionBuilder::new(wallet_ref)?
    .fee_strategy(FeeStrategy::Normal)
    .coin_selection(CoinSelectionStrategy::BranchAndBound)
    .rbf(true);

// Add recipient
let recipient = Address::from_str("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")?;
builder.add_recipient(recipient, Amount::from_sat(100_000))?;

// Build PSBT
let psbt = builder.build_psbt()?;
println!("Created PSBT with {} inputs", psbt.inputs.len());
```

### Script Validation

```rust
use armory_rust::{ScriptEngine, ScriptContext};

let mut engine = ScriptEngine::new();

// Validate a P2WPKH script
let context = ScriptContext {
    transaction: tx.clone(),
    input_index: 0,
    prevout: utxo.clone(),
    block_height: 800_000,
    block_time: 1640995200,
};

let result = engine.validate_script(
    &context,
    &script_pubkey,
    None,
    Some(&witness),
)?;

match result {
    ValidationResult::Valid => println!("Script validation successful"),
    ValidationResult::Invalid(reason) => println!("Validation failed: {}", reason),
    ValidationResult::IncompleteSignatures => println!("Missing signatures"),
}
```

### Working with Descriptors

```rust
use armory_rust::{ScriptDescriptor, DescriptorType};

// Create a Taproot descriptor
let pubkey = wallet.get_public_key(&derivation_path)?;
let descriptor = ScriptDescriptor::taproot_key_only(&pubkey, Network::Bitcoin)?;

// Generate address from descriptor
let address = descriptor.address()?;
println!("Taproot address: {}", address);

// Estimate transaction size
let descriptor_types = vec![DescriptorType::Taproot, DescriptorType::Wpkh];
let tx_size = DescriptorUtils::estimate_tx_size(&descriptor_types, 2)?;
println!("Estimated transaction size: {} bytes", tx_size);
```

---

This API reference provides comprehensive documentation for all public interfaces in the Armory Rust implementation. For more detailed examples and usage patterns, see the integration tests and examples directory.