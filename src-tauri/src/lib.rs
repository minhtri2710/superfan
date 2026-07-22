pub mod smc;

use smc::{get_telemetry, TelemetryData};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Window,
};
use tauri_plugin_autostart::ManagerExt;

#[derive(Default)]
pub struct AppState {
    pub auto_curve_enabled: AtomicBool,
}

#[tauri::command]
fn check_helper_status() -> bool {
    std::path::Path::new("/usr/local/bin/smc-helper").exists()
}

#[tauri::command]
fn fetch_telemetry() -> TelemetryData {
    let mut data = get_telemetry();
    data.is_helper_installed = check_helper_status();
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
fn set_fan_speed(fan_id: usize, rpm: i32) -> Result<String, String> {
    let helper_path = "/usr/local/bin/smc-helper";
    if !std::path::Path::new(helper_path).exists() {
        return Err("SMC Helper tool is not installed. Please install it from Settings.".into());
    }

    let output = Command::new(helper_path)
        .arg("set")
        .arg(fan_id.to_string())
        .arg(rpm.to_string())
        .output()
        .map_err(|e| format!("Failed to execute helper: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Helper error: {}", stderr))
    }
}

#[tauri::command]
fn set_fan_mode(fan_id: usize, mode: String) -> Result<String, String> {
    let helper_path = "/usr/local/bin/smc-helper";
    if !std::path::Path::new(helper_path).exists() {
        return Err("SMC Helper tool is not installed. Please install it from Settings.".into());
    }

    let mut cmd = Command::new(helper_path);
    if mode == "auto" {
        cmd.arg("auto").arg(fan_id.to_string());
    } else {
        cmd.arg("set").arg(fan_id.to_string()).arg("2500");
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute helper: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
fn install_helper() -> Result<String, String> {
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(_) => "/tmp".to_string(),
    };
    let built_helper = format!("{}/smc-helper", out_dir);
    
    // AppleScript command to prompt for admin password once and setuid
    let script = format!(
        "do shell script \"mkdir -p /usr/local/bin && cp '{}' /usr/local/bin/smc-helper && chown root:wheel /usr/local/bin/smc-helper && chmod 4755 /usr/local/bin/smc-helper\" with administrator privileges",
        built_helper
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Admin authentication failed: {}", e))?;

    if output.status.success() {
        Ok("SMC Helper successfully installed with root privileges.".into())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Installation failed: {}", stderr))
    }
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

// Calculate target fan RPM according to temperature curve
fn calculate_smart_curve_rpm(temp: f64, min_rpm: i32, max_rpm: i32) -> i32 {
    let min_f = min_rpm as f64;
    let max_f = max_rpm as f64;

    if temp < 45.0 {
        min_rpm
    } else if temp < 75.0 {
        let ratio = (temp - 45.0) / 30.0;
        let target = min_f + ratio * (max_f * 0.7 - min_f);
        target as i32
    } else if temp < 90.0 {
        let ratio = (temp - 75.0) / 15.0;
        let target = (max_f * 0.7) + ratio * (max_f * 0.3);
        target as i32
    } else {
        max_rpm // Emergency 100% fan speed for 90°C+
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState::default());
    app_state.auto_curve_enabled.store(true, Ordering::Relaxed);

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
            check_helper_status,
            fetch_telemetry,
            is_autostart_enabled,
            toggle_autostart,
            set_fan_speed,
            set_fan_mode,
            install_helper,
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
                    data.is_helper_installed = check_helper_status();

                    // Evaluate Smart Fan Curve for emergency over-temperature (>85°C)
                    if let Some(cpu_t) = data.cpu_temp {
                        if cpu_t > 85.0 && check_helper_status() {
                            for fan in &data.fans {
                                if fan.mode == "auto" {
                                    let target_rpm = calculate_smart_curve_rpm(cpu_t, fan.min_speed, fan.max_speed);
                                    let _ = Command::new("/usr/local/bin/smc-helper")
                                        .arg("set")
                                        .arg(fan.id.to_string())
                                        .arg(target_rpm.to_string())
                                        .output();
                                }
                            }
                        }
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
