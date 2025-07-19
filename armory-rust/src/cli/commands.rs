use bitcoin::Network;
/// CLI commands implementation
///
/// Complete command-line interface for Armory Bitcoin wallet operations
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// CLI commands structure
#[derive(Parser)]
#[command(name = "armory-rust")]
#[command(about = "Modern Rust implementation of Armory Bitcoin wallet")]
#[command(version = "1.0.0")]
pub struct CliCommands {
    /// Set the network (bitcoin, testnet, regtest)
    #[arg(long, default_value = "bitcoin")]
    pub network: NetworkArg,

    /// Data directory for wallet storage
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone)]
pub enum NetworkArg {
    Bitcoin,
    Testnet,
    Regtest,
}

impl From<NetworkArg> for Network {
    fn from(net: NetworkArg) -> Self {
        match net {
            NetworkArg::Bitcoin => Network::Bitcoin,
            NetworkArg::Testnet => Network::Testnet,
            NetworkArg::Regtest => Network::Regtest,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet
    Create {
        /// Wallet name
        name: String,
        /// Use mnemonic seed phrase (default: generate new)
        #[arg(long)]
        mnemonic: Option<String>,
        /// Encrypt wallet with password
        #[arg(long)]
        encrypt: bool,
        /// Account number for HD derivation (default: 0)
        #[arg(long, default_value = "0")]
        account: u32,
    },

    /// List all wallets
    List,

    /// Get wallet information
    Info {
        /// Wallet name
        wallet: String,
    },

    /// Generate or list addresses
    Address {
        /// Wallet name
        wallet: String,
        /// Address type
        #[arg(long, default_value = "native-segwit")]
        address_type: AddressType,
        /// Number of addresses to generate (default: 1)
        #[arg(long, default_value = "1")]
        count: u32,
        /// List existing addresses instead of generating new ones
        #[arg(long)]
        list: bool,
    },

    /// Check wallet balance
    Balance {
        /// Wallet name
        wallet: String,
        /// Include unconfirmed transactions
        #[arg(long)]
        include_unconfirmed: bool,
    },

    /// Send Bitcoin transaction
    Send {
        /// Wallet name
        wallet: String,
        /// Recipient address
        to: String,
        /// Amount in satoshis
        amount: u64,
        /// Fee rate in sat/vB
        #[arg(long)]
        fee_rate: Option<f64>,
        /// Create unsigned PSBT instead of broadcasting
        #[arg(long)]
        psbt_only: bool,
    },

    /// Sign a PSBT (Partially Signed Bitcoin Transaction)
    Sign {
        /// Wallet name
        wallet: String,
        /// PSBT file path or hex string
        psbt: String,
        /// Output file for signed PSBT
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Import a PSBT
    Import {
        /// PSBT file path or hex string
        psbt: String,
        /// Output file path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Export wallet information
    Export {
        /// Wallet name
        wallet: String,
        /// Export type
        #[arg(long, default_value = "psbt")]
        format: ExportFormat,
        /// Output file path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Multi-signature operations
    Multisig {
        #[command(subcommand)]
        command: MultisigCommands,
    },

    /// Import legacy Armory wallet
    LegacyImport {
        /// Path to legacy wallet file
        wallet_file: PathBuf,
        /// New wallet name
        new_name: String,
        /// Legacy wallet passphrase
        #[arg(long)]
        passphrase: Option<String>,
    },

    /// Backup wallet
    Backup {
        /// Wallet name
        wallet: String,
        /// Backup file path
        output: PathBuf,
    },

    /// Restore wallet from backup
    Restore {
        /// Backup file path
        backup: PathBuf,
        /// New wallet name
        new_name: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum AddressType {
    Legacy,
    NestedSegwit,
    NativeSegwit,
    Taproot,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Psbt,
    Descriptors,
    Xpub,
    Addresses,
}

#[derive(Subcommand)]
pub enum MultisigCommands {
    /// Create multi-signature wallet
    Create {
        /// Wallet name
        name: String,
        /// Required signatures (M in M-of-N)
        threshold: u32,
        /// Extended public keys (N keys)
        xpubs: Vec<String>,
    },

    /// Add signature to PSBT
    Sign {
        /// PSBT file path or hex string
        psbt: String,
        /// Wallet name for signing
        wallet: String,
        /// Output file for updated PSBT
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Combine multiple signed PSBTs
    Combine {
        /// PSBT file paths
        psbts: Vec<PathBuf>,
        /// Output file for combined PSBT
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Finalize multi-signature PSBT
    Finalize {
        /// PSBT file path or hex string
        psbt: String,
        /// Output file for finalized transaction
        #[arg(long)]
        output: Option<PathBuf>,
        /// Broadcast transaction after finalization
        #[arg(long)]
        broadcast: bool,
    },
}
