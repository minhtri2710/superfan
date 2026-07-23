pub mod application_preferences;
pub mod fan_actuation;
pub mod hardware_telemetry;
pub mod smc;
pub mod thermal_policy;

use application_preferences::adapters::{TauriAutostartAdapter, TauriPreferencesStore};
use application_preferences::contract::{ApplicationPreferenceChange, ApplicationPreferences};
use application_preferences::preferences::ApplicationPreferencesModule;
use fan_actuation::client::{self, ActuationStatus};
use hardware_telemetry::contract::{FanActuationStatus, HardwareTelemetrySnapshot};
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Window,
};
use thermal_policy::contract::{ThermalPolicyMode, ThermalPolicySettings, ThermalRule};
use thermal_policy::runtime::ThermalPolicyRuntime;

type PreferencesModule = ApplicationPreferencesModule<
    TauriPreferencesStore<tauri::Wry>,
    TauriAutostartAdapter<tauri::Wry>,
>;

struct ApplicationPreferencesState {
    preferences: Mutex<PreferencesModule>,
    telemetry_interval: tokio::sync::watch::Sender<u32>,
}

struct ThermalPolicyState {
    settings: Mutex<ThermalPolicySettings>,
}

#[tauri::command]
fn fan_actuation_status() -> ActuationStatus {
    client::status()
}

fn telemetry_snapshot() -> HardwareTelemetrySnapshot {
    let fan_actuation_status = match client::status() {
        ActuationStatus::NotRegistered => FanActuationStatus::NotRegistered,
        ActuationStatus::Ready => FanActuationStatus::Ready,
        ActuationStatus::Unavailable => FanActuationStatus::Unavailable,
    };
    let captured_at_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    hardware_telemetry::capture(fan_actuation_status, captured_at_unix_ms)
}

#[tauri::command]
fn fetch_telemetry() -> HardwareTelemetrySnapshot {
    telemetry_snapshot()
}

#[tauri::command]
fn application_preferences(
    state: tauri::State<'_, Arc<ApplicationPreferencesState>>,
) -> ApplicationPreferences {
    state.preferences.lock().unwrap().current()
}

#[tauri::command]
fn update_application_preferences(
    state: tauri::State<'_, Arc<ApplicationPreferencesState>>,
    change: ApplicationPreferenceChange,
) -> Result<ApplicationPreferences, String> {
    let mut preferences = state.preferences.lock().unwrap();
    let previous_interval_ms = preferences.current().telemetry_interval_ms;
    let updated = preferences.update(change)?;
    if updated.telemetry_interval_ms != previous_interval_ms {
        state
            .telemetry_interval
            .send_replace(updated.telemetry_interval_ms);
    }
    Ok(updated)
}

#[tauri::command]
fn thermal_policy_settings(
    state: tauri::State<'_, Arc<ThermalPolicyState>>,
) -> ThermalPolicySettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn select_thermal_policy_mode<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, Arc<ThermalPolicyState>>,
    mode: ThermalPolicyMode,
) -> Result<ThermalPolicySettings, String> {
    let settings = {
        let mut current = state.settings.lock().unwrap();
        let mut updated = current.clone();
        updated.mode = mode;
        thermal_policy::settings::save(&app, &updated)?;
        *current = updated.clone();
        updated
    };
    if settings.mode == ThermalPolicyMode::SystemAuto {
        let _ = client::restore_all();
    }
    Ok(settings)
}

#[tauri::command]
fn upsert_thermal_rule<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, Arc<ThermalPolicyState>>,
    rule: ThermalRule,
) -> Result<ThermalPolicySettings, String> {
    let mut current = state.settings.lock().unwrap();
    let mut updated = current.clone();
    if let Some(existing) = updated
        .rules
        .iter_mut()
        .find(|existing| existing.id == rule.id)
    {
        *existing = rule;
    } else {
        updated.rules.push(rule);
    }
    updated.mode = ThermalPolicyMode::Custom;
    thermal_policy::settings::save(&app, &updated)?;
    *current = updated.clone();
    Ok(updated)
}

#[tauri::command]
fn delete_thermal_rule<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, Arc<ThermalPolicyState>>,
    rule_id: String,
) -> Result<ThermalPolicySettings, String> {
    let mut current = state.settings.lock().unwrap();
    let mut updated = current.clone();
    updated.rules.retain(|rule| rule.id != rule_id);
    thermal_policy::settings::save(&app, &updated)?;
    *current = updated.clone();
    Ok(updated)
}

fn ensure_direct_actuation_allowed(
    state: &tauri::State<'_, Arc<ThermalPolicyState>>,
) -> Result<(), String> {
    if state.settings.lock().unwrap().mode == ThermalPolicyMode::SystemAuto {
        Ok(())
    } else {
        Err("direct Fan actuation is disabled while Thermal policy is active".into())
    }
}

