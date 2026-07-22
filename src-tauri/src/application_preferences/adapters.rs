use super::contract::ApplicationPreferences;
use super::preferences::{AutostartAdapter, PreferencesStore};
use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "application-preferences.json";
const PREFERENCES_KEY: &str = "preferences";

pub struct TauriPreferencesStore<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriPreferencesStore<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> PreferencesStore for TauriPreferencesStore<R> {
    fn load(&self) -> Result<Option<ApplicationPreferences>, String> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| error.to_string())?;
        store
            .get(PREFERENCES_KEY)
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| error.to_string())
    }

    fn save(&mut self, preferences: &ApplicationPreferences) -> Result<(), String> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| error.to_string())?;
        store.set(
            PREFERENCES_KEY,
            serde_json::to_value(preferences).map_err(|error| error.to_string())?,
        );
        store.save().map_err(|error| error.to_string())
    }
}

pub struct TauriAutostartAdapter<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriAutostartAdapter<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> AutostartAdapter for TauriAutostartAdapter<R> {
    fn is_enabled(&self) -> Result<bool, String> {
        self.app
            .autolaunch()
            .is_enabled()
            .map_err(|error| error.to_string())
    }

    fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
        if enabled {
            self.app.autolaunch().enable()
        } else {
            self.app.autolaunch().disable()
        }
        .map_err(|error| error.to_string())
    }
}
