use primitive_types::H256;
use sha3::{Digest, Keccak256};

/// A node in the Merkle tree
#[derive(Clone, Debug)]
pub struct MerkleNode {
    pub hash: H256,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    /// Create a new leaf node with the given hash
    pub fn new_leaf(hash: H256) -> Self {
        Self {
            hash,
            left: None,
            right: None,
        }
    }

    /// Create a new internal node with the given children
    pub fn new_internal(hash: H256, left: Option<Self>, right: Option<Self>) -> Self {
        Self {
            hash,
            left: left.map(Box::new),
            right: right.map(Box::new),
        }
    }

    /// Hash a leaf node's data
    pub fn hash_leaf(data: &[u8]) -> H256 {
        let mut hasher = Keccak256::new();
        // Prefix with 0x00 to distinguish from internal nodes
        hasher.update(&[0x00]);
        hasher.update(data);
        H256::from(hasher.finalize().as_ref())
    }

    /// Hash two child hashes together to create a parent hash
    pub fn hash_internal(left: &H256, right: &H256) -> H256 {
        let mut hasher = Keccak256::new();
        // Prefix with 0x01 to distinguish from leaf nodes
        hasher.update(&[0x01]);
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        H256::from(hasher.finalize().as_ref())
    }
}
