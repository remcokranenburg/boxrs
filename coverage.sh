#!/bin/bash

export RUSTFLAGS="-Cinstrument-coverage"
export CARGO_INCREMENTAL="0"
export LLVM_PROFILE_FILE="target/coverage/boxrs-%p-%m.profraw"

rustup component add llvm-tools-preview

pushd target
curl -L https://github.com/mozilla/grcov/releases/latest/download/grcov-x86_64-unknown-linux-gnu.tar.bz2 | tar jxf -
popd

cargo build --verbose
cargo test --verbose
target/grcov . -s . --binary-path target/debug/ -t html --branch --ignore-not-existing -o target/debug/coverage/