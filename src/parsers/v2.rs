use nom::branch::alt;
use nom::bytes::streaming::{tag, take};
use nom::combinator::{map, map_res};
use nom::multi::length_data;
use nom::number::streaming::{be_i16, be_u16, be_u32, be_u64, be_u8};
use nom::sequence::tuple;
use nom::IResult;
use std::convert::TryFrom;

use crate::control::*;
use crate::locale::Locale;
use crate::structures::*;

const VERSION: u8 = 2;

fn sep(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("\t")(input)
}

fn end(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("\n")(input)
}

fn mode(input: &[u8]) -> IResult<&[u8], Mode> {
    let mut parser = alt((
        map(tag(b"\x01"), |_| Mode::Production),
        map(tag(b"\x02"), |_| Mode::Qualification),
        map(tag(b"\x03"), |_| Mode::IntegrationTest),
    ));
    parser(input)
}

fn phase(input: &[u8]) -> IResult<&[u8], Phase> {
    let mut parser = alt((
        map(tag([17u8]), |_| Phase::Inhalation),
        map(tag([68u8]), |_| Phase::Exhalation),
    ));
    parser(input)
}

fn control_setting(input: &[u8]) -> IResult<&[u8], ControlSetting> {
    let mut parser = map_res(be_u8, ControlSetting::try_from);
    parser(input)
}

fn alarm_priority(input: &[u8]) -> IResult<&[u8], AlarmPriority> {
    let mut parser = alt((
        map(tag([4u8]), |_| AlarmPriority::High),
        map(tag([2u8]), |_| AlarmPriority::Medium),
        map(tag([1u8]), |_| AlarmPriority::Low),
    ));
    parser(input)
}

fn u8_array(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let mut parser = map(length_data(be_u8), Vec::from);
    parser(input)
}

fn triggered(input: &[u8]) -> IResult<&[u8], bool> {
    let mut parser = alt((map(tag([240u8]), |_| true), map(tag([15u8]), |_| false)));
    parser(input)
}

fn software_version(input: &[u8]) -> IResult<&[u8], &str> {
    let (rest, len) = be_u8(input)?;
    let mut parser = map_res(take(len), std::str::from_utf8);
    parser(rest)
}

fn device_id(input: &[u8]) -> IResult<&[u8], String> {
    let mut parser = map(tuple((be_u32, be_u32, be_u32)), |(p1, p2, p3)| {
        format!("{}-{}-{}", p1, p2, p3)
    });
    parser(input)
}

fn ventilation_mode(input: &[u8]) -> IResult<&[u8], VentilationMode> {
    let mut parser = map_res(be_u8, VentilationMode::try_from);
    parser(input)
}

fn fatal_error_details(input: &[u8]) -> IResult<&[u8], FatalErrorDetails> {
    use nom::error::{Error, ErrorKind};
    use nom::Err::Failure;
    use FatalErrorDetails::*;

    let (input, error_type) = be_u8(input)?;
    match error_type {
        1 => Ok((input, WatchdogRestart)),
        2 => {
            let mut parser = map(
                tuple((
                    sep, be_i16, sep, be_i16, sep, be_i16, sep, be_i16, sep, be_i16,
                )),
                |(
                    _,
                    pressure_offset,
                    _,
                    min_pressure,
                    _,
                    max_pressure,
                    _,
                    flow_at_starting,
                    _,
                    flow_with_blower_on,
                )| {
                    CalibrationError {
                        pressure_offset,
                        min_pressure,
                        max_pressure,
                        flow_at_starting: if flow_at_starting == i16::MAX {
                            None
                        } else {
                            Some(flow_at_starting)
                        },
                        flow_with_blower_on: if flow_with_blower_on == i16::MAX {
                            None
                        } else {
                            Some(flow_with_blower_on)
                        },
                    }
                },
            );
            parser(input)
        }
        3 => {
            let mut parser = map(tuple((sep, be_u16)), |(_, battery_level)| {
                BatteryDeeplyDischarged { battery_level }
            });
            parser(input)
        }
        4 => Ok((input, MassFlowMeterError)),
        5 => {
            let mut parser = map(tuple((sep, be_u16)), |(_, pressure)| InconsistentPressure {
                pressure,
            });
            parser(input)
        }
        _ => Err(Failure(Error::new(input, ErrorKind::Switch))),
    }
}

