import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Header } from "./components/Header";
import { TemperatureGauge } from "./components/TemperatureGauge";
import { TemperatureChart } from "./components/TemperatureChart";
import { CoreBreakdown } from "./components/CoreBreakdown";
import { FanCard } from "./components/FanCard";
import { BatteryCard } from "./components/BatteryCard";
import { SettingsModal } from "./components/SettingsModal";
import { TelemetryData, AppSettings, FanReading } from "./types";
import { ShieldCheck } from "lucide-react";

export function App() {
  const [telemetry, setTelemetry] = useState<TelemetryData | null>(null);
  const [tempHistory, setTempHistory] = useState<{ time: number; cpu: number; gpu: number }[]>([]);
  const [activeTab, setActiveTab] = useState<"overview" | "dashboard" | "settings">("overview");
  const [settings, setSettings] = useState<AppSettings>({
    tempUnit: "C",
    pollingInterval: 1500,
    launchAtLogin: false,
    demoMode: true, // default to demo mode for initial UI demonstration
  });

  useEffect(() => {
    // Enable demo mode on backend startup if needed
    invoke("set_demo_mode", { enabled: settings.demoMode });

    // Initial fetch
    invoke<TelemetryData>("fetch_telemetry")
      .then((data) => {
        setTelemetry(data);
        if (data.cpu_temp !== null) {
          setTempHistory([{ time: Date.now(), cpu: data.cpu_temp, gpu: data.gpu_temp || data.cpu_temp - 5 }]);
        }
      })
      .catch((err) => console.error("Telemetry fetch error:", err));

    // Listen for real-time telemetry updates from Rust backend
    const unlistenPromise = listen<TelemetryData>("telemetry-update", (event) => {
      const data = event.payload;
      setTelemetry(data);
      if (data.cpu_temp !== null) {
        setTempHistory((prev) => [
          ...prev.slice(-40),
          { time: Date.now(), cpu: data.cpu_temp!, gpu: data.gpu_temp || data.cpu_temp! - 4 },
        ]);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleToggleDemo = async (enabled: boolean) => {
    setSettings((prev) => ({ ...prev, demoMode: enabled }));
    await invoke("set_demo_mode", { enabled });
    const data = await invoke<TelemetryData>("fetch_telemetry");
    setTelemetry(data);
  };

  const handleSetFanSpeed = async (fanId: number, rpm: number) => {
    try {
      await invoke("set_fan_speed", { fanId, rpm });
    } catch (err) {
      console.error("Set fan speed failed:", err);
    }
  };

  const handleSetFanMode = async (fanId: number, mode: "auto" | "manual") => {
    try {
      await invoke("set_fan_mode", { fanId, mode });
    } catch (err) {
      console.error("Set fan mode failed:", err);
    }
  };

  const handleHideWindow = () => {
    invoke("toggle_popover").catch(() => {});
  };

  // Fallback demo data if telemetry is not yet populated
  const currentTelemetry: TelemetryData = telemetry || {
    cpu_temp: 48.5,
    gpu_temp: 43.2,
    max_cpu_temp: 52.0,
    sensors: [
      { key: "Tp01", label: "P-Core 1", value: 48.5 },
      { key: "Tp05", label: "P-Core 2", value: 49.7 },
      { key: "Te05", label: "E-Core 1", value: 45.0 },
      { key: "Tg0D", label: "GPU Core", value: 43.2 },
    ],
    fans: [
      {
        id: 0,
        label: "Fan 1 (Left)",
        speed: 2150,
        min_speed: 1200,
        max_speed: 6000,
        target_speed: 2500,
        mode: "auto",
      },
      {
        id: 1,
        label: "Fan 2 (Right)",
        speed: 2080,
        min_speed: 1200,
        max_speed: 6000,
        target_speed: 2500,
        mode: "auto",
      },
    ],
    battery: {
      percentage: 88,
      temperature: 31.2,
      is_charging: true,
      cycle_count: 142,
      power_watts: 18.5,
    },
    has_smc_access: true,
    is_helper_installed: true,
    is_demo_mode: true,
    timestamp: Date.now(),
  };

  return (
    <div className="w-full h-screen glass-panel flex flex-col rounded-2xl overflow-hidden border border-white/10 shadow-2xl">
      <Header
        isDemoMode={currentTelemetry.is_demo_mode}
        hasAccess={currentTelemetry.has_smc_access}
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        onHideWindow={handleHideWindow}
      />

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto p-3.5 space-y-3">
        {activeTab === "settings" ? (
          <SettingsModal
            settings={settings}
            isHelperInstalled={currentTelemetry.is_helper_installed}
            onUpdateSettings={(newVal) => setSettings((prev) => ({ ...prev, ...newVal }))}
            onToggleDemo={handleToggleDemo}
          />
        ) : (
          <>
            {/* Temperature Section */}
            <TemperatureGauge
              cpuTemp={currentTelemetry.cpu_temp}
              gpuTemp={currentTelemetry.gpu_temp}
              unit={settings.tempUnit}
            />

            {/* Temperature History Chart */}
            <TemperatureChart history={tempHistory} unit={settings.tempUnit} />

            {/* Per-Core Thermal Breakdown */}
            <CoreBreakdown sensors={currentTelemetry.sensors} unit={settings.tempUnit} />

            {/* Fans Section */}
            <div className="space-y-2">
              <div className="flex items-center justify-between px-1">
                <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
                  Fan Speed Controls
                </span>
                <span className="text-[10px] text-cyan-400 font-mono font-semibold">
                  {currentTelemetry.fans.length} Active Fans
                </span>
              </div>

              {currentTelemetry.fans.map((fan: FanReading) => (
                <FanCard
                  key={fan.id}
                  fan={fan}
                  onSetSpeed={handleSetFanSpeed}
                  onSetMode={handleSetFanMode}
                />
              ))}
            </div>

            {/* Battery Section */}
            <BatteryCard battery={currentTelemetry.battery} unit={settings.tempUnit} />
          </>
        )}
      </div>

      {/* Footer Status bar */}
      <div className="px-3.5 py-2 border-t border-white/10 flex items-center justify-between text-[10px] text-slate-400 bg-slate-950/40">
        <span className="flex items-center gap-1">
          <ShieldCheck className="w-3 h-3 text-emerald-400" />
          SMC Status: Active
        </span>
        <span className="font-mono">SuperFan v1.0.0</span>
      </div>
    </div>
  );
}

export default App;
