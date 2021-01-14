#!/usr/bin/env sh
export RUST_LOG="info,makair_telemetry=debug,makair_telemetry_cli=debug"

cargo run --features="build-binary" -- "$@"
