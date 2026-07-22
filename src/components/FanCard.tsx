import React, { useState } from "react";
import { Fan, ShieldCheck } from "lucide-react";
import { FanReading } from "../types";

interface FanCardProps {
  fan: FanReading;
  onSetSpeed: (fanId: number, rpm: number) => void;
  onSetMode: (fanId: number, mode: "auto" | "manual") => void;
}

export const FanCard: React.FC<FanCardProps> = ({ fan, onSetSpeed, onSetMode }) => {
  const [sliderVal, setSliderVal] = useState<number>(fan.speed);

  const percent = Math.round(
    ((fan.speed - fan.min_speed) / (fan.max_speed - fan.min_speed || 1)) * 100
  );
  const clampedPercent = Math.min(Math.max(percent, 0), 100);

  // Rotation speed in seconds based on RPM
  const rotationDuration = fan.speed > 0 ? Math.max(0.3, 3 - (fan.speed / fan.max_speed) * 2.7) : 0;

  return (
    <div className="glass-card p-3.5 rounded-xl flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
            <Fan
              className="w-4 h-4"
              style={{
                animation: rotationDuration > 0 ? `spin ${rotationDuration}s linear infinite` : "none",
              }}
            />
          </div>
          <div>
            <h3 className="text-xs font-bold text-slate-200">{fan.label}</h3>
            <p className="text-[10px] text-slate-400 font-mono">
              {fan.min_speed} - {fan.max_speed} RPM
            </p>
          </div>
        </div>

        <div className="flex items-center gap-1 bg-slate-900/60 p-0.5 rounded-lg border border-white/5 text-[10px]">
          <button
            onClick={() => onSetMode(fan.id, "auto")}
            className={`px-2 py-0.5 rounded-md font-medium transition-all ${
              fan.mode === "auto"
                ? "bg-cyan-500/20 text-cyan-300 border border-cyan-500/30"
                : "text-slate-400 hover:text-white"
            }`}
          >
            Auto
          </button>
          <button
            onClick={() => onSetMode(fan.id, "manual")}
            className={`px-2 py-0.5 rounded-md font-medium transition-all ${
              fan.mode === "manual"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/30"
                : "text-slate-400 hover:text-white"
            }`}
          >
            Manual
          </button>
        </div>
      </div>

      <div className="flex items-baseline justify-between">
        <div className="flex items-baseline gap-1.5">
          <span className="text-2xl font-black text-white tracking-tight font-mono">{fan.speed}</span>
          <span className="text-xs font-semibold text-slate-400">RPM</span>
        </div>
        <span className="text-xs font-bold text-cyan-400 font-mono">{clampedPercent}%</span>
      </div>

      {fan.mode === "manual" ? (
        <div className="flex flex-col gap-1.5">
          <div className="flex justify-between text-[10px] text-slate-400">
            <span>Target Speed</span>
            <span className="font-mono text-slate-200">{sliderVal} RPM</span>
          </div>
          <input
            type="range"
            min={fan.min_speed}
            max={fan.max_speed}
            step={100}
            value={sliderVal}
            onChange={(e) => setSliderVal(Number(e.target.value))}
            onMouseUp={() => onSetSpeed(fan.id, sliderVal)}
            className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-400"
          />
        </div>
      ) : (
        <div className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-slate-900/40 border border-white/5 text-[10px] text-slate-400">
          <ShieldCheck className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
          <span>System & Target Curve managing fan speed dynamically</span>
        </div>
      )}
    </div>
  );
};
