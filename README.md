# ZKStash

**ZKStash** is a privacy-preserving zero-knowledge asset vault (mixer) built natively on Stellar's **Soroban** smart contract platform. It enables shielded transactions by allowing users to deposit standard Stellar assets (like USDC or XLM) into a common pool and later withdraw them to a fresh, unlinked account using Zero-Knowledge proofs (e.g., Groth16 zk-SNARKs).

---

## Architecture Overview

The system is split into three main modules:

1.  **Main Vault Contract (`src/lib.rs`)**
    *   Exposes `deposit` and `withdraw` endpoints.
    *   Tracks valid historical Merkle roots to support proof generation against slightly older states.
    *   Stores spent nullifier hashes in persistent storage to prevent double-spending of commitments.
    *   Ensures strict security parameters (e.g., positive amounts only).

2.  **Incremental Merkle Tree (`src/merkle.rs`)**
    *   Uses a storage-backed Merkle Tree of depth `20` ($2^{20}$ maximum deposit slots).
    *   Efficiently appends commitment hashes to the tree by storing only the filled subtrees (right-most path), dramatically saving on-chain storage costs.
    *   Computes hashes using Soroban's native cryptographic SHA-256 precompiles.

3.  **Zero-Knowledge Verifier (`src/verifier.rs`)**
    *   Defines the contract-level proof validation interface.
    *   Binds withdrawal parameters (recipient address, token address, amount) to the ZK proof to prevent front-running attacks.
    *   Provides a clean mock verification pathway for unit tests to ensure robust contract flow without blowing up the WebAssembly contract binary size limits (64KB).

---

## Directory Structure

```text
zkstash/
├── Cargo.toml                  # Workspace configuration
└── contracts/
    └── zkstash/
        ├── Cargo.toml          # Soroban contract dependencies
        └── src/
            ├── lib.rs          # Main contract entry point
            ├── merkle.rs       # Incremental Merkle tree
            ├── verifier.rs     # ZK-proof verification logic
            └── test.rs         # End-to-end unit tests
```

---

## Getting Started

### Prerequisites
*   [Rust & Cargo](https://www.rust-lang.org/tools/install)
*   WebAssembly target for Rust:
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
*   (Optional) [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli)

### Running Unit Tests
Execute the test suite to verify the deposit, withdrawal, and nullifier spends:
```bash
cargo test
```

### Compiling to WASM
Build the optimized WASM binary for deployment:
```bash
cargo build --target wasm32-unknown-unknown --release
```
The optimized contract binary will be generated at `target/wasm32-unknown-unknown/release/zkstash_contract.wasm`.

## Troubleshooting & Dependency Resolution

During compilation, Cargo may try to resolve transitive dependencies on `ed25519-dalek` to incompatible newer versions (like `v3.0.0`), which causes testutils compilation failures.

To lock dependencies to compatible versions (matching Soroban SDK `v22` requirements), ensure `ed25519-dalek` is pinned to version `2.2.0` in the Cargo lockfile:

```bash
cargo generate-lockfile
cargo update -p ed25519-dalek@3.0.0 --precise 2.2.0
```

## License

MIT — see [LICENSE](LICENSE) for details.
