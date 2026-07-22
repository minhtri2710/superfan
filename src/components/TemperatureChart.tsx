import React from "react";
import { TrendingUp } from "lucide-react";

interface TemperatureChartProps {
  history: { time: number; cpu: number; gpu: number | null }[];
  unit: "C" | "F";
}

export const TemperatureChart: React.FC<TemperatureChartProps> = ({ history, unit }) => {
  if (history.length < 2) return null;

  const maxPoints = 30;
  const data = history.slice(-maxPoints);

  const formatVal = (v: number) => (unit === "F" ? Math.round((v * 9) / 5 + 32) : Math.round(v));

  // Determine min & max for scale
  const cpuVals = data.map((d) => d.cpu);
  const gpuVals = data.flatMap((d) => (d.gpu === null ? [] : [d.gpu]));
  const allVals = [...cpuVals, ...gpuVals];
  const minTemp = Math.max(20, Math.min(...allVals) - 5);
  const maxTemp = Math.min(105, Math.max(...allVals) + 5);
  const range = maxTemp - minTemp || 1;

  const width = 340;
  const height = 64;

  const getX = (idx: number) => (idx / (data.length - 1)) * width;
  const getY = (val: number) => height - ((val - minTemp) / range) * (height - 8) - 4;

  const cpuPoints = data.map((d, i) => `${getX(i)},${getY(d.cpu)}`).join(" ");
  const gpuPoints = data
    .map((d, i) => (d.gpu === null ? null : `${getX(i)},${getY(d.gpu)}`))
    .filter((point): point is string => point !== null)
    .join(" ");

  const latestCpu = data[data.length - 1]?.cpu || 0;
  const latestGpu = data[data.length - 1]?.gpu ?? null;

  return (
    <div className="glass-card p-3.5 rounded-xl flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
          <TrendingUp className="w-3.5 h-3.5 text-cyan-400" />
          Real-time Temp History (30s)
        </div>
        <div className="flex items-center gap-3 text-[10px]">
          <span className="flex items-center gap-1 text-cyan-400 font-mono">
            <span className="w-2 h-2 rounded-full bg-cyan-400 inline-block" />
            CPU: {formatVal(latestCpu)}°{unit}
          </span>
          <span className="flex items-center gap-1 text-amber-400 font-mono">
            <span className="w-2 h-2 rounded-full bg-amber-400 inline-block" />
            GPU: {latestGpu === null ? "--" : `${formatVal(latestGpu)}°${unit}`}
          </span>
        </div>
      </div>

      <div className="relative w-full h-16 bg-slate-950/60 rounded-lg overflow-hidden border border-white/5 pt-1">
        <svg viewBox={`0 0 ${width} ${height}`} className="w-full h-full overflow-visible">
          <defs>
            <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#22d3ee" stopOpacity="0.3" />
              <stop offset="100%" stopColor="#22d3ee" stopOpacity="0.0" />
            </linearGradient>
            <linearGradient id="gpuGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#fbbf24" stopOpacity="0.25" />
              <stop offset="100%" stopColor="#fbbf24" stopOpacity="0.0" />
            </linearGradient>
          </defs>

          {/* Grid lines */}
          <line x1="0" y1={height * 0.25} x2={width} y2={height * 0.25} stroke="rgba(255,255,255,0.05)" strokeDasharray="3 3" />
          <line x1="0" y1={height * 0.75} x2={width} y2={height * 0.75} stroke="rgba(255,255,255,0.05)" strokeDasharray="3 3" />

          {/* Area under CPU */}
          <polygon
            points={`0,${height} ${cpuPoints} ${width},${height}`}
            fill="url(#cpuGradient)"
          />
          {/* CPU Polyline */}
          <polyline
            fill="none"
            stroke="#22d3ee"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            points={cpuPoints}
          />

          {gpuPoints && (
            <>
              <polygon
                points={`0,${height} ${gpuPoints} ${width},${height}`}
                fill="url(#gpuGradient)"
              />
              <polyline
                fill="none"
                stroke="#fbbf24"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                points={gpuPoints}
              />
            </>
          )}
        </svg>
      </div>
    </div>
  );
};
