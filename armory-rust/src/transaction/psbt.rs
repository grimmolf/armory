/// PSBT v2 implementation (BIP-370)
/// 
/// This module implements BIP-370 PSBT version 2 functionality
/// with support for independent input/output addition and improved
/// transaction construction workflows.

use bitcoin::{
    psbt::{Input, Output, Psbt as PsbtV0},
    Transaction, TxIn, TxOut, Witness, OutPoint, Amount,
    ScriptBuf, PublicKey, XOnlyPublicKey,
    taproot::{Signature as TaprootSignature, TapLeafHash, ControlBlock, LeafVersion, TapTree},
    bip32::{Fingerprint, DerivationPath, ExtendedPubKey},
    ecdsa::Signature as EcdsaSignature,
    Txid, Sequence, absolute::LockTime
};
use psbt::PsbtSighashType;
use std::collections::HashMap;
use crate::error::{TransactionResult, TransactionError};

/// PSBT v2 structure implementing BIP-370
#[derive(Debug, Clone)]
pub struct PsbtV2 {
    /// PSBT version (2 for BIP-370)
    pub version: u8,
    /// Transaction inputs with PSBT input data
    pub inputs: Vec<PsbtV2Input>,
    /// Transaction outputs with PSBT output data
    pub outputs: Vec<PsbtV2Output>,
    /// Global PSBT fields
    pub global_fields: HashMap<Vec<u8>, Vec<u8>>,
    /// Fallback locktime for the transaction
    pub fallback_locktime: Option<u32>,
    /// Input count (required in v2)
    pub input_count: u32,
    /// Output count (required in v2)
    pub output_count: u32,
}

/// PSBT v2 Input structure
#[derive(Debug, Clone)]
pub struct PsbtV2Input {
    /// Previous transaction output being spent
    pub previous_txout: Option<TxOut>,
    /// Witness UTXO (required for SegWit inputs)
    pub witness_utxo: Option<TxOut>,
    /// Non-witness UTXO (full previous transaction)
    pub non_witness_utxo: Option<Transaction>,
    /// Partial signatures
    pub partial_sigs: HashMap<PublicKey, EcdsaSignature>,
    /// SIGHASH type
    pub sighash_type: Option<PsbtSighashType>,
    /// Redeem script for P2SH inputs
    pub redeem_script: Option<ScriptBuf>,
    /// Witness script for P2WSH inputs
    pub witness_script: Option<ScriptBuf>,
    /// BIP 32 derivation paths
    pub bip32_derivation: HashMap<PublicKey, (Fingerprint, DerivationPath)>,
    /// Final script signature
    pub final_script_sig: Option<ScriptBuf>,
    /// Final witness
    pub final_script_witness: Option<Witness>,
    /// Taproot key signature
    pub tap_key_sig: Option<TaprootSignature>,
    /// Taproot script signatures
    pub tap_script_sigs: HashMap<(XOnlyPublicKey, TapLeafHash), TaprootSignature>,
    /// Taproot leaf scripts
    pub tap_scripts: HashMap<ControlBlock, (ScriptBuf, LeafVersion)>,
    /// Taproot BIP32 derivations
    pub tap_key_origins: HashMap<XOnlyPublicKey, (Vec<TapLeafHash>, (Fingerprint, DerivationPath))>,
    /// Required time locktime
    pub required_time_locktime: Option<u32>,
    /// Required height locktime
    pub required_height_locktime: Option<u32>,
    /// Previous transaction ID (required in v2)
    pub previous_txid: Txid,
    /// Previous output index (required in v2)
    pub previous_output_index: u32,
    /// Sequence number
    pub sequence: Option<u32>,
}

/// PSBT v2 Output structure
#[derive(Debug, Clone)]
pub struct PsbtV2Output {
    /// Output amount (required in v2)
    pub amount: Amount,
    /// Output script (required in v2)
    pub script: ScriptBuf,
    /// Redeem script for P2SH outputs
    pub redeem_script: Option<ScriptBuf>,
    /// Witness script for P2WSH outputs
    pub witness_script: Option<ScriptBuf>,
    /// BIP 32 derivation paths
    pub bip32_derivation: HashMap<PublicKey, (Fingerprint, DerivationPath)>,
    /// Taproot internal key
    pub tap_internal_key: Option<XOnlyPublicKey>,
    /// Taproot tree
    pub tap_tree: Option<TapTree>,
    /// Taproot BIP32 derivations
    pub tap_key_origins: HashMap<XOnlyPublicKey, (Vec<TapLeafHash>, (Fingerprint, DerivationPath))>,
}

impl PsbtV2 {
    /// Create new empty PSBT v2
    pub fn new() -> TransactionResult<Self> {
        Ok(Self {
            version: 2,
            inputs: Vec::new(),
            outputs: Vec::new(),
            global_fields: HashMap::new(),
            fallback_locktime: None,
            input_count: 0,
            output_count: 0,
        })
    }
    
    /// Create PSBT v2 from transaction template
    pub fn from_tx_template(
        inputs: Vec<(Txid, u32)>,
        outputs: Vec<(ScriptBuf, Amount)>,
        locktime: Option<u32>,
    ) -> TransactionResult<Self> {
        let mut psbt = Self::new()?;
        
        // Add inputs
        for (txid, output_index) in inputs {
            psbt.add_input(txid, output_index, None)?;
        }
        
        // Add outputs
        for (script, amount) in outputs {
            psbt.add_output(amount, script)?;
        }
        
        psbt.fallback_locktime = locktime;
        Ok(psbt)
    }
    
