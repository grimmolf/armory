/// Storage layer module
/// 
/// This module provides encrypted storage functionality to replace
/// the legacy wallet file format with modern standards.

pub mod wallet_storage;
pub mod legacy_import;

// Re-exports for convenience
pub use wallet_storage::{WalletStorage, StorageConfig};
pub use legacy_import::import_armory_wallet;