fn eol_test_step(input: &[u8]) -> IResult<&[u8], EolTestStep> {
    let mut parser = map_res(be_u8, EolTestStep::try_from);
    parser(input)
}

fn eol_test_snapshot_content(input: &[u8]) -> IResult<&[u8], EolTestSnapshotContent> {
    use nom::error::{Error, ErrorKind};
    use nom::Err::Failure;
    use EolTestSnapshotContent::*;

    let (input, content_type) = be_u8(input)?;
    match content_type {
        0 => {
            let mut parser = map(tuple((sep, u8_array)), |(_, message)| {
                InProgress(String::from_utf8_lossy(&message).into_owned())
            });
            parser(input)
        }
        1 => {
            let mut parser = map(tuple((sep, u8_array)), |(_, message)| {
                Error(String::from_utf8_lossy(&message).into_owned())
            });
            parser(input)
        }
        2 => {
            let mut parser = map(tuple((sep, u8_array)), |(_, message)| {
                Success(String::from_utf8_lossy(&message).into_owned())
            });
            parser(input)
        }
        _ => Err(Failure(Error::new(input, ErrorKind::Switch))),
    }
}

fn patient_gender(input: &[u8]) -> IResult<&[u8], PatientGender> {
    let mut parser = map_res(be_u8, PatientGender::try_from);
    parser(input)
}

fn boot(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tag("B:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            mode,
            sep,
            be_u8,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _, mode, _, value128, _)| {
            TelemetryMessage::BootMessage(BootMessage {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                mode,
                value128,
            })
        },
    );
    parser(input)
}

fn stopped(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tuple((
                tag("O:"),
                tag([VERSION]),
                software_version,
                device_id,
                sep,
                be_u64,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
            )),
            tuple((
                be_u8,
                sep,
                be_u8,
                sep,
                ventilation_mode,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
            )),
            tuple((
                sep, be_u8, sep, be_u8, sep, be_u16, sep, be_u16, sep, be_u16, sep, be_u16, sep,
                be_u16, sep, be_u8, sep, be_u16, sep, be_u16, sep,
            )),
            tuple((
                u8_array,
                sep,
                be_u16,
                sep,
                be_u8,
                sep,
                patient_gender,
                sep,
                be_u16,
                end,
            )),
        )),
        |(
            (
                _,
                _,
                software_version,
                device_id,
                _,
                systick,
                _,
                peak_command,
                _,
                plateau_command,
                _,
                peep_command,
                _,
                cpm_command,
                _,
                expiratory_term,
                _,
                trigger_enabled,
                _,
                trigger_offset,
                _,
            ),
            (
                alarm_snoozed,
                _,
                cpu_load,
                _,
                ventilation_mode,
                _,
                inspiratory_trigger_flow,
                _,
                expiratory_trigger_flow,
                _,
                ti_min,
                _,
                ti_max,
                _,
                low_inspiratory_minute_volume_alarm_threshold,
                _,
                high_inspiratory_minute_volume_alarm_threshold,
                _,
                low_expiratory_minute_volume_alarm_threshold,
                _,
                high_expiratory_minute_volume_alarm_threshold,
            ),
            (
                _,
                low_respiratory_rate_alarm_threshold,
                _,
                high_respiratory_rate_alarm_threshold,
                _,
                target_tidal_volume,
                _,
                low_tidal_volume_alarm_threshold,
                _,
                high_tidal_volume_alarm_threshold,
                _,
                plateau_duration,
                _,
                leak_alarm_threshold,
                _,
                target_inspiratory_flow,
                _,
                inspiratory_duration_command,
                _,
                battery_level,
                _,
            ),
            (
                current_alarm_codes,
                _,
                locale,
                _,
                patient_height,
                _,
                patient_gender,
                _,
                peak_pressure_alarm_threshold,
                _,
            ),
        )| {
            TelemetryMessage::StoppedMessage(StoppedMessage {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                peak_command: Some(peak_command),
                plateau_command: Some(plateau_command),
                peep_command: Some(peep_command),
                cpm_command: Some(cpm_command),
                expiratory_term: Some(expiratory_term),
                trigger_enabled: Some(trigger_enabled != 0),
                trigger_offset: Some(trigger_offset),
                alarm_snoozed: Some(alarm_snoozed != 0),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(
                    low_inspiratory_minute_volume_alarm_threshold,
                ),
                high_inspiratory_minute_volume_alarm_threshold: Some(
                    high_inspiratory_minute_volume_alarm_threshold,
                ),
                low_expiratory_minute_volume_alarm_threshold: Some(
                    low_expiratory_minute_volume_alarm_threshold,
                ),
                high_expiratory_minute_volume_alarm_threshold: Some(
                    high_expiratory_minute_volume_alarm_threshold,
                ),
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                battery_level: Some(battery_level),
                current_alarm_codes: Some(current_alarm_codes),
                locale: Locale::try_from_u16(locale),
                patient_height: Some(patient_height),
                patient_gender: Some(patient_gender),
                peak_pressure_alarm_threshold: Some(peak_pressure_alarm_threshold),
            })
        },
    );
    parser(input)
}

