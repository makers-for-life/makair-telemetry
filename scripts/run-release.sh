#!/usr/bin/env sh
export RUST_LOG="info,telemetry=debug,makair_telemetry=debug"

cargo run --features="build-binary" --release -- "$@"
