#!/bin/sh

git config core.hooksPath ./.githooks
chmod +x ./.githooks/pre-commit ./.githooks/pre-push
cargo install cargo-nextest --locked