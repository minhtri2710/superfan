import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Header } from "./components/Header";
import { TemperatureGauge } from "./components/TemperatureGauge";
import { TemperatureChart } from "./components/TemperatureChart";
import { CoreBreakdown } from "./components/CoreBreakdown";
import { FanRuleManager } from "./components/FanRuleManager";
import { FanCard } from "./components/FanCard";
import { BatteryCard } from "./components/BatteryCard";
import { SettingsModal } from "./components/SettingsModal";
import {
  AppSettings,
  BatteryReading,
  FanReading,
  FanRule,
  HardwareTelemetrySnapshot,
  TemperatureReading,
} from "./types";
import { ShieldAlert, ShieldCheck } from "lucide-react";

function unavailableReason(
  availability: { status: "available" } | { status: "not_present" } | { status: "unavailable"; reason: string },
  notPresentMessage: string,
) {
  return availability.status === "unavailable" ? availability.reason : notPresentMessage;
}

export function App() {
  const [telemetry, setTelemetry] = useState<HardwareTelemetrySnapshot | null>(null);
  const [tempHistory, setTempHistory] = useState<
    { time: number; cpu: number; gpu: number | null }[]
  >([]);
  const [customRules, setCustomRules] = useState<FanRule[]>([]);
  const [activeTab, setActiveTab] = useState<"overview" | "dashboard" | "settings">("overview");
  const [settings, setSettings] = useState<AppSettings>({
    tempUnit: "C",
    pollingInterval: 1500,
    launchAtLogin: false,
    activePreset: "auto",
  });

  const recordSnapshot = (snapshot: HardwareTelemetrySnapshot) => {
    setTelemetry(snapshot);
    if (snapshot.temperatures.status !== "available") return;

    const temperatureReadings = snapshot.temperatures.value;
    const cpu = temperatureReadings.cpu_celsius;
    if (cpu === null) return;

    setTempHistory((previous) => [
      ...previous.slice(-40),
      {
        time: snapshot.captured_at_unix_ms,
        cpu,
        gpu: temperatureReadings.gpu_celsius,
      },
    ]);
  };

  useEffect(() => {
    invoke<HardwareTelemetrySnapshot>("fetch_telemetry")
      .then(recordSnapshot)
      .catch((error) => console.error("Hardware telemetry snapshot fetch failed:", error));

    const unlistenPromise = listen<HardwareTelemetrySnapshot>("telemetry-update", (event) => {
      recordSnapshot(event.payload);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleSetFanSpeed = async (fanId: number, rpm: number) => {
    try {
      await invoke("set_fan_speed", { fanId, rpm });
    } catch (err) {
      console.error("Set fan speed failed:", err);
    }
  };

  const handleSetFanMode = async (fanId: number, mode: "auto" | "manual", rpm?: number) => {
    try {
      await invoke("set_fan_mode", { fanId, mode, rpm });
    } catch (err) {
      console.error("Set fan mode failed:", err);
    }
  };

  const handleHideWindow = () => {
    invoke("toggle_popover").catch(() => {});
  };

  const handleSaveRule = (rule: FanRule) => {
    setCustomRules((prev) => [...prev, rule]);
  };

  const handleDeleteRule = (id: string) => {
    setCustomRules((prev) => prev.filter((rule) => rule.id !== id));
  };

  const temperatures = telemetry?.temperatures.status === "available" ? telemetry.temperatures.value : null;
  const sensors: TemperatureReading[] = temperatures?.sensors ?? [];
  const fans: FanReading[] = telemetry?.fans.status === "available" ? telemetry.fans.value : [];
  const battery: BatteryReading | null =
    telemetry?.battery.status === "available" ? telemetry.battery.value : null;
  const fanActuationStatus = telemetry?.fan_actuation_status ?? "not_registered";
  const hasSmcAccess = telemetry !== null && telemetry.temperatures.status !== "unavailable";

  return (
    <div className="w-full h-screen glass-panel flex flex-col rounded-2xl overflow-hidden border border-white/10 shadow-2xl">
      <Header
        hasAccess={hasSmcAccess}
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        onHideWindow={handleHideWindow}
      />

      <div className="flex-1 overflow-y-auto p-3.5 space-y-3">
        {activeTab === "settings" ? (
          <SettingsModal
            settings={settings}
            fanActuationStatus={fanActuationStatus}
            onUpdateSettings={(newValue) => setSettings((previous) => ({ ...previous, ...newValue }))}
          />
        ) : (
          <>
            <TemperatureGauge
              cpuTemp={temperatures?.cpu_celsius ?? null}
              gpuTemp={temperatures?.gpu_celsius ?? null}
              unit={settings.tempUnit}
            />

            {telemetry?.temperatures.status === "unavailable" && (
              <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-[10px] text-rose-200">
                <ShieldAlert className="w-3.5 h-3.5 shrink-0" />
                <span>Temperature telemetry unavailable: {telemetry.temperatures.reason}</span>
              </div>
            )}

            <TemperatureChart history={tempHistory} unit={settings.tempUnit} />

            {sensors.length > 0 && <CoreBreakdown sensors={sensors} unit={settings.tempUnit} />}

            <FanRuleManager
              activePreset={settings.activePreset}
              customRules={customRules}
              sensors={sensors}
              onSelectPreset={(preset) => setSettings((previous) => ({ ...previous, activePreset: preset }))}
              onSaveRule={handleSaveRule}
              onDeleteRule={handleDeleteRule}
            />

            <div className="space-y-2">
              <div className="flex items-center justify-between px-1">
                <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
                  Fan Speed Controls
                </span>
                <span className="text-[10px] text-cyan-400 font-mono font-semibold">
                  {fans.length} Fans Detected
                </span>
              </div>

              {fans.map((fan) => (
                <FanCard
                  key={fan.id}
                  fan={fan}
                  onSetSpeed={handleSetFanSpeed}
                  onSetMode={handleSetFanMode}
                  actuationAvailable={fanActuationStatus === "ready"}
                />
              ))}

              {telemetry && telemetry.fans.status !== "available" && (
                <div className="px-3 py-2 rounded-lg bg-slate-900/40 border border-white/5 text-[10px] text-slate-400">
                  {unavailableReason(telemetry.fans, "No hardware fans were reported.")}
                </div>
              )}
            </div>

            <BatteryCard battery={battery} unit={settings.tempUnit} />

            {telemetry && telemetry.battery.status !== "available" && (
              <div className="px-3 py-2 rounded-lg bg-slate-900/40 border border-white/5 text-[10px] text-slate-400">
                {unavailableReason(telemetry.battery, "No battery is present.")}
              </div>
            )}
          </>
        )}
      </div>

      <div className="px-3.5 py-2 border-t border-white/10 flex items-center justify-between text-[10px] text-slate-400 bg-slate-950/40">
        <span className="flex items-center gap-1">
          <ShieldCheck className={`w-3 h-3 ${hasSmcAccess ? "text-emerald-400" : "text-rose-400"}`} />
          SMC Status: {telemetry === null ? "Waiting" : hasSmcAccess ? "Available" : "Unavailable"}
        </span>
        <span className="font-mono">SuperFan v1.0.0</span>
      </div>
    </div>
  );
}

export default App;
