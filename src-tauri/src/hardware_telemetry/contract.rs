use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export)]
pub enum Availability<T> {
    Available { value: T },
    NotPresent,
    Unavailable { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct TemperatureReading {
    pub key: String,
    pub label: String,
    pub celsius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct TemperatureReadings {
    pub cpu_celsius: Option<f64>,
    pub gpu_celsius: Option<f64>,
    pub sensors: Vec<TemperatureReading>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FanMode {
    SystemAuto,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct FanReading {
    pub id: usize,
    pub label: String,
    pub speed_rpm: i32,
    pub min_speed_rpm: Option<i32>,
    pub max_speed_rpm: Option<i32>,
    pub target_speed_rpm: Option<i32>,
    pub mode: Option<FanMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct BatteryReading {
    pub charge_percent: Option<i32>,
    pub temperature_celsius: Option<f64>,
    pub is_charging: Option<bool>,
    pub cycle_count: Option<i32>,
    pub power_watts: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FanActuationStatus {
    NotRegistered,
    RequiresApproval,
    Ready,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct HardwareTelemetrySnapshot {
    pub temperatures: Availability<TemperatureReadings>,
    pub fans: Availability<Vec<FanReading>>,
    pub battery: Availability<BatteryReading>,
    pub fan_actuation_status: FanActuationStatus,
    #[ts(type = "number")]
    pub captured_at_unix_ms: u64,
}

#[cfg(test)]
impl HardwareTelemetrySnapshot {
    fn fixture() -> Self {
        Self {
            temperatures: Availability::Available {
                value: TemperatureReadings {
                    cpu_celsius: Some(62.5),
                    gpu_celsius: Some(58.0),
                    sensors: vec![TemperatureReading {
                        key: "Tp01".into(),
                        label: "P-Core 1".into(),
                        celsius: 62.5,
                    }],
                },
            },
            fans: Availability::Available {
                value: vec![FanReading {
                    id: 0,
                    label: "Fan 1".into(),
                    speed_rpm: 2400,
                    min_speed_rpm: Some(1200),
                    max_speed_rpm: Some(6000),
                    target_speed_rpm: Some(2400),
                    mode: Some(FanMode::SystemAuto),
                }],
            },
            battery: Availability::Available {
                value: BatteryReading {
                    charge_percent: Some(78),
                    temperature_celsius: Some(31.2),
                    is_charging: Some(false),
                    cycle_count: Some(142),
                    power_watts: Some(18.4),
                },
            },
            fan_actuation_status: FanActuationStatus::Ready,
            captured_at_unix_ms: 1_700_000_000_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_serialization_names_units_explicitly() {
        let snapshot = HardwareTelemetrySnapshot::fixture();

        let value = serde_json::to_value(snapshot).expect("snapshot should serialize");
        assert_eq!(value["temperatures"]["value"]["cpu_celsius"], 62.5);
        assert_eq!(value["fans"]["value"][0]["speed_rpm"], 2400);
        assert_eq!(value["battery"]["value"]["charge_percent"], 78);
        assert_eq!(value["battery"]["value"]["power_watts"], 18.4);
    }

    #[test]
    fn unreadable_optional_measurements_serialize_as_null() {
        let mut snapshot = HardwareTelemetrySnapshot::fixture();
        snapshot.fans = Availability::Available {
            value: vec![FanReading {
                id: 0,
                label: "Fan 1".into(),
                speed_rpm: 2400,
                min_speed_rpm: None,
                max_speed_rpm: None,
                target_speed_rpm: None,
                mode: None,
            }],
        };
        snapshot.battery = Availability::Available {
            value: BatteryReading {
                charge_percent: None,
                temperature_celsius: None,
                is_charging: None,
                cycle_count: None,
                power_watts: None,
            },
        };

        let value = serde_json::to_value(snapshot).expect("snapshot should serialize");
        assert!(value["fans"]["value"][0]["min_speed_rpm"].is_null());
        assert!(value["fans"]["value"][0]["mode"].is_null());
        assert!(value["battery"]["value"]["charge_percent"].is_null());
        assert!(value["battery"]["value"]["is_charging"].is_null());
    }

    #[test]
    fn availability_is_explicit_for_missing_hardware_and_failed_access() {
        let fans: Availability<Vec<FanReading>> = Availability::Unavailable {
            reason: "SMC access denied".into(),
        };
        let battery: Availability<BatteryReading> = Availability::NotPresent;

        let fans_value = serde_json::to_value(fans).expect("fans should serialize");
        let battery_value = serde_json::to_value(battery).expect("battery should serialize");

        assert_eq!(fans_value["status"], "unavailable");
        assert_eq!(fans_value["reason"], "SMC access denied");
        assert_eq!(battery_value["status"], "not_present");
        assert!(battery_value.get("value").is_none());
    }
}
