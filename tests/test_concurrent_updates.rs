use state_channel_node::channel::state::StateUpdate;
use state_channel_node::channel::transitions::validate_state_transition;
use state_channel_node::crypto;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use serial_test::serial;

mod test_helpers;
use test_helpers::{create_test_channel, sign_update, sort_participants};

#[tokio::test]
#[serial]
async fn test_concurrent_state_updates() {
    println!("\n=== Testing Concurrent State Updates ===");
    
    // Create initial channel state
    let participant1 = crypto::generate_keypair();
    let participant2 = crypto::generate_keypair();
    let mut participants = vec![participant1.public_key().clone(), participant2.public_key().clone()];
    sort_participants(&mut participants);
    let initial_balance = 2_000_000;
    let channel = create_test_channel(&participants, initial_balance);
    
    println!("Created channel with {} participants", participants.len());
    println!("Initial channel balance: {}", initial_balance);

    // Generate test updates without signatures
    println!("\nGenerating test updates...");
    let mut rng = rand::thread_rng();
    let num_updates = 5;
    let mut updates = Vec::new();
    
    println!("\nInitial channel state:");
    println!("  Sequence number: {}", channel.sequence_number);
    println!("  Channel ID: {:?}", channel.channel_id);
    println!("  Participants: {:?}", channel.participants);
    
    // Pre-generate all updates with signatures to ensure consistency
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    for i in 0..num_updates {
        let amount = rng.gen_range(50_000..200_000);
        println!("\nPreparing update {}: Transfer {} from P1 to P2", i + 1, amount);
        
        // First create and sort affected participants for deterministic order
        let mut affected_participants = vec![participants[0].clone(), participants[1].clone()];
        sort_participants(&mut affected_participants);
        
        // Create balance changes in the same order as affected_participants
        let mut balance_changes = HashMap::new();
        balance_changes.insert(affected_participants[1].clone(), amount);  // P2 receives
        balance_changes.insert(affected_participants[0].clone(), -amount); // P1 sends
        
        // Generate signatures
        println!("Generating signatures for update {}:", i + 1);
        println!("  Sequence: {}", i + 1);
        println!("  Channel ID: {:?}", channel.channel_id);
        println!("  Balance changes: {:?}", balance_changes);
        println!("  Affected participants: {:?}", affected_participants);
        println!("  Timestamp: {}", timestamp);
        
        let signature1 = sign_update(
            if affected_participants[0] == participant1.public_key() { &participant1 } else { &participant2 },
            &affected_participants,
            (i + 1) as u64,
            &balance_changes,
            timestamp,
            channel.channel_id,
        );
        println!("  Signature from participant 0: {:?}", signature1);
        
        let signature2 = sign_update(
            if affected_participants[1] == participant2.public_key() { &participant2 } else { &participant1 },
            &affected_participants,
            (i + 1) as u64,
            &balance_changes,
            timestamp,
            channel.channel_id,
        );
        println!("  Signature from participant 1: {:?}", signature2);
        
        let update = StateUpdate {
            sequence_number: (i + 1) as u64,
            balance_changes,
            signatures: vec![signature1, signature2],
            affected_participants,
            timestamp,
        };
        
        updates.push(update);
    }

    println!("\nApplying updates in parallel...");
    let channel = Arc::new(RwLock::new(channel));
    let updates = Arc::new(updates);
    let mut handles = vec![];
    
    println!("\nStarting parallel update application process...");
    
    // Launch tasks to apply updates
    for i in 0..num_updates {
        let channel = Arc::clone(&channel);
        let updates = Arc::clone(&updates);
        
        let handle = tokio::spawn(async move {
            let mut success = false;
            let mut retries = 0;
            let max_retries = 10;
            
            while !success && retries < max_retries {
                if retries > 0 {
                    println!("\nRetry {}/{} for update {}", retries + 1, max_retries, i + 1);
                    sleep(Duration::from_millis(10 * retries as u64)).await;
                }
                
                let update = updates[i].clone();
                println!("\nTask attempting update {} (sequence {})", i + 1, update.sequence_number);
                
                // Get current channel state and check sequence atomically
                let mut channel_guard = channel.write().await;
                let current_sequence = channel_guard.sequence_number;
                
                // Only proceed if this is the next update in sequence
                if update.sequence_number != current_sequence + 1 {
                    println!("Waiting for sequence {} (current: {})", 
                        update.sequence_number, current_sequence);
                    retries += 1;
                    continue;
                }

                println!("\nValidating update {} against channel state:", i + 1);
                println!("  Channel sequence: {}", channel_guard.sequence_number);
                println!("  Channel ID: {:?}", channel_guard.channel_id);
                println!("  Update sequence: {}", update.sequence_number);
                println!("  Update timestamp: {}", update.timestamp);
                println!("  Balance changes: {:?}", update.balance_changes);
                
                match validate_state_transition(&mut *channel_guard, &update) {
                    Ok(_) => {
                        channel_guard.apply_update(&update);
                        success = true;
                        println!("✓ Successfully applied update {} (sequence {})", i + 1, i + 1);
                        
                        // Log new channel state
                        println!("New channel state:");
                        println!("  Sequence: {}", channel_guard.sequence_number);
                        for (participant, balance) in &channel_guard.balances {
                            println!("  Balance for {:?}: {}", participant, balance);
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to apply update {}: {} (after {}µs)", i + 1, e, retries * 10);
                        println!("Channel state at failure:");
                        println!("  Sequence number: {}", channel_guard.sequence_number);
                        println!("  Channel ID: {:?}", channel_guard.channel_id);
                        retries += 1;
                    }
                }
            }
            
            success
        });
        
        handles.push(handle);
    }

    // Wait for all tasks and check results
    let results = futures::future::join_all(handles).await;
    let results: Vec<bool> = results.into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Verify all updates were successful
    assert!(results.iter().all(|&x| x), "Not all valid updates were applied successfully");
    
    // Verify final channel state
    let final_channel = channel.read().await;
    assert_eq!(final_channel.sequence_number, num_updates as u64, 
        "Final sequence number should match number of updates");
}
