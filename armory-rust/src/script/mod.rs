/// Bitcoin script engine with Taproot and miniscript support
/// 
/// This module implements script validation, execution, and descriptor parsing
/// with full support for modern Bitcoin script features including:
/// - Taproot script paths (BIP-341) 
/// - Miniscript compilation and analysis
/// - Script witness generation
/// - Descriptor-based script templates

pub mod engine;
pub mod descriptors;
pub mod taproot;
pub mod witness;

// Re-exports for convenience
pub use engine::ScriptEngine;
pub use descriptors::{ScriptDescriptor, DescriptorType};
pub use taproot::{TaprootSpendData, TaprootBuilder};
pub use witness::WitnessGenerator;