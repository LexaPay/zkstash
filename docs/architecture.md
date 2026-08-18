# ZKStash Architecture Design

Modular layout:
1. merkle.rs: storage-backed incremental Merkle Tree of depth 20.
2. verifier.rs: zk-SNARK verifier interface.
3. lib.rs: deposit/withdraw endpoint logic.

### Administrative Actions
Admin can pause/unpause the vault and adjust fee rates dynamically.
