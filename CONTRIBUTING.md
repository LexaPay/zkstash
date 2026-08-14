# Contributing to ZKStash

Thank you for your interest in contributing to ZKStash! We welcome help in making this privacy vault robust, secure, and performant.

## Code of Conduct

Please be respectful and professional in all communications and pull requests.

## Workflow

1. Fork the repository and create your branch from `main`.
2. Ensure your code compiles cleanly and passes all tests:
   ```bash
   cargo test
   ```
3. Check code styling and quality lints:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets
   ```
4. Document any new features or API additions.
5. Open a Pull Request detailing the changes and linking relevant issues.

## Testing Guidelines

Every new feature or bug fix must be accompanied by comprehensive tests in `src/test.rs`. We aim for 100% path coverage on cryptographic checks and state modifications.
