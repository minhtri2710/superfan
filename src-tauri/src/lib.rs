pub mod smc;

use smc::{get_telemetry, TelemetryData};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, Window,
};

#[derive(Default)]
pub struct AppState {
    pub demo_mode: AtomicBool,
}

#[tauri::command]
fn fetch_telemetry(state: State<Arc<AppState>>) -> TelemetryData {
    let demo = state.demo_mode.load(Ordering::Relaxed);
    get_telemetry(demo)
}

#[tauri::command]
fn set_demo_mode(enabled: bool, state: State<Arc<AppState>>) -> bool {
    state.demo_mode.store(enabled, Ordering::Relaxed);
    enabled
}

#[tauri::command]
fn set_fan_speed(fan_id: usize, rpm: i32) -> Result<String, String> {
    // Helper tool integration or SMC direct write
    Ok(format!("Set fan {} target speed to {} RPM", fan_id, rpm))
}

#[tauri::command]
fn set_fan_mode(fan_id: usize, mode: String) -> Result<String, String> {
    Ok(format!("Set fan {} mode to {}", fan_id, mode))
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
    let app_state = Arc::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            fetch_telemetry,
            set_demo_mode,
            set_fan_speed,
            set_fan_mode,
            toggle_popover
        ])
        .setup(move |app| {
            let state_clone = app_state.clone();
            let app_handle = app.handle().clone();

            // Setup Tray Icon
            let quit_i = MenuItem::with_id(app, "quit", "Quit SuperFan", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show SuperFan", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("superfan-tray")
                .tooltip("SuperFan - macOS Temperature & Fan Control")
                .icon(app.default_window_icon().unwrap().clone())
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
                .build(app)?;

            // Telemetry Background Timer Loop (1.5s interval)
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(1500));
                loop {
                    interval.tick().await;
                    let demo = state_clone.demo_mode.load(Ordering::Relaxed);
                    let data = get_telemetry(demo);

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
