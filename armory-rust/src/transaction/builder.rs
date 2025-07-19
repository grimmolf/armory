/// Transaction builder implementation with PSBT v2 support
///
/// This module implements comprehensive transaction building with:
/// - PSBT v2 (BIP-370) transaction construction
/// - Intelligent coin selection algorithms
/// - Dynamic fee estimation
/// - RBF (Replace-by-Fee) support by default
/// - Multi-input transaction construction
/// - Taproot and legacy script support
use crate::error::{TransactionError, TransactionResult};
use crate::transaction::psbt::PsbtV2;
use crate::wallet::descriptor_wallet::{AddressType, Utxo, Wallet};
use bitcoin::{absolute::LockTime, Address, Amount, FeeRate, TxOut};
use std::sync::{Arc, RwLock};

/// Fee estimation strategy
#[derive(Debug, Clone, Copy)]
pub enum FeeStrategy {
    /// Use specified fee rate (sat/vB)
    FeeRate(FeeRate),
    /// Target confirmation within N blocks
    ConfirmationTarget(u32),
    /// Low priority (slower confirmation, lower fee)
    LowPriority,
    /// Normal priority (moderate confirmation, moderate fee)
    Normal,
    /// High priority (fast confirmation, higher fee)
    HighPriority,
}

/// Coin selection strategy
#[derive(Debug, Clone, Copy)]
pub enum CoinSelectionStrategy {
    /// Largest first (minimizes change)
    LargestFirst,
    /// Smallest first (minimizes fees)
    SmallestFirst,
    /// Branch and bound (optimal selection)
    BranchAndBound,
    /// Random selection (enhances privacy)
    Random,
}

/// Transaction builder configuration
#[derive(Debug, Clone)]
pub struct BuilderConfig {
    /// Fee estimation strategy
    pub fee_strategy: FeeStrategy,
    /// Coin selection strategy
    pub coin_selection: CoinSelectionStrategy,
    /// Enable RBF by default
    pub enable_rbf: bool,
    /// Minimum number of confirmations for UTXOs
    pub min_confirmations: u32,
    /// Target number of confirmations
    pub target_confirmations: u32,
    /// Maximum fee rate (sat/vB) to prevent fee overpayment
    pub max_fee_rate: FeeRate,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            fee_strategy: FeeStrategy::Normal,
            coin_selection: CoinSelectionStrategy::BranchAndBound,
            enable_rbf: true,
            min_confirmations: 1,
            target_confirmations: 6,
            max_fee_rate: FeeRate::from_sat_per_vb(1000).expect("Valid fee rate"), // 1000 sat/vB max
        }
    }
}

/// Transaction builder with PSBT v2 support
pub struct TransactionBuilder {
    /// Reference to the wallet
    wallet: Arc<RwLock<Wallet>>,
    /// PSBT v2 under construction
    psbt: PsbtV2,
    /// Transaction outputs (recipient, amount)
    outputs: Vec<(Address, Amount)>,
    /// Selected UTXOs for inputs
    selected_utxos: Vec<Utxo>,
    /// Builder configuration
    config: BuilderConfig,
    /// Current fee estimate
    estimated_fee: Option<Amount>,
    /// Change output if needed
    change_output: Option<(Address, Amount)>,
    /// Explicit locktime
    locktime: Option<LockTime>,
}

impl TransactionBuilder {
    /// Create new transaction builder
    pub fn new(wallet: Arc<RwLock<Wallet>>) -> TransactionResult<Self> {
        Ok(Self {
            wallet,
            psbt: PsbtV2::new()?,
            outputs: Vec::new(),
            selected_utxos: Vec::new(),
            config: BuilderConfig::default(),
            estimated_fee: None,
            change_output: None,
            locktime: None,
        })
    }

