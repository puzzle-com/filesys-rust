#!/bin/bash
set -ex

cargo run --package keystore keygen --tendermint --test-seed "alice.near" -p ~/.tendermint/config/
cargo build --package filesysmint --release
./target/release/filesysmint --devnet & tendermint node --rpc.laddr "tcp://0.0.0.0:3030"