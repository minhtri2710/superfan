# SuperFan Developer & Architecture Guide

Tài liệu hướng dẫn phát triển và kiến trúc chuyên sâu cho dự án **SuperFan**.

---

## 🏗️ Tổng quan Kiến trúc Hệ thống

```
+-----------------------------------------------------------------------+
|                        React 19 Frontend (Vite)                       |
|  [Header] [TemperatureGauge] [TemperatureChart] [FanRuleManager] ...  |
+-----------------------------------+-----------------------------------+
                                    | Tauri v2 IPC (Events & Commands)
+-----------------------------------v-----------------------------------+
|                        Rust Backend (Tauri v2)                        |
|  - Telemetry Loop (1.5s timer)    - IPC Command Handlers             |
|  - Smart Fan Curve Evaluator      - FFI Binding Engine (`smc/mod.rs`) |
+------------------+--------------------------------+-------------------+
                   | Direct IOKit FFI               | Command Execution
+------------------v------------------+   +---------v-------------------+
| macOS Kernel / IOKit / SMC Read     |   | Privileged Helper Tool      |
| - IOServiceMatching("AppleSMC")     |   | /usr/local/bin/smc-helper   |
| - AppleSmartBattery & PowerSources  |   | (setuid root 4755)          |
+-------------------------------------+   +-----------------------------+
```

---

## 🔑 Bảng Tra cứu SMC Keys & IOKit Telemetry

### 1. Cảm biến Nhiệt độ (Temperature Sensors)
- **Apple Silicon (M1/M2/M3/M4)**:
  - `Tp01`, `Tp05`, `Tp09`, `Tp0D`,...: CPU Performance Cores (`P-Core`).
  - `Te05`, `Te0L`, `Te0P`,...: CPU Efficiency Cores (`E-Core`).
  - `Tg05`, `Tg0D`, `Tg0L`, `Tg0T`,...: GPU Cluster Sensors (`GPU Core`).
- **Intel Macs**:
  - `TC0P`, `TCXC`, `TC0E`, `TC0D`: CPU Proximity & Core sensors.
  - `TG0P`, `TG0D`, `TGDD`: GPU Proximity sensors.

### 2. Quạt & Điều khiển Tốc độ (Fan Control Keys)
- `F0Ac`, `F1Ac`: Actual Fan Speed (RPM).
- `F0Mn`, `F1Mn`: Minimum Fan Speed (RPM).
- `F0Mx`, `F1Mx`: Maximum Fan Speed (RPM).
- `F0Tg`, `F1Tg`: Target Fan Speed (RPM).
- `F0Md`, `F1Md`: Fan Control Mode (`0` = Auto/System, `1` = Manual/Target).

### 3. Thông số Pin & Điện năng (Battery Telemetry)
- **`IOPSCopyPowerSourcesInfo` / `IOPSGetPowerSourceDescription`**: Lấy `% Pin` (`kIOPSCurrentCapacityKey`) và `Trạng thái sạc` (`kIOPSIsChargingKey`).
- **`AppleSmartBattery` (IORegistry)**:
  - `CycleCount`: Số chu kỳ sạc.
  - `Temperature`: Nhiệt độ pin (đơn vị 0.1 Kelvin $\rightarrow$ chuyển đổi $(T / 10) - 273.15 = ^\circ\text{C}$).
  - `InstantAmperage` / `Amperage`: Dòng điện (mA).
  - `Voltage`: Điện áp (mV).
  - `PowerWatts`: $P = V \times I$ (Watts).

---

## 🛠️ IPC Commands (Rust Backend Handlers)

| Command | Tham số | Mô tả |
| :--- | :--- | :--- |
| `fetch_telemetry` | `()` | Đọc toàn bộ chỉ số phần cứng hiện tại |
| `set_fan_speed` | `fan_id: usize, rpm: i32` | Đặt tốc độ quạt thủ công qua `smc-helper set <fan_id> <rpm>` |
| `set_fan_mode` | `fan_id: usize, mode: String` | Đặt chế độ quạt (`"auto"` / `"manual"`) |
| `install_helper` | `()` | Cài đặt `smc-helper` vào `/usr/local/bin/` với quyền `setuid root` |
| `toggle_popover` | `()` | Ẩn/Hiện cửa sổ ứng dụng khi click Tray Icon |
