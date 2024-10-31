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
