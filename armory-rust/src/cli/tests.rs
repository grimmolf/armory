/// CLI integration tests for Phase 4 validation gates
///
/// Tests the command-line interface functionality including wallet operations,
/// address generation, and basic CLI workflow as specified in the PRP.
use crate::cli::{AddressType as CliAddressType, CliConfig, CliHandler, Commands};
use crate::wallet::descriptor_wallet::AddressType;
use bitcoin::Network;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a test CLI configuration
fn create_test_cli_config() -> (CliConfig, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config = CliConfig::with_options(
        Some(temp_dir.path().to_path_buf()),
        Network::Regtest,
        false, // verbose
    )
    .unwrap();
    (config, temp_dir)
}

/// Helper function to create a test CLI handler
fn create_test_cli_handler() -> (CliHandler, TempDir) {
    let (config, temp_dir) = create_test_cli_config();
    let handler = CliHandler::new(config).unwrap();
    (handler, temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic wallet operations through CLI - Phase 4 validation gate
    #[tokio::test]
    async fn test_wallet_operations() {
        let (handler, _temp_dir) = create_test_cli_handler();

        // Test wallet creation
        let create_result = handler
            .execute(Commands::Create {
                name: "test_wallet".to_string(),
                mnemonic: None,
                encrypt: false,
                account: 0,
            })
            .await;
        assert!(create_result.is_ok(), "Wallet creation should succeed");

        // Test wallet listing
        let list_result = handler.execute(Commands::List).await;
        assert!(list_result.is_ok(), "Wallet listing should succeed");

        // Test wallet info
        let info_result = handler
            .execute(Commands::Info {
                wallet: "test_wallet".to_string(),
            })
            .await;
        assert!(info_result.is_ok(), "Wallet info should succeed");

        // Test address generation
        let address_result = handler
            .execute(Commands::Address {
                wallet: "test_wallet".to_string(),
                address_type: CliAddressType::NativeSegwit,
                count: 3,
                list: false,
            })
            .await;
        assert!(address_result.is_ok(), "Address generation should succeed");

        // Test address listing
        let list_address_result = handler
            .execute(Commands::Address {
                wallet: "test_wallet".to_string(),
                address_type: CliAddressType::NativeSegwit,
                count: 1,
                list: true,
            })
            .await;
        assert!(
            list_address_result.is_ok(),
            "Address listing should succeed"
        );
    }

    /// Test CLI error handling
    #[tokio::test]
    async fn test_cli_error_handling() {
        let (handler, _temp_dir) = create_test_cli_handler();

        // Test accessing non-existent wallet
        let info_result = handler
            .execute(Commands::Info {
                wallet: "nonexistent_wallet".to_string(),
            })
            .await;
        assert!(
            info_result.is_err(),
            "Accessing non-existent wallet should fail"
        );

        // Test creating duplicate wallet
        let create_result = handler
            .execute(Commands::Create {
                name: "test_wallet".to_string(),
                mnemonic: None,
                encrypt: false,
                account: 0,
            })
            .await;
        assert!(
            create_result.is_ok(),
            "First wallet creation should succeed"
        );

        let duplicate_result = handler
            .execute(Commands::Create {
                name: "test_wallet".to_string(),
                mnemonic: None,
                encrypt: false,
                account: 0,
            })
            .await;
        assert!(
            duplicate_result.is_err(),
            "Duplicate wallet creation should fail"
        );
    }

    /// Test multiple wallet management
    #[tokio::test]
    async fn test_multiple_wallets() {
        let (handler, _temp_dir) = create_test_cli_handler();

        // Create multiple wallets
        let wallets = vec!["wallet1", "wallet2", "wallet3"];

        for wallet_name in &wallets {
            let create_result = handler
                .execute(Commands::Create {
                    name: wallet_name.to_string(),
                    mnemonic: None,
                    encrypt: false,
                    account: 0,
                })
                .await;
            assert!(
                create_result.is_ok(),
                "Wallet creation should succeed for {}",
                wallet_name
            );
        }

        // Verify all wallets exist
        for wallet_name in &wallets {
            let info_result = handler
                .execute(Commands::Info {
                    wallet: wallet_name.to_string(),
                })
                .await;
            assert!(
                info_result.is_ok(),
                "Wallet info should succeed for {}",
                wallet_name
            );
        }
    }

    /// Test address type conversion
    #[test]
    fn test_address_type_conversion() {
        // Test CLI address type to wallet address type conversion
        let cli_legacy = CliAddressType::Legacy;
        let wallet_legacy: AddressType = cli_legacy.into();
        assert!(matches!(wallet_legacy, AddressType::Legacy));

        let cli_native_segwit = CliAddressType::NativeSegwit;
        let wallet_native_segwit: AddressType = cli_native_segwit.into();
        assert!(matches!(wallet_native_segwit, AddressType::NativeSegwit));

        let cli_taproot = CliAddressType::Taproot;
        let wallet_taproot: AddressType = cli_taproot.into();
        assert!(matches!(wallet_taproot, AddressType::Taproot));
    }

    /// Test CLI configuration
    #[test]
    fn test_cli_configuration() {
        let temp_dir = TempDir::new().unwrap();

        // Test with custom data directory
        let config = CliConfig::with_options(
            Some(temp_dir.path().to_path_buf()),
            Network::Testnet,
            true, // verbose
        )
        .unwrap();

        assert_eq!(config.network, Network::Testnet);
        assert!(config.verbose);
        assert_eq!(config.data_dir, temp_dir.path());

        // Test default configuration
        let default_config = CliConfig::default();
        assert_eq!(default_config.network, Network::Bitcoin);
        assert!(!default_config.verbose);
    }
}
