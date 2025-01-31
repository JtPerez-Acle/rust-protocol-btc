# API Specification

## Batch Processing
```rust
/// Atomically update multiple channels
async fn batch_update(channels: Vec<(&[u8], Transaction)>) -> Result<(), ChannelError>;
```

# Timelock Rules
All transactions must include:
```rust
struct Transaction {
    // ... existing fields ...
    timelock: Option<Locktime>, // BIP68-style relative locktime
}
```
Validation rules:
1. Relative locktimes must be ≥ previous transaction's locktime
2. Absolute locktimes use blockchain height-based expiration
3. Settlement transactions require minimum 144-block delay

## Event Subscription
```rust
/// Channel state change notifications
struct ChannelEvent {
    event_type: EventType, // Update|Dispute|Close
    channel_id: Vec<u8>,
    timestamp: u64,
}

/// Subscribe to channel events
async fn subscribe_to_events(sender: mpsc::Sender<ChannelEvent>);
```

## Channel Lifecycle
```rust
/// Creates new payment channel with initial funding UTXO
/// Returns channel ID or Error if:
/// - UTXO ownership invalid
/// - Duplicate channel exists
async fn open_channel(initiator_pub: [u8], participant_pub: [u8], funding_utxo: UTXO) -> Result<Vec<u8>, ChannelError>;

/// Updates channel state with signed transaction
/// Validates:
/// 1. All inputs are current UTXOs
/// 2. Signature matches participant's key
/// 3. Output amounts ≤ Input amounts (no inflation)
async fn update_channel(channel_id: &[u8], tx: Transaction) -> Result<(), ChannelError>;

/// Initiates on-chain settlement process
/// Returns final UTXO set for settlement transaction
async fn close_channel(channel_id: &[u8]) -> Result<Vec<UTXO>, ChannelError>;

/// Emergency closure for suspected malicious activity
/// Bypasses normal dispute period with proof of fraud
async fn force_close_channel(
    channel_id: &[u8],
    proof: MaliciousProof
) -> Result<Vec<UTXO>, ChannelError>;
```

## Dispute Resolution
```rust
/// Submit a disputed transaction for on-chain arbitration
/// Validates:
/// 1. Dispute period is still active
/// 2. Contesting transaction is newer than disputed one
async fn submit_dispute(
    channel_id: &[u8],
    disputed_tx: Transaction,
    contesting_tx: Transaction
) -> Result<Penalty, ChannelError>;

/// Get current UTXO set for a channel
async fn get_channel_state(channel_id: &[u8]) -> Result<Vec<UTXO>, ChannelError>;

/// Get all spent UTXOs (for audit purposes)
async fn get_spent_utxos(channel_id: &[u8]) -> Result<Vec<UTXO>, ChannelError>;
```

## Transaction Verification
```rust
/// Validates transaction cryptographic integrity
/// Returns verified transaction or Error if:
/// - Invalid signature format
/// - Signature verification fails
/// - Input/output ratio mismatch
/// - Signature malleability detected
fn verify_transaction(tx: Transaction) -> Result<VerifiedTx, ValidationError> {
    let mut normalized_tx = tx.clone();
    normalize_signature(&mut normalized_tx);
    // ... original verification logic ...
}

/// Validates transaction timelock constraints
/// Returns success only if:
/// - Relative locktimes are ≥ previous transaction's locktime
/// - Absolute locktimes have not expired
fn validate_timelocks(tx: &Transaction) -> Result<(), ValidationError>;
```
