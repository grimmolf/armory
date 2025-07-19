/// CLI commands implementation
/// 
/// This is a placeholder for command-line interface commands.

use clap::{Parser, Subcommand};

/// CLI commands structure
#[derive(Parser)]
#[command(name = "armory-rust")]
#[command(about = "Modern Rust implementation of Armory Bitcoin wallet")]
pub struct CliCommands {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet
    Create {
        /// Wallet name
        name: String,
    },
    /// List all wallets
    List,
}