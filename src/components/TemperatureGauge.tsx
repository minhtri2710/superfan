import React from "react";
import { Cpu, Zap, Activity } from "lucide-react";

interface TemperatureGaugeProps {
  cpuTemp: number | null;
  gpuTemp: number | null;
  unit: "C" | "F";
}

export const TemperatureGauge: React.FC<TemperatureGaugeProps> = ({ cpuTemp, gpuTemp, unit }) => {
  const formatTemp = (val: number | null) => {
    if (val === null) return "--";
    if (unit === "F") {
      return `${Math.round((val * 9) / 5 + 32)}°F`;
    }
    return `${Math.round(val)}°C`;
  };

  const getTempColor = (val: number | null) => {
    if (val === null) return "from-slate-600 to-slate-700 text-slate-400";
    if (val < 50) return "from-emerald-500 to-teal-400 text-emerald-400";
    if (val < 70) return "from-cyan-500 to-blue-500 text-cyan-400";
    if (val < 85) return "from-amber-500 to-orange-400 text-amber-400";
    return "from-rose-600 to-red-500 text-rose-400 animate-pulse";
  };

  const getProgressWidth = (val: number | null) => {
    if (val === null) return 0;
    const clamped = Math.min(Math.max(val, 20), 100);
    return ((clamped - 20) / 80) * 100;
  };

  return (
    <div className="grid grid-cols-2 gap-3">
      {/* CPU Card */}
      <div className="glass-card glass-card-hover p-3.5 rounded-xl flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
            <Cpu className="w-3.5 h-3.5 text-cyan-400" />
            CPU Temp
          </div>
          <Activity className="w-3 h-3 text-slate-500" />
        </div>

        <div className="my-2">
          <div className={`text-2xl font-extrabold tracking-tight ${getTempColor(cpuTemp).split(" ").pop()}`}>
            {formatTemp(cpuTemp)}
          </div>
          <div className="text-[10px] text-slate-400 mt-0.5">Apple Silicon / Intel Core</div>
        </div>

        {/* Progress Bar */}
        <div className="w-full bg-slate-800/80 rounded-full h-1.5 overflow-hidden border border-white/5">
          <div
            className={`h-full rounded-full bg-gradient-to-r ${getTempColor(cpuTemp).split(" ")[0]} ${
              getTempColor(cpuTemp).split(" ")[1]
            } transition-all duration-500`}
            style={{ width: `${getProgressWidth(cpuTemp)}%` }}
          />
        </div>
      </div>

      {/* GPU Card */}
      <div className="glass-card glass-card-hover p-3.5 rounded-xl flex flex-col justify-between">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
            <Zap className="w-3.5 h-3.5 text-amber-400" />
            GPU Temp
          </div>
          <Activity className="w-3 h-3 text-slate-500" />
        </div>

        <div className="my-2">
          <div className={`text-2xl font-extrabold tracking-tight ${getTempColor(gpuTemp).split(" ").pop()}`}>
            {formatTemp(gpuTemp)}
          </div>
          <div className="text-[10px] text-slate-400 mt-0.5">Graphics Cluster</div>
        </div>

        {/* Progress Bar */}
        <div className="w-full bg-slate-800/80 rounded-full h-1.5 overflow-hidden border border-white/5">
          <div
            className={`h-full rounded-full bg-gradient-to-r ${getTempColor(gpuTemp).split(" ")[0]} ${
              getTempColor(gpuTemp).split(" ")[1]
            } transition-all duration-500`}
            style={{ width: `${getProgressWidth(gpuTemp)}%` }}
          />
        </div>
      </div>
    </div>
  );
};
