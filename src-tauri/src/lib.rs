pub mod fan_actuation;
pub mod hardware_telemetry;
pub mod smc;

use fan_actuation::client::{self, ActuationStatus};
use smc::{get_telemetry, TelemetryData};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Window,
};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
fn fan_actuation_status() -> ActuationStatus {
    client::status()
}

#[tauri::command]
fn fetch_telemetry() -> TelemetryData {
    let mut data = get_telemetry();
    data.fan_actuation_status = serde_json::to_value(client::status())
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unavailable".into());
    data
}

#[tauri::command]
fn is_autostart_enabled<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn toggle_autostart<R: tauri::Runtime>(app: tauri::AppHandle<R>, enable: bool) -> Result<bool, String> {
    if enable {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }
    Ok(enable)
}

#[tauri::command]
fn set_fan_speed(fan_id: usize, rpm: i32) -> Result<(), String> {
    client::set_target(fan_id, rpm)
}

#[tauri::command]
fn set_fan_mode(fan_id: usize, mode: String, rpm: Option<i32>) -> Result<(), String> {
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
fn register_fan_actuation_service() -> Result<ActuationStatus, String> {
    let service_status = fan_actuation::bootstrap::register()?;
    if service_status == fan_actuation::bootstrap::ServiceStatus::RequiresApproval {
        fan_actuation::bootstrap::open_system_settings();
        return Ok(ActuationStatus::RequiresApproval);
    }
    Ok(client::status())
}

#[tauri::command]
fn open_fan_actuation_settings() {
    fan_actuation::bootstrap::open_system_settings();
}

#[tauri::command]
fn toggle_popover(window: Window) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
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
        .invoke_handler(tauri::generate_handler![
            fan_actuation_status,
            fetch_telemetry,
            is_autostart_enabled,
            toggle_autostart,
            set_fan_speed,
            set_fan_mode,
            register_fan_actuation_service,
            open_fan_actuation_settings,
            toggle_popover
        ])
        .setup(move |_app| {
            let app_handle = _app.handle().clone();

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
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(_app)?;

            // Telemetry Background Timer Loop (1.5s interval)
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(1500));
                loop {
                    interval.tick().await;
                    let mut data = get_telemetry();
                    let actuation_status = client::status();
                    data.fan_actuation_status = serde_json::to_value(&actuation_status)
                        .ok()
                        .and_then(|value| value.as_str().map(str::to_owned))
                        .unwrap_or_else(|| "unavailable".into());

                    if actuation_status == ActuationStatus::Ready {
                        let _ = client::heartbeat();
                    }

                    // Update tray title if CPU temp is available
                    if let Some(tray) = app_handle.tray_by_id("superfan-tray") {
                        if let Some(temp) = data.cpu_temp {
                            let title = format!("🔥 {:.0}°C", temp);
                            let _ = tray.set_title(Some(title));
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
