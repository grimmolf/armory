/// Command-line interface module
///
/// This module provides the CLI for wallet operations.
pub mod commands;
pub mod config;
pub mod handlers;

#[cfg(test)]
pub mod tests;

// Re-exports for convenience
pub use commands::{AddressType, CliCommands, Commands, ExportFormat, MultisigCommands};
pub use config::CliConfig;
pub use handlers::CliHandler;
