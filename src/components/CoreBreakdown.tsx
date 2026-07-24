import React, { useState, useMemo } from "react";
import { ChevronDown, Cpu, Flame, ArrowUpDown, Layers, Thermometer, Activity } from "lucide-react";
import { TemperatureReading } from "../types";

interface CoreBreakdownProps {
  sensors: TemperatureReading[];
  unit: "C" | "F";
}

type SortOption = "default" | "warmest" | "coolest";

export const CoreBreakdown: React.FC<CoreBreakdownProps> = ({ sensors, unit }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [sortBy, setSortBy] = useState<SortOption>("default");

  if (!sensors || sensors.length === 0) return null;

  const formatTemp = (val: number) => {
    if (unit === "F") return `${Math.round((val * 9) / 5 + 32)}°F`;
    return `${Math.round(val)}°C`;
  };

  const getTempTheme = (val: number) => {
    if (val < 50) {
      return {
        badgeBg: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
        barGradient: "from-emerald-500 to-teal-400",
        textColor: "text-emerald-400",
        statusText: "Cool",
        dotColor: "bg-emerald-400",
      };
    }
    if (val < 70) {
      return {
        badgeBg: "bg-cyan-500/10 text-cyan-400 border-cyan-500/20",
        barGradient: "from-cyan-500 to-blue-400",
        textColor: "text-cyan-400",
        statusText: "Nominal",
        dotColor: "bg-cyan-400",
      };
    }
    if (val < 85) {
      return {
        badgeBg: "bg-amber-500/10 text-amber-400 border-amber-500/20",
        barGradient: "from-amber-500 to-orange-400",
        textColor: "text-amber-400",
        statusText: "Warm",
        dotColor: "bg-amber-400",
      };
    }
    return {
      badgeBg: "bg-rose-500/10 text-rose-400 border-rose-500/20",
      barGradient: "from-rose-600 to-red-500",
      textColor: "text-rose-400",
      statusText: "Hot",
      dotColor: "bg-rose-400 animate-pulse",
    };
  };

  // Compute thermal metrics
  const celsiusValues = useMemo(() => sensors.map((s) => s.celsius), [sensors]);
  const minTemp = useMemo(() => Math.min(...celsiusValues), [celsiusValues]);
  const maxTemp = useMemo(() => Math.max(...celsiusValues), [celsiusValues]);
  const avgTemp = useMemo(
    () => celsiusValues.reduce((acc, curr) => acc + curr, 0) / celsiusValues.length,
    [celsiusValues]
  );
  const tempSpread = maxTemp - minTemp;
  const overallTheme = getTempTheme(maxTemp);

  // Sorted sensors
  const sortedSensors = useMemo(() => {
    const list = [...sensors];
    if (sortBy === "warmest") {
      return list.sort((a, b) => b.celsius - a.celsius);
    }
    if (sortBy === "coolest") {
      return list.sort((a, b) => a.celsius - b.celsius);
    }
    return list;
  }, [sensors, sortBy]);

  const getProgressWidth = (val: number) => {
    const clamped = Math.min(Math.max(val, 20), 100);
    return ((clamped - 20) / 80) * 100;
  };

  return (
    <div className="glass-card p-3.5 rounded-xl flex flex-col gap-2.5 transition-all duration-300">
      {/* Header Button */}
      <div className="flex items-center justify-between gap-2">
        <button
          type="button"
          onClick={() => setIsExpanded((expanded) => !expanded)}
          aria-expanded={isExpanded}
          aria-controls="per-core-thermal-breakdown"
          className="group flex min-w-0 flex-1 items-center gap-2 rounded-lg text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-cyan-400/60"
        >
          <div className="p-1.5 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 group-hover:bg-cyan-500/20 transition-colors shrink-0">
            <Cpu className="w-4 h-4" />
          </div>

          <div className="flex flex-col min-w-0">
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-slate-200 group-hover:text-white transition-colors truncate">
                Per-Core Thermal Breakdown
              </span>
              <span className="shrink-0 text-[10px] font-mono px-1.5 py-0.5 rounded-md bg-slate-800/80 text-slate-400 border border-white/5">
                {sensors.length} Cores
              </span>
            </div>

            {/* Micro Header Preview */}
            <div className="flex items-center gap-2 text-[10px] text-slate-400 font-mono mt-0.5">
              <span>Avg: <strong className="text-slate-300">{formatTemp(avgTemp)}</strong></span>
              <span>•</span>
              <span>Max: <strong className={overallTheme.textColor}>{formatTemp(maxTemp)}</strong></span>
            </div>
          </div>

          <ChevronDown
            className={`ml-auto h-4 w-4 shrink-0 text-slate-400 transition-transform duration-300 ease-out group-hover:text-slate-200 ${
              isExpanded ? "rotate-180 text-cyan-400" : ""
            }`}
            aria-hidden="true"
          />
        </button>
      </div>

      {/* Mini Heat Spectrum Bar (Always visible overview strip) */}
      <div className="w-full bg-slate-900/60 p-1.5 rounded-lg border border-white/5 flex items-center gap-1.5">
        <div className="text-[9px] font-bold text-slate-400 uppercase tracking-wider shrink-0 pr-1.5 border-r border-white/10">
          Spectrum
        </div>
        <div className="flex-1 flex items-center gap-0.5 h-3.5 px-0.5">
          {sensors.map((s, idx) => {
            const theme = getTempTheme(s.celsius);
            const heightPct = Math.min(Math.max(((s.celsius - 20) / 80) * 100, 25), 100);
            return (
              <div
                key={s.key || idx}
                title={`${s.label}: ${formatTemp(s.celsius)}`}
                className="flex-1 h-full flex items-end justify-center group relative cursor-pointer"
              >
                <div
                  className={`w-full rounded-sm bg-gradient-to-t ${theme.barGradient} transition-all duration-300 group-hover:brightness-125`}
                  style={{ height: `${heightPct}%` }}
                />
              </div>
            );
          })}
        </div>
      </div>

      {/* Collapsible Detailed Breakdown */}
      <div
        id="per-core-thermal-breakdown"
        className={`grid overflow-hidden transition-[grid-template-rows,opacity,margin] duration-300 ease-out ${
          isExpanded ? "mt-1 grid-rows-[1fr] opacity-100" : "mt-0 grid-rows-[0fr] opacity-0"
        }`}
        aria-hidden={!isExpanded}
      >
        <div className="min-h-0 flex flex-col gap-3">
          {/* Summary Dashboard Strip */}
          <div className="grid grid-cols-3 gap-2 p-2.5 rounded-xl bg-slate-900/80 border border-white/5 text-[11px]">
            <div className="flex flex-col gap-0.5">
              <span className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1">
                <Thermometer className="w-3 h-3 text-cyan-400" /> Avg Temp
              </span>
              <span className="font-mono font-bold text-slate-200 text-sm">
                {formatTemp(avgTemp)}
              </span>
            </div>

            <div className="flex flex-col gap-0.5 border-x border-white/10 px-2">
              <span className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1">
                <Flame className="w-3 h-3 text-rose-400" /> Peak Core
              </span>
              <span className={`font-mono font-bold text-sm ${overallTheme.textColor}`}>
                {formatTemp(maxTemp)}
              </span>
            </div>

            <div className="flex flex-col gap-0.5 pl-1">
              <span className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1">
                <Activity className="w-3 h-3 text-amber-400" /> Delta (Spread)
              </span>
              <span className="font-mono font-bold text-slate-200 text-sm">
                Δ {unit === "F" ? `${Math.round((tempSpread * 9) / 5)}°F` : `${Math.round(tempSpread)}°C`}
              </span>
            </div>
          </div>

          {/* Controls Bar: Sort options */}
          <div className="flex items-center justify-between px-0.5">
            <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1">
              <Layers className="w-3 h-3 text-slate-400" /> Cores Detail
            </span>

            <div className="flex items-center gap-1 text-[10px]">
              <ArrowUpDown className="w-3 h-3 text-slate-400" />
              <button
                type="button"
                onClick={() =>
                  setSortBy((current) =>
                    current === "default" ? "warmest" : current === "warmest" ? "coolest" : "default"
                  )
                }
                className="font-mono text-cyan-400 hover:text-cyan-300 font-semibold px-2 py-0.5 rounded bg-cyan-500/10 border border-cyan-500/20 hover:bg-cyan-500/20 transition-colors"
              >
                Sort: {sortBy === "default" ? "Default" : sortBy === "warmest" ? "Hot First" : "Cool First"}
              </button>
            </div>
          </div>

          {/* Core Cards Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {sortedSensors.map((s) => {
              const theme = getTempTheme(s.celsius);
              const progressWidth = getProgressWidth(s.celsius);

              return (
                <div
                  key={s.key}
                  className="p-2.5 rounded-xl border border-white/5 bg-slate-900/50 hover:bg-slate-800/60 hover:border-white/10 transition-all flex flex-col gap-1.5 shadow-sm"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${theme.dotColor}`} />
                      <span className="text-[11px] font-bold text-slate-200 truncate">{s.label}</span>
                    </div>
                    <span className="text-xs font-extrabold font-mono text-slate-100">
                      {formatTemp(s.celsius)}
                    </span>
                  </div>

                  <div className="flex items-center justify-between text-[9px] text-slate-400 font-mono">
                    <span>{s.key}</span>
                    <span className={`px-1.5 py-0.2 rounded font-semibold border ${theme.badgeBg}`}>
                      {theme.statusText}
                    </span>
                  </div>

                  {/* Micro Progress Bar */}
                  <div className="w-full bg-slate-950/80 rounded-full h-1 overflow-hidden border border-white/5 mt-0.5">
                    <div
                      className={`h-full rounded-full bg-gradient-to-r ${theme.barGradient} transition-all duration-500`}
                      style={{ width: `${progressWidth}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};