    /// Create transaction builder with custom configuration
    pub fn with_config(
        wallet: Arc<RwLock<Wallet>>,
        config: BuilderConfig,
    ) -> TransactionResult<Self> {
        Ok(Self {
            wallet,
            psbt: PsbtV2::new()?,
            outputs: Vec::new(),
            selected_utxos: Vec::new(),
            config,
            estimated_fee: None,
            change_output: None,
            locktime: None,
        })
    }

    /// Add recipient output
    pub fn add_recipient(
        &mut self,
        address: Address,
        amount: Amount,
    ) -> TransactionResult<&mut Self> {
        if amount == Amount::ZERO {
            return Err(TransactionError::InvalidAmount(
                "Cannot send zero amount".to_string(),
            ));
        }

        self.outputs.push((address, amount));
        Ok(self)
    }

    /// Set fee strategy
    pub fn fee_strategy(&mut self, strategy: FeeStrategy) -> &mut Self {
        self.config.fee_strategy = strategy;
        self
    }

    /// Set coin selection strategy
    pub fn coin_selection(&mut self, strategy: CoinSelectionStrategy) -> &mut Self {
        self.config.coin_selection = strategy;
        self
    }

    /// Enable or disable RBF
    pub fn rbf(&mut self, enable: bool) -> &mut Self {
        self.config.enable_rbf = enable;
        self
    }

    /// Set explicit locktime
    pub fn locktime(&mut self, locktime: LockTime) -> &mut Self {
        self.locktime = Some(locktime);
        self
    }

    /// Estimate transaction fee based on current configuration
    pub fn estimate_fee(&mut self) -> TransactionResult<Amount> {
        // First, determine fee rate
        let fee_rate = self.get_fee_rate()?;

        // Calculate transaction size estimate
        let tx_size = self.estimate_transaction_size()?;

        // Calculate fee
        let fee = fee_rate.fee_vb(tx_size as u64).ok_or_else(|| {
            TransactionError::FeeEstimation("Fee calculation overflow".to_string())
        })?;

        self.estimated_fee = Some(fee);
        Ok(fee)
    }

    /// Select UTXOs for the transaction
    pub fn select_utxos(&mut self) -> TransactionResult<&mut Self> {
        // Calculate total output amount
        let total_output = self
            .outputs
            .iter()
            .try_fold(Amount::ZERO, |acc, (_, amount)| acc.checked_add(*amount))
            .ok_or_else(|| TransactionError::InvalidAmount("Output amount overflow".to_string()))?;

        // Estimate fee if not already done
        if self.estimated_fee.is_none() {
            self.estimate_fee()?;
        }

        let estimated_fee = self.estimated_fee.unwrap();

        // Get available UTXOs from wallet (estimate current height for now)
        let current_height = 800_000; // TODO: Get actual current block height
        let available_utxos = {
            let wallet = self
                .wallet
                .read()
                .map_err(|_| TransactionError::InvalidInput("Failed to read wallet".to_string()))?;
            let utxos = wallet.spendable_utxos(self.config.min_confirmations, current_height);
            utxos.into_iter().cloned().collect::<Vec<Utxo>>()
        };

        // Select UTXOs based on strategy
        let (selected_utxos, change_amount) =
            self.perform_coin_selection(&available_utxos, total_output, estimated_fee)?;

        self.selected_utxos = selected_utxos;

        // Create change output if needed
        if change_amount > Amount::ZERO {
            let change_address = {
                let mut wallet = self.wallet.write().map_err(|_| {
                    TransactionError::InvalidInput("Failed to write wallet".to_string())
                })?;
                wallet
                    .get_change_address(AddressType::NativeSegwit)
                    .map_err(|e| {
                        TransactionError::InvalidInput(format!("Failed to get change address: {e}"))
                    })?
            };
            self.change_output = Some((change_address, change_amount));
        }

        Ok(self)
    }

