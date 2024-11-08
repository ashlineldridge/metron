alias b := build
alias c := clean

default:
    @just --list

build:
    cargo build

clean:
    cargo clean

test:
    cargo test

gen:
    cargo run --example gen -- ./examples

example-local-test:
    RUST_LOG=info cargo run -- test -f examples/local-test-config.yaml
