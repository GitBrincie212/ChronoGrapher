.PHONY: fmt clippy build test bench ci

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

build:
	cargo build --all-features

test:
	cd tests && cargo test --all-features

bench:
	cd benches && cargo bench

ci: fmt clippy build test
