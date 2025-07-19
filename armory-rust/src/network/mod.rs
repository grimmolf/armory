/// Network communication module
///
/// This module provides BIP-324 encrypted P2P communication
/// and RPC client functionality.
pub mod p2p;
pub mod rpc;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use p2p::{BitcoinP2P, P2PConfig, PeerInfo};
pub use rpc::{BlockchainInfo, NetworkInfo, RpcClient, RpcEndpoint};
