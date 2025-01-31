# 🚀 Rust-Based Mini Payment Channel Protocol

## Goal
Implement a simplified off-chain payment channel, similar to how the Lightning Network works, enabling instant, low-cost transactions without needing to record every transaction on-chain.

## 📌 Why This Project?
- **Blockchain Fundamentals**: Learn how transactions and state channels work.
- **Rust Performance & Safety**: Implement secure memory-safe transactions in Rust.
- **Cryptography & Signatures**: Use Ed25519 or Secp256k1 for digital signatures.
- **Distributed Systems**: Build a lightweight protocol that can simulate multi-user off-chain transactions.
- **UTXO Model Understanding**: Integrate an Unspent Transaction Output (UTXO)-based model to track funds.

---

## 🔨 Project Breakdown

### 1️⃣ Step 1: Set Up a Basic Blockchain Transaction Structure
Create a Rust-based structure to represent a Bitcoin-like transaction.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    inputs: Vec<UTXO>,    // UTXOs being spent
    outputs: Vec<UTXO>,   // New UTXOs created
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct UTXO {
    tx_hash: Vec<u8>,     // Hash of previous transaction
    index: u32,           // Output index in previous transaction
    owner: Vec<u8>,       // Public key of owner
    amount: u64,
}
```
- Use `serde` to serialize transactions.
- Store transactions in a simple in-memory ledger.

### 2️⃣ Step 2: Implement Digital Signatures
Every payment in the channel should be cryptographically signed.

#### Generate and Verify Signatures
Use Ed25519 for signing and verifying transactions.

```rust
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

fn generate_keys() -> Keypair {
    let mut csprng = OsRng;
    Keypair::generate(&mut csprng)
}

fn sign_transaction(tx: &Transaction, keypair: &Keypair) -> Signature {
    keypair.sign(&bincode::serialize(tx).unwrap())
}

fn verify_transaction(tx: &Transaction, public_key: &PublicKey) -> bool {
    let tx_data = bincode::serialize(&(&tx.inputs, &tx.outputs)).unwrap();
    let signature = Signature::from_bytes(&tx.signature).unwrap();
    
    public_key.verify(&tx_data, &signature).is_ok()
}
```
- The sender signs a transaction using their private key.
- The receiver verifies it using the public key.

### 3️⃣ Step 3: Build the Payment Channel Mechanism
A payment channel allows off-chain transactions to happen instantly.

#### Key Ideas:
- **Opening a Channel**: Fund a multi-signature UTXO on-chain.
- **Off-Chain Transactions**: Parties exchange signed state updates instead of broadcasting transactions to the network.
- **Closing a Channel**: The latest signed transaction is broadcast to settle balances on-chain.

#### Example: Creating a Payment Channel

```rust
struct PaymentChannel {
    initiator_public: Vec<u8>,    // Ed25519 public key
    participant_public: Vec<u8>,  // Ed25519 public key
    utxos: Vec<UTXO>,             // Current unspent outputs
    spent_utxos: Vec<UTXO>,       // Spent outputs (for dispute resolution)
    channel_id: Vec<u8>,          // Unique channel identifier
}

impl PaymentChannel {
    pub fn new(initiator_pub: Vec<u8>, participant_pub: Vec<u8>, initial_utxo: UTXO) -> Self {
        // Validate initial funding transaction
        assert!(initial_utxo.owner == initiator_pub, "Initial UTXO must belong to initiator");
        
        Self {
            initiator_public: initiator_pub,
            participant_public: participant_pub,
            utxos: vec![initial_utxo],
            spent_utxos: vec![],
            channel_id: hash(&[&initiator_pub, &participant_pub]),
        }
    }

    pub fn create_transaction(&mut self, inputs: Vec<UTXO>, outputs: Vec<UTXO>, signature: Vec<u8>) {
        // Verify inputs are valid and unspent
        for input in &inputs {
            assert!(self.utxos.contains(input), "Invalid UTXO input");
        }
        
        // Verify cryptographic signature
        let tx_data = bincode::serialize(&(&inputs, &outputs)).unwrap();
        let public_key = PublicKey::from_bytes(&self.initiator_public).unwrap();
        let signature = Signature::from_bytes(&signature).unwrap();
        
        assert!(
            public_key.verify(&tx_data, &signature).is_ok(),
            "Invalid transaction signature"
        );

        // Move inputs to spent, add new outputs
        self.spent_utxos.extend(inputs.iter().cloned());
        self.utxos.retain(|u| !inputs.contains(u));
        self.utxos.extend(outputs);
    }

    pub fn settle_channel(&self) -> Vec<UTXO> {
        // In real implementation, this would create an on-chain settlement tx
        self.utxos.clone()
    }
}
```
- The initiator opens the channel by funding it with an amount.
- Both parties sign transactions off-chain.
- The final state can be broadcast to the blockchain.

### 4️⃣ Step 4: Implement Settlement Mechanism
When the payment channel closes, the last valid transaction should be recorded on-chain.

```rust
```
- If a user tries to cheat by broadcasting an old state, penalties can be added to incentivize honest behavior.

### 5️⃣ Step 5: Optimize for Performance
To make this protocol more efficient, you can:
- Implement a **Merkle tree** structure to store transactions efficiently.
- Use **async Rust (Tokio)** for handling multiple payment channels.
- Implement **zero-knowledge proofs (ZK-SNARKs)** to make transactions private.

---

## 📌 Final Features of Your Protocol
✅ **UTXO-Based Payment Channel** – Similar to the Lightning Network  
✅ **Digital Signatures for Security** – Using Ed25519  
✅ **Instant Payments Without Blockchain Fees** – Off-chain state updates  
✅ **Blockchain Settlement Support** – Closing channel with final balance  
✅ **Rust Memory Safety & Performance Optimizations**  

---
