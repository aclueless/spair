#!/bin/bash

set -ex

TARGET="--target=wasm32-unknown-unknown"

for x in ./examples/*; do
    if [ -f $x/Cargo.toml ]; then
        cargo build $TARGET --manifest-path=$x/Cargo.toml
    fi
done
