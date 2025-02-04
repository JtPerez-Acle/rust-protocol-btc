use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use primitive_types::H256;
use thiserror::Error;

use super::models::{Transaction, Utxo};
use super::store::{SdbStore, StoreError};

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("UTXO not found: {0}")]
    UtxoNotFound(String),
    #[error("Storage error: {0}")]
    StorageError(#[from] StoreError),
    #[error("Lock acquisition failed")]
    LockError,
}

/// Thread-safe cache for UTXOs with persistence layer
pub struct UtxoCache {
    /// In-memory cache of UTXOs
    cache: Arc<RwLock<HashMap<(H256, u32), Utxo>>>,
    /// Persistent storage
    store: Arc<RwLock<SdbStore>>,
}

impl UtxoCache {
    /// Create a new UTXO cache with persistent storage
    pub fn new(store: SdbStore) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            store: Arc::new(RwLock::new(store)),
        }
    }

    /// Add UTXOs from a transaction to the cache
    pub fn add_transaction(&self, tx: &Transaction, block_height: Option<u32>) -> Result<(), CacheError> {
        let mut cache = self.cache.write().map_err(|_| CacheError::LockError)?;
        let mut store = self.store.write().map_err(|_| CacheError::LockError)?;
        
        // Create UTXOs for each output
        for (idx, output) in tx.outputs.iter().enumerate() {
            let utxo = if let Some(height) = block_height {
                Utxo::new(output.clone(), height, idx as u32, tx.hash)
            } else {
                Utxo::new_unconfirmed(output.clone(), idx as u32, tx.hash)
            };
            
            // Add to cache
            cache.insert((tx.hash, idx as u32), utxo.clone());
        }
        
        // Persist to storage
        store.add_outputs(tx)?;
        
        Ok(())
    }

    /// Remove spent UTXOs when inputs reference them
    pub fn remove_spent(&self, tx: &Transaction) -> Result<(), CacheError> {
        let mut cache = self.cache.write().map_err(|_| CacheError::LockError)?;
        let mut store = self.store.write().map_err(|_| CacheError::LockError)?;
        
        // Check each input's referenced UTXO exists and hasn't been spent
        for input in &tx.inputs {
            let key = (input.previous_output, input.index);
            
            // Check if UTXO exists in cache
            if !cache.contains_key(&key) {
                // Check persistent storage
                if !store.contains_input(input)? {
                    return Err(CacheError::UtxoNotFound(format!(
                        "UTXO {}:{} not found or already spent",
                        input.previous_output,
                        input.index
                    )));
                }
            }
        }
        
        // Remove each input's referenced UTXO
        for input in &tx.inputs {
            cache.remove(&(input.previous_output, input.index));
        }
        
        // Update persistent storage
        store.remove_inputs(&tx.inputs)?;
        
        Ok(())
    }

    /// Get a UTXO by its transaction hash and output index
    pub fn get_utxo(&self, tx_hash: H256, output_index: u32) -> Result<Option<Utxo>, CacheError> {
        // Try cache first
        if let Some(utxo) = self.cache
            .read()
            .map_err(|_| CacheError::LockError)?
            .get(&(tx_hash, output_index))
        {
            return Ok(Some(utxo.clone()));
        }
        
        // If not in cache, try persistent storage
        // TODO: Implement get_utxo in SdbStore and load into cache
        
        Ok(None)
    }

    /// Confirm a transaction's UTXOs by updating their block height
    pub fn confirm_transaction(&self, tx_hash: H256, block_height: u32) -> Result<(), CacheError> {
        let mut cache = self.cache.write().map_err(|_| CacheError::LockError)?;
        
        // Update all UTXOs from this transaction
        for (key, utxo) in cache.iter_mut() {
            if key.0 == tx_hash {
                utxo.block_height = block_height;
                utxo.is_confirmed = true;
            }
        }
        
        // TODO: Update persistent storage with confirmation
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::utxo::models::Output;
    
    fn create_test_transaction() -> Transaction {
        // Create a test transaction with one input and two outputs
        Transaction {
            version: 1,
            inputs: vec![],  // No inputs for coinbase-style tx
            outputs: vec![
                Output {
                    value: 50_000_000,  // 0.5 BTC
                    public_key_hash: vec![1; 20],  // Dummy pubkey hash
                    lock_script: vec![],  // Empty lock script for testing
                },
                Output {
                    value: 40_000_000,  // 0.4 BTC
                    public_key_hash: vec![2; 20],
                    lock_script: vec![],
                },
            ],
            lock_time: 0,
            hash: H256::zero(),  // Will be updated by calculate_hash
        }
    }
    
    #[test]
    fn test_add_transaction() {
        let store = SdbStore::new(Path::new("test_db")).unwrap();
        let cache = UtxoCache::new(store);
        
        let mut tx = create_test_transaction();
        tx.hash = tx.calculate_hash();
        
        // Add transaction to cache
        cache.add_transaction(&tx, Some(100)).unwrap();
        
        // Verify UTXOs were added
        for i in 0..tx.outputs.len() {
            let utxo = cache.get_utxo(tx.hash, i as u32).unwrap().unwrap();
            assert_eq!(utxo.block_height, 100);
            assert_eq!(utxo.tx_hash, tx.hash);
            assert_eq!(utxo.output_index, i as u32);
            assert!(utxo.is_confirmed);
        }
    }
}
