/// Legacy Armory wallet migration module
///
/// This module handles importing legacy Armory wallets (Python/C++ implementation)
/// and converting them to the modern Rust descriptor-based format.
pub mod legacy_import;

pub use legacy_import::{ImportError, LegacyWalletData, LegacyWalletImporter};

#[cfg(test)]
pub mod tests;