    /// Build the PSBT v2
    pub fn build_psbt(&mut self) -> TransactionResult<PsbtV2> {
        // Ensure UTXOs are selected
        if self.selected_utxos.is_empty() {
            self.select_utxos()?;
        }

        // Clear existing PSBT
        self.psbt = PsbtV2::new()?;

        // Add inputs from selected UTXOs
        for utxo in &self.selected_utxos {
            self.psbt
                .add_input(utxo.txid, utxo.vout, self.get_sequence_number())?;

            // Set UTXO data
            let input_index = self.psbt.inputs.len() - 1;
            let txout = TxOut {
                value: Amount::from_sat(utxo.value),
                script_pubkey: utxo.script_pubkey.clone(),
            };
            self.psbt.set_witness_utxo(input_index, txout)?;
        }

        // Add recipient outputs
        for (address, amount) in &self.outputs {
            self.psbt.add_output(*amount, address.script_pubkey())?;
        }

        // Add change output if needed
        if let Some((change_address, change_amount)) = &self.change_output {
            self.psbt
                .add_output(*change_amount, change_address.script_pubkey())?;
        }

        // Set locktime
        if let Some(locktime) = self.locktime {
            match locktime {
                LockTime::Blocks(height) => {
                    self.psbt.fallback_locktime = Some(height.to_consensus_u32());
                }
                LockTime::Seconds(time) => {
                    self.psbt.fallback_locktime = Some(time.to_consensus_u32());
                }
            }
        }

        Ok(self.psbt.clone())
    }

    /// Get total input value
    pub fn total_input_value(&self) -> Amount {
        self.selected_utxos
            .iter()
            .fold(Amount::ZERO, |acc, utxo| acc + Amount::from_sat(utxo.value))
    }

    /// Get total output value (including change)
    pub fn total_output_value(&self) -> Amount {
        let recipient_total = self
            .outputs
            .iter()
            .fold(Amount::ZERO, |acc, (_, amount)| acc + *amount);

        let change_amount = self
            .change_output
            .as_ref()
            .map(|(_, amount)| *amount)
            .unwrap_or(Amount::ZERO);

        recipient_total + change_amount
    }

    /// Get current fee estimate
    pub fn fee(&self) -> Option<Amount> {
        self.estimated_fee
    }

    /// Get change amount
    pub fn change_amount(&self) -> Option<Amount> {
        self.change_output.as_ref().map(|(_, amount)| *amount)
    }

    // Private helper methods

    /// Get fee rate based on strategy
    fn get_fee_rate(&self) -> TransactionResult<FeeRate> {
        match self.config.fee_strategy {
            FeeStrategy::FeeRate(rate) => Ok(rate),
            FeeStrategy::ConfirmationTarget(blocks) => {
                // In a real implementation, this would query fee estimation service
                // For now, use simple heuristics
                let rate = match blocks {
                    1..=2 => 50, // High priority
                    3..=6 => 20, // Normal priority
                    _ => 10,     // Low priority
                };
                FeeRate::from_sat_per_vb(rate)
                    .ok_or_else(|| TransactionError::FeeEstimation("Invalid fee rate".to_string()))
            }
            FeeStrategy::LowPriority => FeeRate::from_sat_per_vb(5)
                .ok_or_else(|| TransactionError::FeeEstimation("Invalid fee rate".to_string())),
            FeeStrategy::Normal => FeeRate::from_sat_per_vb(20)
                .ok_or_else(|| TransactionError::FeeEstimation("Invalid fee rate".to_string())),
            FeeStrategy::HighPriority => FeeRate::from_sat_per_vb(50)
                .ok_or_else(|| TransactionError::FeeEstimation("Invalid fee rate".to_string())),
        }
    }

    /// Estimate transaction size in vbytes
    fn estimate_transaction_size(&self) -> TransactionResult<usize> {
        // Base transaction size
        let mut size = 10; // version (4) + input count (1) + output count (1) + locktime (4)

        // Input sizes (estimate based on script type)
        let input_count = self.selected_utxos.len().max(1); // At least 1 for estimation
        for _ in 0..input_count {
            // Conservative estimate: 148 bytes per input (P2WPKH)
            size += 148;
        }

        // Output sizes
        let output_count = self.outputs.len() + if self.change_output.is_some() { 1 } else { 0 };
        for _ in 0..output_count.max(1) {
            // Conservative estimate: 34 bytes per output (P2WPKH)
            size += 34;
        }

        Ok(size)
    }

