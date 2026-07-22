import React, { useState } from "react";
import { Sliders, Shield, Flame, Zap, Plus, Check, Trash2 } from "lucide-react";
import { FanRule, SensorReading } from "../types";

interface FanRuleManagerProps {
  activePreset: "auto" | "quiet" | "performance" | "custom";
  customRules: FanRule[];
  sensors: SensorReading[];
  onSelectPreset: (preset: "auto" | "quiet" | "performance" | "custom") => void;
  onSaveRule: (rule: FanRule) => void;
  onDeleteRule: (id: string) => void;
}

export const FanRuleManager: React.FC<FanRuleManagerProps> = ({
  activePreset,
  customRules,
  sensors,
  onSelectPreset,
  onSaveRule,
  onDeleteRule,
}) => {
  const [editing, setEditing] = useState(false);
  const [targetSensor, setTargetSensor] = useState<string>("hottest");
  const [lowTemp, setLowTemp] = useState<number>(45);
  const [highTemp, setHighTemp] = useState<number>(80);
  const [minFan, setMinFan] = useState<number>(20);
  const [maxFan, setMaxFan] = useState<number>(100);
  const [ruleName, setRuleName] = useState<string>("Custom Thermal Rule");

  const handleCreateRule = () => {
    const newRule: FanRule = {
      id: Date.now().toString(),
      name: ruleName,
      targetSensor,
      lowTemp,
      highTemp,
      minFanPercent: minFan,
      maxFanPercent: maxFan,
      active: true,
    };
    onSaveRule(newRule);
    onSelectPreset("custom");
    setEditing(false);
  };

  return (
    <div className="glass-card p-3.5 rounded-xl flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
          <Sliders className="w-3.5 h-3.5 text-cyan-400" />
          iStat-style Sensor Fan Control
        </div>
        <button
          onClick={() => setEditing(!editing)}
          className="px-2 py-0.5 rounded-md bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 text-[10px] font-semibold flex items-center gap-1 hover:bg-cyan-500/30 transition-all"
        >
          <Plus className="w-3 h-3" />
          New Rule
        </button>
      </div>

      {/* Preset Grid */}
      <div className="grid grid-cols-3 gap-2 text-[11px]">
        {/* macOS Auto */}
        <button
          onClick={() => onSelectPreset("auto")}
          className={`p-2.5 rounded-lg border text-left transition-all ${
            activePreset === "auto"
              ? "bg-cyan-500/20 border-cyan-500/50 text-white shadow-md shadow-cyan-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Shield className="w-3.5 h-3.5 text-emerald-400" />
            {activePreset === "auto" && <Check className="w-3 h-3 text-cyan-400" />}
          </div>
          <div className="font-bold text-xs">System Auto</div>
          <div className="text-[9px] text-slate-400 mt-0.5">Default Apple SMC</div>
        </button>

        {/* Quiet Profile */}
        <button
          onClick={() => onSelectPreset("quiet")}
          className={`p-2.5 rounded-lg border text-left transition-all ${
            activePreset === "quiet"
              ? "bg-cyan-500/20 border-cyan-500/50 text-white shadow-md shadow-cyan-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Zap className="w-3.5 h-3.5 text-cyan-400" />
            {activePreset === "quiet" && <Check className="w-3 h-3 text-cyan-400" />}
          </div>
          <div className="font-bold text-xs">Quiet</div>
          <div className="text-[9px] text-slate-400 mt-0.5">Low noise 50-85°C</div>
        </button>

        {/* Performance Profile */}
        <button
          onClick={() => onSelectPreset("performance")}
          className={`p-2.5 rounded-lg border text-left transition-all ${
            activePreset === "performance"
              ? "bg-amber-500/20 border-amber-500/50 text-white shadow-md shadow-amber-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Flame className="w-3.5 h-3.5 text-amber-400" />
            {activePreset === "performance" && <Check className="w-3 h-3 text-amber-400" />}
          </div>
          <div className="font-bold text-xs">Performance</div>
          <div className="text-[9px] text-slate-400 mt-0.5">Max cooling 40-75°C</div>
        </button>
      </div>

      {/* Custom Rule Builder Modal */}
      {editing && (
        <div className="p-3 rounded-lg bg-slate-900/80 border border-cyan-500/30 flex flex-col gap-2.5">
          <div className="text-xs font-bold text-cyan-300">Configure Custom Sensor Rule</div>

          <div>
            <label className="text-[10px] text-slate-400">Rule Name</label>
            <input
              type="text"
              value={ruleName}
              onChange={(e) => setRuleName(e.target.value)}
              className="w-full bg-slate-950 border border-white/10 rounded px-2 py-1 text-xs text-white outline-none mt-0.5"
            />
          </div>

          <div>
            <label className="text-[10px] text-slate-400">Target Sensor Component</label>
            <select
              value={targetSensor}
              onChange={(e) => setTargetSensor(e.target.value)}
              className="w-full bg-slate-950 border border-white/10 rounded px-2 py-1 text-xs text-white outline-none mt-0.5 cursor-pointer font-mono"
            >
              <option value="hottest">🔥 Hottest Component (Auto Target)</option>
              <option value="cpu">💻 CPU Aggregate</option>
              <option value="gpu">🎮 GPU Cluster</option>
              {sensors.map((s) => (
                <option key={s.key} value={s.key}>
                  {s.label} ({s.key})
                </option>
              ))}
            </select>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-[10px] text-slate-400">Low Temp Trigger (°C)</label>
              <input
                type="number"
                min={30}
                max={70}
                value={lowTemp}
                onChange={(e) => setLowTemp(Number(e.target.value))}
                className="w-full bg-slate-950 border border-white/10 rounded px-2 py-1 text-xs text-white font-mono outline-none mt-0.5"
              />
            </div>
            <div>
              <label className="text-[10px] text-slate-400">High Temp Threshold (°C)</label>
              <input
                type="number"
                min={60}
                max={100}
                value={highTemp}
                onChange={(e) => setHighTemp(Number(e.target.value))}
                className="w-full bg-slate-950 border border-white/10 rounded px-2 py-1 text-xs text-white font-mono outline-none mt-0.5"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-[10px] text-slate-400">Min Fan Speed ({minFan}%)</label>
              <input
                type="range"
                min={10}
                max={50}
                value={minFan}
                onChange={(e) => setMinFan(Number(e.target.value))}
                className="w-full h-1 bg-slate-800 rounded accent-cyan-400 cursor-pointer mt-1"
              />
            </div>
            <div>
              <label className="text-[10px] text-slate-400">Max Fan Speed ({maxFan}%)</label>
              <input
                type="range"
                min={60}
                max={100}
                value={maxFan}
                onChange={(e) => setMaxFan(Number(e.target.value))}
                className="w-full h-1 bg-slate-800 rounded accent-amber-400 cursor-pointer mt-1"
              />
            </div>
          </div>

          <div className="flex gap-2 justify-end mt-1">
            <button
              onClick={() => setEditing(false)}
              className="px-3 py-1 rounded text-xs text-slate-400 hover:text-white"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateRule}
              className="px-3 py-1 rounded bg-cyan-500 text-slate-950 font-bold text-xs hover:bg-cyan-400 shadow-md shadow-cyan-500/20"
            >
              Save Rule
            </button>
          </div>
        </div>
      )}

      {/* Active Custom Rules List */}
      {customRules.length > 0 && (
        <div className="flex flex-col gap-1.5 mt-1">
          <div className="text-[10px] font-bold text-slate-400 uppercase">Custom Rules</div>
          {customRules.map((r) => (
            <div
              key={r.id}
              className="p-2 rounded-lg bg-slate-900/60 border border-white/5 flex items-center justify-between text-xs"
            >
              <div>
                <div className="font-semibold text-white">{r.name}</div>
                <div className="text-[9px] text-slate-400 font-mono">
                  Target: {r.targetSensor} | {r.lowTemp}°C - {r.highTemp}°C ({r.minFanPercent}%-{r.maxFanPercent}%)
                </div>
              </div>
              <button
                onClick={() => onDeleteRule(r.id)}
                className="p-1 rounded text-slate-500 hover:text-rose-400 transition-colors"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
