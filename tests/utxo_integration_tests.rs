mod common;

use std::path::PathBuf;
use state_channel_node::utxo::cache::UtxoCache;
use state_channel_node::utxo::store::SdbStore;
use common::test_utils;
use common::utxo;

#[test]
fn test_utxo_spending() {
    let test_db_path = PathBuf::from("test_utxo.db");
    let store = SdbStore::new(&test_db_path).expect("Failed to create test store");
    let cache = UtxoCache::new(store);
    
    // Create initial transaction (similar to coinbase)
    let initial_tx = utxo::create_transaction(vec![
        utxo::create_output(5_000_000_000), // 50 BTC
        utxo::create_output(3_000_000_000), // 30 BTC
    ]);
    let initial_hash = initial_tx.hash;
    
    cache.add_transaction(&initial_tx, Some(1))
        .expect("Failed to add initial transaction");
        
    // Create spending transaction
    let spending_tx = utxo::create_spending_transaction(
        &initial_tx,
        &[0], // Spend first output
        vec![utxo::create_output(4_900_000_000)], // 49 BTC (0.1 BTC fee)
    );
    
    // Remove spent outputs and add new ones
    cache.remove_spent(&spending_tx)
        .expect("Failed to remove spent outputs");
    cache.add_transaction(&spending_tx, Some(2))
        .expect("Failed to add spending transaction");
        
    // Verify spending transaction structure
    assert_eq!(spending_tx.inputs.len(), 1);
    assert_eq!(spending_tx.inputs[0].previous_output, initial_hash);
    assert_eq!(spending_tx.inputs[0].index, 0);
    
    // Cleanup
    test_utils::cleanup_test_db(&test_db_path);
}

#[test]
fn test_double_spend_prevention() {
    let test_db_path = PathBuf::from("test_double_spend.db");
    let store = SdbStore::new(&test_db_path).expect("Failed to create test store");
    let cache = UtxoCache::new(store);
    
    // Create initial transaction
    let initial_tx = utxo::create_transaction(vec![
        utxo::create_output(5_000_000_000), // 50 BTC
    ]);
    
    cache.add_transaction(&initial_tx, Some(1))
        .expect("Failed to add initial transaction");
        
    // First spending transaction
    let spending_tx = utxo::create_spending_transaction(
        &initial_tx,
        &[0],
        vec![utxo::create_output(4_900_000_000)],
    );
    
    // Remove spent outputs and add new ones
    cache.remove_spent(&spending_tx)
        .expect("Failed to remove spent outputs");
    cache.add_transaction(&spending_tx, Some(2))
        .expect("Failed to add spending transaction");
        
    // Try to create double spend
    let double_spend_tx = utxo::create_spending_transaction(
        &initial_tx,
        &[0],
        vec![utxo::create_output(4_800_000_000)],
    );
    
    // Attempt to spend already spent output should fail
    let result = cache.remove_spent(&double_spend_tx);
    assert!(result.is_err());
    
    // Cleanup
    test_utils::cleanup_test_db(&test_db_path);
}

#[test]
fn test_complex_transaction_chain() {
    let test_db_path = PathBuf::from("test_chain.db");
    let store = SdbStore::new(&test_db_path).expect("Failed to create test store");
    let cache = UtxoCache::new(store);
    
    // Initial transaction with multiple outputs
    let initial_tx = utxo::create_transaction(vec![
        utxo::create_output(3_000_000_000), // 30 BTC
        utxo::create_output(2_000_000_000), // 20 BTC
    ]);
    
    cache.add_transaction(&initial_tx, Some(1))
        .expect("Failed to add initial transaction");
        
    // First spending transaction (spends first output)
    let spending_tx1 = utxo::create_spending_transaction(
        &initial_tx,
        &[0],
        vec![
            utxo::create_output(2_000_000_000), // 20 BTC
            utxo::create_output(900_000_000),   // 9 BTC (0.1 BTC fee)
        ],
    );
    
    cache.remove_spent(&spending_tx1)
        .expect("Failed to remove spent outputs");
    cache.add_transaction(&spending_tx1, Some(2))
        .expect("Failed to add first spending transaction");
        
    // Second spending transaction (spends both outputs from first spending transaction)
    let final_tx = utxo::create_spending_transaction(
        &spending_tx1,
        &[0, 1],
        vec![utxo::create_output(2_800_000_000)], // 28 BTC (0.1 BTC fee)
    );
    
    cache.remove_spent(&final_tx)
        .expect("Failed to remove spent outputs");
    cache.add_transaction(&final_tx, Some(3))
        .expect("Failed to add final transaction");
        
    assert_eq!(final_tx.inputs.len(), 2);
    
    // Cleanup
    test_utils::cleanup_test_db(&test_db_path);
}
