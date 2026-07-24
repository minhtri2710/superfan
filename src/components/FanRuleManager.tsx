import React, { useState } from "react";
import { Sliders, Shield, Flame, Zap, Plus, Check, Trash2, Cpu, Thermometer, Pencil } from "lucide-react";
import { TemperatureReading, ThermalPolicyMode, ThermalRule, ThermalTarget } from "../types";

interface FanRuleManagerProps {
  activePreset: ThermalPolicyMode;
  customRules: ThermalRule[];
  sensors: TemperatureReading[];
  onSelectPreset: (preset: ThermalPolicyMode) => void;
  onSaveRule: (rule: ThermalRule) => void;
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
  const [editingRuleId, setEditingRuleId] = useState<string | null>(null);
  const [targetSensor, setTargetSensor] = useState<string>("hottest");
  const [lowTemp, setLowTemp] = useState<number>(45);
  const [highTemp, setHighTemp] = useState<number>(80);
  const [minFan, setMinFan] = useState<number>(20);
  const [maxFan, setMaxFan] = useState<number>(100);
  const [ruleName, setRuleName] = useState<string>("Custom Thermal Profile");

  const resetForm = () => {
    setEditingRuleId(null);
    setRuleName("Custom Thermal Profile");
    setTargetSensor("hottest");
    setLowTemp(45);
    setHighTemp(80);
    setMinFan(20);
    setMaxFan(100);
  };

  const handleStartNewRule = () => {
    resetForm();
    setEditing(true);
  };

  const handleStartEditRule = (rule: ThermalRule) => {
    setEditingRuleId(rule.id);
    setRuleName(rule.name);
    setTargetSensor(rule.target.type === "sensor_key" ? rule.target.key : rule.target.type);
    setLowTemp(rule.low_celsius);
    setHighTemp(rule.high_celsius);
    setMinFan(rule.min_fan_percent);
    setMaxFan(rule.max_fan_percent);
    setEditing(true);
  };

  const handleSaveRuleSubmit = () => {
    const target: ThermalTarget =
      targetSensor === "hottest" || targetSensor === "cpu" || targetSensor === "gpu"
        ? { type: targetSensor }
        : { type: "sensor_key", key: targetSensor };
    const ruleToSave: ThermalRule = {
      id: editingRuleId || Date.now().toString(),
      name: ruleName,
      target,
      low_celsius: lowTemp,
      high_celsius: highTemp,
      min_fan_percent: minFan,
      max_fan_percent: maxFan,
      active: true,
    };
    onSaveRule(ruleToSave);
    onSelectPreset("custom");
    setEditing(false);
    resetForm();
  };

