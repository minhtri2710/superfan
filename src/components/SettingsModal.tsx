import React from "react";
import { AppSettings } from "../types";
import { ShieldCheck, ToggleLeft, ToggleRight, Sparkles, Clock, Thermometer } from "lucide-react";

interface SettingsModalProps {
  settings: AppSettings;
  onUpdateSettings: (newSettings: Partial<AppSettings>) => void;
  onToggleDemo: (enabled: boolean) => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  settings,
  onUpdateSettings,
  onToggleDemo,
}) => {
  return (
    <div className="flex flex-col gap-3 p-4">
      <h2 className="text-xs font-bold uppercase tracking-wider text-slate-400">Application Settings</h2>

      {/* Temperature Unit */}
      <div className="glass-card p-3 rounded-xl flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Thermometer className="w-4 h-4 text-cyan-400" />
          <div>
            <div className="text-xs font-semibold text-white">Temperature Unit</div>
            <div className="text-[10px] text-slate-400">Display values in Celsius or Fahrenheit</div>
          </div>
        </div>

        <div className="flex items-center p-0.5 bg-slate-900/60 rounded-lg border border-white/5 text-[11px]">
          <button
            onClick={() => onUpdateSettings({ tempUnit: "C" })}
            className={`px-2.5 py-1 rounded-md font-medium transition-all ${
              settings.tempUnit === "C"
                ? "bg-cyan-500/20 text-cyan-300 border border-cyan-500/30"
                : "text-slate-400 hover:text-white"
            }`}
          >
            °C
          </button>
          <button
            onClick={() => onUpdateSettings({ tempUnit: "F" })}
            className={`px-2.5 py-1 rounded-md font-medium transition-all ${
              settings.tempUnit === "F"
                ? "bg-cyan-500/20 text-cyan-300 border border-cyan-500/30"
                : "text-slate-400 hover:text-white"
            }`}
          >
            °F
          </button>
        </div>
      </div>

      {/* Polling Interval */}
      <div className="glass-card p-3 rounded-xl flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Clock className="w-4 h-4 text-amber-400" />
          <div>
            <div className="text-xs font-semibold text-white">Refresh Rate</div>
            <div className="text-[10px] text-slate-400">Telemetry polling frequency</div>
          </div>
        </div>

        <select
          value={settings.pollingInterval}
          onChange={(e) => onUpdateSettings({ pollingInterval: Number(e.target.value) })}
          className="bg-slate-900/80 border border-white/10 text-white text-xs rounded-lg px-2 py-1 outline-none font-mono"
        >
          <option value={1000}>1.0s (Fast)</option>
          <option value={1500}>1.5s (Balanced)</option>
          <option value={2500}>2.5s (Eco)</option>
        </select>
      </div>

      {/* Demo Mode Toggle */}
      <div className="glass-card p-3 rounded-xl flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-purple-400" />
          <div>
            <div className="text-xs font-semibold text-white">Demo Mode</div>
            <div className="text-[10px] text-slate-400">Simulate hardware data for UI testing</div>
          </div>
        </div>

        <button
          onClick={() => onToggleDemo(!settings.demoMode)}
          className="text-slate-300 hover:text-white transition-colors"
        >
          {settings.demoMode ? (
            <ToggleRight className="w-7 h-7 text-cyan-400" />
          ) : (
            <ToggleLeft className="w-7 h-7 text-slate-600" />
          )}
        </button>
      </div>

      {/* Helper Status */}
      <div className="glass-card p-3 rounded-xl flex items-center justify-between">
        <div className="flex items-center gap-2">
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          <div>
            <div className="text-xs font-semibold text-white">SMC Privileged Helper</div>
            <div className="text-[10px] text-slate-400">Required for manual fan speed modification</div>
          </div>
        </div>

        <span className="text-[10px] px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 font-medium">
          Active
        </span>
      </div>
    </div>
  );
};