    /// Add input to PSBT v2
    pub fn add_input(
        &mut self,
        previous_txid: Txid,
        previous_output_index: u32,
        sequence: Option<u32>,
    ) -> TransactionResult<()> {
        let input = PsbtV2Input {
            previous_txout: None,
            witness_utxo: None,
            non_witness_utxo: None,
            partial_sigs: HashMap::new(),
            sighash_type: None,
            redeem_script: None,
            witness_script: None,
            bip32_derivation: HashMap::new(),
            final_script_sig: None,
            final_script_witness: None,
            tap_key_sig: None,
            tap_script_sigs: HashMap::new(),
            tap_scripts: HashMap::new(),
            tap_key_origins: HashMap::new(),
            required_time_locktime: None,
            required_height_locktime: None,
            previous_txid,
            previous_output_index,
            sequence,
        };
        
        self.inputs.push(input);
        self.input_count = self.inputs.len() as u32;
        Ok(())
    }
    
    /// Add output to PSBT v2
    pub fn add_output(&mut self, amount: Amount, script: ScriptBuf) -> TransactionResult<()> {
        let output = PsbtV2Output {
            amount,
            script,
            redeem_script: None,
            witness_script: None,
            bip32_derivation: HashMap::new(),
            tap_internal_key: None,
            tap_tree: None,
            tap_key_origins: HashMap::new(),
        };
        
        self.outputs.push(output);
        self.output_count = self.outputs.len() as u32;
        Ok(())
    }
    
    /// Set witness UTXO for input
    pub fn set_witness_utxo(&mut self, input_index: usize, utxo: TxOut) -> TransactionResult<()> {
        if input_index >= self.inputs.len() {
            return Err(TransactionError::InvalidInput("Input index out of range".to_string()));
        }
        
        self.inputs[input_index].witness_utxo = Some(utxo);
        Ok(())
    }
    
    /// Set non-witness UTXO for input
    pub fn set_non_witness_utxo(&mut self, input_index: usize, tx: Transaction) -> TransactionResult<()> {
        if input_index >= self.inputs.len() {
            return Err(TransactionError::InvalidInput("Input index out of range".to_string()));
        }
        
        self.inputs[input_index].non_witness_utxo = Some(tx);
        Ok(())
    }
    
    /// Get total input value
    pub fn total_input_value(&self) -> TransactionResult<Amount> {
        let mut total = Amount::ZERO;
        
        for input in &self.inputs {
            if let Some(utxo) = &input.witness_utxo {
                total = total.checked_add(utxo.value)
                    .ok_or_else(|| TransactionError::InvalidAmount("Input value overflow".to_string()))?;
            } else if let Some(tx) = &input.non_witness_utxo {
                let utxo = tx.output.get(input.previous_output_index as usize)
                    .ok_or_else(|| TransactionError::InvalidInput("Invalid previous output index".to_string()))?;
                total = total.checked_add(utxo.value)
                    .ok_or_else(|| TransactionError::InvalidAmount("Input value overflow".to_string()))?;
            } else {
                return Err(TransactionError::InvalidInput("Missing UTXO data for input".to_string()));
            }
        }
        
        Ok(total)
    }
    
    /// Get total output value
    pub fn total_output_value(&self) -> TransactionResult<Amount> {
        let mut total = Amount::ZERO;
        
        for output in &self.outputs {
            total = total.checked_add(output.amount)
                .ok_or_else(|| TransactionError::InvalidAmount("Output value overflow".to_string()))?;
        }
        
        Ok(total)
    }
    
    /// Calculate transaction fee
    pub fn fee(&self) -> TransactionResult<Amount> {
        let input_value = self.total_input_value()?;
        let output_value = self.total_output_value()?;
        
        input_value.checked_sub(output_value)
            .ok_or_else(|| TransactionError::InvalidAmount("Insufficient input value".to_string()))
    }
    
    /// Check if PSBT is ready for finalization
    pub fn is_ready_for_finalization(&self) -> bool {
        // All inputs must have either final scripts or sufficient signatures
        for input in &self.inputs {
            let has_final_scripts = input.final_script_sig.is_some() || input.final_script_witness.is_some();
            let has_signature = !input.partial_sigs.is_empty() || input.tap_key_sig.is_some();
            
            if !has_final_scripts && !has_signature {
                return false;
            }
        }
        
        true
    }
    
    /// Finalize PSBT to create final transaction
    pub fn finalize(&self) -> TransactionResult<Transaction> {
        if !self.is_ready_for_finalization() {
            return Err(TransactionError::InvalidPsbt("PSBT not ready for finalization".to_string()));
        }
        
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        
        // Build transaction inputs
        for input in &self.inputs {
            let txin = TxIn {
                previous_output: OutPoint {
                    txid: input.previous_txid,
                    vout: input.previous_output_index,
                },
                script_sig: input.final_script_sig.clone().unwrap_or_default(),
                sequence: Sequence(input.sequence.unwrap_or(0xffffffff)),
                witness: input.final_script_witness.clone().unwrap_or_default(),
            };
            inputs.push(txin);
        }
        
        // Build transaction outputs
        for output in &self.outputs {
            let txout = TxOut {
                value: output.amount,
                script_pubkey: output.script.clone(),
            };
            outputs.push(txout);
        }
        
        Ok(Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: LockTime::from_height(self.fallback_locktime.unwrap_or(0))
                .map_err(|_| TransactionError::InvalidLocktime("Invalid locktime value".to_string()))?,
            input: inputs,
            output: outputs,
        })
    }
}

impl Default for PsbtV2 {
    fn default() -> Self {
        Self::new().expect("Default PSBT v2 creation should not fail")
    }
}