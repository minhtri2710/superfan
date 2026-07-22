export interface SensorReading {
  key: string;
  label: string;
  value: number;
}

export interface FanReading {
  id: number;
  label: string;
  speed: number;
  min_speed: number;
  max_speed: number;
  target_speed?: number;
  mode: "auto" | "manual";
}

export interface BatteryReading {
  percentage: number;
  temperature: number;
  is_charging: boolean;
  cycle_count: number;
  power_watts: number;
}

export interface TelemetryData {
  cpu_temp: number | null;
  gpu_temp: number | null;
  max_cpu_temp: number | null;
  sensors: SensorReading[];
  fans: FanReading[];
  battery: BatteryReading | null;
  has_smc_access: boolean;
  is_helper_installed: boolean;
  timestamp: number;
}

export interface AppSettings {
  tempUnit: "C" | "F";
  pollingInterval: number; // in ms
  launchAtLogin: boolean;
}
