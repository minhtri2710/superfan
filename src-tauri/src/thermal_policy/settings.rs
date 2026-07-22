use super::contract::{ThermalPolicySettings, ThermalRule, ThermalTarget};
use std::collections::HashSet;
use tauri::Runtime;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "thermal-policy.json";
const SETTINGS_KEY: &str = "settings";

pub fn validate(settings: &ThermalPolicySettings) -> Result<(), String> {
    let mut ids = HashSet::new();
    for rule in &settings.rules {
        validate_rule(rule)?;
        if !ids.insert(&rule.id) {
            return Err(format!("duplicate Thermal rule id: {}", rule.id));
        }
    }
    Ok(())
}

pub fn load<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<ThermalPolicySettings, String> {
    let store = app.store(STORE_PATH).map_err(|error| error.to_string())?;
    match store.get(SETTINGS_KEY) {
        Some(value) => {
            let settings = serde_json::from_value(value).map_err(|error| error.to_string())?;
            validate(&settings)?;
            Ok(settings)
        }
        None => Ok(ThermalPolicySettings::default()),
    }
}

pub fn save<R: Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &ThermalPolicySettings,
) -> Result<(), String> {
    validate(settings)?;
    let store = app.store(STORE_PATH).map_err(|error| error.to_string())?;
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(settings).map_err(|error| error.to_string())?,
    );
    store.save().map_err(|error| error.to_string())
}

fn validate_rule(rule: &ThermalRule) -> Result<(), String> {
    if rule.id.trim().is_empty() {
        return Err("Thermal rule id cannot be empty".into());
    }
    if rule.name.trim().is_empty() {
        return Err("Thermal rule name cannot be empty".into());
    }
    if !rule.low_celsius.is_finite()
        || !rule.high_celsius.is_finite()
        || rule.low_celsius >= rule.high_celsius
    {
        return Err("Thermal rule Celsius range is invalid".into());
    }
    if rule.min_fan_percent > 100
        || rule.max_fan_percent > 100
        || rule.min_fan_percent > rule.max_fan_percent
    {
        return Err("Thermal rule fan percent range is invalid".into());
    }
    if matches!(&rule.target, ThermalTarget::SensorKey { key } if key.trim().is_empty()) {
        return Err("Thermal rule sensor key cannot be empty".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thermal_policy::contract::{ThermalPolicyMode, ThermalTarget};

    fn rule(id: &str) -> ThermalRule {
        ThermalRule {
            id: id.into(),
            name: "CPU".into(),
            target: ThermalTarget::Cpu,
            low_celsius: 40.0,
            high_celsius: 80.0,
            min_fan_percent: 20,
            max_fan_percent: 100,
            active: true,
        }
    }

    #[test]
    fn accepts_valid_settings() {
        assert!(validate(&ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule("cpu")],
        })
        .is_ok());
    }

    #[test]
    fn rejects_invalid_ranges_duplicate_ids_and_empty_sensor_keys() {
        let mut invalid_range = rule("cpu");
        invalid_range.low_celsius = 80.0;
        invalid_range.high_celsius = 40.0;
        assert!(validate(&ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![invalid_range],
        })
        .is_err());

        assert!(validate(&ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule("cpu"), rule("cpu")],
        })
        .is_err());

        let mut empty_sensor = rule("sensor");
        empty_sensor.target = ThermalTarget::SensorKey { key: " ".into() };
        assert!(validate(&ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![empty_sensor],
        })
        .is_err());
    }
}
