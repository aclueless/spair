#!/bin/bash
set -ex

TARGET="--target=wasm32-unknown-unknown"

cargo fmt --all -- --check
cargo clippy --all -- -D warnings

cd spair
# --chrome and --firefox on separate lines to easily disable one of them if the driver has problems
# wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
cd ..

for x in ./examples/*; do
    if [ -f $x/Cargo.toml ]; then
        cargo build $TARGET --manifest-path=$x/Cargo.toml
    fi
done
