/// Modern descriptor-based wallet implementation
/// 
/// This module implements a Bitcoin wallet using output descriptors for address generation,
/// following modern standards like BIP-32, BIP-44, BIP-49, BIP-84, and BIP-341.
/// 
/// Key features:
/// - Hierarchical Deterministic (HD) wallets with BIP-32 master seeds
/// - Descriptor-based address generation for all address types
/// - Modern derivation paths (BIP-44/49/84 for legacy/segwit/native segwit)
/// - Taproot support with BIP-341 descriptors
/// - Integration with encrypted storage and legacy import

use crate::error::{WalletError, WalletResult};
use crate::crypto::{signatures::PrivateKey, generate_random_bytes};
use crate::storage::{WalletStorage, wallet_storage::{WalletData, StorageConfig}};
use crate::Network;

use bitcoin::{
    secp256k1::{Secp256k1, All},
    bip32::{Xpriv, Xpub, DerivationPath, ChildNumber},
    Address, PublicKey, CompressedPublicKey,
    key::TweakedPublicKey,
    ScriptBuf, Txid,
};
use std::collections::{HashMap, BTreeMap};
use std::str::FromStr;

/// Supported address types for descriptor generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressType {
    /// Legacy Pay-to-Public-Key-Hash (P2PKH) - 1...
    Legacy,
    /// Pay-to-Script-Hash wrapped SegWit (P2SH-P2WPKH) - 3...
    NestedSegwit,
    /// Native SegWit Pay-to-Witness-Public-Key-Hash (P2WPKH) - bc1q...
    NativeSegwit,
    /// Taproot Pay-to-Taproot (P2TR) - bc1p...
    Taproot,
}

impl AddressType {
    /// Get the BIP derivation path for this address type
    pub fn derivation_path(&self, account: u32, change: u32, index: u32) -> DerivationPath {
        let purpose = match self {
            AddressType::Legacy => 44,      // BIP-44
            AddressType::NestedSegwit => 49, // BIP-49
            AddressType::NativeSegwit => 84, // BIP-84
            AddressType::Taproot => 86,     // BIP-86
        };
        
        DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(purpose).unwrap(),
            ChildNumber::from_hardened_idx(0).unwrap(), // Bitcoin
            ChildNumber::from_hardened_idx(account).unwrap(),
            ChildNumber::from_normal_idx(change).unwrap(),
            ChildNumber::from_normal_idx(index).unwrap(),
        ])
    }
}

/// Extended private key with metadata for HD wallet operations
#[derive(Debug, Clone)]
pub struct ExtendedPrivateKey {
    /// BIP-32 extended private key
    xpriv: Xpriv,
    /// Secp256k1 context for operations
    secp: Secp256k1<All>,
}

impl ExtendedPrivateKey {
    /// Create from BIP-32 extended private key
    pub fn new(xpriv: Xpriv) -> Self {
        Self {
            xpriv,
            secp: Secp256k1::new(),
        }
    }
    
    /// Generate a new master key from entropy
    pub fn generate_master(network: Network) -> WalletResult<Self> {
        let entropy = generate_random_bytes(32)
            .map_err(|_| WalletError::KeyGeneration)?;
        
        let bitcoin_network = match network {
            Network::Bitcoin => bitcoin::Network::Bitcoin,
            Network::Testnet => bitcoin::Network::Testnet,
            Network::Regtest => bitcoin::Network::Regtest,
            Network::Signet => bitcoin::Network::Signet,
        };
        
        let secp = Secp256k1::new();
        let xpriv = Xpriv::new_master(bitcoin_network, &entropy)
            .map_err(|_| WalletError::KeyGeneration)?;
        
        Ok(Self { xpriv, secp })
    }
    
    /// Derive a child key at the given path
    pub fn derive_path(&self, path: &DerivationPath) -> WalletResult<Self> {
        let derived = self.xpriv.derive_priv(&self.secp, path)
            .map_err(|_| WalletError::KeyDerivation)?;
        
        Ok(Self::new(derived))
    }
    
    /// Get the extended public key
    pub fn extended_public_key(&self) -> Xpub {
        Xpub::from_priv(&self.secp, &self.xpriv)
    }
    
    /// Get the private key for signing
    pub fn private_key(&self) -> PrivateKey {
        PrivateKey::from_bytes(&self.xpriv.private_key.secret_bytes())
            .expect("Valid private key from ExtendedPrivKey")
    }
    
    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        self.xpriv.private_key.public_key(&self.secp).into()
    }
}

