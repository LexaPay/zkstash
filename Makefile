.PHONY: all build test fmt lint clean

all: build test

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets

clean:
	cargo clean

# Makefile runtime config
DEBUG ?= false
SOROBAN_SDK_VERSION = "22.0.11"
