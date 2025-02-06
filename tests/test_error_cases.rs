use state_channel_node::crypto;
use state_channel_node::channel::state::StateUpdate;
use state_channel_node::channel::transitions::validate_state_transition;
use std::collections::HashMap;

mod test_helpers;
use test_helpers::{create_test_channel, sign_update, sort_participants};

#[test]
fn test_multi_hop_payment_with_insufficient_funds() {
    println!("\n=== Testing Multi-Hop Payment with Insufficient Funds ===");
    // Create keypairs for three participants
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    
    let participants = vec![kp1.public_key(), kp2.public_key()];
    println!("Created channel with {} participants", participants.len());
    let mut channel = create_test_channel(&participants, 1_000_000);
    println!("Initial channel balance: 1,000,000");
    
    // First transfer: P1 -> P2 (more than balance)
    let mut affected_participants = participants.clone();
    sort_participants(&mut affected_participants);
    
    let mut changes = HashMap::new();
    changes.insert(participants[0].clone(), -2_000_000); // More than available
    changes.insert(participants[1].clone(), 2_000_000);
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut signatures = Vec::new();
    for participant in &affected_participants {
        let kp = if participant == &participants[0] { &kp1 } else { &kp2 };
        signatures.push(sign_update(kp, &affected_participants, 1, &changes, timestamp, channel.channel_id));
    }
    
    let update = StateUpdate {
        sequence_number: 1,
        balance_changes: changes,
        signatures,
        affected_participants,
        timestamp,
    };
    
    println!("\nAttempting to apply update that exceeds balance");
    match validate_state_transition(&channel, &update) {
        Ok(_) => {
            channel.apply_update(&update.clone());
            println!("Successfully applied update");
            println!("New balances: {:?}", channel.balances);
        },
        Err(e) => {
            println!("Failed to apply update as expected: {:?}", e);
        }
    }
    
    // Verify state hasn't changed
    println!("\nFinal channel state:");
    println!("P1 balance: {}", channel.balances[&participants[0]]);
    println!("P2 balance: {}", channel.balances[&participants[1]]);
    
    assert_eq!(
        channel.balances[&participants[0]],
        1_000_000,
        "P1 balance should be unchanged"
    );
    assert_eq!(
        channel.balances[&participants[1]],
        1_000_000,
        "P2 balance should be unchanged"
    );
    assert_eq!(channel.sequence_number, 0, "Sequence number should be unchanged");
}
