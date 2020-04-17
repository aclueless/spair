#!/bin/bash
set -ex

TARGET="--target=wasm32-unknown-unknown"

cargo fmt -- --check
cargo clippy
wasm-pack test --headless --chrome --firefox -- --features="keyed-list"

for x in ./examples/*; do
    if [ -f $x/Cargo.toml ]; then
        cargo build $TARGET --manifest-path=$x/Cargo.toml
    fi
done
