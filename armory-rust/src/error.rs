/// Comprehensive error types for the Armory wallet
/// 
/// Based on the error handling strategy from the PRP, this provides
/// a unified error system that replaces the inconsistent error handling
/// in the legacy Python/C++ codebase.

use thiserror::Error;

/// Main wallet error type with comprehensive error variants
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Cryptographic operation failed: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Network communication error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Storage operation failed: {0}")]  
    Storage(#[from] StorageError),
    
    #[error("Legacy wallet import failed: {0}")]
    LegacyImport(String),
    
    #[error("Hardware wallet error: {0}")]
    Hardware(String),
    
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    
    #[error("Key generation failed")]
    KeyGeneration,
    
    #[error("Key derivation failed")]
    KeyDerivation,
    
    #[error("Address generation failed")]
    AddressGeneration,
    
    #[error("Feature not yet implemented: {0}")]
    NotImplemented(String),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Cryptographic operation errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),
    
    #[error("Encryption failed: {0}")]
    Encryption(String),
    
    #[error("Decryption failed: {0}")]
    Decryption(String),
    
    #[error("Invalid private key")]
    InvalidPrivateKey,
    
    #[error("Invalid public key")]
    InvalidPublicKey,
    
    #[error("Signature verification failed")]
    SignatureVerification,
    
    #[error("Random number generation failed")]
    RandomGeneration,
    
    #[error("ECDSA error: {0}")]
    Ecdsa(#[from] secp256k1::Error),
}

/// Network communication errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    Connection(String),
    
    #[error("Timeout occurred")]
    Timeout,
    
    #[error("Invalid peer response: {0}")]
    InvalidResponse(String),
    
    #[error("P2P protocol error: {0}")]
    Protocol(String),
    
    #[error("BIP-324 handshake failed: {0}")]
    Bip324Handshake(String),
    
    #[error("RPC error: {0}")]
    Rpc(String),
}

/// Storage operation errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Wallet file corrupted")]
    WalletCorrupted,
    
    #[error("Backup failed: {0}")]
    BackupFailed(String),
    
    #[error("Atomic update failed: {0}")]
    AtomicUpdateFailed(String),
}

/// Transaction processing errors
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Insufficient funds: available {available}, required {required}")]
    InsufficientFunds { available: u64, required: u64 },
    
    #[error("Invalid PSBT: {0}")]
    InvalidPsbt(String),
    
    #[error("Fee estimation failed: {0}")]
    FeeEstimation(String),
    
    #[error("Coin selection failed: {0}")]
    CoinSelection(String),
    
    #[error("Script validation failed: {0}")]
    ScriptValidation(String),
    
    #[error("Bitcoin core error: {0}")]
    Bitcoin(#[from] bitcoin::consensus::encode::Error),
}

/// Result type for all wallet operations
pub type WalletResult<T> = Result<T, WalletError>;

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Result type for storage operations  
pub type StorageResult<T> = Result<T, StorageError>;

/// Result type for transaction operations
pub type TransactionResult<T> = Result<T, TransactionError>;

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;