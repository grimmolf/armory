/// Core wallet module
///
/// This module implements the main wallet functionality using modern
/// descriptor-based design patterns.
pub mod descriptor_wallet;
pub mod hd_wallet;

// Re-exports for convenience
pub use descriptor_wallet::Wallet;
pub use hd_wallet::HdWallet;
