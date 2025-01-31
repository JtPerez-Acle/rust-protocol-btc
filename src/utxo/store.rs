use crate::utxo::models::{Output, Transaction, H256};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtxoError {
    #[error("Output already exists in UTXO set")]
    OutputExists,
    #[error("Output not found in UTXO set")]
    OutputNotFound,
}

/// In-memory UTXO set storage
#[derive(Debug, Default)]
pub struct UtxoSet {
    outputs: HashMap<(H256, u32), Output>, // (tx_hash, output_index) -> Output
}

impl UtxoSet {
    /// Add transaction outputs to UTXO set
    pub fn add_outputs(&mut self, tx: &Transaction) {
        for (index, output) in tx.outputs.iter().enumerate() {
            let key = (tx.hash, index as u32);
            self.outputs.insert(key, output.clone());
        }
    }

    /// Remove spent inputs from UTXO set
    pub fn remove_inputs(&mut self, inputs: &[Input]) -> Result<(), UtxoError> {
        for input in inputs {
            let key = (input.previous_output, 0); // Simplified index handling
            if self.outputs.remove(&key).is_none() {
                return Err(UtxoError::OutputNotFound);
            }
        }
        Ok(())
    }

    /// Check if input exists in UTXO set
    pub fn contains_input(&self, input: &Input) -> bool {
        self.outputs.contains_key(&(input.previous_output, 0))
    }

    /// Get current UTXO count
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    /// Check if UTXO set is empty
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}
