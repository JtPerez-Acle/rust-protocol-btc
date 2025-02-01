# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Initial development plan in docs/DEVELOPMENT_PLAN.md
- Basic in-memory UTXO set management
- UTXO storage error handling
- CHANGELOG.md file for tracking project changes
- SdbStore implementation for UTXO persistence
- Documentation comments for transaction models
- Enhanced Input struct with BIP68 sequence numbers and hex serialization

### Changed
- Updated Input struct with index field and documentation
- Improved UTXO storage error handling patterns
- Consolidated changelog entries for better readability
- Project structure initialized based on development plan
- Updated development plan with completed initial setup tasks
- Core dependencies: Tokio, ed25519-dalek, RocksDB
- Basic UTXO transaction models (Input, Output, Transaction)
- Serialization support with serde
- Development toolchain (rustfmt, clippy)
- GitHub Actions CI/CD pipeline

## [0.1.0] - 2025-01-31
### Added
- Basic project documentation framework
- Task tracking system for development phases
- Initial dependency list for Rust components

### Security
- Security audit requirements documented
- Cryptographic implementation guidelines added

---
[Unreleased]: https://github.com/your-org/your-repo/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-org/your-repo/releases/tag/v0.1.0

*Note: Update "your-org/your-repo" with actual repository URL*
