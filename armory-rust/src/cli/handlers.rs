/// CLI command handlers
///
/// Implementation of CLI command execution logic
use crate::cli::{
    AddressType as CliAddressType, CliConfig, Commands, ExportFormat, MultisigCommands,
};
use crate::error::{WalletError, WalletResult};
use crate::storage::WalletStorage;
use crate::wallet::{descriptor_wallet::AddressType, Wallet};
use std::path::PathBuf;

impl From<CliAddressType> for AddressType {
    fn from(cli_type: CliAddressType) -> Self {
        match cli_type {
            CliAddressType::Legacy => AddressType::Legacy,
            CliAddressType::NestedSegwit => AddressType::NestedSegwit,
            CliAddressType::NativeSegwit => AddressType::NativeSegwit,
            CliAddressType::Taproot => AddressType::Taproot,
        }
    }
}

/// CLI command handler
pub struct CliHandler {
    config: CliConfig,
}

impl CliHandler {
    /// Create new CLI handler
    pub fn new(config: CliConfig) -> WalletResult<Self> {
        config.ensure_directories()?;
        Ok(Self { config })
    }

    /// Execute CLI command
    pub async fn execute(&self, command: Commands) -> WalletResult<()> {
        match command {
            Commands::Create {
                name,
                mnemonic,
                encrypt,
                account,
            } => self.handle_create(name, mnemonic, encrypt, account).await,
            Commands::List => self.handle_list().await,
            Commands::Info { wallet } => self.handle_info(wallet).await,
            Commands::Address {
                wallet,
                address_type,
                count,
                list,
            } => self.handle_address(wallet, address_type, count, list).await,
            Commands::Balance {
                wallet,
                include_unconfirmed,
            } => self.handle_balance(wallet, include_unconfirmed).await,
            Commands::Send {
                wallet,
                to,
                amount,
                fee_rate,
                psbt_only,
            } => {
                self.handle_send(wallet, to, amount, fee_rate, psbt_only)
                    .await
            }
            Commands::Sign {
                wallet,
                psbt,
                output,
            } => self.handle_sign(wallet, psbt, output).await,
            Commands::Import { psbt, output } => self.handle_import(psbt, output).await,
            Commands::Export {
                wallet,
                format,
                output,
            } => self.handle_export(wallet, format, output).await,
            Commands::Multisig { command } => self.handle_multisig(command).await,
            Commands::LegacyImport {
                wallet_file,
                new_name,
                passphrase,
            } => {
                self.handle_legacy_import(wallet_file, new_name, passphrase)
                    .await
            }
            Commands::Backup { wallet, output } => self.handle_backup(wallet, output).await,
            Commands::Restore { backup, new_name } => self.handle_restore(backup, new_name).await,
        }
    }

    /// Handle wallet creation
    async fn handle_create(
        &self,
        name: String,
        mnemonic: Option<String>,
        encrypt: bool,
        account: u32,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Creating wallet '{name}' with account {account}");
        }

        // Check if wallet already exists
        let storage_config = crate::storage::wallet_storage::StorageConfig {
            storage_path: self.config.wallet_dir().join(&name),
            auto_backup: true,
            backup_count: 5,
        };

        let storage = WalletStorage::new(storage_config)?;

        if storage.wallet_exists(&name)? {
            return Err(WalletError::InvalidInput(format!(
                "Wallet '{name}' already exists"
            )));
        }

        // Create wallet
        let wallet = if let Some(_mnemonic_str) = mnemonic {
            if self.config.verbose {
                println!("Using provided mnemonic seed phrase");
            }
            println!("⚠️  Mnemonic import not yet implemented, creating new wallet instead");
            Wallet::create_new(name.clone(), self.config.network.into(), storage)?
        } else {
            if self.config.verbose {
                println!("Generating new mnemonic seed phrase");
            }
            Wallet::create_new(name.clone(), self.config.network.into(), storage)?
        };

        // Save the wallet
        wallet.save()?;

        // Handle encryption
        if encrypt {
            // In a real implementation, we'd prompt for password
            println!("⚠️  Wallet encryption not yet implemented in CLI");
        }

        // Display the mnemonic for backup (placeholder)
        println!("\n🔐 IMPORTANT: Save your mnemonic seed phrase:");
        println!("   [Mnemonic display not yet implemented]");
        println!("   Keep this safe - it's needed to recover your wallet!");
        println!();

