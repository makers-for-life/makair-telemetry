// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

/// Parsers for the telemetry protocol version 1
pub mod v1;
/// Parsers for the telemetry protocol version 2
pub mod v2;

use nom::number::streaming::be_u32;
use nom::IResult;

use super::structures::*;

const MAXIMUM_SUPPORTED_VERSION: u8 = 2;

fn header(input: &[u8]) -> IResult<&[u8], &[u8], TelemetryError<&[u8]>> {
    nom::bytes::streaming::tag(b"\x03\x0C")(input)
}

fn footer(input: &[u8]) -> IResult<&[u8], &[u8], TelemetryError<&[u8]>> {
    nom::bytes::streaming::tag(b"\x30\xC0")(input)
}

fn message(input: &[u8]) -> IResult<&[u8], TelemetryMessage, TelemetryError<&[u8]>> {
    nom::branch::alt((v2::message, v1::message))(input).map_err(nom::Err::convert)
}

fn with_input<
    I: Clone + nom::Offset + nom::Slice<nom::lib::std::ops::RangeTo<usize>>,
    O,
    E: nom::error::ParseError<I>,
    F,
>(
    parser: F,
) -> impl Fn(I) -> nom::IResult<I, (I, O), E>
where
    F: Fn(I) -> nom::IResult<I, O, E>,
{
    move |input: I| {
        let i = input.clone();
        match parser(i) {
            Ok((i, o)) => {
                let index = input.offset(&i);
                Ok((i, (input.slice(..index), o)))
            }
            Err(e) => Err(e),
        }
    }
}

/// Try to extract protocol version from message bytes
///
/// * `input` - Bytes of the message.
///
/// This requires the message header and the 3 first bytes of the message body.
/// CRC will not be checked.
pub fn protocol_version(input: &[u8]) -> IResult<&[u8], u8, TelemetryError<&[u8]>> {
    use nom::bytes::streaming::{tag, take};
    use nom::number::streaming::be_u8;
    use nom::sequence::{pair, preceded};

    let mut parser = preceded(header, preceded(pair(take(1usize), tag(":")), be_u8));
    parser(input)
}

/// Transform bytes into a structured telemetry message
///
/// * `input` - Bytes to parse.
///
/// This requires every bytes of the message, including header, CRC and footer.
/// CRC will be checked.
pub fn parse_telemetry_message(
    input: &[u8],
) -> IResult<&[u8], TelemetryMessage, TelemetryError<&[u8]>> {
    use nom::sequence::{pair, preceded, terminated};

    let mut parser = preceded(
        header,
        terminated(pair(with_input(message), be_u32), footer),
    );
    parser(input)
        .and_then(|(rest, ((msg_bytes, msg), expected_crc))| {
            let mut crc = crc32fast::Hasher::new();
            crc.update(msg_bytes);
            let computed_crc = crc.finalize();
            if expected_crc == computed_crc {
                Ok((rest, msg))
            } else {
                Err(nom::Err::Failure(TelemetryError(
                    input,
                    TelemetryErrorKind::CrcError {
                        expected: expected_crc,
                        computed: computed_crc,
                    },
                )))
            }
        })
        .or_else(|e| match e {
            nom::Err::Error(TelemetryError(
                _,
                TelemetryErrorKind::ParserError(nom::error::ErrorKind::Tag),
            )) => protocol_version(input).and_then(|(_rest, version)| {
                if version > MAXIMUM_SUPPORTED_VERSION {
                    Err(nom::Err::Failure(TelemetryError(
                        input,
                        TelemetryErrorKind::UnsupportedProtocolVersion {
                            maximum_supported: MAXIMUM_SUPPORTED_VERSION,
                            found: version,
                        },
                    )))
                } else {
                    Err(e)
                }
            }),
            _ => Err(e),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    pub fn flat(v: &[&[u8]]) -> Vec<u8> {
        v.iter().flat_map(|a| a.iter()).copied().collect()
    }

    pub fn mode_strategy() -> impl Strategy<Value = Mode> {
        prop_oneof![
            Just(Mode::Production),
            Just(Mode::Qualification),
            Just(Mode::IntegrationTest),
        ]
    }

    pub fn mode_ordinal(m: &Mode) -> u8 {
        match m {
            Mode::Production => 1,
            Mode::Qualification => 2,
            Mode::IntegrationTest => 3,
        }
    }

    proptest! {
        #[test]
        fn test_crc_check(
            random_crc in (0u32..),
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            mode in mode_strategy(),
            value128 in (0u8..),
        ) {
            let msg = BootMessage {
                telemetry_version: 1,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                mode,
                value128,
            };

            // This needs to be consistent with sendBootMessage() defined in src/software/firmware/srcs/telemetry.cpp
            let input_message = &flat(&[
                b"B:\x01",
                &[msg.version.len() as u8],
                &msg.version.as_bytes(),
                &device_id1.to_be_bytes(),
                &device_id2.to_be_bytes(),
                &device_id3.to_be_bytes(),
                b"\t",
                &msg.systick.to_be_bytes(),
                b"\t",
                &[mode_ordinal(&msg.mode)],
                b"\t",
                &[msg.value128],
                b"\n",
            ]);
            let mut crc = crc32fast::Hasher::new();
            crc.update(input_message);
            let expected_crc = crc.finalize();

            let fake_crc = if random_crc == expected_crc {
                if random_crc > 0 { random_crc - 1 } else { random_crc + 1 }
            } else {
                random_crc
            };

            let input = &flat(&[
                b"\x03\x0C",
                &input_message,
                &expected_crc.to_be_bytes(),
                b"\x30\xC0",
            ]);
            let fake_input = &flat(&[
                b"\x03\x0C",
                &input_message,
                &fake_crc.to_be_bytes(),
                b"\x30\xC0",
            ]);

            let expected = TelemetryMessage::BootMessage(msg);
            assert_eq!(nom::dbg_dmp(parse_telemetry_message, "parse_telemetry_message")(input), Ok((&[][..], expected)));
            assert_eq!(nom::dbg_dmp(parse_telemetry_message, "parse_telemetry_message")(fake_input), Err(nom::Err::Failure(TelemetryError(
                &fake_input[..],
                TelemetryErrorKind::CrcError{
                    expected: fake_crc,
                    computed: expected_crc,
                }
            ))));
        }
    }

    #[test]
    fn unsuported_protocol_version() {
        let version = MAXIMUM_SUPPORTED_VERSION + 1;
        let input = &flat(&[b"\x03\x0C", b"B:", &[version]]);
        let expected = TelemetryError(
            &input[..],
            TelemetryErrorKind::UnsupportedProtocolVersion {
                maximum_supported: MAXIMUM_SUPPORTED_VERSION,
                found: version,
            },
        );
        assert_eq!(
            nom::dbg_dmp(parse_telemetry_message, "parse_telemetry_message")(input),
            Err(nom::Err::Failure(expected))
        );
    }
}
