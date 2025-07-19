/// Command-line interface module
/// 
/// This module provides the CLI for wallet operations.

pub mod commands;
pub mod config;

// Re-exports for convenience
pub use commands::CliCommands;
pub use config::CliConfig;