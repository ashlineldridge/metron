alias b := build

default:
    @just --list

build:
    cargo build

test:
    cargo test
