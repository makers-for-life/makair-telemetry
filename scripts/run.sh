#!/usr/bin/env sh
export RUST_LOG="info,makair_telemetry=trace,makair_telemetry_cli=trace"

cargo run --features="build-binary" -- "$@"
