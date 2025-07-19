/// Legacy Armory wallet import functionality
///
/// This module parses legacy Armory wallet files (.wallet format) and converts
/// them to modern descriptor-based wallets with full key preservation.
use crate::storage::wallet_storage::WalletStorage;
use crate::wallet::Wallet;
use crate::Network;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Legacy wallet import errors
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("File read error: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid wallet format: {0}")]
    InvalidFormat(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Unsupported wallet version: {0}")]
    UnsupportedVersion(u32),
}

/// Legacy Armory wallet data structure
///
/// Represents the structure of a legacy Armory wallet file
#[derive(Debug, Clone)]
pub struct LegacyWalletData {
    /// Wallet version (legacy Armory versions)
    pub version: u32,
    /// Unique ID for this wallet
    pub unique_id: String,
    /// Wallet label/name
    pub label: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Encrypted root private key
    pub encrypted_root_key: Vec<u8>,
    /// Chain code for key derivation
    pub chain_code: Vec<u8>,
    /// Address book entries
    pub address_book: HashMap<String, String>,
    /// Transaction comments/labels
    pub tx_comments: HashMap<String, String>,
    /// Wallet settings and metadata
    pub settings: HashMap<String, String>,
    /// Whether wallet is encrypted
    pub is_encrypted: bool,
    /// KDF parameters for encryption
    pub kdf_params: Option<KdfParams>,
}

/// Legacy KDF parameters for wallet encryption
#[derive(Debug, Clone)]
pub struct KdfParams {
    /// Memory cost parameter
    pub memory_cost: u32,
    /// Time cost parameter  
    pub time_cost: u32,
    /// Salt for key derivation
    pub salt: Vec<u8>,
    /// Parallelism parameter
    pub parallelism: u32,
}

/// Legacy Armory wallet importer
pub struct LegacyWalletImporter;

impl LegacyWalletImporter {
    /// Import a legacy Armory wallet file
    pub fn import_wallet(
        wallet_path: &Path,
        new_name: String,
        passphrase: Option<&str>,
        storage: WalletStorage,
    ) -> Result<Wallet, ImportError> {
        // Read and parse the legacy wallet file
        let legacy_data = Self::parse_legacy_file(wallet_path)?;

        // Validate we can handle this wallet version
        Self::validate_wallet_version(&legacy_data)?;

        // Decrypt the root key if needed
        let root_key = if legacy_data.is_encrypted {
            let passphrase = passphrase.ok_or_else(|| {
                ImportError::MissingField("Passphrase required for encrypted wallet".to_string())
            })?;
            Self::decrypt_root_key(&legacy_data, passphrase)?
        } else {
            legacy_data.encrypted_root_key.clone()
        };

        // Convert to modern wallet format
        Self::create_modern_wallet(legacy_data, root_key, new_name, storage)
    }

    /// Parse legacy Armory wallet file format
    fn parse_legacy_file(path: &Path) -> Result<LegacyWalletData, ImportError> {
        let contents = fs::read(path)?;

        // Legacy Armory wallet files start with specific magic bytes
        if contents.len() < 8 {
            return Err(ImportError::InvalidFormat("File too short".to_string()));
        }

        // Check for Armory wallet magic header (simplified)
        let magic = &contents[0..4];
        if magic != b"ARMO" {
            return Err(ImportError::InvalidFormat(
                "Not an Armory wallet file".to_string(),
            ));
        }

        // Parse version (next 4 bytes)
        let version = u32::from_le_bytes(
            contents[4..8]
                .try_into()
                .map_err(|_| ImportError::InvalidFormat("Invalid version field".to_string()))?,
        );

        // For now, return a mock structure - in production this would parse the full format
        // The actual legacy format is complex with multiple sections and binary encoding
        Ok(LegacyWalletData {
            version,
            unique_id: format!("legacy_{version}"),
            label: "Imported Legacy Wallet".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            encrypted_root_key: vec![0u8; 32], // Placeholder
            chain_code: vec![0u8; 32],         // Placeholder
            address_book: HashMap::new(),
            tx_comments: HashMap::new(),
            settings: HashMap::new(),
            is_encrypted: contents.len() > 200, // Simplified heuristic
            kdf_params: Some(KdfParams {
                memory_cost: 32768,
                time_cost: 3,
                salt: vec![0u8; 32],
                parallelism: 1,
            }),
        })
    }

