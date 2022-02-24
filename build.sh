#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package ic-cron-recurrent-payments-example && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/ic_cron_recurrent_payments_example.wasm -o ./target/wasm32-unknown-unknown/release/ic-cron-recurrent-payments-example-opt.wasm