        println!("✅ Wallet '{name}' created successfully");
        println!("   Network: {:?}", self.config.network);
        println!("   Account: {account}");
        println!(
            "   Storage: {}",
            self.config.wallet_dir().join(&name).display()
        );

        Ok(())
    }

    /// Handle wallet listing
    async fn handle_list(&self) -> WalletResult<()> {
        if self.config.verbose {
            println!("Listing wallets in {}", self.config.wallet_dir().display());
        }

        // Scan all wallet directories for database files
        let wallet_dir = self.config.wallet_dir();

        if !wallet_dir.exists() {
            println!("No wallets found. Create one with: armory-rust create <name>");
            return Ok(());
        }

        let mut wallets = Vec::new();

        // Check each subdirectory for wallet databases
        for entry in std::fs::read_dir(wallet_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let storage_config = crate::storage::wallet_storage::StorageConfig {
                        storage_path: entry.path(),
                        auto_backup: true,
                        backup_count: 5,
                    };

                    // Try to create storage to check if wallet exists
                    if let Ok(storage) = WalletStorage::new(storage_config) {
                        if let Ok(true) = storage.wallet_exists(name) {
                            wallets.push(name.to_string());
                        }
                    }
                }
            }
        }

        if wallets.is_empty() {
            println!("No wallets found. Create one with: armory-rust create <name>");
        } else {
            println!("Available wallets:");
            for wallet in wallets {
                println!("  • {wallet}");
            }
        }

        Ok(())
    }

    /// Handle wallet info
    async fn handle_info(&self, wallet_name: String) -> WalletResult<()> {
        if self.config.verbose {
            println!("Getting info for wallet '{wallet_name}'");
        }

        let wallet = self.load_wallet(&wallet_name)?;

        println!("📋 Wallet Information");
        println!("  Name: {}", wallet.id);
        println!("  Label: {}", wallet.label);
        println!("  Network: {:?}", wallet.network);
        println!("  Created: {}", format_timestamp(wallet.created_at()));
        println!("  Modified: {}", format_timestamp(wallet.modified_at()));
        println!();

        // Balance information
        println!("💰 Balance");
        let total_balance = wallet.balance();
        let confirmed_balance = wallet.confirmed_balance();
        let unconfirmed_balance = wallet.unconfirmed_balance();

        println!(
            "  Total: {} sats ({:.8} BTC)",
            total_balance,
            total_balance as f64 / 100_000_000.0
        );
        println!(
            "  Confirmed: {} sats ({:.8} BTC)",
            confirmed_balance,
            confirmed_balance as f64 / 100_000_000.0
        );
        println!(
            "  Unconfirmed: {} sats ({:.8} BTC)",
            unconfirmed_balance,
            unconfirmed_balance as f64 / 100_000_000.0
        );
        println!();

        // Address counts
        println!("📍 Addresses");
        for &addr_type in &[
            AddressType::Legacy,
            AddressType::NestedSegwit,
            AddressType::NativeSegwit,
            AddressType::Taproot,
        ] {
            let receiving_count = wallet.get_addresses(addr_type, false).len();
            let change_count = wallet.get_addresses(addr_type, true).len();
            println!("  {addr_type:?}: {receiving_count} receiving, {change_count} change");
        }
        println!();

        // Transaction history
        let transactions = wallet.transactions();
        println!("📈 Transactions: {}", transactions.len());

        // UTXO information
        let utxos = wallet.utxos();
        println!("🔗 UTXOs: {}", utxos.len());

        Ok(())
    }

    /// Handle address generation/listing
    async fn handle_address(
        &self,
        wallet_name: String,
        address_type: CliAddressType,
        count: u32,
        list: bool,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Handling addresses for wallet '{wallet_name}'");
        }

        let mut wallet = self.load_wallet(&wallet_name)?;
        let wallet_address_type: AddressType = address_type.clone().into();

        if list {
            println!("📍 Addresses for wallet '{wallet_name}':");

            // Get existing receiving addresses
            let receiving_addrs = wallet.get_addresses(wallet_address_type, false);
            let change_addrs = wallet.get_addresses(wallet_address_type, true);

            if receiving_addrs.is_empty() && change_addrs.is_empty() {
                println!("  No {address_type:?} addresses found");
            } else {
                if !receiving_addrs.is_empty() {
                    println!("  Receiving addresses:");
                    for (i, addr) in receiving_addrs.iter().enumerate() {
                        println!("    {}. {}", i + 1, addr);
                    }
                }

                if !change_addrs.is_empty() {
                    println!("  Change addresses:");
                    for (i, addr) in change_addrs.iter().enumerate() {
                        println!("    {}. {}", i + 1, addr);
                    }
                }
            }
        } else {
            println!("📍 Generated {count} new {address_type:?} address(es):");
            for i in 0..count {
                let address = wallet.get_new_address(wallet_address_type)?;
                println!("  {}. {}", i + 1, address);
            }

            // Save the wallet with new addresses
            wallet.save()?;

            if self.config.verbose {
                println!("  Wallet saved with new addresses");
            }
        }

        Ok(())
    }

    /// Handle balance checking
    async fn handle_balance(
        &self,
        wallet_name: String,
        include_unconfirmed: bool,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Checking balance for wallet '{wallet_name}'");
        }

        let _wallet = self.load_wallet(&wallet_name)?;

        println!("💰 Balance for wallet '{wallet_name}':");
        println!("  Confirmed: 0.00000000 BTC");
        if include_unconfirmed {
            println!("  Unconfirmed: 0.00000000 BTC");
            println!("  Total: 0.00000000 BTC");
        }
        println!("  (Balance checking requires network integration)");

        Ok(())
    }

    /// Handle sending transactions
    async fn handle_send(
        &self,
        wallet_name: String,
        to: String,
        amount: u64,
        fee_rate: Option<f64>,
        psbt_only: bool,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Sending {amount} sats from '{wallet_name}' to '{to}'");
        }

        let _wallet = self.load_wallet(&wallet_name)?;

        println!("🚀 Transaction preparation:");
        println!("  From: {wallet_name}");
        println!("  To: {to}");
        println!("  Amount: {amount} sats");
        if let Some(rate) = fee_rate {
            println!("  Fee rate: {rate} sat/vB");
        }

        if psbt_only {
            println!("  Creating unsigned PSBT...");
            println!("  (PSBT creation not yet implemented)");
        } else {
            println!("  (Transaction creation and broadcasting not yet implemented)");
        }

        Ok(())
    }

    /// Handle PSBT signing
    async fn handle_sign(
        &self,
        wallet_name: String,
        psbt: String,
        output: Option<PathBuf>,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Signing PSBT with wallet '{wallet_name}'");
        }

        let _wallet = self.load_wallet(&wallet_name)?;

        println!("✍️  PSBT Signing:");
        println!("  Wallet: {wallet_name}");
        println!(
            "  PSBT: {}",
            if psbt.len() > 50 { &psbt[..50] } else { &psbt }
        );
        if let Some(output_path) = output {
            println!("  Output: {}", output_path.display());
        }
        println!("  (PSBT signing not yet implemented)");

        Ok(())
    }

    /// Handle PSBT import
    async fn handle_import(&self, psbt: String, output: Option<PathBuf>) -> WalletResult<()> {
        if self.config.verbose {
            println!("Importing PSBT");
        }

        println!("📥 PSBT Import:");
        println!(
            "  PSBT: {}",
            if psbt.len() > 50 { &psbt[..50] } else { &psbt }
        );
        if let Some(output_path) = output {
            println!("  Output: {}", output_path.display());
        }
        println!("  (PSBT import not yet implemented)");

        Ok(())
    }

    /// Handle wallet export
    async fn handle_export(
        &self,
        wallet_name: String,
        format: ExportFormat,
        output: Option<PathBuf>,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Exporting wallet '{wallet_name}' as {format:?}");
        }

        let _wallet = self.load_wallet(&wallet_name)?;

        println!("📤 Wallet Export:");
        println!("  Wallet: {wallet_name}");
        println!("  Format: {format:?}");
        if let Some(output_path) = output {
            println!("  Output: {}", output_path.display());
        }
        println!("  (Wallet export not yet implemented)");

        Ok(())
    }

    /// Handle multisig commands
    async fn handle_multisig(&self, command: MultisigCommands) -> WalletResult<()> {
        match command {
            MultisigCommands::Create {
                name,
                threshold,
                xpubs,
            } => {
                println!("🔐 Creating multisig wallet:");
                println!("  Name: {name}");
                println!("  Threshold: {} of {}", threshold, xpubs.len());
                println!("  (Multisig creation not yet implemented)");
            }
            MultisigCommands::Sign {
                psbt: _,
                wallet,
                output,
            } => {
                println!("✍️  Multisig PSBT signing:");
                println!("  Wallet: {wallet}");
                if let Some(output_path) = output {
                    println!("  Output: {}", output_path.display());
                }
                println!("  (Multisig signing not yet implemented)");
            }
            MultisigCommands::Combine { psbts, output } => {
                println!("🔗 Combining {} PSBTs", psbts.len());
                if let Some(output_path) = output {
                    println!("  Output: {}", output_path.display());
                }
                println!("  (PSBT combining not yet implemented)");
            }
            MultisigCommands::Finalize {
                psbt: _,
                output,
                broadcast,
            } => {
                println!("🏁 Finalizing PSBT");
                if let Some(output_path) = output {
                    println!("  Output: {}", output_path.display());
                }
                if broadcast {
                    println!("  Broadcasting after finalization");
                }
                println!("  (PSBT finalization not yet implemented)");
            }
        }

        Ok(())
    }

    /// Handle legacy wallet import
    async fn handle_legacy_import(
        &self,
        wallet_file: PathBuf,
        new_name: String,
        passphrase: Option<String>,
    ) -> WalletResult<()> {
        if self.config.verbose {
            println!("Importing legacy wallet from {}", wallet_file.display());
        }

        println!("📦 Legacy Wallet Import:");
        println!("  Source: {}", wallet_file.display());
        println!("  New name: {new_name}");
        if passphrase.is_some() {
            println!("  Using passphrase: Yes");
        }
        println!("  (Legacy import not yet implemented)");

        Ok(())
    }

    /// Handle wallet backup
    async fn handle_backup(&self, wallet_name: String, output: PathBuf) -> WalletResult<()> {
        if self.config.verbose {
            println!(
                "Backing up wallet '{}' to {}",
                wallet_name,
                output.display()
            );
        }

        let _wallet = self.load_wallet(&wallet_name)?;

        println!("💾 Wallet Backup:");
        println!("  Wallet: {wallet_name}");
        println!("  Output: {}", output.display());
        println!("  (Wallet backup not yet implemented)");

        Ok(())
    }

    /// Handle wallet restore
    async fn handle_restore(&self, backup: PathBuf, new_name: String) -> WalletResult<()> {
        if self.config.verbose {
            println!("Restoring wallet from {}", backup.display());
        }

        println!("📥 Wallet Restore:");
        println!("  Backup: {}", backup.display());
        println!("  New name: {new_name}");
        println!("  (Wallet restore not yet implemented)");

        Ok(())
    }

    /// Load wallet by name
    fn load_wallet(&self, name: &str) -> WalletResult<Wallet> {
        let storage_config = crate::storage::wallet_storage::StorageConfig {
            storage_path: self.config.wallet_dir().join(name),
            auto_backup: true,
            backup_count: 5,
        };

        let storage = WalletStorage::new(storage_config.clone())?;

        // Check if wallet exists
        if !storage.wallet_exists(name)? {
            return Err(WalletError::InvalidInput(format!(
                "Wallet '{name}' not found"
            )));
        }

        // Try to load the wallet (will fail for now due to unimplemented loading)
        match Wallet::load(name.to_string(), storage) {
            Ok(wallet) => Ok(wallet),
            Err(WalletError::NotImplemented(_)) => {
                // Fallback: create a dummy wallet for now until loading is implemented
                if self.config.verbose {
                    println!("⚠️  Wallet loading not yet implemented, creating temporary wallet");
                }
                let new_storage = WalletStorage::new(storage_config.clone())?;
                Wallet::create_new(name.to_string(), self.config.network.into(), new_storage)
            }
            Err(e) => Err(e),
        }
    }
}

/// Format timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    match std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp)) {
        Some(time) => {
            if let Ok(datetime) = time.duration_since(std::time::UNIX_EPOCH) {
                let secs = datetime.as_secs();
                let days = secs / 86400;
                let years = days / 365;

                if years > 0 {
                    format!("{years} years ago")
                } else if days > 0 {
                    format!("{days} days ago")
                } else {
                    "Today".to_string()
                }
            } else {
                "Unknown".to_string()
            }
        }
        None => "Invalid timestamp".to_string(),
    }
}
