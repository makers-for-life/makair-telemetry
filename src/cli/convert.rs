// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use crate::structures::*;

#[derive(Debug, PartialEq)]
pub enum Format {
    GTS,
    JSON,
}

impl std::str::FromStr for Format {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "gts" => Ok(Self::GTS),
            "json" => Ok(Self::JSON),
            _ => Err("Supported formats are: gts, json"),
        }
    }
}

pub fn telemetry_to_gts(message: &TelemetryMessage, source_label: &Option<String>) -> String {
    let mut output = vec![];
    match message {
        TelemetryMessage::BootMessage(msg) => {
            output.push(create_gts_line(
                msg.systick,
                "boot_version",
                Value::Str(&msg.version),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "boot_mode",
                Value::Str(format!("{:?}", msg.mode)),
                source_label,
            ));
        }
        TelemetryMessage::StoppedMessage(_) => {
            // Do nothing: we don't want this kind of messages
        }
        TelemetryMessage::DataSnapshot(msg) => {
            output.push(create_gts_line(
                msg.systick,
                "pressure",
                Value::Number(msg.pressure),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "blower_valve_position",
                Value::Number(msg.blower_valve_position),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "patient_valve_position",
                Value::Number(msg.patient_valve_position),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "blower_rpm",
                Value::Number(msg.blower_rpm),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "battery_level",
                Value::Number(msg.battery_level),
                source_label,
            ));
        }
        TelemetryMessage::MachineStateSnapshot(msg) => {
            output.push(create_gts_line(
                msg.systick,
                "cycle",
                Value::Number(msg.cycle),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "peak_command",
                Value::Number(msg.peak_command),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "plateau_command",
                Value::Number(msg.plateau_command),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "peep_command",
                Value::Number(msg.peep_command),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "cpm_command",
                Value::Number(msg.cpm_command),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "previous_peak_pressure",
                Value::Number(msg.previous_peak_pressure),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "previous_plateau_pressure",
                Value::Number(msg.previous_plateau_pressure),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "previous_peep_pressure",
                Value::Number(msg.previous_peep_pressure),
                source_label,
            ));
            if let Some(previous_volume) = msg.previous_volume {
                output.push(create_gts_line(
                    msg.systick,
                    "previous_volume",
                    Value::Number(previous_volume),
                    source_label,
                ));
            }
            output.push(create_gts_line(
                msg.systick,
                "expiratory_term",
                Value::Number(msg.expiratory_term),
                source_label,
            ));
            output.push(create_gts_line::<String>(
                msg.systick,
                "trigger_enabled",
                Value::Bool(msg.trigger_enabled),
                source_label,
            ));
            output.push(create_gts_line(
                msg.systick,
                "trigger_offset",
                Value::Number(msg.trigger_offset),
                source_label,
            ));
        }
        TelemetryMessage::AlarmTrap(msg) => {
            output.push(create_gts_line::<String>(
                msg.systick,
                format!("alarm_{}", msg.alarm_code).as_str(),
                Value::Bool(msg.triggered),
                source_label,
            ));
        }
        TelemetryMessage::ControlAck(_) => {
            // Do nothing: we don't want this kind of messages
        }
        TelemetryMessage::FatalError(_) => {
            // Do nothing: we don't want this kind of messages
        }
        TelemetryMessage::EolTestSnapshot(_) => {
            // Do nothing: we don't want this kind of messages
        }
    };
    output.iter().fold(String::new(), |mut acc, cur| {
        acc.push_str(cur);
        acc.push('\n');
        acc
    })
}

enum Value<N: std::string::ToString> {
    Str(N),
    Number(N),
    Bool(bool),
}

impl<N: std::string::ToString> std::fmt::Display for Value<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(val) => write!(f, "'{}'", val.to_string()),
            Self::Number(val) => write!(f, "{}", val.to_string()),
            Self::Bool(val) => write!(f, "{}", if *val { "T" } else { "F" }),
        }
    }
}

fn create_gts_line<N: std::string::ToString>(
    ts: u64,
    name: &str,
    value: Value<N>,
    source_label: &Option<String>,
) -> String {
    let labels = match source_label {
        Some(source) => format!("{{source={}}}", source),
        None => "{}".to_owned(),
    };
    format!("{}// {}{} {}", ts, name, labels, value)
}

pub fn telemetry_to_json(message: &TelemetryMessage) -> Result<String, serde_json::Error> {
    serde_json::to_string(&message).map(|mut result| {
        result.push('\n');
        result
    })
}
