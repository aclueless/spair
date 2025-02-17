#!/bin/bash
set -ex

TARGET="--target=wasm32-unknown-unknown"

# cargo fmt --all -- --check
# cargo clippy --all -- -D warnings

# --chrome and --firefox on separate lines to easily disable one of them if the driver has problems
# wasm-pack test --headless --chrome -- --all-features
#wasm-pack test --headless --firefox -- --all-features
wasm-pack test --headless --firefox
# wasm-pack test --headless --chrome -- --features=svg,queue-render,keyed-list

# for x in ./examples/*; do
#     if [ -f $x/Cargo.toml ]; then
#         cargo build $TARGET --manifest-path=$x/Cargo.toml
#     fi
# done
