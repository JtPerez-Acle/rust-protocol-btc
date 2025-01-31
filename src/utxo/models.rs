use serde::{Deserialize, Serialize};
use primitive_types::H256;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u64,
    pub hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Input {
    pub previous_output: H256,
    pub signature: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Output {
    pub value: u64,
    pub public_key_hash: Vec<u8>,
    pub lock_script: Vec<u8>,
}

impl Transaction {
    pub fn calculate_hash(&self) -> H256 {
        // TODO: Implement proper transaction hashing
        H256::default()
    }
}
