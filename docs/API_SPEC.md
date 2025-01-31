# API Specification

## Channel Management
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
```

## Transaction Verification
```rust
/// Validates transaction cryptographic integrity
/// Returns verified transaction or Error if:
/// - Invalid signature format
/// - Signature verification fails
/// - Input/output ratio mismatch
fn verify_transaction(tx: Transaction) -> Result<VerifiedTx, ValidationError>;
```
