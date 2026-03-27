# just manual: https://github.com/casey/just#readme

_default:
	just --list

# Generate code coverage report
ci-cov *args:
	#!/bin/bash -eux
	cargo llvm-cov nextest --no-report --locked
	cargo llvm-cov --no-report --doc --locked
	cargo llvm-cov report --doctests --lcov --output-path coverage.lcov --ignore-filename-regex fuzz

# =========== LOCAL COMMANDS ===========

build *args:
	cargo build {{args}} --locked

b *args:
	just build {{args}}

check *args:
    cargo check {{args}} --locked --all-features

c *args:
	just check {{args}}

example *args:
	cargo run --example {{args}}

e *args:
	just example {{args}}

msrv *args:
	cargo +1.82.0 clippy {{args}} --locked --all-features

m *args:
	just msrv {{args}}

test *args:
	#!/bin/bash -eux
	export RUST_BACKTRACE=1
	cargo nextest run {{args}} --locked --all-features
	cargo test {{args}} --doc --locked --all-features

t *args:
	just test {{args}}

clippy *args:
	cargo clippy {{args}} --locked --all-features

cov *args:
	#!/bin/bash -eux
	cargo llvm-cov nextest --no-report --locked
	cargo llvm-cov --no-report --doc --locked
	cargo llvm-cov report --doctests --html --output-dir coverage --ignore-filename-regex fuzz
