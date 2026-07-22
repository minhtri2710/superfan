use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SUPPORTED_TELEMETRY_INTERVALS_MS: [u64; 3] = [1_000, 1_500, 2_500];

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TemperatureUnit {
    #[default]
    Celsius,
    Fahrenheit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ApplicationPreferences {
    pub temperature_unit: TemperatureUnit,
    pub telemetry_interval_ms: u64,
    pub launch_at_login: bool,
}

impl Default for ApplicationPreferences {
    fn default() -> Self {
        Self {
            temperature_unit: TemperatureUnit::Celsius,
            telemetry_interval_ms: 1_500,
            launch_at_login: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ApplicationPreferenceChange {
    SetTemperatureUnit { value: TemperatureUnit },
    SetTelemetryIntervalMs { value: u64 },
    SetLaunchAtLogin { value: bool },
}
