set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

build:
    cargo build

release:
    cargo build --release

run *ARGS:
    cargo run -- {{ARGS}}

run-release *ARGS:
    cargo run --release -- {{ARGS}}

clean:
    cargo clean

test:
    cargo test

test-verbose:
    cargo test -- --nocapture

test-one NAME:
    cargo test {{NAME}} -- --nocapture

test-all:
    cargo test --all-features

lint:
    cargo clippy --all-targets --all-features

lint-strict:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

check: fmt-check lint test

fix:
    cargo fmt
    cargo clippy --fix --allow-dirty --allow-staged

doc:
    cargo doc --no-deps

doc-open:
    cargo doc --no-deps --open

bench:
    cargo run --release -- --bench

size:
    cargo build --release
    @ls -lh target/release/bukvar* 2>/dev/null || dir target\release\bukvar.exe

ci: fmt-check lint-strict test-all
    @echo "All CI checks passed"

prepare-release: ci doc
    cargo build --release

example-json:
    cargo run --release -- ./examples ./output -f json --pretty --verbose

example-dast:
    cargo run --release -- ./examples ./output -f dast --verbose

pre-commit:
    @just fmt-check
    @just lint-strict

hooks-install:
    git config core.hooksPath hooks

hooks-uninstall:
    git config --unset core.hooksPath

install:
    cargo install --path .

uninstall:
    cargo uninstall bukvar

loc:
    @find src -name "*.rs" -exec cat {} + | wc -l 2>/dev/null || powershell -Command "(Get-ChildItem -Recurse -Filter *.rs src | Get-Content | Measure-Object -Line).Lines"

update:
    cargo update
