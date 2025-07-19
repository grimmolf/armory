/// Migration module tests
///
/// Tests for legacy wallet import functionality as specified in Phase 4 validation gates.
use super::legacy_import::{ImportError, LegacyWalletImporter};
use crate::storage::wallet_storage::{StorageConfig, WalletStorage};
use std::fs::write;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

/// Helper function to create a test storage configuration
fn create_test_storage() -> (WalletStorage, TempDir) {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig {
        storage_path: temp_dir.path().to_path_buf(),
        auto_backup: true,
        backup_count: 3,
    };
    let storage = WalletStorage::new(config).unwrap();
    (storage, temp_dir)
}

/// Helper function to create a mock legacy Armory wallet file
fn create_mock_legacy_wallet() -> (TempDir, PathBuf) {
    let temp_dir = tempdir().unwrap();
    let wallet_path = temp_dir.path().join("legacy_wallet.wallet");

    // Create a mock Armory wallet file with proper structure
    let mut wallet_data = Vec::new();

    // Armory magic header
    wallet_data.extend_from_slice(b"ARMO");

    // Version (4 bytes, little endian)
    wallet_data.extend_from_slice(&15u32.to_le_bytes());

    // Mock wallet data (simplified structure)
    // In reality, this would be the full Armory binary format
    let mock_data = vec![
        // Unique ID (8 bytes)
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
        // Encrypted root key (32 bytes of mock data)
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF, 0x00, // Chain code (32 bytes)
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99,
        // Additional mock data to simulate a full wallet file
    ];
    wallet_data.extend_from_slice(&mock_data);

    // Add minimal data to avoid triggering encryption detection
    wallet_data.extend_from_slice(&vec![0u8; 50]);

    write(&wallet_path, wallet_data).unwrap();
    (temp_dir, wallet_path)
}

