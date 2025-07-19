/// Main entry point for Armory Rust CLI
///
/// Modern Rust implementation of Armory Bitcoin wallet
use armory_rust::cli::{CliCommands, CliConfig, CliHandler};
use clap::Parser;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let cli = CliCommands::parse();

    // Initialize logging based on verbosity
    if cli.verbose {
        fmt().with_max_level(tracing::Level::DEBUG).init();
    } else {
        fmt().with_max_level(tracing::Level::INFO).init();
    }

    // Create configuration
    let config = CliConfig::with_options(cli.data_dir.clone(), cli.network.into(), cli.verbose)?;

    if config.verbose {
        println!("🦀 Armory Rust Bitcoin Wallet v1.0.0");
        println!("📁 Data directory: {}", config.data_dir.display());
        println!("🌐 Network: {:?}", config.network);
        println!();
    }

    // Create and run CLI handler
    let handler = CliHandler::new(config)?;

    if let Err(e) = handler.execute(cli.command).await {
        eprintln!("❌ Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
