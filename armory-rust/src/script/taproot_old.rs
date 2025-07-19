/// Taproot script tree and spending data management
/// 
/// This module implements BIP-341 Taproot functionality including:
/// - Script tree construction and merkle path generation
/// - Key and script path spending data
/// - Control block generation for script path spends

use crate::error::{TransactionResult, TransactionError};
use bitcoin::{
    ScriptBuf, XOnlyPublicKey, PublicKey,
    taproot::{
        TapTree, TapLeafHash, ControlBlock, LeafVersion,
        TaprootSpendInfo, TaprootBuilder as BitcoinTaprootBuilder,
    },
    secp256k1::{Secp256k1, All},
    key::{TapTweak, TweakedPublicKey},
};
use std::collections::{HashMap, BTreeMap};

/// Taproot spending data container
#[derive(Debug, Clone)]
pub struct TaprootSpendData {
    /// The internal key used for key-path spending
    pub internal_key: XOnlyPublicKey,
    /// The tweaked output key
    pub output_key: TweakedPublicKey,
    /// Script tree information
    pub script_tree: Option<TapTree>,
    /// Map from script to its merkle path
    pub script_paths: HashMap<ScriptBuf, (TapLeafHash, Vec<TapLeafHash>)>,
    /// Control blocks for script path spending
    pub control_blocks: HashMap<ScriptBuf, ControlBlock>,
    /// Merkle root of the script tree (if any)
    pub merkle_root: Option<TapLeafHash>,
}

impl TaprootSpendData {
    /// Create new Taproot spend data for key-only spending
    pub fn key_only(internal_key: XOnlyPublicKey) -> TransactionResult<Self> {
        let secp = Secp256k1::new();
        
        // For key-only spending, there's no script tree
        let output_key = internal_key.tap_tweak(&secp, None)
            .map_err(|_| TransactionError::ScriptValidation("Failed to create Taproot tweak".to_string()))?;

        Ok(Self {
            internal_key,
            output_key,
            script_tree: None,
            script_paths: HashMap::new(),
            control_blocks: HashMap::new(),
            merkle_root: None,
        })
    }

    /// Create Taproot spend data with script tree
    pub fn with_script_tree(
        internal_key: XOnlyPublicKey,
        script_tree: TapTree,
    ) -> TransactionResult<Self> {
        let secp = Secp256k1::new();
        
        // Calculate merkle root from script tree
        let merkle_root = script_tree.root();
        
        // Tweak the internal key with the merkle root
        let output_key = internal_key.tap_tweak(&secp, Some(merkle_root))
            .map_err(|_| TransactionError::ScriptValidation("Failed to create Taproot tweak with script tree".to_string()))?;

        // Extract script paths and control blocks
        let mut script_paths = HashMap::new();
        let mut control_blocks = HashMap::new();

        // TODO: Extract script paths from TapTree
        // This would involve traversing the tree and building merkle paths
        
        Ok(Self {
            internal_key,
            output_key,
            script_tree: Some(script_tree),
            script_paths,
            control_blocks,
            merkle_root: Some(merkle_root),
        })
    }

    /// Get control block for a specific script
    pub fn control_block(&self, script: &ScriptBuf) -> Option<&ControlBlock> {
        self.control_blocks.get(script)
    }

    /// Check if this can be spent via key path
    pub fn supports_key_path(&self) -> bool {
        true // All Taproot outputs support key path spending
    }

    /// Check if this can be spent via script path
    pub fn supports_script_path(&self) -> bool {
        self.script_tree.is_some()
    }

    /// Get all available scripts for script path spending
    pub fn available_scripts(&self) -> Vec<&ScriptBuf> {
        self.script_paths.keys().collect()
    }
}

/// Builder for creating Taproot script trees
pub struct TaprootBuilder {
    /// Internal key for Taproot construction
    internal_key: XOnlyPublicKey,
    /// Script leaves to include in the tree
    script_leaves: Vec<(ScriptBuf, LeafVersion)>,
    /// Secp256k1 context
    secp: Secp256k1<All>,
}