  return (
    <div className="glass-card p-3.5 rounded-xl flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
          <Sliders className="w-3.5 h-3.5 text-cyan-400" />
          Smart Thermal Control
        </div>
        <button
          onClick={handleStartNewRule}
          className="px-2.5 py-1 rounded-lg bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 text-[10px] font-bold flex items-center gap-1 hover:bg-cyan-500/30 transition-all shadow-sm"
        >
          <Plus className="w-3 h-3" />
          New Rule
        </button>
      </div>

      {/* Preset Grid - 4 Columns */}
      <div className="grid grid-cols-4 gap-1.5 text-[11px]">
        {/* macOS Auto */}
        <button
          onClick={() => onSelectPreset("system_auto")}
          className={`p-2 rounded-xl border text-left transition-all ${
            activePreset === "system_auto"
              ? "bg-cyan-500/20 border-cyan-500/50 text-white shadow-md shadow-cyan-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Shield className="w-3.5 h-3.5 text-emerald-400" />
            {activePreset === "system_auto" && <Check className="w-3 h-3 text-cyan-400" />}
          </div>
          <div className="font-bold text-[11px] truncate">System Auto</div>
          <div className="text-[9px] text-slate-400 mt-0.5 truncate">Default SMC</div>
        </button>

        {/* Quiet Profile */}
        <button
          onClick={() => onSelectPreset("quiet")}
          className={`p-2 rounded-xl border text-left transition-all ${
            activePreset === "quiet"
              ? "bg-cyan-500/20 border-cyan-500/50 text-white shadow-md shadow-cyan-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Zap className="w-3.5 h-3.5 text-cyan-400" />
            {activePreset === "quiet" && <Check className="w-3 h-3 text-cyan-400" />}
          </div>
          <div className="font-bold text-[11px] truncate">Quiet</div>
          <div className="text-[9px] text-slate-400 mt-0.5 truncate">50-85°C</div>
        </button>

        {/* Performance Profile */}
        <button
          onClick={() => onSelectPreset("performance")}
          className={`p-2 rounded-xl border text-left transition-all ${
            activePreset === "performance"
              ? "bg-amber-500/20 border-amber-500/50 text-white shadow-md shadow-amber-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Flame className="w-3.5 h-3.5 text-amber-400" />
            {activePreset === "performance" && <Check className="w-3 h-3 text-amber-400" />}
          </div>
          <div className="font-bold text-[11px] truncate">Performance</div>
          <div className="text-[9px] text-slate-400 mt-0.5 truncate">40-75°C</div>
        </button>

        {/* Custom Profile */}
        <button
          onClick={() => onSelectPreset("custom")}
          className={`p-2 rounded-xl border text-left transition-all ${
            activePreset === "custom"
              ? "bg-purple-500/20 border-purple-500/50 text-white shadow-md shadow-purple-500/10"
              : "bg-slate-900/40 border-white/5 text-slate-400 hover:text-white"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <Sliders className="w-3.5 h-3.5 text-purple-400" />
            {activePreset === "custom" && <Check className="w-3 h-3 text-purple-400" />}
          </div>
          <div className="font-bold text-[11px] truncate">Custom</div>
          <div className="text-[9px] text-slate-400 mt-0.5 truncate">User Rules</div>
        </button>
      </div>

      {/* Redesigned Custom Rule Builder Panel */}
      {editing && (
        <div className="p-3.5 rounded-xl bg-slate-950/90 border border-cyan-500/30 flex flex-col gap-3 shadow-xl backdrop-blur-md">
          <div className="flex items-center justify-between border-b border-white/10 pb-2">
            <div className="text-xs font-bold text-cyan-400 flex items-center gap-1.5">
              <Cpu className="w-3.5 h-3.5 text-cyan-400" />
              {editingRuleId ? "Edit Thermal Rule" : "Configure Thermal Rule"}
            </div>
            <span className="text-[9px] font-mono text-slate-400">
              {editingRuleId ? "Editing Rule" : "New Profile"}
            </span>
          </div>

          <div className="space-y-1">
            <label className="text-[10px] font-medium text-slate-300">Profile Name</label>
            <input
              type="text"
              value={ruleName}
              onChange={(e) => setRuleName(e.target.value)}
              placeholder="Rule Name"
              className="w-full bg-slate-900/90 border border-white/10 rounded-lg px-2.5 py-1.5 text-xs text-white outline-none focus:border-cyan-400 transition-colors"
            />
          </div>

          <div className="space-y-1">
            <label className="text-[10px] font-medium text-slate-300">Target Sensor Component</label>
            <select
              value={targetSensor}
              onChange={(e) => setTargetSensor(e.target.value)}
              className="w-full bg-slate-900/90 border border-white/10 rounded-lg px-2.5 py-1.5 text-xs text-white outline-none focus:border-cyan-400 cursor-pointer font-mono transition-colors"
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

          {/* Temperature Triggers */}
          <div className="grid grid-cols-2 gap-2.5">
            <div className="space-y-1">
              <div className="flex justify-between items-center text-[10px] text-slate-300 font-medium">
                <span className="flex items-center gap-1">
                  <Thermometer className="w-3 h-3 text-cyan-400" /> Low Trigger
                </span>
                <span className="font-mono text-cyan-400 font-bold">{lowTemp}°C</span>
              </div>
              <input
                type="number"
                min={30}
                max={70}
                value={lowTemp}
                onChange={(e) => setLowTemp(Number(e.target.value))}
                className="w-full bg-slate-900/90 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-white font-mono outline-none focus:border-cyan-400 transition-colors"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between items-center text-[10px] text-slate-300 font-medium">
                <span className="flex items-center gap-1">
                  <Thermometer className="w-3 h-3 text-amber-400" /> High Threshold
                </span>
                <span className="font-mono text-amber-400 font-bold">{highTemp}°C</span>
              </div>
              <input
                type="number"
                min={60}
                max={100}
                value={highTemp}
                onChange={(e) => setHighTemp(Number(e.target.value))}
                className="w-full bg-slate-900/90 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-white font-mono outline-none focus:border-cyan-400 transition-colors"
              />
            </div>
          </div>

          {/* Fan Speeds % */}
          <div className="grid grid-cols-2 gap-2.5">
            <div className="space-y-1">
              <div className="flex justify-between text-[10px] text-slate-300">
                <span>Min Speed</span>
                <span className="font-mono text-cyan-400 font-semibold">{minFan}%</span>
              </div>
              <input
                type="range"
                min={10}
                max={50}
                value={minFan}
                onChange={(e) => setMinFan(Number(e.target.value))}
                className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
              />
            </div>

            <div className="space-y-1">
              <div className="flex justify-between text-[10px] text-slate-300">
                <span>Max Speed</span>
                <span className="font-mono text-amber-400 font-semibold">{maxFan}%</span>
              </div>
              <input
                type="range"
                min={60}
                max={100}
                value={maxFan}
                onChange={(e) => setMaxFan(Number(e.target.value))}
                className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-amber-400"
              />
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-2 justify-end pt-1">
            <button
              onClick={() => {
                setEditing(false);
                resetForm();
              }}
              className="px-3 py-1.5 rounded-lg text-xs text-slate-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSaveRuleSubmit}
              className="px-3.5 py-1.5 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-slate-950 font-bold text-xs hover:from-cyan-400 hover:to-blue-400 shadow-md shadow-cyan-500/20 transition-all"
            >
              {editingRuleId ? "Update Rule" : "Save Rule"}
            </button>
          </div>
        </div>
      )}

      {/* Active Custom Rules List */}
      {customRules.length > 0 && (
        <div className="flex flex-col gap-1.5 mt-1">
          <div className="flex items-center justify-between text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            <span>Custom Profiles</span>
            {activePreset === "custom" && (
              <span className="text-purple-400 flex items-center gap-1 font-mono normal-case text-[9px] bg-purple-500/10 border border-purple-500/30 px-1.5 py-0.5 rounded">
                <span className="w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse"></span>
                ACTIVE POLICY
              </span>
            )}
          </div>
          {customRules.map((r) => (
            <div
              key={r.id}
              onClick={() => onSelectPreset("custom")}
              className={`p-2.5 rounded-xl border flex items-center justify-between text-xs cursor-pointer transition-all ${
                activePreset === "custom"
                  ? "bg-purple-950/30 border-purple-500/40 hover:border-purple-500/60"
                  : "bg-slate-900/60 border-white/5 hover:border-white/10"
              }`}
            >
              <div className="flex-1 pr-2">
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-white">{r.name}</span>
                  {activePreset === "custom" && (
                    <span className="text-[8px] bg-purple-500/20 text-purple-300 font-mono px-1.5 py-0.5 rounded border border-purple-500/30">
                      ENFORCED
                    </span>
                  )}
                </div>
                <div className="text-[9px] text-slate-400 font-mono mt-0.5">
                  Target: {r.target.type === "sensor_key" ? r.target.key : r.target.type} | {r.low_celsius}°C - {r.high_celsius}°C ({r.min_fan_percent}%-{r.max_fan_percent}%)
                </div>
              </div>
              <div className="flex items-center gap-1">
                <button
                  title="Edit Rule"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleStartEditRule(r);
                  }}
                  className="p-1.5 rounded-lg text-slate-400 hover:text-cyan-300 hover:bg-cyan-500/10 transition-all"
                >
                  <Pencil className="w-3.5 h-3.5" />
                </button>
                <button
                  title="Delete Rule"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteRule(r.id);
                  }}
                  className="p-1.5 rounded-lg text-slate-500 hover:text-rose-400 hover:bg-white/5 transition-all"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

