#!/bin/bash
set -e
cargo fmt --check
cargo clippy --all-targets
cargo test
