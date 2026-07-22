import React from "react";
import { Cpu } from "lucide-react";
import { TemperatureReading } from "../types";

interface CoreBreakdownProps {
  sensors: TemperatureReading[];
  unit: "C" | "F";
}

export const CoreBreakdown: React.FC<CoreBreakdownProps> = ({ sensors, unit }) => {
  if (!sensors || sensors.length === 0) return null;

  const formatTemp = (val: number) => {
    if (unit === "F") return `${Math.round((val * 9) / 5 + 32)}°F`;
    return `${Math.round(val)}°C`;
  };

  const getTempColor = (val: number) => {
    if (val < 50) return "text-emerald-400 bg-emerald-500/10 border-emerald-500/20";
    if (val < 70) return "text-cyan-400 bg-cyan-500/10 border-cyan-500/20";
    if (val < 85) return "text-amber-400 bg-amber-500/10 border-amber-500/20";
    return "text-rose-400 bg-rose-500/10 border-rose-500/20";
  };

  return (
    <div className="glass-card p-3.5 rounded-xl flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
          <Cpu className="w-3.5 h-3.5 text-cyan-400" />
          Per-Core Thermal Breakdown
        </div>
        <span className="text-[10px] text-slate-400 font-mono">{sensors.length} Core Sensors</span>
      </div>

      <div className="grid grid-cols-2 gap-2 mt-1">
        {sensors.map((s) => (
          <div
            key={s.key}
            className={`p-2 rounded-lg border flex items-center justify-between transition-all ${getTempColor(
              s.celsius
            )}`}
          >
            <div>
              <div className="text-[10px] font-bold text-slate-200">{s.label}</div>
              <div className="text-[9px] font-mono text-slate-400">{s.key}</div>
            </div>
            <div className="text-xs font-extrabold font-mono">{formatTemp(s.celsius)}</div>
          </div>
        ))}
      </div>
    </div>
  );
};
