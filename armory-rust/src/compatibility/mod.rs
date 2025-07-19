/// Bitcoin Core RPC compatibility module
///
/// This module provides compatibility with Bitcoin Core RPC interface
/// and ensures interoperability with the broader Bitcoin ecosystem.
pub mod rpc_compatibility;

pub use rpc_compatibility::{CompatibilityError, CompatibilityResult, RpcCompatibilityTester};

#[cfg(test)]
pub mod tests;