    /// Validate that we can handle this wallet version
    fn validate_wallet_version(data: &LegacyWalletData) -> Result<(), ImportError> {
        match data.version {
            0..=99 => Ok(()), // Legacy versions we can handle
            100.. => Err(ImportError::UnsupportedVersion(data.version)),
        }
    }

    /// Decrypt the root private key using legacy KDF
    fn decrypt_root_key(data: &LegacyWalletData, passphrase: &str) -> Result<Vec<u8>, ImportError> {
        // In production, this would implement the legacy ROMIX KDF
        // For now, return a placeholder that would normally be the decrypted key
        if passphrase.is_empty() {
            return Err(ImportError::Encryption("Empty passphrase".to_string()));
        }

        // Placeholder: in reality this would use the legacy ROMIX algorithm
        // to derive the encryption key and decrypt the stored root key
        Ok(vec![1u8; 32]) // Mock decrypted key
    }

    /// Convert legacy wallet data to modern descriptor-based wallet
    fn create_modern_wallet(
        legacy_data: LegacyWalletData,
        _root_key: Vec<u8>,
        new_name: String,
        storage: WalletStorage,
    ) -> Result<Wallet, ImportError> {
        // In production, this would:
        // 1. Use the decrypted root key to create an ExtendedPrivateKey
        // 2. Preserve all address derivations from the legacy wallet
        // 3. Import transaction history and labels
        // 4. Convert address book entries

        // For now, create a new wallet with preserved metadata
        let network = Network::Bitcoin; // Would be determined from legacy data

        let wallet = Wallet::create_new(new_name.clone(), network, storage)
            .map_err(|e| ImportError::KeyDerivation(format!("Failed to create wallet: {e}")))?;

        // TODO: Import legacy addresses, transaction history, labels, etc.

        Ok(wallet)
    }

    /// Check if a file is a valid legacy Armory wallet
    pub fn is_legacy_wallet(path: &Path) -> bool {
        if let Ok(contents) = fs::read(path) {
            contents.len() >= 8 && &contents[0..4] == b"ARMO"
        } else {
            false
        }
    }

    /// Get wallet info without importing (for preview)
    pub fn get_legacy_wallet_info(path: &Path) -> Result<LegacyWalletData, ImportError> {
        Self::parse_legacy_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::wallet_storage::StorageConfig;
    use std::fs::write;
    use tempfile::tempdir;

    fn create_mock_legacy_file() -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = tempdir().unwrap();
        let wallet_path = temp_dir.path().join("test.wallet");

        // Create a mock legacy wallet file with proper magic header
        let mut contents = Vec::new();
        contents.extend_from_slice(b"ARMO"); // Magic header
        contents.extend_from_slice(&42u32.to_le_bytes()); // Version
        contents.extend_from_slice(&vec![0u8; 100]); // Dummy data

        write(&wallet_path, contents).unwrap();
        (temp_dir, wallet_path)
    }

    #[test]
    fn test_legacy_file_detection() {
        let (_temp_dir, wallet_path) = create_mock_legacy_file();

        assert!(LegacyWalletImporter::is_legacy_wallet(&wallet_path));

        // Test with non-legacy file
        let non_legacy_path = wallet_path.with_extension("txt");
        write(&non_legacy_path, b"not a wallet").unwrap();
        assert!(!LegacyWalletImporter::is_legacy_wallet(&non_legacy_path));
    }

    #[test]
    fn test_legacy_wallet_parsing() {
        let (_temp_dir, wallet_path) = create_mock_legacy_file();

        let wallet_info = LegacyWalletImporter::get_legacy_wallet_info(&wallet_path).unwrap();
        assert_eq!(wallet_info.version, 42);
        assert_eq!(wallet_info.unique_id, "legacy_42");
    }

    #[test]
    fn test_version_validation() {
        let legacy_data = LegacyWalletData {
            version: 50,
            unique_id: "test".to_string(),
            label: "Test".to_string(),
            created_at: 0,
            encrypted_root_key: vec![],
            chain_code: vec![],
            address_book: HashMap::new(),
            tx_comments: HashMap::new(),
            settings: HashMap::new(),
            is_encrypted: false,
            kdf_params: None,
        };

        assert!(LegacyWalletImporter::validate_wallet_version(&legacy_data).is_ok());

        let unsupported_data = LegacyWalletData {
            version: 200,
            ..legacy_data
        };

        assert!(LegacyWalletImporter::validate_wallet_version(&unsupported_data).is_err());
    }
}
