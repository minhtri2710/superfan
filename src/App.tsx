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
import { UpdateModal } from "./components/UpdateModal";
import { checkForUpdates, ReleaseInfo } from "./services/updater";
import {
  ApplicationPreferenceChange,
  ApplicationPreferences,
  BatteryReading,
  FanReading,
  HardwareTelemetrySnapshot,
  TemperatureReading,
  ThermalPolicyMode,
  ThermalPolicySettings,
  ThermalRule,
} from "./types";
import { ShieldAlert, ShieldCheck, Sparkles } from "lucide-react";

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
  const [thermalPolicy, setThermalPolicy] = useState<ThermalPolicySettings>({
    mode: "system_auto",
    rules: [],
  });
  const [activeTab, setActiveTab] = useState<"overview" | "settings">("overview");
  const [policyError, setPolicyError] = useState<string | null>(null);
  const [preferences, setPreferences] = useState<ApplicationPreferences>({
    temperature_unit: "celsius",
    telemetry_interval_ms: 1500,
    launch_at_login: false,
  });
  const [preferencesError, setPreferencesError] = useState<string | null>(null);

  // Software Update States
  const [updateRelease, setUpdateRelease] = useState<ReleaseInfo | null>(null);
  const [hasUpdateAvailable, setHasUpdateAvailable] = useState<boolean>(false);
  const [showUpdateModal, setShowUpdateModal] = useState<boolean>(false);

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
    invoke<ApplicationPreferences>("application_preferences")
      .then(setPreferences)
      .catch((error) => console.error("Application preferences fetch failed:", error));
    invoke<ThermalPolicySettings>("thermal_policy_settings")
      .then(setThermalPolicy)
      .catch((error) => console.error("Thermal policy settings fetch failed:", error));

    // Check for software updates on launch
    checkForUpdates()
      .then((res) => {
        if (res.hasUpdate && res.latestRelease) {
          setHasUpdateAvailable(true);
          setUpdateRelease(res.latestRelease);
        }
      })
      .catch(() => {});

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

  const handleUpdatePreferences = async (change: ApplicationPreferenceChange) => {
    try {
      const updated = await invoke<ApplicationPreferences>("update_application_preferences", {
        change,
      });
      setPreferences(updated);
      setPreferencesError(null);
    } catch (error) {
      setPreferencesError(String(error));
      invoke<ApplicationPreferences>("application_preferences")
        .then(setPreferences)
        .catch(() => {});
      console.error("Application preferences update failed:", error);
    }
  };

  const handleSelectPolicyMode = async (mode: ThermalPolicyMode) => {
    try {
      const updated = await invoke<ThermalPolicySettings>("select_thermal_policy_mode", { mode });
      setThermalPolicy(updated);
      setPolicyError(null);
    } catch (error) {
      setPolicyError(String(error));
      console.error("Select Thermal policy mode failed:", error);
    }
  };

  const handleSaveRule = async (rule: ThermalRule) => {
    try {
      const updated = await invoke<ThermalPolicySettings>("upsert_thermal_rule", { rule });
      setThermalPolicy(updated);
      setPolicyError(null);
    } catch (error) {
      setPolicyError(String(error));
      console.error("Save Thermal rule failed:", error);
    }
  };

  const handleDeleteRule = async (ruleId: string) => {
    try {
      const updated = await invoke<ThermalPolicySettings>("delete_thermal_rule", { ruleId });
      setThermalPolicy(updated);
      setPolicyError(null);
    } catch (error) {
      setPolicyError(String(error));
      console.error("Delete Thermal rule failed:", error);
    }
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
          <>
            {preferencesError && (
              <div className="px-3 py-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-[10px] text-rose-200">
                Application preferences update failed: {preferencesError}
              </div>
            )}
            <SettingsModal
              preferences={preferences}
              fanActuationStatus={fanActuationStatus}
              onUpdatePreferences={handleUpdatePreferences}
              onShowUpdateModal={(release) => {
                setUpdateRelease(release);
                setShowUpdateModal(true);
              }}
            />
          </>
        ) : (
          <>
            <TemperatureGauge
              cpuTemp={temperatures?.cpu_celsius ?? null}
              gpuTemp={temperatures?.gpu_celsius ?? null}
              unit={preferences.temperature_unit === "celsius" ? "C" : "F"}
            />

            {telemetry?.temperatures.status === "unavailable" && (
              <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-[10px] text-rose-200">
                <ShieldAlert className="w-3.5 h-3.5 shrink-0" />
                <span>Temperature telemetry unavailable: {telemetry.temperatures.reason}</span>
              </div>
            )}

            <TemperatureChart
              history={tempHistory}
              unit={preferences.temperature_unit === "celsius" ? "C" : "F"}
            />

            {sensors.length > 0 && (
              <CoreBreakdown
                sensors={sensors}
                unit={preferences.temperature_unit === "celsius" ? "C" : "F"}
              />
            )}

            {policyError && (
              <div className="px-3 py-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-[10px] text-rose-200">
                Thermal policy update failed: {policyError}
              </div>
            )}

            <FanRuleManager
              activePreset={thermalPolicy.mode}
              customRules={thermalPolicy.rules}
              sensors={sensors}
              onSelectPreset={handleSelectPolicyMode}
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
                  actuationAvailable={
                    fanActuationStatus === "ready" && thermalPolicy.mode === "system_auto"
                  }
                />
              ))}

              {telemetry && telemetry.fans.status !== "available" && (
                <div className="px-3 py-2 rounded-lg bg-slate-900/40 border border-white/5 text-[10px] text-slate-400">
                  {unavailableReason(telemetry.fans, "No hardware fans were reported.")}
                </div>
              )}
            </div>

            <BatteryCard
              battery={battery}
              unit={preferences.temperature_unit === "celsius" ? "C" : "F"}
            />

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

        {hasUpdateAvailable && updateRelease ? (
          <button
            onClick={() => setShowUpdateModal(true)}
            className="flex items-center gap-1 font-mono text-amber-400 font-bold hover:underline transition-all"
          >
            <Sparkles className="w-3 h-3 animate-pulse" />
            Update v{updateRelease.version}
          </button>
        ) : (
          <span className="font-mono">SuperFan v1.0.2</span>
        )}
      </div>

      {showUpdateModal && updateRelease && (
        <UpdateModal
          currentVersion="1.0.2"
          release={updateRelease}
          onClose={() => setShowUpdateModal(false)}
        />
      )}
    </div>
  );
}

export default App;
