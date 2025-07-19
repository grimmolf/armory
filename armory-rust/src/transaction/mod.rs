/// Transaction processing module
///
/// This module handles PSBT v2 transaction creation and processing.
pub mod builder;
pub mod psbt;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use builder::TransactionBuilder;
pub use psbt::PsbtV2;
