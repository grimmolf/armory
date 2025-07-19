/// Modern Rust implementation of Armory Bitcoin wallet
/// 
/// This library provides a comprehensive Bitcoin wallet implementation that
/// modernizes the legacy Armory wallet while maintaining feature parity
/// and adding support for modern Bitcoin protocols.

pub mod error;
pub mod crypto;
pub mod storage;
pub mod wallet;
pub mod transaction;
pub mod script;
pub mod network;
pub mod cli;

// Re-export common types for convenience
pub use error::{WalletError, WalletResult};
pub use wallet::Wallet;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Bitcoin network types supported by the wallet
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

impl From<Network> for bitcoin::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Bitcoin => bitcoin::Network::Bitcoin,
            Network::Testnet => bitcoin::Network::Testnet, 
            Network::Signet => bitcoin::Network::Signet,
            Network::Regtest => bitcoin::Network::Regtest,
        }
    }
}

impl From<bitcoin::Network> for Network {
    fn from(network: bitcoin::Network) -> Self {
        match network {
            bitcoin::Network::Bitcoin => Network::Bitcoin,
            bitcoin::Network::Testnet => Network::Testnet,
            bitcoin::Network::Signet => Network::Signet,
            bitcoin::Network::Regtest => Network::Regtest,
            _ => Network::Bitcoin, // Default fallback for new network types
        }
    }
}

/// Initialize logging for the wallet
pub fn init_logging() {
    tracing_subscriber::fmt::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_conversion() {
        assert_eq!(
            bitcoin::Network::Bitcoin,
            Network::Bitcoin.into()
        );
        assert_eq!(
            Network::Testnet,
            bitcoin::Network::Testnet.into()
        );
    }
}