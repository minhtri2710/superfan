import React from "react";
import { Flame, Settings, X, ShieldAlert } from "lucide-react";

interface HeaderProps {
  isDemoMode: boolean;
  hasAccess: boolean;
  activeTab: "overview" | "dashboard" | "settings";
  setActiveTab: (tab: "overview" | "dashboard" | "settings") => void;
  onHideWindow: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  isDemoMode,
  hasAccess,
  activeTab,
  setActiveTab,
  onHideWindow,
}) => {
  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between px-4 py-3 border-b border-white/10 select-none cursor-grab active:cursor-grabbing"
    >
      <div className="flex items-center gap-2">
        <div className="p-1.5 rounded-lg bg-gradient-to-tr from-amber-500 to-red-500 shadow-lg shadow-orange-500/20">
          <Flame className="w-4 h-4 text-white animate-pulse" />
        </div>
        <div>
          <h1 className="text-sm font-bold tracking-tight text-white flex items-center gap-1.5">
            SuperFan
            {isDemoMode && (
              <span className="text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30">
                DEMO
              </span>
            )}
          </h1>
          <p className="text-[10px] text-slate-400 font-medium">macOS Fan Control</p>
        </div>
      </div>

      <div className="flex items-center gap-1">
        {!hasAccess && !isDemoMode && (
          <div
            title="SMC Access Limited"
            className="flex items-center gap-1 px-2 py-0.5 rounded bg-red-500/20 text-red-300 text-[10px] border border-red-500/30 mr-1"
          >
            <ShieldAlert className="w-3 h-3" />
            Limited
          </div>
        )}

        <button
          onClick={() => setActiveTab(activeTab === "settings" ? "overview" : "settings")}
          className={`p-1.5 rounded-md transition-colors ${
            activeTab === "settings"
              ? "bg-white/15 text-white"
              : "text-slate-400 hover:text-white hover:bg-white/10"
          }`}
          title="Settings"
        >
          <Settings className="w-4 h-4" />
        </button>

        <button
          onClick={onHideWindow}
          className="p-1.5 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
          title="Hide Window"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
