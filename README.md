# MakAir Telemetry

## Versions

| Version | Last Changelog | Ready? |
| ------- | -------------- | ------ |
| V1.0.0 | Working serial parsing from firmware | ✅

## Telemetry Library

This crate is a library that handles reading and parsing the MakAir's telemetry binary protocol.

➡ [API documentation](https://makers-for-life.github.io/makair-telemetry)

## Telemetry CLI Tool

This crate also contains a CLI tool that leverages the library.

It has the following commands:

| Command | Description |
| --- | --- |
| debug | Reads telemetry from a serial port, parses it and streams result to stdout |
| play | Reads telemetry from a recorded file, parses it and streams result to stdout |
| record | Reads telemetry from a serial port and save bytes to a file |
| stats | Reads telemetry from a recorded file, parses it and compute some statistics |

You can use the scripts provided in the `scripts/` directory to run it through Cargo (you need a working Rust development environment).
