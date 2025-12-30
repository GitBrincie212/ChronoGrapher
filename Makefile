.PHONY: fmt clippy build test ci

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

build:
	cargo build --all-features

test:
	cargo test --all-features

ci: fmt clippy build test
