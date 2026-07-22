export type { Availability } from "./generated/Availability";
export type { BatteryReading } from "./generated/BatteryReading";
export type { FanActuationStatus } from "./generated/FanActuationStatus";
export type { FanMode } from "./generated/FanMode";
export type { FanReading } from "./generated/FanReading";
export type { HardwareTelemetrySnapshot } from "./generated/HardwareTelemetrySnapshot";
export type { TemperatureReading } from "./generated/TemperatureReading";
export type { TemperatureReadings } from "./generated/TemperatureReadings";

export interface FanRule {
  id: string;
  name: string;
  targetSensor: "hottest" | "cpu" | "gpu" | string;
  lowTemp: number;
  highTemp: number;
  minFanPercent: number;
  maxFanPercent: number;
  active: boolean;
}

export interface AppSettings {
  tempUnit: "C" | "F";
  pollingInterval: number; // in ms
  launchAtLogin: boolean;
  activePreset: "auto" | "quiet" | "performance" | "custom";
}
