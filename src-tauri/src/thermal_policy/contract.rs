use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ThermalPolicyMode {
    #[default]
    SystemAuto,
    Quiet,
    Performance,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ThermalTarget {
    Hottest,
    Cpu,
    Gpu,
    SensorKey { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct ThermalRule {
    pub id: String,
    pub name: String,
    pub target: ThermalTarget,
    pub low_celsius: f64,
    pub high_celsius: f64,
    pub min_fan_percent: u8,
    pub max_fan_percent: u8,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, TS)]
#[ts(export)]
pub struct ThermalPolicySettings {
    pub mode: ThermalPolicyMode,
    pub rules: Vec<ThermalRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct FanTarget {
    pub fan_id: usize,
    pub rpm: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum FanPlan {
    SystemAuto,
    Targets { targets: Vec<FanTarget> },
}

pub const QUIET_RULE: ThermalRule = ThermalRule {
    id: String::new(),
    name: String::new(),
    target: ThermalTarget::Hottest,
    low_celsius: 50.0,
    high_celsius: 85.0,
    min_fan_percent: 20,
    max_fan_percent: 75,
    active: true,
};

pub const PERFORMANCE_RULE: ThermalRule = ThermalRule {
    id: String::new(),
    name: String::new(),
    target: ThermalTarget::Hottest,
    low_celsius: 40.0,
    high_celsius: 75.0,
    min_fan_percent: 40,
    max_fan_percent: 100,
    active: true,
};