    /// Perform coin selection based on strategy
    fn perform_coin_selection(
        &self,
        available_utxos: &[Utxo],
        target_amount: Amount,
        estimated_fee: Amount,
    ) -> TransactionResult<(Vec<Utxo>, Amount)> {
        let total_needed = target_amount + estimated_fee;

        let mut sorted_utxos = available_utxos.to_vec();

        // Sort based on strategy
        match self.config.coin_selection {
            CoinSelectionStrategy::LargestFirst => {
                sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));
            }
            CoinSelectionStrategy::SmallestFirst => {
                sorted_utxos.sort_by(|a, b| a.value.cmp(&b.value));
            }
            CoinSelectionStrategy::BranchAndBound => {
                // For now, use largest first as approximation
                // TODO: Implement proper branch and bound algorithm
                sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));
            }
            CoinSelectionStrategy::Random => {
                // For now, use largest first as approximation
                // TODO: Implement proper random selection
                sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));
            }
        }

        // Select UTXOs until we have enough
        let mut selected = Vec::new();
        let mut total_selected = Amount::ZERO;

        for utxo in sorted_utxos {
            selected.push(utxo.clone());
            total_selected = total_selected
                .checked_add(Amount::from_sat(utxo.value))
                .ok_or_else(|| {
                    TransactionError::InvalidAmount("Selected value overflow".to_string())
                })?;

            if total_selected >= total_needed {
                break;
            }
        }

        // Check if we have enough funds
        if total_selected < total_needed {
            return Err(TransactionError::InsufficientFunds {
                available: total_selected.to_sat(),
                required: total_needed.to_sat(),
            });
        }

        // Calculate change
        let change = total_selected - total_needed;

        Ok((selected, change))
    }

    /// Get sequence number (enables RBF if configured)
    fn get_sequence_number(&self) -> Option<u32> {
        if self.config.enable_rbf {
            Some(0xfffffffd) // RBF signal
        } else {
            Some(0xffffffff) // Final
        }
    }
}

/// Fee estimation utilities
pub struct FeeEstimator;

impl FeeEstimator {
    /// Estimate fee for transaction size
    pub fn estimate_fee(tx_size_vbytes: usize, fee_rate: FeeRate) -> TransactionResult<Amount> {
        fee_rate
            .fee_vb(tx_size_vbytes as u64)
            .ok_or_else(|| TransactionError::FeeEstimation("Fee calculation overflow".to_string()))
    }

    /// Get recommended fee rate for confirmation target
    pub fn get_fee_rate_for_target(blocks: u32) -> FeeRate {
        // Simple heuristic - in production this would query mempool/fee estimation service
        let sat_per_vb = match blocks {
            1 => 100,     // Next block
            2..=3 => 50,  // Within 3 blocks
            4..=6 => 20,  // Within 6 blocks
            7..=12 => 10, // Within 12 blocks
            _ => 5,       // Low priority
        };

        FeeRate::from_sat_per_vb(sat_per_vb)
            .unwrap_or_else(|| FeeRate::from_sat_per_vb(1).expect("Minimum fee rate"))
    }

    /// Calculate transaction weight/vsize
    pub fn calculate_tx_weight(inputs: usize, outputs: usize) -> usize {
        // Base weight
        let mut weight = 4 * (4 + 1 + 1 + 4); // version + input_count + output_count + locktime

        // Input weight (assuming P2WPKH)
        weight += inputs * (4 * (32 + 4 + 1 + 4) + 1 + 73 + 1 + 33); // outpoint + script_len + sequence + witness

        // Output weight
        weight += outputs * 4 * (8 + 1 + 22); // value + script_len + script (P2WPKH)

        // Convert to vsize (weight / 4, rounded up)
        weight.div_ceil(4)
    }
}
