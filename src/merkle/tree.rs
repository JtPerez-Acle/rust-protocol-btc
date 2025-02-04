use primitive_types::H256;
use crate::merkle::{MerkleNode, MerkleProof};

/// A Merkle tree implementation
#[derive(Default)]
pub struct MerkleTree {
    root: Option<MerkleNode>,
    leaf_count: usize,
    levels: Vec<Vec<H256>>,
}

impl MerkleTree {
    /// Create a new empty Merkle tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the root hash of the tree
    pub fn root_hash(&self) -> Option<H256> {
        self.root.as_ref().map(|node| node.hash)
    }

    /// Construct a Merkle tree from a vector of leaf data
    pub fn from_leaves(leaves: Vec<Vec<u8>>) -> Self {
        if leaves.is_empty() {
            return Self::default();
        }

        println!("\nConstructing Merkle tree with {} leaves", leaves.len());
        let mut nodes = leaves.iter()
            .map(|data| {
                let hash = MerkleNode::hash_leaf(data);
                println!("  Leaf hash: {:?} for data: {:?}", hash, data);
                MerkleNode::new_leaf(hash)
            })
            .collect::<Vec<_>>();

        // If we have an odd number of nodes, duplicate the last one
        if nodes.len() % 2 == 1 {
            println!("  Duplicating last leaf for odd number of leaves");
            let last = nodes.last().unwrap().clone();
            nodes.push(last);
        }

        let mut level = 0;
        let mut levels = Vec::new();
        levels.push(nodes.iter().map(|n| n.hash.clone()).collect::<Vec<_>>());
        while nodes.len() > 1 {
            println!("\nProcessing level {}", level);
            let mut new_nodes = Vec::new();

            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    let left = &chunk[0];
                    let right = &chunk[1];
                    let combined = MerkleNode::hash_internal(&left.hash, &right.hash);
                    println!("  Combining {:?} + {:?} = {:?}", left.hash, right.hash, combined);
                    let parent = MerkleNode::new_internal(combined, Some(left.clone()), Some(right.clone()));
                    new_nodes.push(parent);
                }
            }

            levels.push(new_nodes.iter().map(|n| n.hash.clone()).collect::<Vec<_>>());
            nodes = new_nodes;
            level += 1;
        }

        println!("\nFinal tree:");
        println!("  Root hash: {:?}", nodes.first().map(|n| n.hash));
        println!("  Leaf count: {}", leaves.len());

        Self {
            root: nodes.into_iter().next(),
            leaf_count: leaves.len(),
            levels,
        }
    }

    /// Generate a Merkle proof for a leaf at the given index
    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= self.leaf_count {
            return None;
        }

        println!("\nGenerating proof for leaf {}", leaf_index);
        println!("Tree has {} leaves", self.leaf_count);

        // Calculate the initial level size including padding for odd number of leaves
        let mut initial_level_size = self.leaf_count;
        if initial_level_size > 1 && initial_level_size % 2 == 1 {
            initial_level_size += 1;
        }
        println!("Initial level size: {}", initial_level_size);

        let mut proof_hashes = Vec::new();
        let mut current_level = 0;
        let mut current_index = leaf_index;
        let mut level_size = initial_level_size;

        while level_size > 1 {
            println!("  Level size: {}, Index: {}, Is left: {}", level_size, current_index, current_index % 2 == 0);
            
            // Get the sibling hash
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            println!("  Collecting proof at level:");
            println!("    Level size: {}", level_size);
            println!("    Current index: {}", current_index);
            println!("    Is left: {}", current_index % 2 == 0);

            // Get the current and sibling hashes from the correct level
            let current_hash = if current_index < self.levels[current_level].len() {
                self.levels[current_level][current_index].clone()
            } else {
                // If we're beyond the level size, duplicate the last hash
                self.levels[current_level].last().unwrap().clone()
            };

            let sibling_hash = if sibling_index < self.levels[current_level].len() {
                self.levels[current_level][sibling_index].clone()
            } else {
                // If the sibling is beyond the level size, duplicate the current hash
                current_hash.clone()
            };

            println!("    Left hash: {:#x}", current_hash);
            println!("    Right hash: {:#x}", sibling_hash);

            if current_index % 2 == 0 {
                println!("    Adding right sibling: {:#x}", sibling_hash);
            } else {
                println!("    Adding left sibling: {:#x}", sibling_hash);
            }

            proof_hashes.push(sibling_hash);

            current_level += 1;
            current_index /= 2;
            level_size = (level_size + 1) / 2;
        }

        println!("\nProof summary:");
        println!("  Target leaf index: {}", leaf_index);
        println!("  Number of proof hashes: {}", proof_hashes.len());
        for (i, hash) in proof_hashes.iter().enumerate() {
            println!("  Hash {}: {:#x}", i, hash);
        }

        Some(MerkleProof::new(proof_hashes, leaf_index, initial_level_size))
    }
}
