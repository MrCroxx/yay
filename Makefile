SHELL := /bin/bash
.PHONY: deps check test test-ignored test-all all fast monitor clear madsim example msrv udeps

deps:
	./scripts/install-deps.sh

check:
	typos
	shellcheck ./scripts/*
	./.github/template/generate.sh
	cargo sort -w
	cargo fmt --all
	cargo clippy --all-targets

check-all:
	shellcheck ./scripts/*
	./.github/template/generate.sh
	cargo sort -w
	cargo fmt --all
	cargo clippy --all-targets

test:
	RUST_BACKTRACE=1 cargo nextest run --all
	RUST_BACKTRACE=1 cargo test --doc

test-ignored:
	RUST_BACKTRACE=1 cargo nextest run --run-ignored ignored-only --no-capture --workspace

test-all: test test-ignored

full: check-all test-all udeps

fast: check test example

udeps:
	RUSTFLAGS="--cfg tokio_unstable -Awarnings" cargo +nightly-2024-03-17 udeps --all-targets
