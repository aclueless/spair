#!/bin/bash
set -ex

TARGET="--target=wasm32-unknown-unknown"

cargo fmt -- --check
cargo clippy -- -D warnings

# --chrome and --firefox on separate lines to easily disable one of them if the drive has problems
wasm-pack test --headless --chrome -- --features="keyed-list"
#wasm-pack test --headless --firefox -- --features="keyed-list"

for x in ./examples/*; do
    if [ -f $x/Cargo.toml ]; then
        cargo build $TARGET --manifest-path=$x/Cargo.toml
    fi
done
