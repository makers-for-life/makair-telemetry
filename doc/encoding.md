# Introduction

This document is a work in progress and open for comments, please feel free to open [GitHub issues](https://github.com/makers-for-life/makair/issues) to comment.

# Goal

The goal of the present document is to describe the encoding of data between the MakAir devices and a collection point where data can be centralized and displayed.

# Problem description

The software running the MakAir devices produces telemetry data output on a serial link. These data contain technical and functional parameters. When an additional computer is installed within the MakAir device, such as a Raspberry Pi for example, it can be connected to the serial link mentioned above and read all telemetry data.

The telemetry data can be stored locally on this additional computer but it is highly suitable that the data also be transmitted to a central location where dashboards can be built for supervision personnel. In a typical hostpital, teams of a few nurses will supervize from a single room the operations of multiple MakAir devices dispatched in patient rooms.

This introduces the need for a communication channel from the MakAir devices to a centralizing computer. To limit the need to certify MakAir network use, it has been decided to rely on a unidirectional low bandwidth network based on the LoRaWAN technology. This communication channel does not require any specific deployment besides an embedded module in each MakAir device and a gateway next to the centralizing computer.

This choice of technology allows to transmit data from several 10s of MakAir ventilators. The regulations around the use of the ISM band used by LoRaWAN impose a limit on the air time that any single device can claim. In Europe, any device sending data using LoRaWAN must limit its transmission time to 1% of the time, this proportion being measured over a 1 hour period. Note that this limit does not apply when data is sent to prevent a life threatening situation, in that case the air time may be temporarily increased.

This limit in the amount of air time available to each device has a direct impact on the amount of data that can be sent by the MakAir devices.

The LoRaWAN communications between the MakAir devices and the LoRaWAN Gateway will make use of a bandwidth of 125kHz with spreading factors 7 and 8. This gives a maximum theoretical transmission bitrate of 5470 bits/s for SF7 and 3125 bits/s for SF8 both with maximum payloads of 230 bytes. Our budget for transmitting data is therefore roughly 200 bytes per packet. With added headers, a 200 bytes payload takes ~340ms to transmit using SF7/125kHz (DR7), so in order to respect the air time limit, one such packet could be transmitted every 34s.

Given this transmission budget, an encoding needs to be designed which will enable the transmission of a maximum of data from the MakAir.

# Available data

The telemetry protocol of the STM32 MCU in the MakAir exports 5 types of messages, *Boot*, *Stopped*, *DataSnapshot*, *MachineStateSnapshot* and *AlarmTrap*. The *Boot* and *Stopped* messages are emitted when the device starts and stops. The *DataSnapshot* messages are emitted at a frequency of 100Hz and contain various sensor readings. The *MachineStateSnapshot* are emitted at the end of each cycle and contain summary information about that cycle, the current setting of the MakAir and the current alarms. Lastly the *AlarmTrap* messages contain information about alarms whenever they are triggered.

The messages of utmost importance are those of type *MachineStateSnapshot* and *AlarmTrap*, they allow the medical body to supervize the use of the MakAir and react to abnormal situations, these messages should be considered **MUST HAVEs**. The other types of messages can be considered **NICE TO HAVEs**.

# Constraints

There are multiple constraints which must be taken into account in the design of the data encoding. The constraints are identified by an id of the form `DATA-XX` in the rest of this document.

## Functional constraints

The *AlarmTrap* messages **MUST** be sent as soon as possible after the alarm has been raised (`DATA-1`).

The *MachineStateSnapshot* messages should not be sent more than 40s after their cycle (`DATA-2`). The data for each cycle should be sent if possible (`DATA-3`).

## Transmission constraints

The LoRaWAN network imposes packets of less than 200 bytes (`DATA-4`) and 1% of air time (`DATA-5`). For packets of 200 bytes, the time between packets **MUST** be at least 34s to respect the 1% air time.

Smaller packets have a lower SNR given the constant size of the LoRa headers and checksum, so sending small packets, even though they can be sent more often, does not make an efficient use of the allowed air time.

## Technical constraints

The encoding should allow the sending and receiving points to be updated independently (`DATA-6`). A change in the version of software deployed on either end should in the worst case lead to the availability of partial data or no data but should not cause errors (`DATA-7`).

The encoding should also be as agnostic as possible of the target environments. While there are efforts within the MakAir team to use [Warp 10](https://warp10.io/) as the backend for storing the collected data, it is important that other organizations adopting the MakAir can make different choices. This constraint is identified as `DATA-8`.

## Security

The encoding should allow the use of optional security measures such as encryption, integrity checks and/or signature (`DATA-9`).

# Proposed solution

## Description

The proposed encoding addresses `DATA-6` by leveraging a serialization/deserialization framework with support for schema evolution. Two such frameworks were considered, [Thrift](https://thrift.apache.org/) and [Protocol Buffers](https://developers.google.com/protocol-buffers). Due to its greater popularity, Protocol Buffers (protobuf) was chosen, even though the same approach is 100% doable with Thrift. Going with the most popular technology will avoid long lasting debates and religious wars which would distract everyone from the importance of the situation.

The use of Protocol Buffers also addresses `DATA-8` as the chosen format is independent of the selected data backend.

The proposed protobuf schema is in file [`telemetry.proto`](telemetry.proto).

The associated constraint `DATA-7` is taken care of by the use of the `encoding` and `encoded` fields in the protobuf structure `MakairTelemetry`.

The data contained in the `encoded` field can be of one of the defined types in the `Encoding` enum. The type of each entry is stored in the `encoding` repeated field at the same index as the entry it describes. This mechanism allows the decoding end to skip entries which it does not know how to decode, thus addressing `DATA-7`.

The `MAKAIR_DELTA_OPTGZIP` encoding is a special encoding of the raw telemetry output by the MakAir with the following optimizations:

* The `version` and `device_id` fields are removed, the values from the `MakAirTelemetry` message are used.
* The `alarm_codes` portion is not transmitted. This can be discussed, but the alarms are sent in dedicated frames so should be received. A recap of current alarms could be sent without respecting the air time as they are there to solve a life threatening situation.
* The other fields are delta encoded between frames.
* The resulting numbers (N per frame) are then zig-zag [VARINT](https://developers.google.com/protocol-buffers/docs/encoding) encoded.
* The resulting byte array is compressed using `OPTGZIP`.

The resulting blob is then added in the `encoded` field of a new `MakAirTelemetry` message. The `encoding` field can be left empty as `MAKAIR_DELTA_OPTGZIP` is the default value.

The `OPTGZIP` compression is a regular [`gzip`](https://en.wikipedia.org/wiki/Gzip) compression with a post-processing which removes 9 or 10 of the first 10 bytes (the magic number, compression algorithm, header flags, timestamp, eXtra FLags and OS id), as those can always be forced to `1f8b080000000000XX00` for decompression (we retain `XX` as the first byte of the output if the first byte of the compressed output is `0x00`, `0x02` or `0x04` or if `XFL` is not `0x00` so we can correctly initialize the decompressor). Those 9 or 10 bytes are important to save to be able to fit as much data as possible within the limited payload budget.

The `Encoding` enum can be extended to add an `ENCRYPTED` value to be used when the content of the `encoded` field is an encrypted blob. A convention between the sending and receiving parties could then allow for secure payloads. This would address `DATA-9`, but this is not an immediate goal as encryption is also performed at the packet level via the [LoRaWAN end-to-end encryption mechanism](https://lora-alliance.org/sites/default/files/2019-05/lorawan_security_whitepaper.pdf).

## Performance evaluation

The `MAKAIR_DELTA_OPTGZIP` format leads to messages which average 7.75 to 9.5 bytes per original frame for a total of 20 frames per message. That includes the base timestamp and the `device_id`. The `version` is not transmitted.

20 frames occupying less than 200 bytes is what needs to be transmitted to transmit all the data for a `cpm_command` of 35 (the max), thus respecting `DATA-2`. For other values of `cpm_command`, the constraint is a little looser and the footprint tests seem to indicate all data can be transmitted without problem, thus respecting `DATA-3`.

In order to conform to `DATA-4` the packets will be limited to 200 bytes. After each `MachineStateSnsapshot` frame is received from the MCU, it is added to a in-memory buffer, if the number of frames in the buffer allows to respect the air time (`DATA-5`), the frames are encoded and the size of the resulting packet is checked. If that packet is over 200 bytes, the first frame of the buffer is discarded and the encoding is retried. If the packet is below 200 bytes, it is sent.

The alarms are sent as soon as they are received from the MCU in a `MakAirTelemetry` message containing a single `AlarmTrap` message, thus respecting `DATA-1`.
