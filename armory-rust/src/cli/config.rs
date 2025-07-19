/// CLI configuration management
///
/// Handles configuration for the Armory CLI application
use crate::error::WalletResult;
use bitcoin::Network;
use std::env;
use std::path::PathBuf;

/// CLI configuration structure
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub data_dir: PathBuf,
    pub network: Network,
    pub verbose: bool,
}

impl CliConfig {
    /// Create new CLI config with defaults
    pub fn new() -> WalletResult<Self> {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let data_dir = PathBuf::from(home_dir).join(".armory");

        Ok(Self {
            data_dir,
            network: Network::Bitcoin,
            verbose: false,
        })
    }

    /// Create CLI config with custom options
    pub fn with_options(
        data_dir: Option<PathBuf>,
        network: Network,
        verbose: bool,
    ) -> WalletResult<Self> {
        let mut config = Self::new()?;

        if let Some(dir) = data_dir {
            config.data_dir = dir;
        }

        config.network = network;
        config.verbose = verbose;

        Ok(config)
    }

    /// Get wallet storage directory
    pub fn wallet_dir(&self) -> PathBuf {
        self.data_dir.join("wallets")
    }

    /// Get backup directory
    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    /// Get logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    /// Ensure all directories exist
    pub fn ensure_directories(&self) -> WalletResult<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(self.wallet_dir())?;
        std::fs::create_dir_all(self.backup_dir())?;
        std::fs::create_dir_all(self.logs_dir())?;
        Ok(())
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default CLI config")
    }
}
