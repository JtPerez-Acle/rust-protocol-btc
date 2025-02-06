use state_channel_node::crypto::{self, PublicKey, Signature};
use state_channel_node::channel::state::{ChannelState, StateUpdate};
use state_channel_node::channel::transitions::{validate_state_transition, ChannelError, StateUpdateForSigning};
use signature::Signer;
use std::collections::HashMap;

// Test helpers
fn create_test_channel(participants: &[PublicKey], initial_balance: i64) -> ChannelState {
    let mut balances = HashMap::new();
    for pk in participants {
        balances.insert(pk.clone(), initial_balance);
    }
    ChannelState::new(participants.to_vec(), balances)
}

fn create_test_state_update(
    sequence_number: u64,
    changes: Vec<(PublicKey, i64)>,
    signatures: Vec<Signature>,
    affected_participants: Vec<PublicKey>,
    timestamp: u64,
) -> StateUpdate {
    // Create balance changes in the same order as affected_participants
    let mut balance_changes = HashMap::new();
    for participant in &affected_participants {
        if let Some((_, amount)) = changes.iter().find(|(p, _)| p == participant) {
            balance_changes.insert(participant.clone(), *amount);
        }
    }
    
    StateUpdate {
        sequence_number,
        balance_changes,
        signatures,
        affected_participants,
        timestamp,
    }
}

#[test]
fn test_sequence_number_validation() {
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    let participants = vec![
        kp1.verifying_key.clone(),
        kp2.verifying_key.clone(),
    ];
    
    let mut channel = create_test_channel(&participants, 1_000_000);
    channel.sequence_number = 5; // Simulate previous updates
    
    let mut affected_participants = vec![participants[0].clone(), participants[1].clone()];
    affected_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
    
    let changes = vec![
        (participants[0].clone(), -100_000),
        (participants[1].clone(), 100_000),
    ];
    
    let balance_changes: HashMap<_, _> = changes.iter().cloned().collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut message_for_signing = StateUpdateForSigning::new(
        1, 
        channel.channel_id,
        &balance_changes,
        &affected_participants,
    );
    message_for_signing.timestamp = timestamp;
    let message = bincode::serialize(&message_for_signing).unwrap();
    
    let signatures = vec![
        Signature(kp1.signing_key.try_sign(&message).unwrap()),
        Signature(kp2.signing_key.try_sign(&message).unwrap()),
    ];
    
    let update = create_test_state_update(1, changes, signatures, affected_participants, timestamp);
    
    // Should fail due to sequence number being less than current
    assert!(matches!(
        validate_state_transition(&channel, &update),
        Err(ChannelError::InvalidSequence)
    ));
}

#[test]
fn test_balance_overflow_protection() {
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    let participants = vec![
        kp1.verifying_key.clone(),
        kp2.verifying_key.clone(),
    ];
    
    let channel = create_test_channel(&participants, 1_000_000);
    
    let mut affected_participants = vec![participants[0].clone(), participants[1].clone()];
    affected_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
    
    let changes = vec![
        (participants[0].clone(), -i64::MAX),
        (participants[1].clone(), i64::MAX),
    ];
    
    let balance_changes: HashMap<_, _> = changes.iter().cloned().collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut message_for_signing = StateUpdateForSigning::new(
        1,
        channel.channel_id,
        &balance_changes,
        &affected_participants,
    );
    message_for_signing.timestamp = timestamp;
    let message = bincode::serialize(&message_for_signing).unwrap();
    
    let signatures = vec![
        Signature(kp1.signing_key.try_sign(&message).unwrap()),
        Signature(kp2.signing_key.try_sign(&message).unwrap()),
    ];
    
    let update = create_test_state_update(1, changes, signatures, affected_participants, timestamp);
    
    // Should fail due to potential overflow
    assert!(matches!(
        validate_state_transition(&channel, &update),
        Err(ChannelError::InsufficientFunds)
    ));
}

