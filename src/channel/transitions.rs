use super::state::{ChannelState, StateUpdate};
use crate::crypto;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug, PartialEq)]
pub enum ChannelError {
    #[error("Invalid sequence number")]
    InvalidSequence,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Stale update")]
    StaleUpdate,
    #[error("Unknown participant")]
    UnknownParticipant,
    #[error("Non-zero balance change")]
    NonZeroBalanceChange,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid signature count")]
    InvalidSignatureCount,
    #[error("Serialization error")]
    SerializationError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateUpdateForSigning {
    pub sequence_number: u64,
    pub channel_id: [u8; 32],  // Unique channel identifier
    pub balance_changes: Vec<(crypto::PublicKey, i64)>,  // Sorted list of balance changes
    pub affected_participants: Vec<crypto::PublicKey>,  // Sorted list of affected participants
    pub timestamp: u64,  // For replay protection
}

impl StateUpdateForSigning {
    pub fn new(
        sequence_number: u64,
        channel_id: [u8; 32],
        balance_changes: &HashMap<crypto::PublicKey, i64>,
        affected_participants: &[crypto::PublicKey],
    ) -> Self {
        // Sort affected participants by public key for deterministic ordering
        let mut sorted_participants = affected_participants.to_vec();
        sorted_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
        
        // Create a sorted Vec of balance changes
        let mut sorted_changes: Vec<_> = balance_changes.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted_changes.sort_by(|a, b| a.0.as_bytes().cmp(&b.0.as_bytes()));
        
        Self {
            sequence_number,
            channel_id,
            balance_changes: sorted_changes,
            affected_participants: sorted_participants,
            timestamp: 0, // Default to 0, let caller set the timestamp if needed
        }
    }
    
    // Helper function to verify signatures in deterministic order
    pub fn verify_signatures(&self, signatures: &[crypto::Signature]) -> Result<(), ChannelError> {
        if signatures.len() != self.affected_participants.len() {
            return Err(ChannelError::InvalidSignatureCount);
        }
        
        let message = bincode::serialize(&self).map_err(|_| ChannelError::SerializationError)?;
        
        // Verify signatures in the same order as affected_participants
        for (i, participant) in self.affected_participants.iter().enumerate() {
            // Verify that the signature at index i corresponds to the participant at index i
            if !participant.verify_signature(&signatures[i], &message) {
                println!("Signature verification failed for participant at index {}", i);
                return Err(ChannelError::InvalidSignature);
            }
        }
        
        Ok(())
    }
}

pub fn validate_state_transition(
    channel: &ChannelState,
    update: &StateUpdate,
) -> Result<(), ChannelError> {
    println!("\nValidating state transition:");
    println!("Current sequence: {}", channel.sequence_number);
    println!("Update sequence: {}", update.sequence_number);
    println!("Participants: {:?}", channel.participants);
    println!("Balance changes: {:?}", update.balance_changes);
    println!("Signatures: {} provided", update.signatures.len());
    println!("Affected participants: {:?}", update.affected_participants);

    // Validate sequence number - must be exactly one more than current
    if update.sequence_number != channel.sequence_number + 1 {
        println!("Invalid sequence number: expected {}, got {}", 
                channel.sequence_number + 1, update.sequence_number);
        return Err(ChannelError::InvalidSequence);
    }

    // Verify all participants in balance changes are channel participants
    for participant in update.balance_changes.keys() {
        if !channel.participants.contains(participant) {
            return Err(ChannelError::UnknownParticipant);
        }
    }

    // Verify sum of balance changes is zero
    let sum: i64 = update.balance_changes.values().sum();
    if sum != 0 {
        println!("Non-zero balance change sum: {}", sum);
        return Err(ChannelError::NonZeroBalanceChange);
    }

    // Get a sorted list of affected participants for consistent ordering
    let mut sorted_affected = update.affected_participants.clone();
    sorted_affected.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));

    // Verify affected_participants list is properly sorted
    if sorted_affected != update.affected_participants {
        println!("Affected participants list is not properly sorted");
        return Err(ChannelError::InvalidSignatureCount);
    }

    // Verify each participant has sufficient funds for their balance changes
    for (participant, change) in &update.balance_changes {
        let current_balance = channel.balances.get(participant).unwrap_or(&0);
        
        // Check for overflow when adding the change to the current balance
        let new_balance = current_balance.checked_add(*change)
            .ok_or(ChannelError::InsufficientFunds)?;
            
        if new_balance < 0 {
            println!("Insufficient funds for participant {:?}: balance={}, change={}", 
                    participant, current_balance, change);
            return Err(ChannelError::InsufficientFunds);
        }
    }

    // Verify all participants with balance changes are in affected_participants
    for participant in update.balance_changes.keys() {
        if !update.affected_participants.contains(participant) {
            println!("Participant with balance change not in affected_participants list: {:?}", participant);
            return Err(ChannelError::InvalidSignatureCount);
        }
    }

    // Validate that all affected participants have provided signatures
    if update.signatures.len() != update.affected_participants.len() {
        println!("Signature count mismatch: {} signatures for {} affected participants", 
                update.signatures.len(), update.affected_participants.len());
        return Err(ChannelError::InvalidSignatureCount);
    }

    // Create message for signature verification
    let mut message_for_signing = StateUpdateForSigning::new(
        update.sequence_number,
        channel.channel_id,
        &update.balance_changes,
        &update.affected_participants, // Use the original order of affected participants
    );
    message_for_signing.timestamp = update.timestamp;
    
    // Verify signatures using the helper function
    message_for_signing.verify_signatures(&update.signatures)?;

    Ok(())
}