fn data_snapshot(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tuple((
                tag("D:"),
                tag([VERSION]),
                software_version,
                device_id,
                sep,
                be_u64,
                sep,
                be_u16,
                sep,
                be_i16,
                sep,
                phase,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
            )),
            tuple((be_i16, sep, be_i16, end)),
        )),
        |(
            (
                _,
                _,
                software_version,
                device_id,
                _,
                systick,
                _,
                centile,
                _,
                pressure,
                _,
                phase,
                _,
                blower_valve_position,
                _,
                patient_valve_position,
                _,
                blower_rpm,
                _,
                battery_level,
                _,
            ),
            (inspiratory_flow, _, expiratory_flow, _),
        )| {
            TelemetryMessage::DataSnapshot(DataSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                centile,
                pressure,
                phase,
                subphase: None,
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: Some(inspiratory_flow),
                expiratory_flow: Some(expiratory_flow),
            })
        },
    );
    parser(input)
}

fn machine_state_snapshot(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tuple((
                tag("S:"),
                tag([VERSION]),
                software_version,
                device_id,
                sep,
                be_u64,
                sep,
                be_u32,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
            )),
            tuple((
                be_u16,
                sep,
                u8_array,
                sep,
                be_u16,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                be_u8,
                sep,
                ventilation_mode,
                sep,
                be_u8,
            )),
            tuple((
                sep, be_u8, sep, be_u16, sep, be_u16, sep, be_u8, sep, be_u8, sep, be_u8, sep,
                be_u8, sep, be_u8, sep, be_u8, sep, be_u16, sep,
            )),
            tuple((
                be_u16,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                be_u8,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                be_u16,
                sep,
                be_u8,
                sep,
                patient_gender,
            )),
            tuple((sep, be_u16, end)),
        )),
        |(
            (
                _,
                _,
                software_version,
                device_id,
                _,
                systick,
                _,
                cycle,
                _,
                peak_command,
                _,
                plateau_command,
                _,
                peep_command,
                _,
                cpm_command,
                _,
                previous_peak_pressure,
                _,
                previous_plateau_pressure,
                _,
            ),
            (
                previous_peep_pressure,
                _,
                current_alarm_codes,
                _,
                previous_volume,
                _,
                expiratory_term,
                _,
                trigger_enabled,
                _,
                trigger_offset,
                _,
                previous_cpm,
                _,
                alarm_snoozed,
                _,
                cpu_load,
                _,
                ventilation_mode,
                _,
                inspiratory_trigger_flow,
            ),
            (
                _,
                expiratory_trigger_flow,
                _,
                ti_min,
                _,
                ti_max,
                _,
                low_inspiratory_minute_volume_alarm_threshold,
                _,
                high_inspiratory_minute_volume_alarm_threshold,
                _,
                low_expiratory_minute_volume_alarm_threshold,
                _,
                high_expiratory_minute_volume_alarm_threshold,
                _,
                low_respiratory_rate_alarm_threshold,
                _,
                high_respiratory_rate_alarm_threshold,
                _,
                target_tidal_volume,
                _,
            ),
            (
                low_tidal_volume_alarm_threshold,
                _,
                high_tidal_volume_alarm_threshold,
                _,
                plateau_duration,
                _,
                leak_alarm_threshold,
                _,
                target_inspiratory_flow,
                _,
                inspiratory_duration_command,
                _,
                previous_inspiratory_duration,
                _,
                battery_level,
                _,
                locale,
                _,
                patient_height,
                _,
                patient_gender,
            ),
            (_, peak_pressure_alarm_threshold, _),
        )| {
            TelemetryMessage::MachineStateSnapshot(MachineStateSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                cycle,
                peak_command,
                plateau_command,
                peep_command,
                cpm_command,
                previous_peak_pressure,
                previous_plateau_pressure,
                previous_peep_pressure,
                current_alarm_codes,
                previous_volume: if previous_volume == 0xFFFF {
                    None
                } else {
                    Some(previous_volume)
                },
                expiratory_term,
                trigger_enabled: trigger_enabled != 0,
                trigger_offset,
                previous_cpm: Some(previous_cpm),
                alarm_snoozed: Some(alarm_snoozed != 0),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(
                    low_inspiratory_minute_volume_alarm_threshold,
                ),
                high_inspiratory_minute_volume_alarm_threshold: Some(
                    high_inspiratory_minute_volume_alarm_threshold,
                ),
                low_expiratory_minute_volume_alarm_threshold: Some(
                    low_expiratory_minute_volume_alarm_threshold,
                ),
                high_expiratory_minute_volume_alarm_threshold: Some(
                    high_expiratory_minute_volume_alarm_threshold,
                ),
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                previous_inspiratory_duration: Some(previous_inspiratory_duration),
                battery_level: Some(battery_level),
                locale: Locale::try_from_u16(locale),
                patient_height: Some(patient_height),
                patient_gender: Some(patient_gender),
                peak_pressure_alarm_threshold: Some(peak_pressure_alarm_threshold),
            })
        },
    );
    parser(input)
}

