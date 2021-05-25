# MakAir Telemetry

[![Telemetry All](https://github.com/makers-for-life/makair-telemetry/workflows/Telemetry%20All/badge.svg)](https://github.com/makers-for-life/makair-telemetry/actions?query=workflow%3A%22Telemetry+All%22) [![dependency status](https://deps.rs/repo/github/makers-for-life/makair-telemetry/status.svg)](https://deps.rs/repo/github/makers-for-life/makair-telemetry)

**ℹ️ Telemetry tools are intended to be used as a library of the Control UI runtime, or as a CLI (eg. to test the telemetry protocol with the firmware).**

## Versions

| Version | Last Changelog | Ready? |
| ------- | -------------- | ------ |
| V1.0.x | Working serial parsing from firmware | ✅
| V1.1.x | Extend telemetry protocol and add control protocol | ✅
| V1.2.x | Alarm code management has been reworked to use ENUMs (interoperability) | ✅
| V2.0.x | New lib API, protocol V2, with ventilation modes and new alarms | ✅
| V2.1.x | EOL tests, advanced errors | ❌

## Telemetry Library

This crate is a library that handles reading, parsing and serializing the MakAir's telemetry binary protocol, and optionally sending new settings values using the Makair's control binary protocol.

Crate name (for imports and `RUST_LOG`) is `makair_telemetry`.

➡ [API documentation](https://makers-for-life.github.io/makair-telemetry)

### Available Cargo features

- **rand** *(enabled by default)*: Provide standard random distribution implementations to generate control messages
- **serial** *(enabled by default)*: Enable serial support (for communicating with a MakAir)
- **serde-messages**: Provide serde implementations for telemetry and control structures (`Serialize` and `Deserialize`)
- **websocket** *(beta)*: Allow to use WebSocket as transport in addition to serial or file

## Telemetry CLI Tool

This crate also contains a CLI tool that leverages the library.

It has the following commands:

| Command | Description |
| --- | --- |
| control | Send one specific control message to a serial port, then run debug mode |
| convert | Read telemetry from a recorded file, parse it and convert it to another format (Warp10 GTS, JSON Text Sequences) |
| disable-rpi-watchdog | Send a control message to disable the RPi watchdog (until MCU is restarted) |
| debug | Read telemetry from a serial port, parse it and stream result to stdout |
| play | Read telemetry from a recorded file, parse it and stream result to stdout |
| record | Read telemetry from a serial port and save bytes to a file |
| stats | Read telemetry from a recorded file, parse it and compute some statistics |
| storm | Send a lot of control messages and/or bytes to a serial port |

You can use the scripts provided in the `scripts/` directory to run it through Cargo (you need a working Rust development environment).

To see documentation, you can run:

- `makair_telemetry_cli --help` to see a list of available commands
- `makair_telemetry_cli [COMMAND] --help` to see a list of flags and options for a given `[COMMAND]`