#[tauri::command]
fn set_fan_speed(
    state: tauri::State<'_, Arc<ThermalPolicyState>>,
    fan_id: usize,
    rpm: i32,
) -> Result<(), String> {
    ensure_direct_actuation_allowed(&state)?;
    client::set_target(fan_id, rpm)
}

#[tauri::command]
fn set_fan_mode(
    state: tauri::State<'_, Arc<ThermalPolicyState>>,
    fan_id: usize,
    mode: String,
    rpm: Option<i32>,
) -> Result<(), String> {
    ensure_direct_actuation_allowed(&state)?;
    match mode.as_str() {
        "auto" => client::system_auto(fan_id),
        "manual" => client::set_target(
            fan_id,
            rpm.ok_or_else(|| "manual mode requires a target RPM".to_string())?,
        ),
        _ => Err("fan mode must be auto or manual".into()),
    }
}

#[tauri::command]
fn install_fan_actuation_helper(app: tauri::AppHandle) -> Result<ActuationStatus, String> {
    let resource_directory = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("fan-actuation");
    fan_actuation::installer::install(&resource_directory)?;
    Ok(client::status())
}

use tauri_plugin_positioner::{Position, WindowExt};

#[tauri::command]
fn toggle_popover(window: Window) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.move_window(Position::TrayBottomCenter);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            fan_actuation_status,
            fetch_telemetry,
            thermal_policy_settings,
            select_thermal_policy_mode,
            upsert_thermal_rule,
            delete_thermal_rule,
            application_preferences,
            update_application_preferences,
            set_fan_speed,
            set_fan_mode,
            install_fan_actuation_helper,
            toggle_popover
        ])
        .setup(move |_app| {
            #[cfg(target_os = "macos")]
            _app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let app_handle = _app.handle().clone();
            let preferences = ApplicationPreferencesModule::load(
                TauriPreferencesStore::new(app_handle.clone()),
                TauriAutostartAdapter::new(app_handle.clone()),
            )?;
            let (telemetry_interval, mut telemetry_interval_updates) =
                tokio::sync::watch::channel(preferences.current().telemetry_interval_ms);
            _app.manage(Arc::new(ApplicationPreferencesState {
                preferences: Mutex::new(preferences),
                telemetry_interval,
            }));

            let policy_settings = thermal_policy::settings::load(&app_handle)
                .unwrap_or_else(|_| ThermalPolicySettings::default());
            let policy_state = Arc::new(ThermalPolicyState {
                settings: Mutex::new(policy_settings),
            });
            _app.manage(policy_state.clone());

            // Setup Tray Icon (No icon set, title only for clean menu bar text)
            let quit_i = MenuItem::with_id(_app, "quit", "Quit SuperFan", true, None::<&str>)?;
            let show_i = MenuItem::with_id(_app, "show", "Show SuperFan", true, None::<&str>)?;
            let menu = Menu::with_items(_app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("superfan-tray")
                .tooltip("SuperFan - macOS Temperature & Fan Control")
                .title("🔥 SuperFan")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.move_window(Position::TrayBottomCenter);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.move_window(Position::TrayBottomCenter);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(_app)?;

            tauri::async_runtime::spawn(async move {
                let mut policy_runtime = ThermalPolicyRuntime::default();
                loop {
                    application_preferences::cadence::wait_for_next_tick(
                        &mut telemetry_interval_updates,
                    )
                    .await;
                    let data = telemetry_snapshot();
                    let settings = policy_state.settings.lock().unwrap().clone();
                    let now_unix_ms = data.captured_at_unix_ms;
                    let policy_result =
                        policy_runtime.evaluate_and_apply(&settings, &data, now_unix_ms);
                    if policy_result.is_err() {
                        let _ = policy_runtime.restore_system_auto();
                    } else if settings.mode == ThermalPolicyMode::SystemAuto
                        && data.fan_actuation_status == FanActuationStatus::Ready
                    {
                        let _ = client::heartbeat();
                    }

                    // Update tray title if CPU temp is available
                    if let Some(tray) = app_handle.tray_by_id("superfan-tray") {
                        if let hardware_telemetry::contract::Availability::Available {
                            value: temperatures,
                        } = &data.temperatures
                        {
                            if let Some(temp) = temperatures.cpu_celsius {
                                let title = format!("🔥 {:.0}°C", temp);
                                let _ = tray.set_title(Some(title));
                            }
                        }
                    }

                    // Emit event to frontend
                    let _ = app_handle.emit("telemetry-update", &data);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
