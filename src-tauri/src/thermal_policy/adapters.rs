use super::contract::ThermalPolicySettings;
use super::state::{FanActuation, SettingsStore};
use crate::fan_actuation::client;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "thermal-policy.json";
const SETTINGS_KEY: &str = "settings";

pub(crate) struct TauriSettingsStore<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriSettingsStore<R> {
    pub(crate) fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> SettingsStore for TauriSettingsStore<R> {
    fn load(&self) -> Result<Option<ThermalPolicySettings>, String> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| error.to_string())?;
        store
            .get(SETTINGS_KEY)
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| error.to_string())
    }

    fn save(&mut self, settings: &ThermalPolicySettings) -> Result<(), String> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| error.to_string())?;
        store.set(
            SETTINGS_KEY,
            serde_json::to_value(settings).map_err(|error| error.to_string())?,
        );
        store.save().map_err(|error| error.to_string())
    }
}

#[derive(Default)]
pub(crate) struct ProductionFanActuation;

impl FanActuation for ProductionFanActuation {
    fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String> {
        client::set_target(fan_id, rpm)
    }

    fn system_auto(&mut self, fan_id: usize) -> Result<(), String> {
        client::system_auto(fan_id)
    }

    fn restore_all(&mut self) -> Result<(), String> {
        client::restore_all()
    }

    fn heartbeat(&mut self) -> Result<(), String> {
        client::heartbeat()
    }
}
