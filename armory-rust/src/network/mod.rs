/// Network communication module
/// 
/// This module provides BIP-324 encrypted P2P communication
/// and RPC client functionality.

pub mod p2p;
pub mod rpc;

// Re-exports for convenience
pub use p2p::BitcoinP2P;
pub use rpc::RpcClient;