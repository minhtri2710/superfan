use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SMCKeyDataKeyInfo {
    pub data_size: u32,
    pub data_type: u32,
    pub data_attributes: c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SMCVal {
    pub key: [c_char; 5],
    pub data_size: u32,
    pub data_type: [c_char; 5],
    pub bytes: [c_uchar; 32],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BatteryInfoC {
    pub percentage: c_int,
    pub is_charging: c_int,
    pub cycle_count: c_int,
    pub temperature: f64,
    pub power_watts: f64,
    pub has_battery: c_int,
}

extern "C" {
    pub fn SMCOpen(conn: *mut u32) -> c_int;
    pub fn SMCClose(conn: u32) -> c_int;
    pub fn SMCReadKey(key: *const c_char, val: *mut SMCVal, conn: u32) -> c_int;
    pub fn SMCWriteKey(write_val: SMCVal, conn: u32) -> c_int;
    pub fn getFloatFromVal(val: SMCVal) -> f32;
    pub fn getFanCount(conn: u32) -> c_int;
    pub fn getFanMinSpeed(fan_num: c_int, conn: u32) -> f32;
    pub fn getFanMaxSpeed(fan_num: c_int, conn: u32) -> f32;
    pub fn setFanSpeed(fan_num: c_int, speed: c_int, conn: u32) -> c_int;
    pub fn setFanAuto(fan_num: c_int, conn: u32) -> c_int;
    pub fn fetch_battery_info(info: *mut BatteryInfoC) -> c_int;
}

static SMC_CONNECTION: Mutex<u32> = Mutex::new(0);
static LAST_GPU_TEMP: Mutex<Option<(f64, Instant)>> = Mutex::new(None);

pub fn ensure_smc_open() -> bool {
    let mut conn = SMC_CONNECTION.lock().unwrap();
    if *conn != 0 {
        return true;
    }
    let mut new_conn: u32 = 0;
    let res = unsafe { SMCOpen(&mut new_conn) };
    if res == 0 && new_conn != 0 {
        *conn = new_conn;
        true
    } else {
        false
    }
}

pub fn close_smc() {
    let mut conn = SMC_CONNECTION.lock().unwrap();
    if *conn != 0 {
        unsafe { SMCClose(*conn) };
        *conn = 0;
    }
}

pub fn with_smc_connection<T>(
    operation: impl FnOnce(u32) -> Result<T, String>,
) -> Result<T, String> {
    if !ensure_smc_open() {
        return Err("SMC access is unavailable".into());
    }
    let conn = *SMC_CONNECTION.lock().unwrap();
    operation(conn)
}

pub fn read_smc_key(key_str: &str) -> Option<f32> {
    if !ensure_smc_open() {
        return None;
    }
    let conn = *SMC_CONNECTION.lock().unwrap();
    let c_key = CString::new(key_str).ok()?;
    let mut val: SMCVal = unsafe { std::mem::zeroed() };

    let res = unsafe { SMCReadKey(c_key.as_ptr(), &mut val, conn) };
    if res == 0 {
        let fval = unsafe { getFloatFromVal(val) };
        if (0.0..65535.0).contains(&fval) {
            return Some(fval);
        }
    }
    None
}

const CPU_INTEL_KEYS: &[&str] = &[
    "TC0P", "TCXC", "TC0E", "TC0F", "TC0D", "TC1C", "TC2C", "TC3C", "TC4C",
];
const GPU_INTEL_KEYS: &[&str] = &["TGDD", "TG0P", "TG0D", "TG0E", "TG0F"];

const CPU_APPLE_SILICON_KEYS: &[&str] = &[
    "Te05", "Te0L", "Te0P", "Te0S", "Tp01", "Tp05", "Tp09", "Tp0D", "Tp0H", "Tp0L", "Tp0P", "Tp0T",
    "Tp0X", "Tp0b", "Tp0f", "Tp0j", "Tp0n", "Tp0r", "Tp0v", "Tp0z", "Tp19", "Tp1d", "Tp1f", "Tp1h",
    "Tp1n", "Tp1p", "Tp1t", "Tp1v",
];
const GPU_APPLE_SILICON_KEYS: &[&str] = &[
    "Tg05", "Tg0D", "Tg0L", "Tg0T", "Tg0V", "Tg0f", "Tg0j", "Tg1f", "Tg1j",
];

pub fn get_cpu_temperature() -> Option<f64> {
    let mut valid_temps = Vec::new();

    for key in CPU_APPLE_SILICON_KEYS.iter().chain(CPU_INTEL_KEYS.iter()) {
        if let Some(t) = read_smc_key(key) {
            if t > 15.0 && t < 125.0 {
                valid_temps.push(t as f64);
            }
        }
    }

    if valid_temps.is_empty() {
        None
    } else {
        valid_temps.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let top_count = std::cmp::min(4, valid_temps.len());
        let sum: f64 = valid_temps.iter().take(top_count).sum();
        Some((sum / top_count as f64 * 10.0).round() / 10.0)
    }
}

pub fn get_gpu_temperature() -> Option<f64> {
    let mut max_t: Option<f64> = None;
    for key in GPU_APPLE_SILICON_KEYS.iter().chain(GPU_INTEL_KEYS.iter()) {
        if let Some(t) = read_smc_key(key) {
            if t > 15.0 && t < 125.0 {
                match max_t {
                    Some(cur) => {
                        if (t as f64) > cur {
                            max_t = Some(t as f64);
                        }
                    }
                    None => max_t = Some(t as f64),
                }
            }
        }
    }

    let mut last_gpu = LAST_GPU_TEMP.lock().unwrap();
    let now = Instant::now();

    if let Some(t) = max_t {
        let rounded = (t * 10.0).round() / 10.0;
        *last_gpu = Some((rounded, now));
        Some(rounded)
    } else if let Some((cached_t, time)) = *last_gpu {
        if now.duration_since(time) < Duration::from_secs(8) {
            Some(cached_t)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_all_sensors() -> Vec<(String, String, f64)> {
    let mut sensors = Vec::new();
    let mut core_idx = 1;

    for key in CPU_APPLE_SILICON_KEYS.iter().chain(CPU_INTEL_KEYS.iter()) {
        if let Some(t) = read_smc_key(key) {
            if t > 15.0 && t < 125.0 {
                let label = if key.starts_with("Te") {
                    format!("E-Core {}", core_idx)
                } else if key.starts_with("Tp") {
                    format!("P-Core {}", core_idx)
                } else {
                    format!("CPU Core {}", core_idx)
                };
                sensors.push((key.to_string(), label, (t as f64 * 10.0).round() / 10.0));
                core_idx += 1;
            }
        }
    }
    sensors
}

#[derive(Debug, Clone)]
pub struct RawFanReading {
    pub id: usize,
    pub label: String,
    pub speed_rpm: i32,
    pub min_speed_rpm: Option<i32>,
    pub max_speed_rpm: Option<i32>,
    pub target_speed_rpm: Option<i32>,
    pub is_manual: Option<bool>,
}

pub fn get_fan_readings() -> Vec<RawFanReading> {
    let mut fans = Vec::new();
    let num_fans = read_smc_key("FNum").map(|v| v as usize).unwrap_or(0);
    let count = if num_fans > 0 { num_fans } else { 4 }; // Probe up to 4 fans if FNum is unreadable

    for i in 0..count {
        let actual_key = format!("F{}Ac", i);
        let min_key = format!("F{}Mn", i);
        let max_key = format!("F{}Mx", i);
        let target_key = format!("F{}Tg", i);
        let mode_key = format!("F{}Md", i);

        let speed = read_smc_key(&actual_key).map(|v| v as i32);
        let min_speed_rpm = read_smc_key(&min_key).map(|v| v as i32);
        let max_speed_rpm = read_smc_key(&max_key).map(|v| v as i32);
        let target_speed_rpm = read_smc_key(&target_key).map(|v| v as i32);
        let is_manual = read_smc_key(&mode_key).map(|v| (v as i32) == 1);

        if let Some(speed_rpm) = speed {
            fans.push(RawFanReading {
                id: i,
                label: if i == 0 {
                    "Fan 1 (Left)".into()
                } else if i == 1 {
                    "Fan 2 (Right)".into()
                } else {
                    format!("Fan {}", i + 1)
                },
                speed_rpm,
                min_speed_rpm,
                max_speed_rpm,
                target_speed_rpm,
                is_manual,
            });
        }
    }

    fans
}

#[derive(Debug, Clone)]
pub struct RawBatteryReading {
    pub charge_percent: Option<i32>,
    pub temperature_celsius: Option<f64>,
    pub is_charging: Option<bool>,
    pub cycle_count: Option<i32>,
    pub power_watts: Option<f64>,
}

pub fn get_battery_reading() -> Option<RawBatteryReading> {
    let mut c_info: BatteryInfoC = unsafe { std::mem::zeroed() };
    let has = unsafe { fetch_battery_info(&mut c_info) };

    if has != 0 {
        Some(RawBatteryReading {
            charge_percent: (0..=100)
                .contains(&c_info.percentage)
                .then_some(c_info.percentage),
            temperature_celsius: (c_info.temperature > 10.0 && c_info.temperature <= 80.0)
                .then_some((c_info.temperature * 10.0).round() / 10.0),
            is_charging: Some(c_info.is_charging != 0),
            cycle_count: (c_info.cycle_count >= 0).then_some(c_info.cycle_count),
            power_watts: (c_info.power_watts.abs() > 0.1).then_some(c_info.power_watts),
        })
    } else {
        None
    }
}
