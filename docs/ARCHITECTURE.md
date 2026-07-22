# SuperFan Developer & Architecture Guide

## System shape

```text
React UI
  └─ generated TypeScript interfaces
       │ Tauri commands and telemetry events
       ▼
Rust backend
  ├─ Application preferences module
  │    ├─ Tauri-store adapter
  │    └─ macOS autostart adapter
  ├─ Hardware telemetry snapshot module
  │    ├─ SMC adapter
  │    ├─ IOKit adapter (AppleSmartBattery + AppleSmartBatteryPack fallback)
  │    └─ fixture adapter for tests
  ├─ Windowing & Activation Policy
  │    ├─ LSUIElement / ActivationPolicy::Accessory (Dockless menu bar app)
  │    └─ tauri-plugin-positioner (Position::TrayBottomCenter)
  ├─ Thermal policy module
  │    └─ Hardware telemetry snapshot → Fan plan
  └─ Fan actuation module
       ├─ administrator-authorized installer adapter
       └─ Unix socket → privileged launchd daemon → Apple SMC
```

The Application preferences module owns display temperature unit, telemetry cadence, and launch-at-login state. Its small interface returns one authoritative snapshot and accepts one tagged change. It validates supported cadence values, persists display and cadence preferences, reconciles launch-at-login with macOS, and notifies the telemetry loop when cadence changes.

The Hardware telemetry snapshot module normalizes temperatures, fan measurements, battery measurements, and availability. Its interface uses explicit Celsius, RPM, percent, watts, and capture-time fields. Each hardware group reports `available`, `not_present`, or `unavailable`; the module does not invent display values.

The Windowing & Activation Policy module configures SuperFan as a macOS Accessory application. In `Info.plist`, `LSUIElement` is set to `true`, and Rust setup calls `set_activation_policy(tauri::ActivationPolicy::Accessory)`. The app icon is hidden from the macOS Dock and Cmd+Tab app switcher. Tray clicks and `toggle_popover` invoke `window.move_window(Position::TrayBottomCenter)` from `tauri-plugin-positioner` to position the frameless window directly underneath the menu bar icon.

The Thermal policy module owns presets and custom rules in Rust. It evaluates one Hardware telemetry snapshot at a time and produces a Fan plan. Policy settings persist through Tauri store. TypeScript interfaces are generated from the Rust contracts with `npm run generate:types`.

The Fan actuation module owns privileged writes. An administrator-authorized installer places the helper in `/Library/PrivilegedHelperTools` and its traditional plist in `/Library/LaunchDaemons`, with transactional rollback on launchd failure. The application communicates with the installed daemon through a narrow Unix socket protocol. The daemon authorizes the active console user, validates targets against hardware ranges, and restores all fans to System Auto when communication or its heartbeat lease fails.

## Thermal policy behavior

- System Auto always produces a System Auto Fan plan.
- Missing, unavailable, or more than five-second-old hardware data produces System Auto.
- Quiet uses 50–85°C and 20–75% fan output.
- Performance uses 40–75°C and 40–100% fan output.
- Custom rules can target CPU, GPU, a sensor key, or the hottest available temperature.
- When multiple rules apply, the highest requested fan target wins.
- RPM increases apply immediately.
- RPM decreases require a 2°C temperature drop and are limited to 400 RPM per second.
- Direct manual controls are disabled while Quiet, Performance, or Custom policy is active, preserving one writer for Fan actuation.

## Tauri commands

| Command | Purpose |
| --- | --- |
| `application_preferences` | Return the authoritative Application preferences snapshot. |
| `update_application_preferences` | Validate and apply one tagged preference change. |
| `fetch_telemetry` | Return the current Hardware telemetry snapshot. |
| `thermal_policy_settings` | Return persisted Thermal policy settings. |
| `select_thermal_policy_mode` | Select System Auto, Quiet, Performance, or Custom. |
| `upsert_thermal_rule` | Validate and persist a custom Thermal rule. |
| `delete_thermal_rule` | Delete and persist a custom Thermal rule. |
| `fan_actuation_status` | Report installed Fan actuation helper readiness. |
| `set_fan_speed` | Apply a direct manual RPM target while Thermal policy is in System Auto. |
| `set_fan_mode` | Select direct manual behavior or System Auto while automatic policy is inactive. |
| `install_fan_actuation_helper` | Prompt for administrator authorization and install or repair the privileged helper. |
| `toggle_popover` | Show or hide the application window, positioned at `Position::TrayBottomCenter`. |

## SMC and IOKit readings

Temperature adapters probe known Apple Silicon and Intel SMC keys. Fan measurements use `F*Ac`, `F*Mn`, `F*Mx`, `F*Tg`, and `F*Md`. Battery measurements use `IOPowerSources` and `AppleSmartBattery` properties, with fallback probing of `AppleSmartBatteryPack` (and nested `BatteryData` dictionaries). Unreadable optional measurements remain absent rather than receiving fallback values.

## Fail-safe authority

macOS remains the fail-safe authority through System Auto. Thermal policy evaluation, persistence, socket errors, stale telemetry, unavailable hardware, application exit, daemon restart, and heartbeat timeout must never leave an unowned manual Fan plan active.

## CI/CD Pipeline

The GitHub Actions workflow in `.github/workflows/build-macos.yml` builds Universal macOS binary targets (`aarch64-apple-darwin` and `x86_64-apple-darwin`), verifies fan actuation helper bundling integrity, and packages `.app.tar.gz` and `.dmg` release artifacts.
