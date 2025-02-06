use state_channel_node::crypto;
use state_channel_node::channel::state::StateUpdate;
use state_channel_node::channel::transitions::validate_state_transition;
use std::collections::HashMap;
use rand::prelude::*;

mod test_helpers;
use test_helpers::{create_test_channel, sign_update, sort_participants};

#[test]
fn test_multi_participant_circular_payment() {
    println!("\n=== Testing Multi-Participant Circular Payment ===");
    // Create keypairs for three participants
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    let kp3 = crypto::generate_keypair();
    
    let participants = vec![kp1.public_key(), kp2.public_key(), kp3.public_key()];
    println!("Created channel with {} participants", participants.len());
    let mut channel = create_test_channel(&participants, 1_000_000);
    println!("Initial channel balance: 1,000,000");
    
    // First transfer: P1 -> P2
    println!("\nStep 1: P1 -> P2 (100,000)");
    let mut affected_participants1 = vec![participants[0].clone(), participants[1].clone()];
    sort_participants(&mut affected_participants1);
    
    let mut changes1 = HashMap::new();
    changes1.insert(participants[0].clone(), -100_000);
    changes1.insert(participants[1].clone(), 100_000);
    
    let timestamp1 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut signatures1 = Vec::new();
    for participant in &affected_participants1 {
        let kp = if participant == &participants[0] { &kp1 } else { &kp2 };
        signatures1.push(sign_update(kp, &affected_participants1, 1, &changes1, timestamp1, channel.channel_id));
    }
    
    let update1 = StateUpdate {
        sequence_number: 1,
        balance_changes: changes1,
        signatures: signatures1,
        affected_participants: affected_participants1,
        timestamp: timestamp1,
    };
    
    // Second transfer: P2 -> P3
    println!("\nStep 2: P2 -> P3 (100,000)");
    let mut affected_participants2 = vec![participants[1].clone(), participants[2].clone()];
    sort_participants(&mut affected_participants2);
    
    let mut changes2 = HashMap::new();
    changes2.insert(participants[1].clone(), -100_000);
    changes2.insert(participants[2].clone(), 100_000);
    
    let timestamp2 = timestamp1 + 1;
    let mut signatures2 = Vec::new();
    for participant in &affected_participants2 {
        let kp = if participant == &participants[1] { &kp2 } else { &kp3 };
        signatures2.push(sign_update(kp, &affected_participants2, 2, &changes2, timestamp2, channel.channel_id));
    }
    
    let update2 = StateUpdate {
        sequence_number: 2,
        balance_changes: changes2,
        signatures: signatures2,
        affected_participants: affected_participants2,
        timestamp: timestamp2,
    };
    
    // Third transfer: P3 -> P1
    println!("\nStep 3: P3 -> P1 (100,000)");
    let mut affected_participants3 = vec![participants[2].clone(), participants[0].clone()];
    sort_participants(&mut affected_participants3);
    
    let mut changes3 = HashMap::new();
    changes3.insert(participants[2].clone(), -100_000);
    changes3.insert(participants[0].clone(), 100_000);
    
    let timestamp3 = timestamp2 + 1;
    let mut signatures3 = Vec::new();
    for participant in &affected_participants3 {
        let kp = if participant == &participants[2] { &kp3 } else { &kp1 };
        signatures3.push(sign_update(kp, &affected_participants3, 3, &changes3, timestamp3, channel.channel_id));
    }
    
    let update3 = StateUpdate {
        sequence_number: 3,
        balance_changes: changes3,
        signatures: signatures3,
        affected_participants: affected_participants3,
        timestamp: timestamp3,
    };
    
    // Apply all updates
    for (i, update) in [update1, update2, update3].iter().enumerate() {
        println!("\nApplying update {}", i + 1);
        match validate_state_transition(&channel, update) {
            Ok(_) => {
                channel.apply_update(&update.clone());
                println!("Successfully applied update {}", i + 1);
                println!("New balances: {:?}", channel.balances);
            },
            Err(e) => {
                println!("Failed to apply update {}: {:?}", i + 1, e);
            }
        }
    }
    
    // Verify final state
    println!("\nFinal channel state:");
    for (i, participant) in participants.iter().enumerate() {
        println!("P{} balance: {}", i + 1, channel.balances[participant]);
    }
    
    // All balances should be back to initial state
    for participant in &participants {
        assert_eq!(
            channel.balances[participant],
            1_000_000,
            "Final balance incorrect"
        );
    }
    
    assert_eq!(channel.sequence_number, 3, "Final sequence number should be 3");
}

#[test]
fn test_realistic_multi_participant_scenario() {
    println!("\n=== Testing Realistic Multi-Participant Scenario ===");
    // Create 5 participants
    let keypairs: Vec<_> = (0..5).map(|_| crypto::generate_keypair()).collect();
    let participants: Vec<_> = keypairs.iter().map(|kp| kp.public_key()).collect();
    
    println!("Created channel with {} participants", participants.len());
    let mut channel = create_test_channel(&participants, 1_000_000);
    println!("Initial channel balance: 1,000,000");
    
    // Create a series of random updates
    let mut rng = rand::thread_rng();
    let mut updates = Vec::new();
    let mut timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    for seq in 1..=10 {
        // Randomly select two participants for this update
        let mut update_participants = participants.clone();
        update_participants.shuffle(&mut rng);
        let affected = &update_participants[0..2];
        
        // Create the update
        let mut affected_participants = affected.to_vec();
        sort_participants(&mut affected_participants);
        
        let amount = rng.gen_range(50_000..=100_000);
        let mut changes = HashMap::new();
        changes.insert(affected[0].clone(), -amount);
        changes.insert(affected[1].clone(), amount);
        
        // Get the keypairs for signing
        let kp_map: HashMap<_, _> = vec![
            (participants[0].clone(), &keypairs[0]),
            (participants[1].clone(), &keypairs[1]),
            (participants[2].clone(), &keypairs[2]),
            (participants[3].clone(), &keypairs[3]),
            (participants[4].clone(), &keypairs[4]),
        ].into_iter().collect();
        
        // Create signatures
        let mut signatures = Vec::new();
        for participant in &affected_participants {
            let kp = kp_map.get(participant).unwrap();
            signatures.push(sign_update(kp, &affected_participants, seq, &changes, timestamp, channel.channel_id));
        }
        
        let update = StateUpdate {
            sequence_number: seq,
            balance_changes: changes,
            signatures,
            affected_participants,
            timestamp,
        };
        
        updates.push(update);
        timestamp += 1;  // Increment timestamp for each update
    }
    
    // Apply all updates
    for (i, update) in updates.iter().enumerate() {
        println!("\nApplying update {}", i + 1);
        println!("Participants: {:?}", update.affected_participants);
        println!("Changes: {:?}", update.balance_changes);
        
        match validate_state_transition(&channel, update) {
            Ok(_) => {
                channel.apply_update(&update.clone());
                println!("Successfully applied update {}", i + 1);
                println!("New balances: {:?}", channel.balances);
            },
            Err(e) => {
                println!("Failed to apply update {}: {:?}", i + 1, e);
            }
        }
    }
    
    // Verify final state
    println!("\nFinal channel state:");
    for (i, participant) in participants.iter().enumerate() {
        println!("P{} balance: {}", i + 1, channel.balances[participant]);
    }
    
    // Verify that total balance remains constant
    let total_balance: i64 = channel.balances.values().sum();
    assert_eq!(
        total_balance,
        5_000_000,
        "Total balance should remain constant"
    );
    
    assert_eq!(channel.sequence_number, 10, "Final sequence number should be 10");
}
