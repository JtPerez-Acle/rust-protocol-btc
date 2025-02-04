use primitive_types::H256;
use state_channel_node::utxo::models::{Transaction, Input, Output};
use std::path::PathBuf;

/// Test utilities and common setup
#[allow(dead_code)]
pub mod test_utils {
    use super::*;
    use std::fs;

    /// Clean up test database directory
    pub fn cleanup_test_db(test_db_path: &PathBuf) {
        if test_db_path.exists() {
            fs::remove_dir_all(test_db_path).expect("Failed to cleanup test database");
        }
    }
    
    /// Create a test transaction with specified outputs
    pub fn create_transaction(outputs: Vec<Output>) -> Transaction {
        let mut tx = Transaction {
            version: 1,
            inputs: Vec::new(),
            outputs,
            lock_time: 0,
            hash: H256::zero(),
        };
        tx.hash = tx.calculate_hash();
        tx
    }

    /// Create a test output with specified value
    pub fn create_output(value: u64) -> Output {
        Output {
            value,
            public_key_hash: vec![1; 20], // Dummy public key hash
            lock_script: vec![], // Empty lock script for testing
        }
    }
}

/// UTXO test utilities
#[allow(dead_code)]
pub mod utxo {
    use super::*;
    
    /// Create a test transaction with specified outputs
    pub fn create_transaction(outputs: Vec<Output>) -> Transaction {
        let mut tx = Transaction {
            version: 1,
            inputs: vec![],
            outputs,
            lock_time: 0,
            hash: H256::zero(),
        };
        tx.hash = tx.calculate_hash();
        tx
    }
    
    /// Create a test transaction with inputs spending from another transaction
    pub fn create_spending_transaction(
        prev_tx: &Transaction,
        output_indices: &[u32],
        new_outputs: Vec<Output>
    ) -> Transaction {
        let inputs: Vec<Input> = output_indices
            .iter()
            .map(|&index| Input {
                previous_output: prev_tx.hash,
                index,
                signature: vec![1, 2, 3, 4], // Dummy signature
                sequence: 0xffffffff,
            })
            .collect();
            
        let mut tx = Transaction {
            version: 1,
            inputs,
            outputs: new_outputs,
            lock_time: 0,
            hash: H256::zero(),
        };
        tx.hash = tx.calculate_hash();
        tx
    }
    
    /// Create a test output with specified value
    pub fn create_output(value: u64) -> Output {
        Output {
            value,
            public_key_hash: vec![1; 20], // Dummy public key hash
            lock_script: vec![], // Empty lock script for testing
        }
    }
}