impl TaprootBuilder {
    /// Create new Taproot builder
    pub fn new(internal_key: XOnlyPublicKey) -> Self {
        Self {
            internal_key,
            script_leaves: Vec::new(),
            secp: Secp256k1::new(),
        }
    }

    /// Add a script leaf to the tree
    pub fn add_script(mut self, script: ScriptBuf, leaf_version: LeafVersion) -> TransactionResult<Self> {
        self.script_leaves.push((script, leaf_version));
        Ok(self)
    }

    /// Add a script leaf with default version (0xc0)
    pub fn add_script_default(self, script: ScriptBuf) -> TransactionResult<Self> {
        self.add_script(script, LeafVersion::TapScript)
    }

    /// Build the Taproot spend data
    pub fn build(self) -> TransactionResult<TaprootSpendData> {
        if self.script_leaves.is_empty() {
            // Key-only spending
            return TaprootSpendData::key_only(self.internal_key);
        }

        // Build script tree using Bitcoin's TaprootBuilder
        let mut builder = BitcoinTaprootBuilder::new();
        
        for (script, leaf_version) in &self.script_leaves {
            builder = builder.add_leaf(0, script.clone())
                .map_err(|_| TransactionError::ScriptValidation("Failed to add leaf to Taproot tree".to_string()))?;
        }

        let tree_info = builder.finalize(&self.secp, self.internal_key)
            .map_err(|_| TransactionError::ScriptValidation("Failed to finalize Taproot tree".to_string()))?;

        // Extract spending information
        let mut script_paths = HashMap::new();
        let mut control_blocks = HashMap::new();

        for (script, leaf_version) in &self.script_leaves {
            // Get control block for this script
            if let Some((leaf_hash, merkle_path)) = tree_info.script_map().get(&(script.clone(), *leaf_version)) {
                let control_block = tree_info.control_block(&(script.clone(), *leaf_version))
                    .ok_or_else(|| TransactionError::ScriptValidation("Control block not found".to_string()))?;

                script_paths.insert(script.clone(), (*leaf_hash, merkle_path.clone()));
                control_blocks.insert(script.clone(), control_block);
            }
        }

        Ok(TaprootSpendData {
            internal_key: self.internal_key,
            output_key: tree_info.output_key(),
            script_tree: Some(*tree_info.tap_tree()),
            script_paths,
            control_blocks,
            merkle_root: tree_info.merkle_root(),
        })
    }
}

/// Utilities for Taproot script construction
pub struct TaprootUtils;

impl TaprootUtils {
    /// Create a simple key-path only Taproot output
    pub fn key_only_output(pubkey: &PublicKey) -> TransactionResult<TaprootSpendData> {
        let x_only = pubkey.inner.x_only_public_key().0;
        TaprootSpendData::key_only(x_only)
    }

    /// Create a Taproot output with a single script
    pub fn single_script_output(
        internal_key: &PublicKey,
        script: ScriptBuf,
    ) -> TransactionResult<TaprootSpendData> {
        let x_only = internal_key.inner.x_only_public_key().0;
        TaprootBuilder::new(x_only)
            .add_script_default(script)?
            .build()
    }

    /// Create a Taproot output with multiple scripts
    pub fn multi_script_output(
        internal_key: &PublicKey,
        scripts: Vec<ScriptBuf>,
    ) -> TransactionResult<TaprootSpendData> {
        let x_only = internal_key.inner.x_only_public_key().0;
        let mut builder = TaprootBuilder::new(x_only);
        
        for script in scripts {
            builder = builder.add_script_default(script)?;
        }
        
        builder.build()
    }

