use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::crypto::{PublicKey, Signature, verify_partial_multisig, CryptoError};
use crate::channel::transitions::StateUpdateForSigning;
use sha2::{Sha256, Digest};
use bincode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelStatus {
    Open,
    Closed,
    Disputed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelState {
    pub channel_id: [u8; 32],  // Unique channel identifier
    pub participants: Vec<PublicKey>,
    pub balances: HashMap<PublicKey, i64>,
    pub sequence_number: u64,
    pub status: ChannelStatus,
    pub latest_update: Option<StateUpdate>,
}

impl ChannelState {
    pub fn new(participants: Vec<PublicKey>, initial_balances: HashMap<PublicKey, i64>) -> Self {
        // Create a unique channel ID by hashing the sorted participants
        let mut sorted_participants = participants.clone();
        sorted_participants.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
        
        let mut hasher = Sha256::new();
        for participant in &sorted_participants {
            hasher.update(participant.as_bytes());
        }
        let channel_id = hasher.finalize().into();
        
        Self {
            channel_id,
            participants,
            balances: initial_balances,
            sequence_number: 0,
            status: ChannelStatus::Open,
            latest_update: None,
        }
    }
    
    pub fn apply_update(&mut self, update: &StateUpdate) -> Result<(), &'static str> {
        // Sort participants for consistent message construction
        let mut sorted_participants = update.affected_participants.clone();
        sorted_participants.sort_by_key(|k| k.as_bytes());
        
        // Sort balance changes by public key for consistent order
        let mut sorted_changes: Vec<_> = update.balance_changes.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted_changes.sort_by(|a, b| a.0.as_bytes().cmp(&b.0.as_bytes()));
        
        // Construct message for verification using StateUpdateForSigning
        let message = StateUpdateForSigning {
            sequence_number: update.sequence_number,
            channel_id: self.channel_id,
            balance_changes: sorted_changes,
            affected_participants: sorted_participants.clone(),
            timestamp: update.timestamp,
        };
        
        // Calculate message hash for verification
        let message_bytes = bincode::serialize(&message).map_err(|_| "Serialization failed")?;
        
        // Verify signatures using partial multisig verification
        verify_partial_multisig(
            &update.signatures,
            &self.participants,
            &sorted_participants,
            &message_bytes
        ).map_err(|_| "Invalid signatures")?;
        
        // Update sequence number
        if update.sequence_number != self.sequence_number + 1 {
            return Err("Invalid sequence number");
        }
        self.sequence_number = update.sequence_number;
        
        // Apply balance changes
        for (participant, change) in &update.balance_changes {
            let balance = self.balances.get_mut(participant).ok_or("Unknown participant")?;
            *balance = balance.checked_add(*change).ok_or("Balance overflow")?;
            if *balance < 0 {
                return Err("Negative balance");
            }
        }
        
        // Store latest update
        self.latest_update = Some(update.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateUpdate {
    pub sequence_number: u64,
    pub balance_changes: HashMap<PublicKey, i64>,
    pub signatures: Vec<Signature>,
    pub affected_participants: Vec<PublicKey>,
    pub timestamp: u64,
}
