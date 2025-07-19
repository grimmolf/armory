/// Compatibility module tests
///
/// Tests for Bitcoin Core RPC compatibility as specified in Phase 4 validation gates.
use super::rpc_compatibility::{CompatibilityError, RpcCompatibilityTester};
use crate::storage::wallet_storage::{StorageConfig, WalletStorage};
use crate::wallet::Wallet;
use crate::Network;
use tempfile::{tempdir, TempDir};

/// Helper function to create a test wallet
fn create_test_wallet() -> (Wallet, TempDir) {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig {
        storage_path: temp_dir.path().to_path_buf(),
        auto_backup: true,
        backup_count: 3,
    };
    let storage = WalletStorage::new(config).unwrap();
    let wallet =
        Wallet::create_new("rpc_test_wallet".to_string(), Network::Regtest, storage).unwrap();
    (wallet, temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test RPC compatibility - Phase 4 validation gate
    ///
    /// This test validates Bitcoin Core RPC compatibility as required by the PRP.
    #[tokio::test]
    async fn test_rpc_compatibility() {
        let (wallet, _temp_dir) = create_test_wallet();
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // Run all compatibility tests
        let result = tester.run_all_tests(&wallet).await;
        assert!(
            result.is_ok(),
            "RPC compatibility tests should run successfully"
        );

        // Verify test results
        let results = tester.get_results();
        assert!(
            !results.is_empty(),
            "Should have compatibility test results"
        );

        // Check specific compatibility areas
        let mut has_rpc_methods = false;
        let mut has_address_format = false;
        let mut has_transaction_compat = false;
        let mut has_psbt_compat = false;
        let mut has_network_compat = false;

        for result in results {
            match result.test_name.as_str() {
                name if name.starts_with("rpc_method_") => {
                    has_rpc_methods = true;
                    assert!(result.passed, "RPC method test '{}' should pass", name);
                }
                "address_format_compatibility" => {
                    has_address_format = true;
                    assert!(result.passed, "Address format compatibility should pass");
                }
                "transaction_compatibility" => {
                    has_transaction_compat = true;
                    assert!(result.passed, "Transaction compatibility should pass");
                }
                "psbt_compatibility" => {
                    has_psbt_compat = true;
                    assert!(result.passed, "PSBT compatibility should pass");
                }
                "network_protocol_compatibility" => {
                    has_network_compat = true;
                    assert!(result.passed, "Network protocol compatibility should pass");
                }
                _ => {}
            }
        }

        // Ensure all major compatibility areas were tested
        assert!(has_rpc_methods, "Should test RPC method compatibility");
        assert!(
            has_address_format,
            "Should test address format compatibility"
        );
        assert!(
            has_transaction_compat,
            "Should test transaction compatibility"
        );
        assert!(has_psbt_compat, "Should test PSBT compatibility");
        assert!(
            has_network_compat,
            "Should test network protocol compatibility"
        );

        // Print summary for visibility
        tester.print_summary();

        // Verify overall success
        assert!(
            tester.all_tests_passed(),
            "All RPC compatibility tests should pass"
        );

        println!("✅ RPC compatibility test completed successfully");
    }

    /// Test RPC method compatibility
    #[tokio::test]
    async fn test_rpc_method_compatibility() {
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // Test basic RPC methods
        let result = tester.test_basic_rpc_methods().await;
        assert!(result.is_ok(), "Basic RPC method tests should succeed");

        let results = tester.get_results();
        let rpc_results: Vec<_> = results
            .iter()
            .filter(|r| r.test_name.starts_with("rpc_method_"))
            .collect();

        assert!(
            !rpc_results.is_empty(),
            "Should have RPC method test results"
        );

        // Check that all standard Bitcoin Core methods are supported
        let expected_methods = vec![
            "getblockchaininfo",
            "getnetworkinfo",
            "getwalletinfo",
            "getnewaddress",
            "getbalance",
            "listunspent",
            "sendtoaddress",
            "signrawtransactionwithwallet",
        ];

        for method in expected_methods {
            let method_result = rpc_results
                .iter()
                .find(|r| r.test_name == format!("rpc_method_{}", method));
            assert!(
                method_result.is_some(),
                "Should test RPC method: {}",
                method
            );
            assert!(
                method_result.unwrap().passed,
                "RPC method '{}' should be supported",
                method
            );
        }

        println!("✅ RPC method compatibility test passed");
    }

    /// Test wallet compatibility with Bitcoin Core
    #[tokio::test]
    async fn test_wallet_bitcoin_core_compatibility() {
        let (wallet, _temp_dir) = create_test_wallet();
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // Test wallet compatibility
        let result = tester.test_wallet_compatibility(&wallet).await;
        assert!(result.is_ok(), "Wallet compatibility tests should succeed");

        let results = tester.get_results();

        // Check address format compatibility
        let address_result = results
            .iter()
            .find(|r| r.test_name == "address_format_compatibility");
        assert!(
            address_result.is_some(),
            "Should test address format compatibility"
        );
        assert!(
            address_result.unwrap().passed,
            "Address formats should be compatible"
        );

        // Verify address type details
        let details = &address_result.unwrap().details;
        assert!(
            details.contains_key("legacy_address"),
            "Should test legacy addresses"
        );
        assert!(
            details.contains_key("native_segwit_address"),
            "Should test native SegWit addresses"
        );
        assert!(
            details.contains_key("taproot_address"),
            "Should test Taproot addresses"
        );

        // Check transaction compatibility
        let tx_result = results
            .iter()
            .find(|r| r.test_name == "transaction_compatibility");
        assert!(tx_result.is_some(), "Should test transaction compatibility");
        assert!(
            tx_result.unwrap().passed,
            "Transactions should be compatible"
        );

        // Check PSBT compatibility
        let psbt_result = results.iter().find(|r| r.test_name == "psbt_compatibility");
        assert!(psbt_result.is_some(), "Should test PSBT compatibility");
        assert!(psbt_result.unwrap().passed, "PSBTs should be compatible");

        println!("✅ Wallet Bitcoin Core compatibility test passed");
    }

    /// Test network protocol compatibility
    #[tokio::test]
    async fn test_network_protocol_compatibility() {
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // Test network compatibility
        let result = tester.test_network_compatibility().await;
        assert!(result.is_ok(), "Network compatibility tests should succeed");

        let results = tester.get_results();
        let network_result = results
            .iter()
            .find(|r| r.test_name == "network_protocol_compatibility");

        assert!(
            network_result.is_some(),
            "Should test network protocol compatibility"
        );
        assert!(
            network_result.unwrap().passed,
            "Network protocols should be compatible"
        );

        // Verify protocol support details
        let details = &network_result.unwrap().details;
        assert!(
            details.contains_key("p2p_v1_supported"),
            "Should support standard P2P"
        );
        assert!(
            details.contains_key("bip324_supported"),
            "Should support BIP-324 encrypted transport"
        );

        println!("✅ Network protocol compatibility test passed");
    }

    /// Test RPC connection handling
    #[tokio::test]
    async fn test_rpc_connection_handling() {
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // Test connection to a non-existent endpoint (should succeed at client creation but fail at actual RPC call)
        let result = tester
            .connect_rpc("http://localhost:18443".to_string())
            .await;
        // RPC client creation should succeed, but actual calls will fail
        assert!(result.is_ok(), "RPC client creation should succeed");

        let results = tester.get_results();
        let connection_result = results.iter().find(|r| r.test_name == "rpc_connection");

        assert!(
            connection_result.is_some(),
            "Should record RPC connection attempt"
        );
        assert!(
            connection_result.unwrap().passed,
            "RPC client creation should pass"
        );

        println!("✅ RPC connection handling test passed");
    }

    /// Test error handling in compatibility tests
    #[tokio::test]
    async fn test_compatibility_error_handling() {
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // This should work since we're using mock implementations
        let (wallet, _temp_dir) = create_test_wallet();
        let result = tester.run_all_tests(&wallet).await;
        assert!(
            result.is_ok(),
            "Compatibility tests should handle errors gracefully"
        );

        // Even if some individual tests fail, the overall test runner should succeed
        let results = tester.get_results();
        assert!(
            !results.is_empty(),
            "Should have test results even with errors"
        );

        println!("✅ Compatibility error handling test passed");
    }

    /// Integration test for complete RPC compatibility workflow
    #[tokio::test]
    async fn test_complete_rpc_compatibility_workflow() {
        let (wallet, _temp_dir) = create_test_wallet();
        let mut tester = RpcCompatibilityTester::new(Network::Regtest);

        // Step 1: Test RPC methods
        assert!(tester.test_basic_rpc_methods().await.is_ok());

        // Step 2: Test wallet compatibility
        assert!(tester.test_wallet_compatibility(&wallet).await.is_ok());

        // Step 3: Test network compatibility
        assert!(tester.test_network_compatibility().await.is_ok());

        // Step 4: Verify comprehensive results
        let results = tester.get_results();
        assert!(
            results.len() >= 10,
            "Should have comprehensive test coverage"
        );

        // Step 5: Check success rate
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let success_rate = passed_tests as f64 / results.len() as f64;
        assert!(
            success_rate >= 0.9,
            "Should have high compatibility success rate: {:.1}%",
            success_rate * 100.0
        );

        tester.print_summary();

        println!("✅ Complete RPC compatibility workflow test passed");
    }
}
