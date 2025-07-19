pub mod legacy_import;
/// Storage layer module
///
/// This module provides encrypted storage functionality to replace
/// the legacy wallet file format with modern standards.
pub mod wallet_storage;

// Re-exports for convenience
pub use legacy_import::import_armory_wallet;
pub use wallet_storage::{StorageConfig, WalletStorage};