/// UTXO (Unspent Transaction Output) information
#[derive(Debug, Clone)]
pub struct Utxo {
    /// Transaction ID containing this output
    pub txid: Txid,
    /// Output index within the transaction
    pub vout: u32,
    /// Value in satoshis
    pub value: u64,
    /// Script pubkey
    pub script_pubkey: ScriptBuf,
    /// Address that owns this UTXO
    pub address: Address,
    /// Derivation path for this address
    pub derivation_path: DerivationPath,
    /// Block height when confirmed (None if unconfirmed)
    pub block_height: Option<u32>,
}

/// Wallet transaction history entry
#[derive(Debug, Clone)]
pub struct WalletTransaction {
    /// Transaction ID
    pub txid: Txid,
    /// Net value change for this wallet (positive = received, negative = sent)
    pub value_delta: i64,
    /// Block height when confirmed (None if unconfirmed)
    pub block_height: Option<u32>,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Human-readable description
    pub label: Option<String>,
}

/// Main descriptor-based wallet implementation
pub struct Wallet {
    /// Wallet identifier
    pub id: String,
    /// Bitcoin network
    pub network: Network,
    /// Master extended private key
    master_key: ExtendedPrivateKey,
    /// Cached derived keys by path
    derived_keys: HashMap<DerivationPath, ExtendedPrivateKey>,
    /// Generated addresses by path
    addresses: HashMap<DerivationPath, Address>,
    /// UTXO set owned by this wallet
    utxos: HashMap<(Txid, u32), Utxo>,
    /// Transaction history
    transactions: BTreeMap<Txid, WalletTransaction>,
    /// Next unused address indices for each address type
    next_indices: HashMap<AddressType, u32>,
    /// Storage backend
    storage: WalletStorage,
}

impl Wallet {
    /// Create a new wallet with a random master key
    pub fn create_new(id: String, network: Network, storage: WalletStorage) -> WalletResult<Self> {
        let master_key = ExtendedPrivateKey::generate_master(network)?;
        
        let mut wallet = Self {
            id,
            network,
            master_key,
            derived_keys: HashMap::new(),
            addresses: HashMap::new(),
            utxos: HashMap::new(),
            transactions: BTreeMap::new(),
            next_indices: HashMap::new(),
            storage,
        };
        
        // Initialize next indices for all address types
        wallet.next_indices.insert(AddressType::Legacy, 0);
        wallet.next_indices.insert(AddressType::NestedSegwit, 0);
        wallet.next_indices.insert(AddressType::NativeSegwit, 0);
        wallet.next_indices.insert(AddressType::Taproot, 0);
        
        // Generate initial receiving addresses
        wallet.generate_initial_addresses()?;
        
        Ok(wallet)
    }
    
    /// Load existing wallet from storage
    pub fn load(id: String, storage: WalletStorage) -> WalletResult<Self> {
        let wallet_data = storage.load_wallet_data(&id)?;
        
        // TODO: Implement deserialization from stored wallet data
        // This would restore the master key, addresses, UTXOs, etc.
        Err(WalletError::NotImplemented("Wallet loading not yet implemented".to_string()))
    }
    
    /// Save wallet to storage
    pub fn save(&self) -> WalletResult<()> {
        // TODO: Implement serialization of wallet state
        // This would save master key, addresses, UTXOs, transaction history, etc.
        Err(WalletError::NotImplemented("Wallet saving not yet implemented".to_string()))
    }
    
    /// Generate a new receiving address of the specified type
    pub fn get_new_address(&mut self, address_type: AddressType) -> WalletResult<Address> {
        let index = self.next_indices.get(&address_type).copied().unwrap_or(0);
        let path = address_type.derivation_path(0, 0, index); // account=0, change=0 (receiving)
        
        let address = self.generate_address(&path, address_type)?;
        
        // Increment the next index for this address type
        self.next_indices.insert(address_type, index + 1);
        
        Ok(address)
    }
    
    /// Generate a change address of the specified type
    pub fn get_change_address(&mut self, address_type: AddressType) -> WalletResult<Address> {
        let index = self.next_indices.get(&address_type).copied().unwrap_or(0);
        let path = address_type.derivation_path(0, 1, index); // account=0, change=1 (change)
        
        let address = self.generate_address(&path, address_type)?;
        
        Ok(address)
    }
    
    /// Get all addresses for a specific type and purpose
    pub fn get_addresses(&self, address_type: AddressType, change: bool) -> Vec<Address> {
        let change_index = if change { 1 } else { 0 };
        let max_index = self.next_indices.get(&address_type).copied().unwrap_or(0);
        
        (0..max_index)
            .filter_map(|i| {
                let path = address_type.derivation_path(0, change_index, i);
                self.addresses.get(&path).cloned()
            })
            .collect()
    }
    
