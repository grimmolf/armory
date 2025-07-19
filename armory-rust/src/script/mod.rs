pub mod descriptors;
/// Bitcoin script engine with Taproot and miniscript support
///
/// This module implements script validation, execution, and descriptor parsing
/// with full support for modern Bitcoin script features including:
/// - Taproot script paths (BIP-341)
/// - Miniscript compilation and analysis
/// - Script witness generation
/// - Descriptor-based script templates
pub mod engine;
pub mod taproot;
pub mod witness_simple;

// Re-exports for convenience
pub use descriptors::{DescriptorType, ScriptDescriptor};
pub use engine::ScriptEngine;
pub use taproot::{TaprootBuilder, TaprootSpendData};
pub use witness_simple::WitnessGenerator;
