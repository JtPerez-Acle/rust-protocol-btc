# Rust Protocol BTC: Lightning-style Payment Channels

[![Project Status: Active - Under Development](https://img.shields.io/badge/Project%20Status-Under%20Development-yellow.svg)]()
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)]()

> ⚠️ **Project Status**: This project is currently under active development. Current progress: 6.74% (6/89 tasks completed). See [Development Status](#development-status) for details.

## Overview

Rust Protocol BTC implements a simplified off-chain payment channel system inspired by Bitcoin's Lightning Network. It enables instant, low-cost transactions without recording every transaction on-chain, while maintaining the security guarantees of the Bitcoin blockchain.

### Key Features

- ✅ **UTXO-Based Payment Channels**: Lightning Network-style channels
- ✅ **Off-Chain Transactions**: Instant payments without blockchain fees
- ✅ **Secure State Management**: Cryptographically secure state transitions
- ✅ **Byzantine Fault Tolerance**: Robust against malicious participants
- ✅ **Rust Implementation**: Memory safety and high performance

## Architecture

### System Components

```mermaid
graph TB
    subgraph Core Components
        A[UTXO Ledger] --> |State Updates| B[Channel Manager]
        B --> |Signatures| C[Cryptography Module]
        B --> |Messages| D[Network Layer]
    end
    
    subgraph External
        E[Blockchain] --> |Funding/Settlement| A
        F[Peers] --> |P2P Communication| D
    end
```

### Channel Lifecycle

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

## Implementation Status

### Completed Components
- [x] Project initialization and setup
- [x] Basic UTXO structure
- [x] Transaction input/output models
- [x] Serialization/deserialization
- [x] UTXO set management
- [x] CI/CD pipeline configuration

### Under Development
- [ ] State management system
- [ ] Channel operations
- [ ] Cryptography module
- [ ] Network layer
- [ ] Dispute resolution
- [ ] Batch processing

## Testing Strategy

Our testing approach ensures protocol correctness and security through multiple layers:

```mermaid
graph LR
    subgraph Test Categories
        A[Unit Tests] --> B[Integration Tests]
        B --> C[Security Tests]
        C --> D[Performance Tests]
    end
    
    subgraph Coverage Requirements
        E[85% Line Coverage]
        F[100% Critical Paths]
    end
```

### Key Test Areas

1. **Unit Tests**
   ```rust
   #[test]
   fn test_utxo_state_transitions() {
       // Test UTXO creation, spending, and validation
   }
   ```

2. **Integration Tests**
   ```rust
   #[test]
   fn test_complete_channel_flow() {
       // 1. Open channel
       // 2. Multiple state updates
       // 3. Graceful closure
   }
   ```

3. **Security Tests**
   ```rust
   #[test]
   fn test_byzantine_behavior() {
       // Test system response to malicious participants
   }
   ```

## Performance Targets

| Metric | Target |
|--------|--------|
| Transaction Processing | < 100ms |
| State Updates | < 50ms |
| Memory Usage | < 4GB |

## Getting Started

### Prerequisites
- Rust 1.75+
- Cargo
- Linux-based OS (tested on Ubuntu 20.04+)

### Building from Source
```bash
# Clone the repository
git clone https://github.com/JtPerez-Acle/rust-protocol-btc.git
cd rust-protocol-btc

# Build the project
cargo build --release

# Run tests
cargo test
```

## API Example

```rust
// Create a new payment channel
async fn example_usage() -> Result<(), ChannelError> {
    let channel_id = open_channel(
        initiator_pub,
        participant_pub,
        funding_utxo
    ).await?;

    // Update channel state
    update_channel(
        &channel_id,
        signed_transaction
    ).await?;

    // Close channel
    close_channel(&channel_id).await?;
    
    Ok(())
}
```

## Contributing

While the project is under active development, no contributions are accepted. Please see our [Development Plan](docs/DEVELOPMENT_PLAN.md) for current priorities and progress.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Bitcoin Lightning Network whitepaper
- Rust community and crates.io ecosystem
- Cheap Instant Coffee for maintaining me awake during development 