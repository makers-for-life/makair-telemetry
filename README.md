# MakAir Telemetry

**ℹ️ Telemetry tools are intended to be used as a library of the Control UI runtime, or as a CLI (eg. to test the telemetry protocol with the firmware).**

## Versions

| Version | Last Changelog | Ready? |
| ------- | -------------- | ------ |
| V1.0.x | Working serial parsing from firmware | ✅
| V1.1.x | Extend telemetry protocol and add control protocol | ✅
| V1.2.x | Alarm code management has been reworked to use ENUMs (interoperability) | ✅

## Telemetry Library

This crate is a library that handles reading and parsing the MakAir's telemetry binary protocol, and optionally sending new settings values using the Makair's control binary protocol.

➡ [API documentation](https://makers-for-life.github.io/makair-telemetry)

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
