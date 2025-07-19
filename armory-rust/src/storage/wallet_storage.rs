use crate::crypto::{decrypt_data, encrypt_data, EncryptedData, SecureKey};
/// Modern wallet storage implementation
///
/// This module replaces the custom wallet file format with modern
/// encrypted storage using ChaCha20Poly1305 and atomic updates.
use crate::error::{StorageError, StorageResult};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for wallet storage
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Path to the storage directory
    pub storage_path: PathBuf,
    /// Enable automatic backups
    pub auto_backup: bool,
    /// Backup retention count
    pub backup_count: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from(".armory"),
            auto_backup: true,
            backup_count: 5,
        }
    }
}

/// Wallet data structure for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    /// Wallet unique identifier
    pub id: String,
    /// Wallet label/name
    pub label: String,
    /// Encrypted seed or root key
    pub encrypted_seed: EncryptedData,
    /// Wallet descriptors
    pub descriptors: Vec<String>,
    /// Address book entries
    pub address_book: HashMap<String, String>,
    /// Transaction comments
    pub tx_comments: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Wallet version
    pub version: u32,
}

/// Main wallet storage implementation
pub struct WalletStorage {
    db: Db,
    config: StorageConfig,
    encryption_key: Option<SecureKey>,
}

impl WalletStorage {
    /// Create a new wallet storage instance
    pub fn new(config: StorageConfig) -> StorageResult<Self> {
        // Ensure storage directory exists
        std::fs::create_dir_all(&config.storage_path)?;

        // Open SLED database
        let db_path = config.storage_path.join("wallets.db");
        let db = sled::open(db_path).map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(Self {
            db,
            config,
            encryption_key: None,
        })
    }

    /// Set encryption key for wallet data
    pub fn set_encryption_key(&mut self, key: SecureKey) {
        self.encryption_key = Some(key);
    }

    /// Save wallet data to storage
    pub fn save_wallet_data(&self, data: &WalletData) -> StorageResult<()> {
        // Serialize wallet data
        let serialized = serde_json::to_vec(data)?;

        // Encrypt if key is available
        let final_data = if let Some(key) = &self.encryption_key {
            let encrypted = encrypt_data(key, &serialized, None)
                .map_err(|e| StorageError::Database(format!("Encryption failed: {e}")))?;
            serde_json::to_vec(&encrypted)?
        } else {
            serialized
        };

        // Store in database with atomic update
        let key = format!("wallet:{}", data.id);
        self.db
            .insert(key.as_bytes(), final_data)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        // Flush to ensure data is written
        self.db
            .flush()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        // Create backup if enabled
        if self.config.auto_backup {
            self.create_backup(&data.id)?;
        }

        Ok(())
    }

    /// Load wallet data from storage
    pub fn load_wallet_data(&self, wallet_id: &str) -> StorageResult<WalletData> {
        let key = format!("wallet:{wallet_id}");
        let stored_data = self
            .db
            .get(key.as_bytes())
            .map_err(|e| StorageError::Database(e.to_string()))?
            .ok_or_else(|| StorageError::Database("Wallet not found".to_string()))?;

        // Decrypt if key is available
        let decrypted_data = if let Some(key) = &self.encryption_key {
            let encrypted: EncryptedData = serde_json::from_slice(&stored_data)?;
            decrypt_data(key, &encrypted, None)
                .map_err(|e| StorageError::Database(format!("Decryption failed: {e}")))?
        } else {
            stored_data.to_vec()
        };

        // Deserialize wallet data
        let wallet_data: WalletData = serde_json::from_slice(&decrypted_data)?;
        Ok(wallet_data)
    }

    /// List all wallet IDs
    pub fn list_wallets(&self) -> StorageResult<Vec<String>> {
        let mut wallet_ids = Vec::new();

        for result in self.db.scan_prefix(b"wallet:") {
            let (key, _) = result.map_err(|e| StorageError::Database(e.to_string()))?;
            let key_str = String::from_utf8_lossy(&key);
            if let Some(wallet_id) = key_str.strip_prefix("wallet:") {
                wallet_ids.push(wallet_id.to_string());
            }
        }

        Ok(wallet_ids)
    }

