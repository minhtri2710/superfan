export type { Availability } from "./generated/Availability";
export type { BatteryReading } from "./generated/BatteryReading";
export type { FanActuationStatus } from "./generated/FanActuationStatus";
export type { FanMode } from "./generated/FanMode";
export type { FanReading } from "./generated/FanReading";
export type { HardwareTelemetrySnapshot } from "./generated/HardwareTelemetrySnapshot";
export type { TemperatureReading } from "./generated/TemperatureReading";
export type { TemperatureReadings } from "./generated/TemperatureReadings";
export type { ThermalPolicyMode } from "./generated/ThermalPolicyMode";
export type { ThermalPolicySettings } from "./generated/ThermalPolicySettings";
export type { ThermalRule } from "./generated/ThermalRule";
export type { ThermalTarget } from "./generated/ThermalTarget";

export interface AppSettings {
  tempUnit: "C" | "F";
  pollingInterval: number; // in ms
  launchAtLogin: boolean;
}
