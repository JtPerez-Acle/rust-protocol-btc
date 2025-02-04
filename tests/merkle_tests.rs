use state_channel_node::merkle::MerkleTree;

mod common;

#[test]
fn test_empty_tree() {
    let tree = MerkleTree::new();
    assert_eq!(tree.root_hash(), None);
}

#[test]
fn test_single_leaf() {
    let data = vec![1, 2, 3, 4];
    let tree = MerkleTree::from_leaves(vec![data.clone()]);
    let root_hash = tree.root_hash().unwrap();
    let proof = tree.generate_proof(0).unwrap();
    
    assert!(proof.verify(root_hash, &data));
}

#[test]
fn test_multiple_leaves() {
    let leaves = vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
        vec![13, 14, 15, 16],
    ];
    
    let tree = MerkleTree::from_leaves(leaves.clone());
    let root_hash = tree.root_hash().unwrap();
    
    for (i, leaf) in leaves.iter().enumerate() {
        let proof = tree.generate_proof(i).unwrap();
        assert!(proof.verify(root_hash, leaf), "Failed to verify leaf {}", i);
    }
}

#[test]
fn test_invalid_proof() {
    let data = vec![1, 2, 3, 4];
    let tree = MerkleTree::from_leaves(vec![data.clone()]);
    let root_hash = tree.root_hash().unwrap();
    let proof = tree.generate_proof(0).unwrap();
    
    // Try to verify with wrong data
    let wrong_data = vec![9, 10, 11, 12];
    assert!(!proof.verify(root_hash, &wrong_data));
}

#[test]
fn test_odd_number_of_leaves() {
    let leaves = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];
    
    let tree = MerkleTree::from_leaves(leaves.clone());
    let root_hash = tree.root_hash().unwrap();
    
    // Verify each leaf
    for (i, leaf) in leaves.iter().enumerate() {
        let proof = tree.generate_proof(i).unwrap();
        assert!(proof.verify(root_hash, leaf), "Failed to verify leaf {}", i);
    }
}

#[test]
fn test_power_of_two_leaves() {
    let mut leaves = Vec::new();
    for i in 0..8 {
        leaves.push(vec![i as u8]);
    }
    
    let tree = MerkleTree::from_leaves(leaves.clone());
    let root_hash = tree.root_hash().unwrap();
    
    // Verify each leaf
    for (i, leaf) in leaves.iter().enumerate() {
        let proof = tree.generate_proof(i).unwrap();
        assert!(proof.verify(root_hash, leaf), "Failed to verify leaf {}", i);
    }
}

#[test]
fn test_out_of_bounds_proof() {
    let leaves = vec![vec![1, 2, 3]];
    let tree = MerkleTree::from_leaves(leaves);
    
    // Try to get proof for non-existent leaf
    assert!(tree.generate_proof(1).is_none());
}
