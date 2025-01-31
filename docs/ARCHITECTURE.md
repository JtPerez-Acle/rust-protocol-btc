# Payment Channel Architecture

## Core Components
1. **UTXO Ledger** - Tracks all unspent outputs
2. **Channel Manager** - Handles channel lifecycle:
   - Channel opening (Funding TX validation)
   - State updates (Signed transactions)
   - Dispute resolution (Spent UTXO tracking)
   - Settlement (Final TX generation)

3. **Cryptography Module** - Standardized interface for:
   - Ed25519 signature creation/verification
   - Transaction serialization format
   - Channel ID derivation (Hash of participants' keys)

## Data Flow
1. Channel Opening:
   On-chain TX → Funding UTXO → Channel Creation

2. State Update:
   Previous UTXOs → Signed TX → New UTXOs → State Versioning

3. Settlement:
   Latest UTXO Set → On-chain Settlement TX → Channel Closure
