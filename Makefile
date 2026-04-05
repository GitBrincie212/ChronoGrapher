.PHONY: fmt clippy build test ci

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

build:
	cargo build --all-features

test:
	cd tests && cargo test --all-features && cd ..

bench:
	cd benches && cargo bench && cd ..

ci:
	fmt clippy build test
