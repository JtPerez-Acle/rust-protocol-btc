use state_channel_node::merkle::MerkleTree;

#[test]
fn test_single_leaf() {
    let data = vec![1, 2, 3, 4];
    let tree = MerkleTree::from_leaves(vec![data.clone()]);
    let root_hash = tree.root_hash().unwrap();
    let proof = tree.generate_proof(0).unwrap();
    
    assert!(proof.verify(root_hash, &data));
}

#[test]
fn test_multiple_utxo_leaves() {
    let utxo_data = vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
        vec![13, 14, 15, 16],
    ];
    
    let tree = MerkleTree::from_leaves(utxo_data.clone());
    let root_hash = tree.root_hash().unwrap();
    
    for (i, data) in utxo_data.iter().enumerate() {
        let proof = tree.generate_proof(i).unwrap();
        assert!(proof.verify(root_hash, data));
    }
}

#[test]
fn test_invalid_proof() {
    let data = vec![1, 2, 3, 4];
    let tree = MerkleTree::from_leaves(vec![data.clone()]);
    let root_hash = tree.root_hash().unwrap();
    let proof = tree.generate_proof(0).unwrap();
    
    let modified_data = vec![9, 10, 11, 12];
    assert!(!proof.verify(root_hash, &modified_data));
}

#[test]
fn test_single_leaf_tree() {
    let single_data = vec![vec![1, 2, 3, 4]];
    let tree = MerkleTree::from_leaves(single_data.clone());
    let root_hash = tree.root_hash().unwrap();
    let proof = tree.generate_proof(0).unwrap();
    
    assert!(proof.verify(root_hash, &single_data[0]));
}

#[test]
fn test_odd_number_of_leaves() {
    let odd_data = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];
    
    let tree = MerkleTree::from_leaves(odd_data.clone());
    let odd_root = tree.root_hash().unwrap();
    let last_proof = tree.generate_proof(2).unwrap();
    
    assert!(last_proof.verify(odd_root, &odd_data[2]));
}
