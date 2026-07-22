# SuperFan - macOS Temperature Monitor & Fan Control 🚀

A high-performance, ultra-lightweight menu bar application for monitoring CPU/GPU temperatures, per-core thermal sensors, battery health, and controlling fan speeds on macOS.

Built with **Tauri v2, Rust, React, TypeScript, and Tailwind CSS**.

---

## ⚡ Key Features

- 🌡️ **Real-Time Hardware Telemetry**: Accurate CPU (Apple Silicon P/E Cores & Intel), GPU, and Battery metrics.
- 💻 **Per-Core Thermal Breakdown**: View individual core temperatures (`P-Core`, `E-Core`, `GPU Cluster`).
- 💨 **Smart Thermal Control**:
  - **System Auto**: Default Apple SMC thermal control.
  - **Quiet Profile**: Ultra-quiet fan acoustic curve ($50^\circ\text{C} - 85^\circ\text{C}$).
  - **Performance Profile**: Aggressive thermal dissipation ($40^\circ\text{C} - 75^\circ\text{C}$).
  - **Custom Sensor Rules**: Target specific sensors (e.g. `P-Core 1`, `GPU Core`, or `Hottest Component`) with custom temperature triggers and Min/Max Fan RPM %.
- 📈 **Real-Time Temp History Sparkline**: Ultra-smooth SVG area chart tracking temperature trends with non-scaling stroke rendering.
- ⚡ **Battery & Power Telemetry**: Real-time wattage draw (W), battery percentage, temperature, and cycle count via macOS `IOKit` (`IOPowerSources`, `AppleSmartBattery`, and `AppleSmartBatteryPack` fallback).
- 📌 **Native Menu Bar & Dockless Accessory App**: Runs as a macOS Menu Bar utility (`LSUIElement` / `ActivationPolicy::Accessory`), automatically hiding from Dock and `Cmd + Tab` switcher, positioning its popover window directly below the `🔥 SuperFan` tray icon.
- 🔒 **One-Time Privileged Helper**: Install `smc-helper` (`/usr/local/bin/smc-helper`) with `setuid root` once so no administrator password is required afterwards.
- 🎨 **Liquid Glass UI**: Premium macOS dark mode aesthetic with frameless rounded glass window, custom typography, and native window dragging.

---

## 🛠️ Architecture & Tech Stack

| Layer | Technology |
| :--- | :--- |
| **Frontend** | React 19, TypeScript, Vite, Tailwind CSS v4, Lucide Icons |
| **Backend Core** | Rust (Tauri v2), Tokio async event loop |
| **Window Positioning** | `tauri-plugin-positioner` (`Position::TrayBottomCenter`) |
| **Hardware FFI** | Apple SMC FFI (`smc.c`/`smc.h`), `IOKitLib`, `IOPowerSources`, `AppleSmartBattery`, `AppleSmartBatteryPack` |
| **Privileged Daemon** | Standalone C `smc-helper` binary installed with `setuid root (4755)` |
| **CI/CD** | GitHub Actions Universal macOS App & DMG release pipeline |

---

## 📦 Development & Building

### Requirements
- **macOS** 13.0 or later (Apple Silicon M1-M4 or Intel)
- **Node.js** v20+ & **npm**
- **Rust** 1.80+ & Xcode Command Line Tools

### 1. Run in Development Mode
```bash
# Install node dependencies
npm install

# Start Tauri development environment
npm run tauri dev
```

### 2. Build Production Application & DMG Bundle locally
```bash
# Run automated build script
./scripts/build-dmg.sh
```
The output `.dmg` package will be generated inside the `releases/` directory.

### 3. CI/CD & Automated GitHub Actions Build
The project includes a GitHub Actions pipeline ([`.github/workflows/build-macos.yml`](.github/workflows/build-macos.yml)) that automatically:
- Builds Universal macOS App (`.app` and `.dmg`) supporting Apple Silicon and Intel Macs.
- Packages and uploads release artifacts when a version tag (`v*`) is pushed.

---

## 🔒 Security & Privileges

Writing fan speed target values (`F0Tg`, `F0Md`, `FS!`) on modern macOS requires root privileges. **SuperFan** handles this securely:

1. **Temperature & Telemetry Reading**: Runs completely unprivileged in user-space via IOKit.
2. **Fan Control**: Prompt the administrator password once in **Settings -> Install Helper**. This installs `/usr/local/bin/smc-helper` owned by `root:wheel` with permission `4755` (`setuid`). Afterwards, fan adjustments execute instantly without password popups.

---

## 📄 Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Detailed developer & system architecture guide.

---

## 📜 License

Distributed under the GPL-3.0 License.