fn alarm_trap(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tuple((
                tag("T:"),
                tag([VERSION]),
                software_version,
                device_id,
                sep,
                be_u64,
                sep,
                be_u16,
                sep,
                be_i16,
                sep,
                phase,
                sep,
                be_u32,
                sep,
                be_u8,
                sep,
                alarm_priority,
                sep,
                triggered,
                sep,
            )),
            tuple((be_u32, sep, be_u32, sep, be_u32, end)),
        )),
        |(
            (
                _,
                _,
                software_version,
                device_id,
                _,
                systick,
                _,
                centile,
                _,
                pressure,
                _,
                phase,
                _,
                cycle,
                _,
                alarm_code,
                _,
                alarm_priority,
                _,
                triggered,
                _,
            ),
            (expected, _, measured, _, cycles_since_trigger, _),
        )| {
            TelemetryMessage::AlarmTrap(AlarmTrap {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                centile,
                pressure,
                phase,
                subphase: None,
                cycle,
                alarm_code,
                alarm_priority,
                triggered,
                expected,
                measured,
                cycles_since_trigger,
            })
        },
    );
    parser(input)
}

fn control_ack(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tag("A:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            control_setting,
            sep,
            be_u16,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _, setting, _, value, _)| {
            TelemetryMessage::ControlAck(ControlAck {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                setting,
                value,
            })
        },
    );
    parser(input)
}

