import React, { useState } from "react";
import { ChevronDown, Cpu } from "lucide-react";
import { TemperatureReading } from "../types";

interface CoreBreakdownProps {
  sensors: TemperatureReading[];
  unit: "C" | "F";
}

export const CoreBreakdown: React.FC<CoreBreakdownProps> = ({ sensors, unit }) => {
  const [isExpanded, setIsExpanded] = useState(false);

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
      <div className="flex items-center justify-between gap-2">
        <button
          type="button"
          onClick={() => setIsExpanded((expanded) => !expanded)}
          aria-expanded={isExpanded}
          aria-controls="per-core-thermal-breakdown"
          className="group flex min-w-0 flex-1 items-center gap-1.5 rounded-md text-left text-xs font-semibold text-slate-300 outline-none transition-colors hover:text-slate-100 focus-visible:ring-2 focus-visible:ring-cyan-400/60"
        >
          <Cpu className="w-3.5 h-3.5 shrink-0 text-cyan-400" />
          <span className="truncate">Per-Core Thermal Breakdown</span>
          <ChevronDown
            className={`h-3.5 w-3.5 shrink-0 text-slate-500 transition-transform duration-200 ease-out ${
              isExpanded ? "rotate-180" : ""
            }`}
            aria-hidden="true"
          />
        </button>
        <span className="shrink-0 text-[10px] text-slate-400 font-mono">
          {sensors.length} Core Sensors
        </span>
      </div>

      <div
        id="per-core-thermal-breakdown"
        className={`grid overflow-hidden transition-[grid-template-rows,opacity,margin] duration-200 ease-out ${
          isExpanded ? "mt-1 grid-rows-[1fr] opacity-100" : "mt-0 grid-rows-[0fr] opacity-0"
        }`}
        aria-hidden={!isExpanded}
      >
        <div className="min-h-0 grid-cols-2 gap-2 sm:grid">
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
    </div>
  );
};