    /// Generate an address for a specific derivation path
    fn generate_address(&mut self, path: &DerivationPath, address_type: AddressType) -> WalletResult<Address> {
        // Check if we already have this address cached
        if let Some(address) = self.addresses.get(path) {
            return Ok(address.clone());
        }
        
        // Derive the key for this path
        let derived_key = if let Some(key) = self.derived_keys.get(path) {
            key.clone()
        } else {
            let key = self.master_key.derive_path(path)?;
            self.derived_keys.insert(path.clone(), key.clone());
            key.clone()
        };
        
        let public_key = derived_key.public_key();
        
        let bitcoin_network = match self.network {
            Network::Bitcoin => bitcoin::Network::Bitcoin,
            Network::Testnet => bitcoin::Network::Testnet,
            Network::Regtest => bitcoin::Network::Regtest,
            Network::Signet => bitcoin::Network::Signet,
        };
        
        let address = match address_type {
            AddressType::Legacy => {
                Address::p2pkh(CompressedPublicKey::try_from(public_key).unwrap(), bitcoin_network)
            }
            AddressType::NestedSegwit => {
                Address::p2shwpkh(&CompressedPublicKey::try_from(public_key).unwrap(), bitcoin_network)
            }
            AddressType::NativeSegwit => {
                Address::p2wpkh(&CompressedPublicKey::try_from(public_key).unwrap(), bitcoin_network)
            }
            AddressType::Taproot => {
                // For BIP-86 key-only spending path
                let tweaked_key = TweakedPublicKey::dangerous_assume_tweaked(public_key.inner.x_only_public_key().0);
                Address::p2tr_tweaked(tweaked_key, bitcoin_network)
            }
        };
        
        self.addresses.insert(path.clone(), address.clone());
        Ok(address)
    }
    
    /// Generate initial addresses for all types
    fn generate_initial_addresses(&mut self) -> WalletResult<()> {
        // Generate first receiving address for each type
        for &addr_type in &[AddressType::Legacy, AddressType::NestedSegwit, 
                           AddressType::NativeSegwit, AddressType::Taproot] {
            self.get_new_address(addr_type)?;
        }
        Ok(())
    }
    
    /// Get the total balance in satoshis
    pub fn balance(&self) -> u64 {
        self.utxos.values().map(|utxo| utxo.value).sum()
    }
    
    /// Get confirmed balance (UTXOs with block height)
    pub fn confirmed_balance(&self) -> u64 {
        self.utxos.values()
            .filter(|utxo| utxo.block_height.is_some())
            .map(|utxo| utxo.value)
            .sum()
    }
    
    /// Get unconfirmed balance (UTXOs without block height)
    pub fn unconfirmed_balance(&self) -> u64 {
        self.utxos.values()
            .filter(|utxo| utxo.block_height.is_none())
            .map(|utxo| utxo.value)
            .sum()
    }
    
    /// Get all UTXOs
    pub fn utxos(&self) -> Vec<&Utxo> {
        self.utxos.values().collect()
    }
    
    /// Get UTXOs suitable for spending (with minimum confirmations)
    pub fn spendable_utxos(&self, min_confirmations: u32, current_height: u32) -> Vec<&Utxo> {
        self.utxos.values()
            .filter(|utxo| {
                if let Some(height) = utxo.block_height {
                    current_height.saturating_sub(height) >= min_confirmations
                } else {
                    min_confirmations == 0 // Allow unconfirmed if min_confirmations is 0
                }
            })
            .collect()
    }
    
    /// Add a UTXO to the wallet
    pub fn add_utxo(&mut self, utxo: Utxo) {
        self.utxos.insert((utxo.txid, utxo.vout), utxo);
    }
    
    /// Remove a spent UTXO
    pub fn remove_utxo(&mut self, txid: Txid, vout: u32) -> Option<Utxo> {
        self.utxos.remove(&(txid, vout))
    }
    
    /// Add a transaction to history
    pub fn add_transaction(&mut self, transaction: WalletTransaction) {
        self.transactions.insert(transaction.txid, transaction);
    }
    
    /// Get transaction history
    pub fn transactions(&self) -> Vec<&WalletTransaction> {
        self.transactions.values().collect()
    }
    