fn fatal_error(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tag("E:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            fatal_error_details,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _, error, _)| {
            TelemetryMessage::FatalError(FatalError {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                error,
            })
        },
    );
    parser(input)
}

fn eol_test_snapshot(input: &[u8]) -> IResult<&[u8], TelemetryMessage> {
    let mut parser = map(
        tuple((
            tag("L:"),
            tag([VERSION]),
            software_version,
            device_id,
            sep,
            be_u64,
            sep,
            eol_test_step,
            sep,
            eol_test_snapshot_content,
            end,
        )),
        |(_, _, software_version, device_id, _, systick, _, current_step, _, content, _)| {
            TelemetryMessage::EolTestSnapshot(EolTestSnapshot {
                telemetry_version: VERSION,
                version: software_version.to_owned(),
                device_id,
                systick,
                current_step,
                content,
            })
        },
    );
    parser(input)
}

/// Transform bytes into a structured telemetry message
///
/// * `input` - Bytes to parse.
///
/// This only decodes the message body: header, CRC and footer must be stripped beforehand.
pub fn message(input: &[u8]) -> IResult<&[u8], TelemetryMessage, TelemetryError<&[u8]>> {
    nom::branch::alt((
        boot,
        stopped,
        data_snapshot,
        machine_state_snapshot,
        alarm_trap,
        control_ack,
        fatal_error,
        eol_test_snapshot,
    ))(input)
    .map_err(nom::Err::convert)
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;
    use crate::serializers::ToBytes;
    use proptest::bool;
    use proptest::collection;
    use proptest::num;
    use proptest::option;
    use proptest::prelude::*;

    fn phase_strategy() -> impl Strategy<Value = Phase> {
        prop_oneof![Just(Phase::Inhalation), Just(Phase::Exhalation)]
    }

    fn alarm_priority_strategy() -> impl Strategy<Value = AlarmPriority> {
        prop_oneof![
            Just(AlarmPriority::Low),
            Just(AlarmPriority::Medium),
            Just(AlarmPriority::High),
        ]
    }

    fn control_setting_strategy() -> impl Strategy<Value = ControlSetting> {
        proptest::num::u8::ANY.prop_filter_map("Invalid control setting", |n| {
            ControlSetting::try_from(n).ok()
        })
    }

    fn ventilation_mode_strategy() -> impl Strategy<Value = VentilationMode> {
        prop_oneof![
            Just(VentilationMode::PC_CMV),
            Just(VentilationMode::PC_AC),
            Just(VentilationMode::VC_CMV),
            Just(VentilationMode::PC_VSAI),
            Just(VentilationMode::VC_AC),
        ]
    }

    fn fatal_error_details_strategy() -> BoxedStrategy<FatalErrorDetails> {
        prop_oneof![
            Just(FatalErrorDetails::WatchdogRestart),
            fatal_error_details_calibration_error_strategy(),
            fatal_error_details_battery_deeply_discharged_strategy(),
            Just(FatalErrorDetails::MassFlowMeterError),
            fatal_error_details_inconsistent_pressure_strategy(),
        ]
        .boxed()
    }

    prop_compose! {
        fn fatal_error_details_calibration_error_strategy()(
            pressure_offset in num::i16::ANY,
            min_pressure in num::i16::ANY,
            max_pressure in num::i16::ANY,
            flow_at_starting in option::of(num::i16::ANY),
            flow_with_blower_on in option::of(num::i16::ANY),
        ) -> FatalErrorDetails {
            FatalErrorDetails::CalibrationError { pressure_offset, min_pressure, max_pressure, flow_at_starting, flow_with_blower_on }
        }
    }

    prop_compose! {
        fn fatal_error_details_battery_deeply_discharged_strategy()(battery_level in num::u16::ANY) -> FatalErrorDetails {
            FatalErrorDetails::BatteryDeeplyDischarged { battery_level }
        }
    }

    prop_compose! {
        fn fatal_error_details_inconsistent_pressure_strategy()(pressure in num::u16::ANY) -> FatalErrorDetails {
            FatalErrorDetails::InconsistentPressure { pressure }
        }
    }

    fn eol_test_step_strategy() -> impl Strategy<Value = EolTestStep> {
        proptest::num::u8::ANY
            .prop_filter_map("Invalid test step", |n| EolTestStep::try_from(n).ok())
    }

    fn eol_test_snapshot_content_strategy() -> BoxedStrategy<EolTestSnapshotContent> {
        prop_oneof![
            eol_test_snapshot_content_in_progress_strategy(),
            eol_test_snapshot_content_error_strategy(),
            eol_test_snapshot_content_success_strategy(),
        ]
        .boxed()
    }

    prop_compose! {
        fn eol_test_snapshot_content_in_progress_strategy()(message in ".+") -> EolTestSnapshotContent {
            EolTestSnapshotContent::InProgress(message)
        }
    }

    prop_compose! {
        fn eol_test_snapshot_content_error_strategy()(message in ".+") -> EolTestSnapshotContent {
            EolTestSnapshotContent::Error(message)
        }
    }

    prop_compose! {
        fn eol_test_snapshot_content_success_strategy()(message in ".+") -> EolTestSnapshotContent {
            EolTestSnapshotContent::Success(message)
        }
    }

    fn patient_gender_strategy() -> impl Strategy<Value = PatientGender> {
        prop_oneof![Just(PatientGender::Male), Just(PatientGender::Female),]
    }

    proptest! {
        #[test]
        fn test_boot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            mode in mode_strategy(),
            value128 in (0u8..),
        ) {
            let msg = BootMessage {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                mode,
                value128,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::BootMessage(msg);

            assert_eq!(nom::error::dbg_dmp(boot, "boot")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_stopped_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            peak_command in (0u8..),
            plateau_command in (0u8..),
            peep_command in (0u8..),
            cpm_command in (0u8..),
            expiratory_term in (0u8..),
            trigger_enabled in bool::ANY,
            trigger_offset in (0u8..),
            alarm_snoozed in bool::ANY,
            cpu_load in num::u8::ANY,
            ventilation_mode in ventilation_mode_strategy(),
            inspiratory_trigger_flow in num::u8::ANY,
            expiratory_trigger_flow in num::u8::ANY,
            ti_min in num::u16::ANY,
            ti_max in num::u16::ANY,
            low_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_respiratory_rate_alarm_threshold in num::u8::ANY,
            high_respiratory_rate_alarm_threshold in num::u8::ANY,
            target_tidal_volume in num::u16::ANY,
            low_tidal_volume_alarm_threshold in num::u16::ANY,
            high_tidal_volume_alarm_threshold in num::u16::ANY,
            plateau_duration in num::u16::ANY,
            leak_alarm_threshold in num::u16::ANY,
            target_inspiratory_flow in num::u8::ANY,
            inspiratory_duration_command in num::u16::ANY,
            battery_level in num::u16::ANY,
            current_alarm_codes in collection::vec(0u8.., 0..100),
            patient_height in num::u8::ANY,
            patient_gender in patient_gender_strategy(),
            peak_pressure_alarm_threshold in num::u16::ANY,
        ) {
            let msg = StoppedMessage {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                peak_command: Some(peak_command),
                plateau_command: Some(plateau_command),
                peep_command: Some(peep_command),
                cpm_command: Some(cpm_command),
                expiratory_term: Some(expiratory_term),
                trigger_enabled: Some(trigger_enabled),
                trigger_offset: Some(trigger_offset),
                alarm_snoozed: Some(alarm_snoozed),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(low_inspiratory_minute_volume_alarm_threshold),
                high_inspiratory_minute_volume_alarm_threshold: Some(high_inspiratory_minute_volume_alarm_threshold),
                low_expiratory_minute_volume_alarm_threshold: Some(low_expiratory_minute_volume_alarm_threshold),
                high_expiratory_minute_volume_alarm_threshold: Some(high_expiratory_minute_volume_alarm_threshold),
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                battery_level: Some(battery_level),
                current_alarm_codes: Some(current_alarm_codes),
                locale: Some(Locale::default()),
                patient_height: Some(patient_height),
                patient_gender: Some(patient_gender),
                peak_pressure_alarm_threshold: Some(peak_pressure_alarm_threshold),
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::StoppedMessage(msg);

            assert_eq!(nom::error::dbg_dmp(stopped, "stopped")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_data_snapshot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            centile in (0u16..),
            pressure in (0i16..),
            phase in phase_strategy(),
            blower_valve_position in (0u8..),
            patient_valve_position in (0u8..),
            blower_rpm in (0u8..),
            battery_level in (0u8..),
            inspiratory_flow in num::i16::ANY,
            expiratory_flow in num::i16::ANY,
        ) {
            let msg = DataSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure,
                phase,
                subphase: None,
                blower_valve_position,
                patient_valve_position,
                blower_rpm,
                battery_level,
                inspiratory_flow: Some(inspiratory_flow),
                expiratory_flow: Some(expiratory_flow),
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::DataSnapshot(msg);

            assert_eq!(nom::error::dbg_dmp(data_snapshot, "data_snapshot")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_machine_state_snapshot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            cycle in (0u32..),
            peak_command in (0u8..),
            plateau_command in (0u8..),
            peep_command in (0u8..),
            cpm_command in (0u8..),
            previous_peak_pressure in (0u16..),
            previous_plateau_pressure in (0u16..),
            previous_peep_pressure in (0u16..),
            current_alarm_codes in collection::vec(0u8.., 0..100),
            previous_volume in option::of(0u16..0xFFFE),
            expiratory_term in (0u8..),
            trigger_enabled in bool::ANY,
            trigger_offset in (0u8..),
            previous_cpm in (0u8..),
            alarm_snoozed in bool::ANY,
            cpu_load in num::u8::ANY,
            ventilation_mode in ventilation_mode_strategy(),
            inspiratory_trigger_flow in num::u8::ANY,
            expiratory_trigger_flow in num::u8::ANY,
            ti_min in num::u16::ANY,
            ti_max in num::u16::ANY,
            low_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_inspiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            high_expiratory_minute_volume_alarm_threshold in num::u8::ANY,
            low_respiratory_rate_alarm_threshold in num::u8::ANY,
            high_respiratory_rate_alarm_threshold in num::u8::ANY,
            target_tidal_volume in num::u16::ANY,
            low_tidal_volume_alarm_threshold in num::u16::ANY,
            high_tidal_volume_alarm_threshold in num::u16::ANY,
            plateau_duration in num::u16::ANY,
            leak_alarm_threshold in num::u16::ANY,
            target_inspiratory_flow in num::u8::ANY,
            inspiratory_duration_command in num::u16::ANY,
            previous_inspiratory_duration in num::u16::ANY,
            battery_level in num::u16::ANY,
            patient_height in num::u8::ANY,
            patient_gender in patient_gender_strategy(),
            peak_pressure_alarm_threshold in num::u16::ANY,
        ) {
            let msg = MachineStateSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                cycle,
                peak_command,
                plateau_command,
                peep_command,
                cpm_command,
                previous_peak_pressure,
                previous_plateau_pressure,
                previous_peep_pressure,
                current_alarm_codes,
                previous_volume,
                expiratory_term,
                trigger_enabled,
                trigger_offset,
                previous_cpm: Some(previous_cpm),
                alarm_snoozed: Some(alarm_snoozed),
                cpu_load: Some(cpu_load),
                ventilation_mode,
                inspiratory_trigger_flow: Some(inspiratory_trigger_flow),
                expiratory_trigger_flow: Some(expiratory_trigger_flow),
                ti_min: Some(ti_min),
                ti_max: Some(ti_max),
                low_inspiratory_minute_volume_alarm_threshold: Some(low_inspiratory_minute_volume_alarm_threshold),
                high_inspiratory_minute_volume_alarm_threshold: Some(high_inspiratory_minute_volume_alarm_threshold),
                low_expiratory_minute_volume_alarm_threshold: Some(low_expiratory_minute_volume_alarm_threshold),
                high_expiratory_minute_volume_alarm_threshold: Some(high_expiratory_minute_volume_alarm_threshold),
                low_respiratory_rate_alarm_threshold: Some(low_respiratory_rate_alarm_threshold),
                high_respiratory_rate_alarm_threshold: Some(high_respiratory_rate_alarm_threshold),
                target_tidal_volume: Some(target_tidal_volume),
                low_tidal_volume_alarm_threshold: Some(low_tidal_volume_alarm_threshold),
                high_tidal_volume_alarm_threshold: Some(high_tidal_volume_alarm_threshold),
                plateau_duration: Some(plateau_duration),
                leak_alarm_threshold: Some(leak_alarm_threshold),
                target_inspiratory_flow: Some(target_inspiratory_flow),
                inspiratory_duration_command: Some(inspiratory_duration_command),
                previous_inspiratory_duration: Some(previous_inspiratory_duration),
                battery_level: Some(battery_level),
                locale: Some(Locale::default()),
                patient_height: Some(patient_height),
                patient_gender: Some(patient_gender),
                peak_pressure_alarm_threshold: Some(peak_pressure_alarm_threshold),
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::MachineStateSnapshot(msg);

            assert_eq!(nom::error::dbg_dmp(machine_state_snapshot, "machine_state_snapshot")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_alarm_trap_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            centile in (0u16..),
            pressure in (0i16..),
            phase in phase_strategy(),
            cycle in (0u32..),
            alarm_code in (0u8..),
            alarm_priority in alarm_priority_strategy(),
            triggered in proptest::bool::ANY,
            expected in (0u32..),
            measured in (0u32..),
            cycles_since_trigger in (0u32..),
        ) {
            let msg = AlarmTrap {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                centile,
                pressure,
                phase,
                subphase: None,
                cycle,
                alarm_code,
                alarm_priority,
                triggered,
                expected,
                measured,
                cycles_since_trigger,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::AlarmTrap(msg);

            assert_eq!(nom::error::dbg_dmp(alarm_trap, "alarm_trap")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_control_ack_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            setting in control_setting_strategy(),
            value in (0u16..),
        ) {
            let msg = ControlAck {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                setting,
                value,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::ControlAck(msg);

            assert_eq!(nom::error::dbg_dmp(control_ack, "control_ack")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_fatal_error_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            error in fatal_error_details_strategy(),
        ) {
            let msg = FatalError {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                error,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::FatalError(msg);

            assert_eq!(nom::error::dbg_dmp(fatal_error, "fatal_error")(input), Ok((&[][..], expected)));
        }
    }

    proptest! {
        #[test]
        fn test_eol_test_snapshot_message_parser(
            version in ".*",
            device_id1 in (0u32..),
            device_id2 in (0u32..),
            device_id3 in (0u32..),
            systick in (0u64..),
            current_step in eol_test_step_strategy(),
            content in eol_test_snapshot_content_strategy(),
        ) {
            let msg = EolTestSnapshot {
                telemetry_version: VERSION,
                version,
                device_id: format!("{}-{}-{}", device_id1, device_id2, device_id3),
                systick,
                current_step,
                content,
            };
            let input = &msg.to_bytes_v2();
            let expected = TelemetryMessage::EolTestSnapshot(msg);

            assert_eq!(nom::error::dbg_dmp(eol_test_snapshot, "eol_test_snapshot")(input), Ok((&[][..], expected)));
        }
    }
}
