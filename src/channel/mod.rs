use std::collections::HashMap;
use crate::crypto::{PublicKey, Signature};

pub mod state;
pub mod transitions;

#[derive(Debug, Clone)]
pub struct StateUpdate {
    pub sequence_number: u64,
    pub balance_changes: HashMap<PublicKey, i64>,
    pub signatures: Vec<Signature>,
    pub affected_participants: Vec<PublicKey>,
}