    /// Generate a timelock script for Taproot
    pub fn timelock_script(
        pubkey: &PublicKey,
        locktime: u32,
    ) -> TransactionResult<ScriptBuf> {
        // Create a simple timelock script: <pubkey> OP_CHECKSIG <locktime> OP_CHECKLOCKTIMEVERIFY
        let script = bitcoin::script::Builder::new()
            .push_slice(&pubkey.to_bytes())
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG)
            .push_int(locktime as i64)
            .push_opcode(bitcoin::opcodes::all::OP_CLTV)
            .into_script();
        Ok(script)
    }

    /// Generate a hash preimage script for Taproot
    pub fn hash_preimage_script(
        pubkey: &PublicKey,
        hash: &[u8; 32],
    ) -> TransactionResult<ScriptBuf> {
        // Create a hash preimage script: <hash> OP_SHA256 <pubkey> OP_CHECKSIG OP_BOOLAND
        let script = bitcoin::script::Builder::new()
            .push_slice(hash)
            .push_opcode(bitcoin::opcodes::all::OP_SHA256)
            .push_slice(&pubkey.to_bytes())
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG)
            .push_opcode(bitcoin::opcodes::all::OP_BOOLAND)
            .into_script();
        Ok(script)
    }

    /// Generate a multisig script for Taproot using Schnorr signatures
    pub fn schnorr_multisig_script(
        pubkeys: &[PublicKey],
        threshold: usize,
    ) -> TransactionResult<ScriptBuf> {
        if threshold == 0 || threshold > pubkeys.len() {
            return Err(TransactionError::ScriptValidation("Invalid threshold for multisig".to_string()));
        }

        // Create a simple threshold script for Taproot using Builder pattern
        // This is a simplified version - real implementations would use more efficient constructions
        let mut builder = bitcoin::script::Builder::new();
        
        // Add threshold value
        builder = builder.push_int(threshold as i64);
        
        // Add all public keys
        for pubkey in pubkeys {
            builder = builder.push_slice(&pubkey.to_bytes());
        }
        
        // Add pubkey count and multisig op
        builder = builder.push_int(pubkeys.len() as i64);
        builder = builder.push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG);
        
        Ok(builder.into_script())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{PrivateKey, Network};

    fn create_test_keys() -> (PrivateKey, PublicKey, XOnlyPublicKey) {
        let secp = Secp256k1::new();
        let private_key = PrivateKey::generate(Network::Regtest);
        let public_key = private_key.public_key(&secp);
        let x_only = public_key.inner.x_only_public_key().0;
        (private_key, public_key, x_only)
    }

    #[test]
    fn test_key_only_taproot() {
        let (_, _, x_only) = create_test_keys();
        let spend_data = TaprootSpendData::key_only(x_only);
        
        assert!(spend_data.is_ok());
        let spend_data = spend_data.unwrap();
        assert!(spend_data.supports_key_path());
        assert!(!spend_data.supports_script_path());
        assert_eq!(spend_data.internal_key, x_only);
    }

    #[test]
    fn test_taproot_builder() {
        let (_, pubkey, x_only) = create_test_keys();
        
        // Create a simple script
        let script = TaprootUtils::timelock_script(&pubkey, 100).unwrap();
        
        let builder = TaprootBuilder::new(x_only);
        let spend_data = builder.add_script_default(script).unwrap().build();
        
        assert!(spend_data.is_ok());
        let spend_data = spend_data.unwrap();
        assert!(spend_data.supports_key_path());
        assert!(spend_data.supports_script_path());
    }

    #[test]
    fn test_taproot_utils() {
        let (_, pubkey, _) = create_test_keys();
        
        // Test key-only output
        let spend_data = TaprootUtils::key_only_output(&pubkey);
        assert!(spend_data.is_ok());
        
        // Test timelock script generation
        let script = TaprootUtils::timelock_script(&pubkey, 144);
        assert!(script.is_ok());
        assert!(!script.unwrap().is_empty());
        
        // Test hash preimage script
        let hash = [0u8; 32];
        let script = TaprootUtils::hash_preimage_script(&pubkey, &hash);
        assert!(script.is_ok());
        assert!(!script.unwrap().is_empty());
    }

    #[test]
    fn test_multisig_script() {
        let (_, pubkey1, _) = create_test_keys();
        let (_, pubkey2, _) = create_test_keys();
        let (_, pubkey3, _) = create_test_keys();
        
        let pubkeys = vec![pubkey1, pubkey2, pubkey3];
        let script = TaprootUtils::schnorr_multisig_script(&pubkeys, 2);
        
        assert!(script.is_ok());
        assert!(!script.unwrap().is_empty());
    }
}