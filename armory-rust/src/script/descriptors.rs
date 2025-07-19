/// Script descriptor utilities and templates
/// 
/// This module provides high-level interfaces for working with Bitcoin script descriptors,
/// including common patterns and templates for wallet operations.

use crate::error::{TransactionResult, TransactionError};
use bitcoin::{PublicKey, ScriptBuf, Address, Network};
use miniscript::Descriptor;
use std::fmt;

/// Supported descriptor types for the wallet
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DescriptorType {
    /// Legacy Pay-to-Public-Key-Hash
    Legacy,
    /// SegWit v0 Pay-to-Witness-Public-Key-Hash
    Wpkh,
    /// SegWit v0 Pay-to-Witness-Script-Hash
    Wsh,
    /// SegWit v0 Pay-to-Script-Hash wrapped
    ShWpkh,
    /// Taproot Pay-to-Taproot
    Taproot,
    /// Custom miniscript descriptor
    Custom(String),
}

impl fmt::Display for DescriptorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescriptorType::Legacy => write!(f, "pkh"),
            DescriptorType::Wpkh => write!(f, "wpkh"),
            DescriptorType::Wsh => write!(f, "wsh"),
            DescriptorType::ShWpkh => write!(f, "sh(wpkh)"),
            DescriptorType::Taproot => write!(f, "tr"),
            DescriptorType::Custom(desc) => write!(f, "{}", desc),
        }
    }
}

/// Script descriptor wrapper with additional metadata
#[derive(Debug, Clone)]
pub struct ScriptDescriptor {
    /// The descriptor type
    pub descriptor_type: DescriptorType,
    /// Raw descriptor string
    pub descriptor: String,
    /// Parsed miniscript descriptor
    parsed: Option<Descriptor<PublicKey>>,
    /// Network for address generation
    pub network: Network,
}

impl ScriptDescriptor {
    /// Create a new script descriptor
    pub fn new(descriptor_type: DescriptorType, descriptor: String, network: Network) -> TransactionResult<Self> {
        let mut result = Self {
            descriptor_type,
            descriptor: descriptor.clone(),
            parsed: None,
            network,
        };
        
        // Parse the descriptor to validate it
        result.parse()?;
        Ok(result)
    }

    /// Create a Legacy P2PKH descriptor
    pub fn legacy(pubkey: &PublicKey, network: Network) -> TransactionResult<Self> {
        let descriptor = format!("pkh({})", pubkey);
        Self::new(DescriptorType::Legacy, descriptor, network)
    }

    /// Create a SegWit P2WPKH descriptor
    pub fn wpkh(pubkey: &PublicKey, network: Network) -> TransactionResult<Self> {
        let descriptor = format!("wpkh({})", pubkey);
        Self::new(DescriptorType::Wpkh, descriptor, network)
    }

    /// Create a SegWit P2SH-wrapped P2WPKH descriptor
    pub fn sh_wpkh(pubkey: &PublicKey, network: Network) -> TransactionResult<Self> {
        let descriptor = format!("sh(wpkh({}))", pubkey);
        Self::new(DescriptorType::ShWpkh, descriptor, network)
    }

    /// Create a Taproot descriptor with key-only spending
    pub fn taproot_key_only(pubkey: &PublicKey, network: Network) -> TransactionResult<Self> {
        // Convert to x-only public key for Taproot
        let x_only = pubkey.inner.x_only_public_key().0;
        let descriptor = format!("tr({})", x_only);
        Self::new(DescriptorType::Taproot, descriptor, network)
    }

    /// Create a custom descriptor from string
    pub fn custom(descriptor: String, network: Network) -> TransactionResult<Self> {
        Self::new(DescriptorType::Custom(descriptor.clone()), descriptor, network)
    }

    /// Parse the descriptor string into a miniscript descriptor
    fn parse(&mut self) -> TransactionResult<()> {
        if self.parsed.is_some() {
            return Ok(());
        }

        let parsed: Descriptor<PublicKey> = self.descriptor.parse()
            .map_err(|e| TransactionError::ScriptValidation(format!("Invalid descriptor '{}': {}", self.descriptor, e)))?;

        self.parsed = Some(parsed);
        Ok(())
    }

    /// Get the parsed descriptor
    pub fn parsed(&mut self) -> TransactionResult<Descriptor<PublicKey>> {
        self.parse()?;
        Ok(self.parsed.as_ref().unwrap().clone())
    }

