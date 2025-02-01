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

/// Represents a transaction input that spends an existing UTXO
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Input {
    /// Hash of the previous transaction containing the output being spent
    pub previous_output: H256,
    /// Index of the output in the previous transaction (0-based)
    pub index: u32,
    /// Cryptographic signature proving ownership of the UTXO
    #[serde(with = "hex")]
    pub signature: Vec<u8>,
    /// Sequence number for relative timelock (BIP68)
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