    /// Delete wallet data
    pub fn delete_wallet(&self, wallet_id: &str) -> StorageResult<()> {
        let key = format!("wallet:{wallet_id}");
        self.db
            .remove(key.as_bytes())
            .map_err(|e| StorageError::Database(e.to_string()))?;

        self.db
            .flush()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    /// Create a backup of wallet data
    fn create_backup(&self, wallet_id: &str) -> StorageResult<()> {
        let backup_dir = self.config.storage_path.join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_file = backup_dir.join(format!("{wallet_id}_{timestamp}.backup"));

        // Copy wallet data to backup file
        let key = format!("wallet:{wallet_id}");
        if let Some(data) = self
            .db
            .get(key.as_bytes())
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            std::fs::write(&backup_file, &data)?;
        }

        // Cleanup old backups
        self.cleanup_backups(wallet_id)?;

        Ok(())
    }

    /// Clean up old backup files
    fn cleanup_backups(&self, wallet_id: &str) -> StorageResult<()> {
        let backup_dir = self.config.storage_path.join("backups");
        if !backup_dir.exists() {
            return Ok(());
        }

        // Find all backup files for this wallet
        let mut backups = Vec::new();
        for entry in std::fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with(wallet_id) && file_name_str.ends_with(".backup") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((entry.path(), modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess backups
        for (path, _) in backups.into_iter().skip(self.config.backup_count as usize) {
            let _ = std::fs::remove_file(path);
        }

        Ok(())
    }

    /// Check if wallet exists
    pub fn wallet_exists(&self, wallet_id: &str) -> StorageResult<bool> {
        let key = format!("wallet:{wallet_id}");
        self.db
            .contains_key(key.as_bytes())
            .map_err(|e| StorageError::Database(e.to_string()))
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> StorageResult<StorageStats> {
        let size_on_disk = self
            .db
            .size_on_disk()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let wallet_count = self.list_wallets()?.len();

        Ok(StorageStats {
            size_on_disk,
            wallet_count,
            database_path: self.config.storage_path.clone(),
        })
    }
}

/// Storage statistics
#[derive(Debug)]
pub struct StorageStats {
    pub size_on_disk: u64,
    pub wallet_count: usize,
    pub database_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (WalletStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            storage_path: temp_dir.path().to_path_buf(),
            auto_backup: false,
            backup_count: 3,
        };
        let storage = WalletStorage::new(config).unwrap();
        (storage, temp_dir)
    }

    fn create_test_wallet_data() -> WalletData {
        use crate::crypto::{encrypt_data, SecureKey};

        let key = SecureKey::generate().unwrap();
        let seed_data = b"test seed data";
        let encrypted_seed = encrypt_data(&key, seed_data, None).unwrap();

        WalletData {
            id: "test_wallet_123".to_string(),
            label: "Test Wallet".to_string(),
            encrypted_seed,
            descriptors: vec!["wpkh(xpub123...)".to_string(), "tr(xpub456...)".to_string()],
            address_book: HashMap::new(),
            tx_comments: HashMap::new(),
            created_at: 1640995200, // 2022-01-01
            modified_at: 1640995200,
            version: 1,
        }
    }

    #[test]
    fn test_wallet_storage_creation() {
        let (storage, _temp_dir) = create_test_storage();
        assert!(storage.list_wallets().unwrap().is_empty());
    }

    #[test]
    fn test_save_and_load_wallet() {
        let (storage, _temp_dir) = create_test_storage();
        let wallet_data = create_test_wallet_data();

        // Save wallet
        storage.save_wallet_data(&wallet_data).unwrap();

        // Load wallet
        let loaded_data = storage.load_wallet_data(&wallet_data.id).unwrap();
        assert_eq!(loaded_data.id, wallet_data.id);
        assert_eq!(loaded_data.label, wallet_data.label);
    }

    #[test]
    fn test_list_wallets() {
        let (storage, _temp_dir) = create_test_storage();
        let wallet_data = create_test_wallet_data();

        storage.save_wallet_data(&wallet_data).unwrap();

        let wallets = storage.list_wallets().unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0], wallet_data.id);
    }

    #[test]
    fn test_delete_wallet() {
        let (storage, _temp_dir) = create_test_storage();
        let wallet_data = create_test_wallet_data();

        storage.save_wallet_data(&wallet_data).unwrap();
        assert!(storage.wallet_exists(&wallet_data.id).unwrap());

        storage.delete_wallet(&wallet_data.id).unwrap();
        assert!(!storage.wallet_exists(&wallet_data.id).unwrap());
    }
}
