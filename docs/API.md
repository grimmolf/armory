# API Reference

This document provides comprehensive API documentation for the Armory Rust implementation, including programmatic interfaces and CLI commands.

## 📖 Table of Contents

- [Rust Library API](#rust-library-api)
- [CLI Interface](#cli-interface)
- [JSON-RPC API](#json-rpc-api)
- [Configuration API](#configuration-api)
- [Error Handling](#error-handling)

## 🦀 Rust Library API

### Core Types

#### Network

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Bitcoin,    // Mainnet
    Testnet,    // Bitcoin testnet
    Regtest,    // Regression testing
    Signet,     // Signet test network
}
```

#### Address Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressType {
    Legacy,        // P2PKH (1...)
    NestedSegwit,  // P2SH-P2WPKH (3...)
    NativeSegwit,  // P2WPKH (bc1q...)
    Taproot,       // P2TR (bc1p...)
}
```

### Wallet API

#### Creating Wallets

```rust
use armory_rust::wallet::{Wallet, AddressType};
use armory_rust::storage::{WalletStorage, StorageConfig};
use armory_rust::Network;

// Create storage configuration
let config = StorageConfig {
    storage_path: PathBuf::from("~/.armory-rust"),
    auto_backup: true,
    backup_count: 5,
};

// Initialize storage
let storage = WalletStorage::new(config)?;

// Create new wallet
let wallet = Wallet::create_new(
    "my-wallet".to_string(),
    Network::Bitcoin,
    storage
)?;
```

#### Address Generation

```rust
// Generate receiving addresses
let legacy_addr = wallet.get_new_address(AddressType::Legacy)?;
let segwit_addr = wallet.get_new_address(AddressType::NativeSegwit)?;
let taproot_addr = wallet.get_new_address(AddressType::Taproot)?;

// Generate change address
let change_addr = wallet.get_change_address(AddressType::NativeSegwit)?;

// Get all addresses of a specific type
let all_segwit = wallet.get_addresses(AddressType::NativeSegwit, false); // receiving
let all_change = wallet.get_addresses(AddressType::NativeSegwit, true);  // change
```

#### UTXO Management

```rust
use armory_rust::wallet::{Utxo, WalletTransaction};
use bitcoin::{Txid, Address, ScriptBuf};

// Add UTXO to wallet
let utxo = Utxo {
    txid: Txid::from_str("1234...")?,
    vout: 0,
    value: 100_000, // satoshis
    script_pubkey: ScriptBuf::new(),
    address: address.clone(),
    derivation_path: path,
    block_height: Some(700_000),
};
wallet.add_utxo(utxo);

// Check balances
let total_balance = wallet.balance();
let confirmed_balance = wallet.confirmed_balance();
let unconfirmed_balance = wallet.unconfirmed_balance();

// Get spendable UTXOs
let spendable = wallet.spendable_utxos(6, current_block_height); // 6 confirmations
```

### Cryptography API

#### Key Derivation

```rust
use armory_rust::crypto::{derive_key_from_password, KdfParams};

// Create KDF parameters
let params = KdfParams {
    memory_cost: 65536,  // 64 MiB
    time_cost: 3,        // 3 iterations
    parallelism: 4,      // 4 threads
};

// Derive key from password
let salt = generate_salt()?;
let derived_key = derive_key_from_password("password", &salt, &params)?;
```

#### Encryption

```rust
use armory_rust::crypto::{encrypt_data, decrypt_data, SecureKey, EncryptedData};

// Create encryption key
let key = SecureKey::generate()?;

// Encrypt data
let plaintext = b"sensitive data";
let encrypted = encrypt_data(&key, plaintext, None)?;

// Decrypt data
let decrypted = decrypt_data(&key, &encrypted, None)?;
assert_eq!(decrypted, plaintext);
```

#### Digital Signatures

```rust
use armory_rust::crypto::{sign_schnorr, verify_schnorr, PrivateKey};

// Generate private key
let private_key = PrivateKey::generate()?;

// Sign message with Schnorr signature
let message = b"message to sign";
let signature = sign_schnorr(message, &private_key)?;

// Verify signature
let public_key = private_key.public_key();
let is_valid = verify_schnorr(message, &signature, &public_key)?;
```

### Storage API

#### Wallet Storage

```rust
use armory_rust::storage::{WalletStorage, StorageConfig, WalletData};

// Initialize storage
let config = StorageConfig {
    storage_path: PathBuf::from("/path/to/storage"),
    auto_backup: true,
    backup_count: 3,
};
let storage = WalletStorage::new(config)?;

// Store wallet data
let wallet_data = WalletData {
    id: "wallet-id".to_string(),
    network: Network::Bitcoin,
    encrypted_seed: encrypted_seed_data,
    metadata: metadata,
};
storage.save_wallet_data(&wallet_data)?;

// Load wallet data
let loaded_data = storage.load_wallet_data("wallet-id")?;
```

#### Legacy Import

```rust
use armory_rust::storage::legacy_import::{import_armory_wallet, ImportResult};

// Import legacy Armory wallet
let import_result = import_armory_wallet(
    Path::new("armory_ABC123.wallet"),
    Some("passphrase"),  // Optional passphrase
    &storage
)?;

match import_result {
    ImportResult::Success { wallet_id, address_count, .. } => {
        println!("Imported wallet {} with {} addresses", wallet_id, address_count);
    }
    ImportResult::PartialSuccess { issues, .. } => {
        println!("Import completed with issues: {:?}", issues);
    }
}
```

### Error Handling

```rust
use armory_rust::error::{WalletError, WalletResult};

// All fallible operations return Result types
fn example_operation() -> WalletResult<String> {
    // Operation that might fail
    let wallet = Wallet::create_new("test".to_string(), Network::Regtest, storage)
        .map_err(|e| WalletError::Storage(e))?;
    
    Ok("success".to_string())
}

// Error matching
match example_operation() {
    Ok(result) => println!("Success: {}", result),
    Err(WalletError::Crypto(e)) => println!("Crypto error: {}", e),
    Err(WalletError::Storage(e)) => println!("Storage error: {}", e),
    Err(WalletError::Network(e)) => println!("Network error: {}", e),
    Err(e) => println!("Other error: {}", e),
}
```

## 💻 CLI Interface

### Wallet Management

#### Create Wallet

```bash
# Create new wallet
cargo run -- create <wallet-name> [OPTIONS]

OPTIONS:
    --network <NETWORK>      Bitcoin network [default: bitcoin] [possible values: bitcoin, testnet, regtest, signet]
    --passphrase             Set wallet passphrase
    --mnemonic-words <N>     Mnemonic word count [default: 24] [possible values: 12, 15, 18, 21, 24]
    --derivation <PATH>      Custom derivation path
```

**Examples:**
```bash
# Create mainnet wallet
cargo run -- create "my-wallet" --network bitcoin

# Create testnet wallet with passphrase
cargo run -- create "test-wallet" --network testnet --passphrase

# Create wallet with 12-word mnemonic
cargo run -- create "simple-wallet" --mnemonic-words 12
```

#### List Wallets

```bash
# List all wallets
cargo run -- list [OPTIONS]

OPTIONS:
    --format <FORMAT>    Output format [default: table] [possible values: table, json, csv]
    --show-balance       Include balance information
    --network <NETWORK>  Filter by network
```

**Example output:**
```
┌─────────────┬─────────┬──────────┬───────────┬─────────────────┐
│ Name        │ Network │ Balance  │ Addresses │ Last Activity   │
├─────────────┼─────────┼──────────┼───────────┼─────────────────┤
│ main-wallet │ bitcoin │ 0.05 BTC │ 127       │ 2024-01-15 14:30│
│ test-wallet │ testnet │ 1.23 BTC │ 45        │ 2024-01-14 09:15│
└─────────────┴─────────┴──────────┴───────────┴─────────────────┘
```

#### Wallet Information

```bash
# Get wallet details
cargo run -- info <wallet-name> [OPTIONS]

OPTIONS:
    --show-addresses     Include address list
    --show-utxos        Include UTXO details
    --show-transactions Include transaction history
    --format <FORMAT>   Output format [default: human] [possible values: human, json]
```

### Address Operations

#### Generate Address

```bash
# Generate new receiving address
cargo run -- address <wallet-name> [OPTIONS]

OPTIONS:
    --type <TYPE>        Address type [default: native-segwit] [possible values: legacy, nested-segwit, native-segwit, taproot]
    --label <LABEL>      Address label
    --change             Generate change address
    --index <INDEX>      Specific derivation index
```

**Examples:**
```bash
# Generate native SegWit address
cargo run -- address "my-wallet" --type native-segwit

# Generate Taproot address with label
cargo run -- address "my-wallet" --type taproot --label "Savings"

# Generate change address
cargo run -- address "my-wallet" --change
```

#### List Addresses

```bash
# List wallet addresses
cargo run -- addresses <wallet-name> [OPTIONS]

OPTIONS:
    --type <TYPE>        Filter by address type
    --change             Show change addresses
    --used               Show only used addresses
    --unused             Show only unused addresses
    --limit <N>          Limit number of results
    --format <FORMAT>    Output format [default: table] [possible values: table, json, csv]
```

### Balance and UTXO Operations

#### Check Balance

```bash
# Check wallet balance
cargo run -- balance <wallet-name> [OPTIONS]

OPTIONS:
    --confirmed          Show only confirmed balance
    --unconfirmed        Show only unconfirmed balance
    --by-address         Group by address
    --min-confirmations <N>  Minimum confirmations [default: 1]
    --format <FORMAT>    Output format [default: human] [possible values: human, json]
```

**Example output:**
```
Wallet Balance: my-wallet
═══════════════════════════
Confirmed:     0.05234567 BTC
Unconfirmed:   0.00100000 BTC
Total:         0.05334567 BTC

UTXOs: 3 confirmed, 1 unconfirmed
```

#### List UTXOs

```bash
# List unspent transaction outputs
cargo run -- utxos <wallet-name> [OPTIONS]

OPTIONS:
    --min-value <SATS>   Minimum UTXO value in satoshis
    --max-value <SATS>   Maximum UTXO value in satoshis
    --confirmed          Show only confirmed UTXOs
    --spendable          Show only spendable UTXOs
    --min-confirmations <N>  Minimum confirmations [default: 1]
```

### Transaction Operations

#### Send Transaction

```bash
# Send Bitcoin transaction
cargo run -- send <wallet-name> [OPTIONS]

OPTIONS:
    --to <ADDRESS>       Recipient address
    --amount <AMOUNT>    Amount to send (BTC or satoshis)
    --fee-rate <RATE>    Fee rate in sat/vB
    --rbf                Enable Replace-by-Fee
    --subtract-fee       Subtract fee from amount
    --message <MSG>      Transaction message/memo
```

**Examples:**
```bash
# Send 0.001 BTC
cargo run -- send "my-wallet" --to "bc1q..." --amount 0.001

# Send with custom fee rate
cargo run -- send "my-wallet" --to "bc1q..." --amount 100000 --fee-rate 20

# Send with RBF enabled
cargo run -- send "my-wallet" --to "bc1q..." --amount 0.001 --rbf
```

#### Transaction History

```bash
# Show transaction history
cargo run -- history <wallet-name> [OPTIONS]

OPTIONS:
    --limit <N>          Number of transactions to show
    --since <DATE>       Show transactions since date
    --until <DATE>       Show transactions until date
    --incoming           Show only incoming transactions
    --outgoing           Show only outgoing transactions
    --format <FORMAT>    Output format [default: table] [possible values: table, json, csv]
```

### Import and Export

#### Import Legacy Wallet

```bash
# Import legacy Armory wallet
cargo run -- import legacy <source-file> [OPTIONS]

OPTIONS:
    --name <NAME>        New wallet name
    --passphrase         Wallet is encrypted (will prompt for passphrase)
    --preserve-labels    Import address labels
    --verify             Verify import integrity
    --backup-original    Create backup of original file
```

#### Export Wallet

```bash
# Export wallet data
cargo run -- export <wallet-name> [OPTIONS]

OPTIONS:
    --format <FORMAT>    Export format [possible values: json, csv, private-keys, mnemonic]
    --output <FILE>      Output file (default: stdout)
    --include-private    Include private keys (dangerous)
    --password           Encrypt exported data
```

### Backup and Recovery

#### Create Backup

```bash
# Create wallet backup
cargo run -- backup <wallet-name> [OPTIONS]

OPTIONS:
    --output <FILE>      Backup file path
    --encrypt            Encrypt backup file
    --verify             Verify backup integrity
    --include-metadata   Include labels and transaction history
```

#### Restore from Backup

```bash
# Restore wallet from backup
cargo run -- restore <backup-file> [OPTIONS]

OPTIONS:
    --name <NAME>        Restored wallet name
    --verify             Verify restore integrity
    --decrypt            Backup is encrypted (will prompt for password)
```

## 🌐 JSON-RPC API

### Server Configuration

```toml
# ~/.armory-rust/config.toml
[rpc]
enabled = true
bind_address = "127.0.0.1"
port = 8444
username = "armory"
password = "secure_password"
ssl_cert = "/path/to/cert.pem"
ssl_key = "/path/to/key.pem"
```

### Authentication

```bash
# HTTP Basic Authentication
curl -u armory:secure_password \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getwalletinfo", "params": ["my-wallet"], "id": 1}' \
  http://localhost:8444/
```

### Available Methods

#### Wallet Information

```json
// Get wallet information
{
  "jsonrpc": "2.0",
  "method": "getwalletinfo",
  "params": ["wallet-name"],
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "wallet_name": "my-wallet",
    "network": "bitcoin",
    "balance": 5234567,
    "confirmed_balance": 5134567,
    "unconfirmed_balance": 100000,
    "address_count": 127,
    "utxo_count": 3
  },
  "id": 1
}
```

#### Address Generation

```json
// Generate new address
{
  "jsonrpc": "2.0",
  "method": "getnewaddress",
  "params": {
    "wallet": "my-wallet",
    "address_type": "native-segwit",
    "label": "Payment #123"
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "address": "bc1q...",
    "derivation_path": "m/84'/0'/0'/0/127",
    "address_type": "native-segwit"
  },
  "id": 2
}
```

#### Send Transaction

```json
// Send transaction
{
  "jsonrpc": "2.0",
  "method": "sendtoaddress",
  "params": {
    "wallet": "my-wallet",
    "address": "bc1q...",
    "amount": 100000,
    "fee_rate": 20,
    "rbf": true
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "txid": "abc123...",
    "fee": 4400,
    "size": 220,
    "virtual_size": 140
  },
  "id": 3
}
```

### Error Responses

```json
// Error response format
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Wallet 'nonexistent' not found"
  },
  "id": 1
}
```

**Error Codes:**
- `-32700`: Parse error
- `-32600`: Invalid request  
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-1`: Wallet error
- `-2`: Crypto error
- `-3`: Network error

## ⚙️ Configuration API

### Configuration File Format

```toml
# ~/.armory-rust/config.toml

[wallet]
default_network = "bitcoin"
data_directory = "~/.armory-rust"
auto_backup = true
backup_count = 5
session_timeout = 3600

[network]
bitcoin_rpc_url = "http://localhost:8332"
bitcoin_rpc_user = "bitcoin"
bitcoin_rpc_password = "password"
use_tor = false
tor_proxy = "127.0.0.1:9050"

[security]
require_passphrase = true
passphrase_timeout = 300
memory_lock = true
secure_delete = true

[rpc]
enabled = false
bind_address = "127.0.0.1"
port = 8444
username = "armory"
password = "changeme"

[logging]
level = "info"
file = "~/.armory-rust/logs/armory.log"
max_size = "100MB"
max_files = 5
```

### CLI Configuration

```bash
# Get configuration value
cargo run -- config get wallet.auto_backup

# Set configuration value
cargo run -- config set wallet.auto_backup true

# List all configuration
cargo run -- config list

# Reset to defaults
cargo run -- config reset
```

## 🔧 Error Handling

### Error Types

```rust
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Cryptographic operation failed: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Network communication error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Storage operation failed: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Key generation failed")]
    KeyGeneration,
    
    #[error("Address generation failed")]
    AddressGeneration,
    
    #[error("Feature not yet implemented: {0}")]
    NotImplemented(String),
}
```

### CLI Error Codes

| Exit Code | Description |
|-----------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Wallet not found |
| 4 | Authentication failed |
| 5 | Network error |
| 6 | Storage error |
| 7 | Crypto error |

### Error Examples

```bash
# CLI error with details
$ cargo run -- send "nonexistent" --to "bc1q..." --amount 0.001
Error: Wallet 'nonexistent' not found
Available wallets: main-wallet, test-wallet

# JSON-RPC error
$ curl -u user:pass -H "Content-Type: application/json" \
  -d '{"method": "invalid_method"}' http://localhost:8444/
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": "Available methods: getwalletinfo, getnewaddress, sendtoaddress, ..."
  },
  "id": null
}
```

## 📚 Usage Examples

### Complete Wallet Workflow

```rust
use armory_rust::prelude::*;

async fn main() -> WalletResult<()> {
    // 1. Initialize storage
    let config = StorageConfig::default();
    let storage = WalletStorage::new(config)?;
    
    // 2. Create wallet
    let mut wallet = Wallet::create_new(
        "example-wallet".to_string(),
        Network::Regtest,
        storage
    )?;
    
    // 3. Generate addresses
    let receive_addr = wallet.get_new_address(AddressType::NativeSegwit)?;
    println!("Send coins to: {}", receive_addr);
    
    // 4. Add received UTXO (simulated)
    let utxo = Utxo {
        txid: Txid::from_str("abc123...")?,
        vout: 0,
        value: 100_000,
        script_pubkey: receive_addr.script_pubkey(),
        address: receive_addr,
        derivation_path: AddressType::NativeSegwit.derivation_path(0, 0, 0),
        block_height: Some(800_000),
    };
    wallet.add_utxo(utxo);
    
    // 5. Check balance
    println!("Balance: {} satoshis", wallet.balance());
    
    // 6. Create transaction (when implemented)
    // let transaction = wallet.create_transaction(
    //     recipient_address,
    //     amount,
    //     fee_rate
    // )?;
    
    Ok(())
}
```

This API reference provides the foundation for integrating with and extending the Armory Rust implementation. For more examples and detailed usage, see the [examples/](../examples/) directory in the source repository.