    /// Generate the script pubkey for this descriptor
    pub fn script_pubkey(&mut self) -> TransactionResult<ScriptBuf> {
        let desc = self.parsed()?;
        Ok(desc.script_pubkey())
    }

    /// Generate an address for this descriptor
    pub fn address(&mut self) -> TransactionResult<Address> {
        let desc = self.parsed()?;
        let network = self.network;
        Ok(desc.address(network)
            .map_err(|e| TransactionError::ScriptValidation(format!("Address generation failed: {}", e)))?)
    }

    /// Check if this descriptor requires signatures
    pub fn requires_signatures(&mut self) -> TransactionResult<bool> {
        let _desc = self.parsed()?;
        
        // Most descriptors require signatures except for some custom scripts
        match &self.descriptor_type {
            DescriptorType::Legacy | 
            DescriptorType::Wpkh | 
            DescriptorType::ShWpkh |
            DescriptorType::Taproot => Ok(true),
            DescriptorType::Wsh => {
                // WSH can contain complex scripts that may not require signatures
                Ok(true) // Conservative assumption
            },
            DescriptorType::Custom(_) => {
                // Analyze the descriptor to determine if signatures are needed
                Ok(true) // Conservative assumption
            }
        }
    }

    /// Get maximum witness size for fee estimation
    pub fn max_witness_size(&mut self) -> TransactionResult<usize> {
        match &self.descriptor_type {
            DescriptorType::Legacy => Ok(0), // No witness for legacy
            DescriptorType::Wpkh => Ok(107), // signature (73) + pubkey (33) + length bytes
            DescriptorType::ShWpkh => Ok(107), // Same as WPKH for witness part
            DescriptorType::Taproot => Ok(65), // Schnorr signature (64) + sighash flag (1)
            DescriptorType::Wsh => Ok(500), // Conservative estimate for complex scripts
            DescriptorType::Custom(_) => {
                // Try to estimate based on descriptor analysis
                Ok(200) // Conservative estimate
            }
        }
    }

    /// Get maximum script_sig size for fee estimation
    pub fn max_script_sig_size(&mut self) -> TransactionResult<usize> {
        match &self.descriptor_type {
            DescriptorType::Legacy => Ok(108), // signature (73) + pubkey (33) + length bytes
            DescriptorType::Wpkh => Ok(0), // No script_sig for native SegWit
            DescriptorType::ShWpkh => Ok(23), // Just the redeem script (0014{20-byte-pubkey-hash})
            DescriptorType::Taproot => Ok(0), // No script_sig for Taproot
            DescriptorType::Wsh => Ok(0), // Native SegWit WSH has no script_sig
            DescriptorType::Custom(_) => {
                // Analyze the descriptor
                Ok(50) // Conservative estimate
            }
        }
    }

    /// Check if this is a SegWit descriptor
    pub fn is_segwit(&self) -> bool {
        matches!(self.descriptor_type, 
            DescriptorType::Wpkh | 
            DescriptorType::Wsh | 
            DescriptorType::ShWpkh |
            DescriptorType::Taproot
        )
    }

    /// Check if this is a Taproot descriptor
    pub fn is_taproot(&self) -> bool {
        matches!(self.descriptor_type, DescriptorType::Taproot)
    }

    /// Generate descriptor templates for common wallet patterns
    pub fn wallet_templates() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Legacy P2PKH", "pkh(KEY)"),
            ("SegWit P2WPKH", "wpkh(KEY)"),
            ("SegWit Nested P2SH", "sh(wpkh(KEY))"),
            ("Taproot Key Only", "tr(KEY)"),
            ("2-of-3 Multisig Legacy", "sh(multi(2,KEY1,KEY2,KEY3))"),
            ("2-of-3 Multisig SegWit", "wsh(multi(2,KEY1,KEY2,KEY3))"),
            ("Timelock Script", "wsh(and_v(v:pk(KEY),after(BLOCKHEIGHT)))"),
            ("Hash Preimage Script", "wsh(and_v(v:pk(KEY),sha256(HASH)))"),
        ]
    }
}

/// Descriptor utilities for common operations
pub struct DescriptorUtils;

impl DescriptorUtils {
    /// Estimate transaction size using descriptors
    pub fn estimate_tx_size(
        input_descriptor_types: &[DescriptorType],
        output_count: usize,
    ) -> TransactionResult<usize> {
        let mut size = 10; // Base transaction size (version, input/output counts, locktime)
        
        // Add input sizes
        for desc_type in input_descriptor_types {
            size += 32 + 4 + 4; // txid + vout + sequence
            size += Self::estimate_script_sig_size(desc_type);
            
            if Self::is_segwit_type(desc_type) {
                // SegWit inputs have witness data
                size += Self::estimate_witness_size_for_type(desc_type) / 4; // Witness discount
            }
        }
        
        // Add output sizes (conservative estimate)
        size += output_count * 34; // value (8) + script length (1) + script (~25)
        
        Ok(size)
    }

