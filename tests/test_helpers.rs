use state_channel_node::crypto::{KeyPair, PublicKey, Signature};
use state_channel_node::channel::state::{ChannelState, StateUpdate};
use state_channel_node::channel::transitions::StateUpdateForSigning;
use std::collections::HashMap;
use ed25519_dalek::Signer;

// Helper function to create a test channel with initial balances
pub fn create_test_channel(participants: &[PublicKey], initial_balance: i64) -> ChannelState {
    let mut balances = HashMap::new();
    for participant in participants {
        balances.insert(participant.clone(), initial_balance);
    }
    ChannelState::new(participants.to_vec(), balances)
}

// Helper function to create a state update with signatures
pub fn create_state_update(
    sequence_number: u64,
    changes: Vec<(PublicKey, i64)>,
    signatures: Vec<Signature>,
    affected_participants: Vec<PublicKey>,
    timestamp: u64,
) -> StateUpdate {
    let balance_changes: HashMap<_, _> = changes.into_iter().collect();
    StateUpdate {
        sequence_number,
        balance_changes,
        signatures,
        affected_participants,
        timestamp,
    }
}

pub fn sign_update(
    kp: &KeyPair,
    affected_participants: &[PublicKey],
    sequence: u64,
    changes: &HashMap<PublicKey, i64>,
    timestamp: u64,
    channel_id: [u8; 32],
) -> Signature {
    // Create a new HashMap with the same changes
    let mut message_for_signing = StateUpdateForSigning::new(
        sequence,
        channel_id,
        changes,
        affected_participants,
    );
    message_for_signing.timestamp = timestamp;
    let message = bincode::serialize(&message_for_signing).unwrap();
    Signature(kp.signing_key.try_sign(&message).unwrap())
}

// Helper function to sort public keys consistently
pub fn sort_participants(participants: &mut [PublicKey]) {
    participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
}
