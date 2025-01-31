# Payment Channel Sequence Diagrams

## 1. Channel Lifecycle

### 1.1 Channel Opening
This diagram illustrates the process of opening a new payment channel between two users. The sequence shows:
- Initial funding transaction validation
- Channel request and acknowledgment between parties
- State initialization and channel ID assignment
- Safety checks to ensure proper channel establishment

```mermaid
sequenceDiagram
    participant A as User A
    participant N as Node
    participant B as User B
    participant BC as Blockchain

    A->>N: open_channel(B_pub, funding_utxo)
    activate N
    N->>BC: validate_funding_utxo()
    BC-->>N: utxo_valid
    N->>B: channel_open_request
    activate B
    B->>N: channel_open_ack
    deactivate B
    N->>N: initialize_channel_state()
    N->>A: channel_id
    deactivate N

    Note right of N: Channel state transitions:<br/>INITIALIZED -> OPENING -> ACTIVE
```

### 1.2 Channel Operation
This diagram shows the core operation of an active payment channel, demonstrating:
- How state updates are processed and validated
- The signature verification flow between parties
- UTXO set management for each transaction
- Concurrent operation handling with proper activation/deactivation

```mermaid
sequenceDiagram
    participant A as User A
    participant N as Node
    participant B as User B
    participant UTXO as UTXO Set

    loop While channel active
        A->>N: update_channel(tx_signed)
        activate N
        N->>UTXO: validate_inputs()
        UTXO-->>N: inputs_valid
        N->>N: verify_signature()
        N->>B: tx_validation_request
        activate B
        B->>B: verify_signature()
        B->>N: tx_validation_ack
        deactivate B
        N->>UTXO: update_state()
        N->>A: state_update_confirmation
        deactivate N
    end

    Note right of N: State updates include:<br/>- UTXO set changes<br/>- Balance updates<br/>- Version tracking
```

### 1.3 Channel Settlement
This diagram details the channel closure process, showing:
- How final states are computed and verified
- The settlement transaction creation and broadcast
- Cleanup procedures for channel resources
- Confirmation flow between all parties

```mermaid
sequenceDiagram
    participant A as User A
    participant N as Node
    participant B as User B
    participant BC as Blockchain
    participant UTXO as UTXO Set

    A->>N: close_channel()
    activate N
    N->>UTXO: get_final_state()
    N->>N: create_settlement_tx()
    N->>B: settlement_proposal
    activate B
    B->>B: verify_final_state()
    B->>N: settlement_ack
    deactivate B
    N->>BC: broadcast_settlement_tx
    BC-->>N: tx_confirmed
    N->>UTXO: cleanup_channel_state()
    N->>A: channel_closed
    N->>B: channel_closed
    deactivate N

    Note right of BC: Settlement includes:<br/>- Final balances<br/>- Fee calculation<br/>- UTXO consolidation
```

## 2. Dispute Resolution

### 2.1 Malicious State Update
This diagram demonstrates the protocol's handling of malicious behavior, specifically:
- Detection of outdated state submissions
- The dispute resolution process
- Penalty enforcement mechanisms
- Fund protection for honest participants

```mermaid
sequenceDiagram
    participant H as Honest User
    participant N as Node
    participant M as Malicious User
    participant BC as Blockchain
    participant UTXO as UTXO Set

    M->>N: submit_old_state
    activate N
    N->>UTXO: detect_version_mismatch()
    N->>H: dispute_notification
    activate H
    H->>N: latest_signed_state
    N->>N: verify_signatures()
    N->>N: compare_versions()
    N->>BC: submit_penalty_tx
    BC-->>N: penalty_confirmed
    N->>UTXO: slash_malicious_funds()
    N->>H: penalty_funds_released
    deactivate H
    N->>M: channel_terminated
    deactivate N

    Note right of N: Penalties apply for:<br/>- Version rollbacks<br/>- Invalid signatures<br/>- Double spends
```

### 2.2 Forced Channel Closure
This diagram illustrates the emergency closure mechanism, showing:
- Proof verification for forced closure
- Channel state freezing process
- Penalty application logic
- Different outcomes based on proof validity

```mermaid
sequenceDiagram
    participant A as User A
    participant N as Node
    participant B as User B
    participant BC as Blockchain
    participant UTXO as UTXO Set

    A->>N: force_close_channel(proof)
    activate N
    N->>N: verify_malicious_proof()
    N->>UTXO: freeze_channel_state()
    N->>BC: submit_force_close_tx
    BC-->>N: force_close_confirmed

    alt Valid Proof
        N->>UTXO: apply_penalties()
        N->>A: penalty_funds_released
        N->>B: channel_terminated
    else Invalid Proof
        N->>A: proof_rejected
        N->>UTXO: unfreeze_channel()
    end
    deactivate N

    Note right of BC: Force close requires:<br/>- Cryptographic proof<br/>- Timelock expiration<br/>- Penalty calculation
```

## 3. Batch Operations

### 3.1 Multi-Channel Update
This diagram shows how multiple channel updates are processed atomically:
- Transaction batching mechanism
- Validation steps for each update
- Atomic commit/rollback process
- Error handling for partial failures

```mermaid
sequenceDiagram
    participant A as User A
    participant N as Node
    participant UTXO as UTXO Set
    participant CH as Channels

    A->>N: batch_update(channel_updates[])
    activate N
    N->>N: begin_transaction()

    loop for each update
        N->>CH: validate_channel_state()
        N->>UTXO: verify_inputs()
        N->>N: verify_signatures()
    end

    alt All Valid
        N->>UTXO: apply_batch_updates()
        N->>N: commit_transaction()
        N->>A: batch_success
    else Any Invalid
        N->>N: rollback_transaction()
        N->>A: batch_failed
    end
    deactivate N

    Note right of N: Batch updates ensure:<br/>- Atomic operations<br/>- Consistency<br/>- Rollback safety
```

### 3.2 Network Synchronization
This diagram demonstrates how nodes maintain consistent state across the network:
- Multi-node synchronization process
- Parallel state retrieval
- Conflict detection and resolution
- Network-wide consistency maintenance

```mermaid
sequenceDiagram
    participant NA as Node A
    participant NB as Node B
    participant NC as Node C
    participant UTXOA as UTXO Set A

    NA->>NB: sync_request()
    NA->>NC: sync_request()
    activate NA

    par
        NB->>NA: channel_states_b
    and
        NC->>NA: channel_states_c
    end

    NA->>NA: verify_states()
    NA->>UTXOA: merge_states()

    alt Conflict Detected
        NA->>NB: resolve_conflict()
        NA->>NC: resolve_conflict()
        NA->>UTXOA: apply_resolution()
    end

    NA->>NB: sync_complete
    NA->>NC: sync_complete
    deactivate NA

    Note right of NA: Sync ensures:<br/>- State consistency<br/>- Conflict resolution<br/>- Network consensus