    /// Estimate witness size for fee calculation
    pub fn estimate_witness_size_for_types(descriptor_types: &[DescriptorType]) -> usize {
        let mut size = 0;
        
        for desc_type in descriptor_types {
            if Self::is_segwit_type(desc_type) {
                size += Self::estimate_witness_size_for_type(desc_type);
            }
        }
        
        size
    }

    /// Check if all descriptors are compatible with each other
    pub fn are_compatible(descriptors: &[&ScriptDescriptor]) -> bool {
        // All descriptors should target the same network
        if let Some(first) = descriptors.first() {
            descriptors.iter().all(|desc| desc.network == first.network)
        } else {
            true
        }
    }
    
    /// Helper function to check if descriptor type is SegWit
    fn is_segwit_type(desc_type: &DescriptorType) -> bool {
        matches!(desc_type, 
            DescriptorType::Wpkh | 
            DescriptorType::Wsh | 
            DescriptorType::ShWpkh |
            DescriptorType::Taproot
        )
    }
    
    /// Helper function to estimate script sig size by type
    fn estimate_script_sig_size(desc_type: &DescriptorType) -> usize {
        match desc_type {
            DescriptorType::Legacy => 108, // signature (73) + pubkey (33) + length bytes
            DescriptorType::Wpkh => 0, // No script_sig for native SegWit
            DescriptorType::ShWpkh => 23, // Just the redeem script
            DescriptorType::Taproot => 0, // No script_sig for Taproot
            DescriptorType::Wsh => 0, // Native SegWit WSH has no script_sig
            DescriptorType::Custom(_) => 50, // Conservative estimate
        }
    }
    
    /// Helper function to estimate witness size by type
    fn estimate_witness_size_for_type(desc_type: &DescriptorType) -> usize {
        match desc_type {
            DescriptorType::Legacy => 0, // No witness for legacy
            DescriptorType::Wpkh => 107, // signature (73) + pubkey (33) + length bytes
            DescriptorType::ShWpkh => 107, // Same as WPKH for witness part
            DescriptorType::Taproot => 65, // Schnorr signature (64) + sighash flag (1)
            DescriptorType::Wsh => 500, // Conservative estimate for complex scripts
            DescriptorType::Custom(_) => 200, // Conservative estimate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{Network, PrivateKey};
    use bitcoin::secp256k1::rand::thread_rng;

    fn create_test_pubkey() -> PublicKey {
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let private_key = PrivateKey::generate(Network::Regtest);
        private_key.public_key(&secp)
    }

    #[test]
    fn test_descriptor_creation() {
        let pubkey = create_test_pubkey();
        let desc = ScriptDescriptor::wpkh(&pubkey, Network::Regtest);
        assert!(desc.is_ok());
        
        let desc = desc.unwrap();
        assert_eq!(desc.descriptor_type, DescriptorType::Wpkh);
        assert!(desc.is_segwit());
    }

    #[test]
    fn test_descriptor_script_generation() {
        let pubkey = create_test_pubkey();
        let mut desc = ScriptDescriptor::wpkh(&pubkey, Network::Regtest).unwrap();
        
        let script = desc.script_pubkey();
        assert!(script.is_ok());
        
        let script = script.unwrap();
        assert!(script.is_p2wpkh());
    }

    #[test]
    fn test_descriptor_address_generation() {
        let pubkey = create_test_pubkey();
        let mut desc = ScriptDescriptor::wpkh(&pubkey, Network::Regtest).unwrap();
        
        let address = desc.address();
        assert!(address.is_ok());
        
        let address = address.unwrap();
        assert_eq!(address.network(), Network::Regtest);
    }

    #[test]
    fn test_size_estimation() {
        let descriptor_types = vec![DescriptorType::Wpkh, DescriptorType::Taproot];
        
        let size = DescriptorUtils::estimate_tx_size(&descriptor_types, 2);
        assert!(size.is_ok());
        assert!(size.unwrap() > 100); // Should be reasonable transaction size
        
        let witness_size = DescriptorUtils::estimate_witness_size_for_types(&descriptor_types);
        assert!(witness_size > 0); // SegWit descriptors should have witness data
    }
}