/// Helper function to create an encrypted legacy wallet file
fn create_encrypted_legacy_wallet() -> (TempDir, PathBuf) {
    let temp_dir = tempdir().unwrap();
    let wallet_path = temp_dir.path().join("encrypted_wallet.wallet");

    let mut wallet_data = Vec::new();
    wallet_data.extend_from_slice(b"ARMO");
    wallet_data.extend_from_slice(&20u32.to_le_bytes()); // Version 20

    // Longer data to trigger encryption detection heuristic
    wallet_data.extend_from_slice(&vec![0x42u8; 500]);

    write(&wallet_path, wallet_data).unwrap();
    (temp_dir, wallet_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test legacy wallet import - Phase 4 validation gate
    ///
    /// This test validates the core legacy import functionality required by the PRP.
    #[test]
    fn test_legacy_import() {
        let (_wallet_temp_dir, wallet_path) = create_mock_legacy_wallet();
        let (storage, _storage_temp_dir) = create_test_storage();

        // Test basic legacy wallet detection
        assert!(
            LegacyWalletImporter::is_legacy_wallet(&wallet_path),
            "Should detect valid legacy wallet file"
        );

        // Test legacy wallet info parsing
        let wallet_info = LegacyWalletImporter::get_legacy_wallet_info(&wallet_path)
            .expect("Should parse legacy wallet info");

        assert_eq!(wallet_info.version, 15, "Should parse correct version");
        assert_eq!(
            wallet_info.unique_id, "legacy_15",
            "Should generate correct ID"
        );
        assert_eq!(
            wallet_info.label, "Imported Legacy Wallet",
            "Should have default label"
        );

        // Test importing unencrypted wallet
        let imported_wallet = LegacyWalletImporter::import_wallet(
            &wallet_path,
            "imported_test_wallet".to_string(),
            None, // No passphrase for unencrypted
            storage,
        )
        .expect("Should import unencrypted legacy wallet");

        assert_eq!(
            imported_wallet.id, "imported_test_wallet",
            "Should have correct ID"
        );
        assert_eq!(
            imported_wallet.label, "imported_test_wallet",
            "Should have correct label"
        );

        println!("✅ Legacy wallet import test passed");
    }

    /// Test legacy wallet import error handling
    #[test]
    fn test_legacy_import_errors() {
        let temp_dir = tempdir().unwrap();
        let (storage, _storage_temp_dir) = create_test_storage();

        // Test with non-existent file
        let non_existent = temp_dir.path().join("does_not_exist.wallet");
        let result =
            LegacyWalletImporter::import_wallet(&non_existent, "test".to_string(), None, storage);
        assert!(result.is_err(), "Should fail for non-existent file");

        // Test with invalid wallet file
        let invalid_file = temp_dir.path().join("invalid.wallet");
        write(&invalid_file, b"not a wallet file").unwrap();

        let (storage2, _temp_dir2) = create_test_storage();
        let result =
            LegacyWalletImporter::import_wallet(&invalid_file, "test".to_string(), None, storage2);
        assert!(result.is_err(), "Should fail for invalid wallet file");

        println!("✅ Legacy import error handling test passed");
    }

    /// Test encrypted legacy wallet import
    #[test]
    fn test_encrypted_legacy_import() {
        let (_wallet_temp_dir, wallet_path) = create_encrypted_legacy_wallet();
        let (storage, _storage_temp_dir) = create_test_storage();

        // Test that encrypted wallet is detected as encrypted
        let wallet_info = LegacyWalletImporter::get_legacy_wallet_info(&wallet_path)
            .expect("Should parse encrypted wallet info");
        assert!(wallet_info.is_encrypted, "Should detect encrypted wallet");

        // Test import with missing passphrase fails
        let result = LegacyWalletImporter::import_wallet(
            &wallet_path,
            "encrypted_test".to_string(),
            None, // No passphrase
            storage,
        );
        assert!(result.is_err(), "Should fail without passphrase");

        // Test import with passphrase succeeds
        let (storage2, _temp_dir2) = create_test_storage();
        let imported_wallet = LegacyWalletImporter::import_wallet(
            &wallet_path,
            "encrypted_test".to_string(),
            Some("test_password"),
            storage2,
        )
        .expect("Should import with passphrase");

        assert_eq!(
            imported_wallet.id, "encrypted_test",
            "Should have correct ID"
        );

        println!("✅ Encrypted legacy wallet import test passed");
    }

    /// Test legacy wallet version validation
    #[test]
    fn test_version_validation() {
        let temp_dir = tempdir().unwrap();

        // Create wallet with unsupported version
        let unsupported_path = temp_dir.path().join("unsupported.wallet");
        let mut wallet_data = Vec::new();
        wallet_data.extend_from_slice(b"ARMO");
        wallet_data.extend_from_slice(&999u32.to_le_bytes()); // Unsupported version
        wallet_data.extend_from_slice(&vec![0u8; 100]);
        write(&unsupported_path, wallet_data).unwrap();

        let (storage, _temp_dir) = create_test_storage();
        let result = LegacyWalletImporter::import_wallet(
            &unsupported_path,
            "test".to_string(),
            None,
            storage,
        );

        assert!(result.is_err(), "Should fail for unsupported version");
        if let Err(ImportError::UnsupportedVersion(version)) = result {
            assert_eq!(version, 999, "Should report correct unsupported version");
        } else {
            panic!("Expected UnsupportedVersion error");
        }

        println!("✅ Legacy wallet version validation test passed");
    }

    /// Test legacy wallet file format detection
    #[test]
    fn test_file_format_detection() {
        let temp_dir = tempdir().unwrap();

        // Test various file types
        let files_and_expected = vec![
            (
                b"ARMO\x01\x00\x00\x00".as_slice(),
                true,
                "Valid Armory wallet",
            ),
            (
                b"BTCW\x01\x00\x00\x00".as_slice(),
                false,
                "Different magic header",
            ),
            (b"ARM".as_slice(), false, "Too short"),
            (b"ARMO".as_slice(), false, "Missing version"),
            (b"".as_slice(), false, "Empty file"),
            (b"random data here".as_slice(), false, "Random data"),
        ];

        for (i, (data, expected, description)) in files_and_expected.into_iter().enumerate() {
            let test_path = temp_dir.path().join(format!("test_{}.wallet", i));
            write(&test_path, data).unwrap();

            let is_legacy = LegacyWalletImporter::is_legacy_wallet(&test_path);
            assert_eq!(is_legacy, expected, "Failed for: {}", description);
        }

        println!("✅ File format detection test passed");
    }

    /// Integration test for complete legacy import workflow
    #[test]
    fn test_complete_legacy_import_workflow() {
        let (_wallet_temp_dir, wallet_path) = create_mock_legacy_wallet();
        let (storage, _storage_temp_dir) = create_test_storage();

        // Step 1: Validate file is a legacy wallet
        assert!(LegacyWalletImporter::is_legacy_wallet(&wallet_path));

        // Step 2: Get wallet info for preview
        let info = LegacyWalletImporter::get_legacy_wallet_info(&wallet_path)
            .expect("Should get wallet info");
        assert!(info.version < 100, "Should be a supported version");

        // Step 3: Import the wallet
        let wallet = LegacyWalletImporter::import_wallet(
            &wallet_path,
            "complete_test_wallet".to_string(),
            None,
            storage,
        )
        .expect("Should import wallet");

        // Step 4: Verify wallet properties
        assert_eq!(wallet.id, "complete_test_wallet");
        assert!(!wallet.id.is_empty());

        // Step 5: Verify wallet can be saved (basic functionality)
        assert!(wallet.save().is_ok(), "Imported wallet should be saveable");

        println!("✅ Complete legacy import workflow test passed");
    }
}
