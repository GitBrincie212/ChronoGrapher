msrv := "1.92.0"

alias setup := bootstrap
alias build := build_stable
alias check := check_stable
alias utest := utest_stable
alias doctest := doctest_stable
alias test := test_stable
alias bench := benchmark_stable
alias benchmark := benchmark_stable

# Setup the development enviorement
bootstrap:
    git config core.hooksPath ./.githooks
    chmod +x ./.githooks/pre-commit ./.githooks/pre-push
    cargo install cargo-nextest --locked

# Build the project in the recent stable rust version
build_stable:
    cargo build

# Build the project in the minimum supported rust version
build_msrv:
    cargo +{{ msrv }} build

# Check if the project compiles in the recent stable rust version
check_stable:
    cargo check --all-features

# Check if the project compiles in the minimum supported rust version
check_msrv:
    cargo +{{ msrv }} check --all-features

# Run the project's unit test in the recent stable rust version
utest_stable:
    cargo nextest run

# Run the project's unit test in the minimum supported rust version
utest_msrv:
    cargo +{{ msrv }} nextest run

# Run a project's fuzzy test in the recent stable rust version
ftest_stable TARGET:
    cargo fuzz run {{ TARGET }}

# Run the project's doctests in the recent stable rust version
doctest_stable:
    cargo test --doc --all-features --workspace

# Run the project's doctests in the minimum supported rust version
doctest_msrv:
    cargo +{{ msrv }} test --doc --all-features --workspace

# Run all the project's tests in the recent stable rust version
test_stable:
    just doctest_stable utest_stable

# Run all the project's tests in the recent stable rust version
test_msrv:
    just doctest_msrv utest_msrv

# Benchmark the project in the recent stable rust version
benchmark_stable:
    cargo bench

# Benchmark the project in the minimum supported rust version
benchmark_msrv:
    cargo +{{ msrv }} bench

_lint_cargo:
    cargo clippy --workspace --exclude chronographer_bin --all-targets --all-features --locked -- -D warnings

_format_cargo:
    cargo fmt --all

# Lint the entire project
lint:
    just _lint_cargo
    pnpm -C website run lint

# Format the entire project
format:
    just _format_cargo
    pnpm -C website run format

# Lint & Format the entire project
quality:
    just _lint_cargo, _format_cargo
    pnpm -C website run biome

# Run the website in local development
website:
    cd website && pnpm run dev