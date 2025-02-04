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

/// Represents an Unspent Transaction Output (UTXO)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Utxo {
    /// The transaction output
    pub output: Output,
    /// Height of the block containing this UTXO
    pub block_height: u32,
    /// Index in the transaction's output vector
    pub output_index: u32,
    /// Hash of the transaction containing this output
    pub tx_hash: H256,
    /// Whether this UTXO is confirmed (included in a block)
    pub is_confirmed: bool,
}

impl Transaction {
    pub fn calculate_hash(&self) -> H256 {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        
        // Add transaction data to hasher
        hasher.update(&self.version.to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_output.as_bytes());
            hasher.update(&input.index.to_le_bytes());
            hasher.update(&input.signature);
            hasher.update(&input.sequence.to_le_bytes());
        }
        for output in &self.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.public_key_hash);
            hasher.update(&output.lock_script);
        }
        hasher.update(&self.lock_time.to_le_bytes());
        
        // Convert hash to H256
        let result = hasher.finalize();
        H256::from_slice(&result)
    }
}

impl Utxo {
    /// Create a new UTXO from a transaction output
    pub fn new(output: Output, block_height: u32, output_index: u32, tx_hash: H256) -> Self {
        Self {
            output,
            block_height,
            output_index,
            tx_hash,
            is_confirmed: true,
        }
    }

    /// Create a new unconfirmed UTXO from a transaction output
    pub fn new_unconfirmed(output: Output, output_index: u32, tx_hash: H256) -> Self {
        Self {
            output,
            block_height: 0,
            output_index,
            tx_hash,
            is_confirmed: false,
        }
    }
}
