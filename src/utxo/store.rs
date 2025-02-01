use crate::utxo::models::{Input, Transaction};
use primitive_types::H256;
use sled::Db;
use std::path::Path;
use thiserror::Error;

/// Unified error type for UTXO storage operations
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Output already exists in UTXO set")]
    OutputExists,
    #[error("Output not found in UTXO set: {0:?}")]
    OutputNotFound(H256),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Persistent UTXO storage implementation using Sled key-value store
/// 
/// # Features
/// - Atomic batch operations
/// - Crash-resistant storage
/// - Efficient key lookups using transaction hashes and output indices
#[derive(Debug)]
pub struct SdbStore {
    db: Db,
}

impl SdbStore {
    /// Initialize persistent storage with sled database
    /// 
    /// # Arguments
    /// * `path` - Filesystem path for database storage
    /// 
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use rust_protocol_btc::utxo::store::SdbStore;
    /// 
    /// let store = SdbStore::new(Path::new("./utxo-db")).unwrap();
    /// ```
    pub fn new(path: &Path) -> Result<Self, StoreError> {
        let db = sled::open(path)
            .map_err(|e| StoreError::StorageError(e.to_string()))?;
            
        Ok(Self { db })
    }

    /// Add transaction outputs to UTXO set with batch insertion
    /// 
    /// # Arguments
    /// * `tx` - Transaction containing outputs to add
    pub fn add_outputs(&mut self, tx: &Transaction) -> Result<(), StoreError> {
        let mut batch = sled::Batch::default();

        for (index, output) in tx.outputs.iter().enumerate() {
            let key = (tx.hash, index as u32);
            let key_bytes = self.serialize(&key)?;
            let value_bytes = self.serialize(output)?;

            batch.insert(key_bytes, value_bytes);
        }

        self.db.apply_batch(batch)
            .map_err(|e| StoreError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Remove spent inputs from UTXO set with atomic operations
    /// 
    /// # Arguments
    /// * `inputs` - List of inputs to remove
    pub fn remove_inputs(&mut self, inputs: &[Input]) -> Result<(), StoreError> {
        let mut batch = sled::Batch::default();

        for input in inputs {
            let key = (input.previous_output, input.index);
            let key_bytes = self.serialize(&key)?;
            batch.remove(key_bytes);
        }

        self.db.apply_batch(batch)
            .map_err(|e| StoreError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Check if input exists in UTXO set
    /// 
    /// # Arguments
    /// * `input` - Input to verify existence
    pub fn contains_input(&self, input: &Input) -> Result<bool, StoreError> {
        let key = (input.previous_output, input.index);
        let key_bytes = self.serialize(&key)?;
        
        self.db.contains_key(key_bytes)
            .map_err(|e| StoreError::StorageError(e.to_string()))
    }

    /// Get current UTXO count with proper error handling
    pub fn len(&self) -> Result<usize, StoreError> {
        Ok(self.db.len())
    }

    /// Check if UTXO set is empty
    pub fn is_empty(&self) -> Result<bool, StoreError> {
        self.db.is_empty()
            .map_err(|e| StoreError::StorageError(e.to_string()))
    }

    /// Generic serialization helper with unified error handling
    fn serialize<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, StoreError> {
        bincode::serialize(value)
            .map_err(|e| StoreError::SerializationError(e.to_string()))
    }
}
