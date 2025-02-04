use primitive_types::H256;
use crate::merkle::node::MerkleNode;

/// A Merkle proof that proves a leaf exists in the tree
#[derive(Debug)]
pub struct MerkleProof {
    /// The hashes needed to reconstruct the path from leaf to root
    pub proof_hashes: Vec<H256>,
    /// The index of the leaf in the tree
    pub leaf_index: usize,
    /// The initial level size of the tree
    pub initial_level_size: usize,
}

impl MerkleProof {
    /// Create a new Merkle proof
    pub fn new(proof_hashes: Vec<H256>, leaf_index: usize, initial_level_size: usize) -> Self {
        Self {
            proof_hashes,
            leaf_index,
            initial_level_size,
        }
    }

    /// Verify a Merkle proof against a root hash and leaf data
    pub fn verify(&self, root_hash: H256, data: &[u8]) -> bool {
        let mut current_hash = MerkleNode::hash_leaf(data);
        let mut current_index = self.leaf_index;
        let mut level_size = self.initial_level_size;

        println!("\nVerifying Merkle proof:");
        println!("  Leaf index: {}", self.leaf_index);
        println!("  Number of proof hashes: {}", self.proof_hashes.len());
        println!("  Initial level size: {}", self.initial_level_size);
        println!("  Initial leaf hash: {:#x}", current_hash);
        println!("  Target root hash: {:#x}", root_hash);

        // Special case: if there's only one leaf, we need to duplicate it like in tree construction
        if level_size == 1 {
            current_hash = MerkleNode::hash_internal(&current_hash, &current_hash);
            return current_hash == root_hash;
        }

        for (i, sibling_hash) in self.proof_hashes.iter().enumerate() {
            println!("\n  Level {} verification:", i);
            println!("    Current index: {}", current_index);
            println!("    Level size: {}", level_size);
            let is_left = current_index % 2 == 0;
            println!("    Is left child: {}", is_left);
            println!("    Current hash: {:#x}", current_hash);
            println!("    Sibling hash: {:#x}", sibling_hash);

            // Handle odd-sized levels by duplicating the last node
            let (left, right) = if is_left {
                if current_index + 1 >= level_size {
                    // We're the last node in an odd-sized level, duplicate ourselves
                    (current_hash, current_hash)
                } else {
                    (current_hash, *sibling_hash)
                }
            } else {
                (*sibling_hash, current_hash)
            };

            println!("    Combining as left + right: {:#x} + {:#x}", left, right);
            current_hash = MerkleNode::hash_internal(&left, &right);
            println!("    Combined hash: {:#x}", current_hash);

            current_index /= 2;
            level_size = (level_size + 1) / 2;
        }

        println!("\nFinal verification:");
        println!("  Computed root: {:#x}", current_hash);
        println!("  Expected root: {:#x}", root_hash);
        println!("  Match: {}", current_hash == root_hash);

        current_hash == root_hash
    }
}
