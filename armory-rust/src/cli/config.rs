/// CLI configuration
/// 
/// This is a placeholder for CLI configuration management.

use crate::error::WalletResult;

/// CLI configuration structure
pub struct CliConfig {
    pub data_dir: String,
}

impl CliConfig {
    /// Create new CLI config (placeholder)
    pub fn new() -> WalletResult<Self> {
        Ok(Self {
            data_dir: ".armory".to_string(),
        })
    }
}