#[test]
fn test_invalid_signature_order() {
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    
    let participants = vec![kp1.public_key(), kp2.public_key()];
    let channel = create_test_channel(&participants, 1_000_000);
    
    // Create a sorted list of affected participants
    let mut affected_participants = vec![participants[0].clone(), participants[1].clone()];
    affected_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
    
    // Create balance changes in the same order as affected_participants
    let changes: Vec<_> = affected_participants.iter().enumerate().map(|(i, p)| {
        if i == 0 {
            (p.clone(), -100_000)
        } else {
            (p.clone(), 100_000)
        }
    }).collect();
    
    let balance_changes: HashMap<_, _> = changes.iter().cloned().collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Create message for signing with sorted participants
    let mut message_for_signing = StateUpdateForSigning::new(
        channel.sequence_number + 1,
        channel.channel_id,
        &balance_changes,
        &affected_participants,
    );
    message_for_signing.timestamp = timestamp;
    let message = bincode::serialize(&message_for_signing).unwrap();
    
    // Create signatures in WRONG order (reversed from affected_participants)
    let keypair_map: HashMap<_, _> = vec![
        (kp1.public_key(), &kp1),
        (kp2.public_key(), &kp2),
    ].into_iter().collect();
    
    let signatures: Vec<_> = affected_participants.iter().rev().map(|p| {
        let kp = keypair_map.get(p).unwrap();
        Signature(kp.signing_key.try_sign(&message).unwrap())
    }).collect();
    
    let update = create_test_state_update(
        channel.sequence_number + 1,
        changes,
        signatures,
        affected_participants,
        timestamp
    );
    
    // Should fail with InvalidSignature error since signatures are in wrong order
    assert!(matches!(validate_state_transition(&channel, &update),
        Err(ChannelError::InvalidSignature)));
}

#[test]
fn test_missing_participant_signature() {
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    let participants = vec![
        kp1.verifying_key.clone(),
        kp2.verifying_key.clone(),
    ];
    
    let channel = create_test_channel(&participants, 1_000_000);
    
    let mut affected_participants = vec![participants[0].clone(), participants[1].clone()];
    affected_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
    
    let changes = vec![
        (participants[0].clone(), -100_000),
        (participants[1].clone(), 100_000),
    ];
    
    let balance_changes: HashMap<_, _> = changes.iter().cloned().collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut message_for_signing = StateUpdateForSigning::new(
        1,
        channel.channel_id,
        &balance_changes,
        &affected_participants,
    );
    message_for_signing.timestamp = timestamp;
    let message = bincode::serialize(&message_for_signing).unwrap();
    
    // Only include one signature
    let signatures = vec![
        Signature(kp1.signing_key.try_sign(&message).unwrap()),
    ];
    
    let update = create_test_state_update(1, changes, signatures, affected_participants, timestamp);
    
    // Should fail due to missing signature
    assert!(matches!(
        validate_state_transition(&channel, &update),
        Err(ChannelError::InvalidSignatureCount)
    ));
}

#[test]
fn test_valid_transition() {
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    
    let participants = vec![kp1.public_key(), kp2.public_key()];
    let channel = create_test_channel(&participants, 1_000_000);
    
    // Create a sorted list of affected participants
    let mut affected_participants = vec![participants[0].clone(), participants[1].clone()];
    affected_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
    
    // Create balance changes in the same order as affected_participants
    let changes: Vec<_> = vec![
        (affected_participants[0].clone(), -100_000),
        (affected_participants[1].clone(), 100_000),
    ];
    
    let balance_changes: HashMap<_, _> = changes.iter().cloned().collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Create message for signing with sorted participants
    let mut message_for_signing = StateUpdateForSigning::new(
        channel.sequence_number + 1,  // Use the next sequence number
        channel.channel_id,
        &balance_changes,
        &affected_participants,
    );
    message_for_signing.timestamp = timestamp;
    let message = bincode::serialize(&message_for_signing).unwrap();
    
    // Create a map of public keys to keypairs for easier lookup
    let keypair_map: HashMap<_, _> = vec![
        (kp1.public_key(), &kp1),
        (kp2.public_key(), &kp2),
    ].into_iter().collect();
    
    // Create signatures in the same order as affected_participants
    let signatures: Vec<_> = affected_participants.iter().map(|p| {
        let kp = keypair_map.get(p).unwrap();
        Signature(kp.signing_key.try_sign(&message).unwrap())
    }).collect();
    
    let update = create_test_state_update(
        channel.sequence_number + 1,  // Use the next sequence number
        changes,
        signatures,
        affected_participants,
        timestamp
    );
    
    // Should succeed
    assert!(validate_state_transition(&channel, &update).is_ok());
}
