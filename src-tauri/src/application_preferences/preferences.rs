use super::contract::{
    ApplicationPreferenceChange, ApplicationPreferences, SUPPORTED_TELEMETRY_INTERVALS_MS,
};

pub trait PreferencesStore {
    fn load(&self) -> Result<Option<ApplicationPreferences>, String>;
    fn save(&mut self, preferences: &ApplicationPreferences) -> Result<(), String>;
}

pub trait AutostartAdapter {
    fn is_enabled(&self) -> Result<bool, String>;
    fn set_enabled(&mut self, enabled: bool) -> Result<(), String>;
}

pub struct ApplicationPreferencesModule<S, A> {
    store: S,
    autostart: A,
    current: ApplicationPreferences,
}

impl<S: PreferencesStore, A: AutostartAdapter> ApplicationPreferencesModule<S, A> {
    pub fn load(store: S, autostart: A) -> Result<Self, String> {
        let mut current = store.load()?.unwrap_or_default();
        validate(&current)?;
        current.launch_at_login = autostart.is_enabled().unwrap_or(false);
        Ok(Self {
            store,
            autostart,
            current,
        })
    }

    pub fn current(&self) -> ApplicationPreferences {
        self.current.clone()
    }

    pub fn update(
        &mut self,
        change: ApplicationPreferenceChange,
    ) -> Result<ApplicationPreferences, String> {
        let mut updated = self.current.clone();
        match change {
            ApplicationPreferenceChange::SetTemperatureUnit { value } => {
                updated.temperature_unit = value;
                self.store.save(&updated)?;
            }
            ApplicationPreferenceChange::SetTelemetryIntervalMs { value } => {
                updated.telemetry_interval_ms = value;
                validate(&updated)?;
                self.store.save(&updated)?;
            }
            ApplicationPreferenceChange::SetLaunchAtLogin { value } => {
                if let Err(error) = self.autostart.set_enabled(value) {
                    updated.launch_at_login = self
                        .autostart
                        .is_enabled()
                        .unwrap_or(self.current.launch_at_login);
                    self.current = updated;
                    return Err(format!(
                        "{error}; launch at login remains {}",
                        self.current.launch_at_login
                    ));
                }
                updated.launch_at_login = self.autostart.is_enabled()?;
            }
        }
        self.current = updated.clone();
        Ok(updated)
    }
}

pub fn validate(preferences: &ApplicationPreferences) -> Result<(), String> {
    if !SUPPORTED_TELEMETRY_INTERVALS_MS.contains(&preferences.telemetry_interval_ms) {
        return Err("telemetry interval must be 1000, 1500, or 2500 milliseconds".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application_preferences::contract::TemperatureUnit;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Default)]
    struct MemoryStore {
        value: Rc<RefCell<Option<ApplicationPreferences>>>,
        fail_save: bool,
    }

    impl PreferencesStore for MemoryStore {
        fn load(&self) -> Result<Option<ApplicationPreferences>, String> {
            Ok(self.value.borrow().clone())
        }

        fn save(&mut self, preferences: &ApplicationPreferences) -> Result<(), String> {
            if self.fail_save {
                return Err("save failed".into());
            }
            *self.value.borrow_mut() = Some(preferences.clone());
            Ok(())
        }
    }

    struct MemoryAutostart {
        enabled: bool,
        fail_set: bool,
    }

    impl AutostartAdapter for MemoryAutostart {
        fn is_enabled(&self) -> Result<bool, String> {
            Ok(self.enabled)
        }

        fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
            if self.fail_set {
                return Err("autostart failed".into());
            }
            self.enabled = enabled;
            Ok(())
        }
    }

    fn module(store: MemoryStore) -> ApplicationPreferencesModule<MemoryStore, MemoryAutostart> {
        ApplicationPreferencesModule::load(
            store,
            MemoryAutostart {
                enabled: false,
                fail_set: false,
            },
        )
        .unwrap()
    }

    #[test]
    fn first_launch_uses_defaults_and_actual_autostart_state() {
        let loaded = ApplicationPreferencesModule::load(
            MemoryStore::default(),
            MemoryAutostart {
                enabled: true,
                fail_set: false,
            },
        )
        .unwrap();
        assert_eq!(loaded.current().telemetry_interval_ms, 1_500);
        assert!(loaded.current().launch_at_login);
    }

    #[test]
    fn persisted_preferences_survive_reload() {
        let store = MemoryStore::default();
        let shared = store.clone();
        let mut preferences = module(store);
        preferences
            .update(ApplicationPreferenceChange::SetTemperatureUnit {
                value: TemperatureUnit::Fahrenheit,
            })
            .unwrap();
        assert_eq!(
            module(shared).current().temperature_unit,
            TemperatureUnit::Fahrenheit
        );
    }

    #[test]
    fn invalid_cadence_is_rejected_without_mutation() {
        let mut preferences = module(MemoryStore::default());
        assert!(preferences
            .update(ApplicationPreferenceChange::SetTelemetryIntervalMs { value: 999 })
            .is_err());
        assert_eq!(preferences.current().telemetry_interval_ms, 1_500);
    }

    #[test]
    fn persistence_failure_leaves_current_state_unchanged() {
        let mut preferences = module(MemoryStore {
            fail_save: true,
            ..Default::default()
        });
        assert!(preferences
            .update(ApplicationPreferenceChange::SetTemperatureUnit {
                value: TemperatureUnit::Fahrenheit,
            })
            .is_err());
        assert_eq!(
            preferences.current().temperature_unit,
            TemperatureUnit::Celsius
        );
    }

    #[test]
    fn autostart_failure_reports_actual_state() {
        let mut preferences = ApplicationPreferencesModule::load(
            MemoryStore::default(),
            MemoryAutostart {
                enabled: false,
                fail_set: true,
            },
        )
        .unwrap();
        let error = preferences
            .update(ApplicationPreferenceChange::SetLaunchAtLogin { value: true })
            .unwrap_err();
        assert!(error.contains("remains false"));
        assert!(!preferences.current().launch_at_login);
    }
}
