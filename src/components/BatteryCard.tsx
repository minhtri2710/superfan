import React from "react";
import { BatteryCharging, Battery, Zap, Thermometer } from "lucide-react";
import { BatteryReading } from "../types";

interface BatteryCardProps {
  battery: BatteryReading | null;
  unit: "C" | "F";
}

export const BatteryCard: React.FC<BatteryCardProps> = ({ battery, unit }) => {
  if (!battery) return null;

  const formatTemp = (val: number) => {
    if (unit === "F") return `${Math.round((val * 9) / 5 + 32)}°F`;
    return `${val.toFixed(1)}°C`;
  };

  return (
    <div className="glass-card p-3 rounded-xl flex items-center justify-between text-slate-300">
      <div className="flex items-center gap-2.5">
        <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          {battery.is_charging ? (
            <BatteryCharging className="w-4 h-4 animate-pulse" />
          ) : (
            <Battery className="w-4 h-4" />
          )}
        </div>
        <div>
          <div className="flex items-center gap-1.5">
            <span className="text-sm font-bold text-white font-mono">{battery.percentage}%</span>
            <span className="text-[10px] text-slate-400">
              {battery.is_charging ? "Charging" : "Discharging"}
            </span>
          </div>
          <div className="flex items-center gap-2 text-[10px] text-slate-400 mt-0.5">
            <span className="flex items-center gap-0.5">
              <Thermometer className="w-3 h-3 text-cyan-400" />
              {formatTemp(battery.temperature)}
            </span>
            <span>•</span>
            <span className="flex items-center gap-0.5">
              <Zap className="w-3 h-3 text-amber-400" />
              {battery.power_watts.toFixed(1)}W
            </span>
          </div>
        </div>
      </div>

      <div className="text-right text-[10px] text-slate-400">
        <div className="font-mono text-slate-300 font-semibold">{battery.cycle_count} cycles</div>
        <div>Health Nominal</div>
      </div>
    </div>
  );
};