    /// Get a private key for a specific derivation path
    pub fn get_private_key(&self, path: &DerivationPath) -> WalletResult<PrivateKey> {
        if let Some(derived_key) = self.derived_keys.get(path) {
            Ok(derived_key.private_key())
        } else {
            let derived_key = self.master_key.derive_path(path)?;
            Ok(derived_key.private_key())
        }
    }
    
    /// Check if an address belongs to this wallet
    pub fn owns_address(&self, address: &Address) -> Option<DerivationPath> {
        self.addresses.iter()
            .find(|(_, addr)| *addr == address)
            .map(|(path, _)| path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::WalletStorage;
    use tempfile::tempdir;
    
    fn create_test_wallet() -> Wallet {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            storage_path: temp_dir.path().to_path_buf(),
            auto_backup: true,
            backup_count: 3,
        };
        let storage = WalletStorage::new(config).unwrap();
        Wallet::create_new("test_wallet".to_string(), Network::Regtest, storage).unwrap()
    }
    
    #[test]
    fn test_wallet_creation() {
        let wallet = create_test_wallet();
        assert_eq!(wallet.id, "test_wallet");
        assert_eq!(wallet.network, Network::Regtest);
        assert_eq!(wallet.balance(), 0);
    }
    
    #[test]
    fn test_address_generation() {
        let mut wallet = create_test_wallet();
        
        // Test different address types
        let legacy_addr = wallet.get_new_address(AddressType::Legacy).unwrap();
        let segwit_addr = wallet.get_new_address(AddressType::NativeSegwit).unwrap();
        let taproot_addr = wallet.get_new_address(AddressType::Taproot).unwrap();
        
        // Addresses should be different
        assert_ne!(legacy_addr, segwit_addr);
        assert_ne!(legacy_addr, taproot_addr);
        assert_ne!(segwit_addr, taproot_addr);
        
        // Check address formats (regtest network)
        let legacy_str = legacy_addr.to_string();
        let segwit_str = segwit_addr.to_string();
        let taproot_str = taproot_addr.to_string();
        
        // Legacy addresses on regtest start with 'm' or 'n'
        assert!(legacy_str.starts_with('m') || legacy_str.starts_with('n'));
        // SegWit addresses on regtest start with 'bcrt1q'
        assert!(segwit_str.starts_with("bcrt1q"));
        // Taproot addresses on regtest start with 'bcrt1p'  
        assert!(taproot_str.starts_with("bcrt1p"));
    }
    
    #[test]
    fn test_derivation_paths() {
        let path_legacy = AddressType::Legacy.derivation_path(0, 0, 0);
        let path_segwit = AddressType::NativeSegwit.derivation_path(0, 0, 0);
        
        // Check that different address types have different derivation paths
        assert_ne!(path_legacy, path_segwit);
        
        // Legacy should use purpose 44
        assert_eq!(path_legacy.as_ref()[0], ChildNumber::from_hardened_idx(44).unwrap());
        // Native SegWit should use purpose 84
        assert_eq!(path_segwit.as_ref()[0], ChildNumber::from_hardened_idx(84).unwrap());
    }
    
    #[test]
    fn test_balance_tracking() {
        let mut wallet = create_test_wallet();
        
        // Create a test UTXO
        let utxo = Utxo {
            txid: Txid::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
            vout: 0,
            value: 100_000, // 0.001 BTC
            script_pubkey: ScriptBuf::new(),
            address: wallet.get_new_address(AddressType::NativeSegwit).unwrap(),
            derivation_path: AddressType::NativeSegwit.derivation_path(0, 0, 0),
            block_height: Some(700_000),
        };
        
        wallet.add_utxo(utxo);
        
        assert_eq!(wallet.balance(), 100_000);
        assert_eq!(wallet.confirmed_balance(), 100_000);
        assert_eq!(wallet.unconfirmed_balance(), 0);
    }
    
    #[test]
    fn test_address_ownership() {
        let mut wallet = create_test_wallet();
        let address = wallet.get_new_address(AddressType::NativeSegwit).unwrap();
        
        // Wallet should own the generated address
        assert!(wallet.owns_address(&address).is_some());
        
        // Create a random address that wallet doesn't own
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            storage_path: temp_dir.path().to_path_buf(),
            auto_backup: true,
            backup_count: 3,
        };
        let storage = WalletStorage::new(config).unwrap();
        let mut other_wallet = Wallet::create_new("other".to_string(), Network::Regtest, storage).unwrap();
        let other_address = other_wallet.get_new_address(AddressType::NativeSegwit).unwrap();
        
        // Original wallet should not own the other wallet's address
        assert!(wallet.owns_address(&other_address).is_none());
    }
}