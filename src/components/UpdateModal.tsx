import React, { useState } from "react";
import { ReleaseInfo, performAutoUpdate, openReleasePage } from "../services/updater";
import { Sparkles, ExternalLink, Download, X, Calendar, ArrowRight, RefreshCw } from "lucide-react";

interface UpdateModalProps {
  currentVersion: string;
  release: ReleaseInfo;
  onClose: () => void;
}

export const UpdateModal: React.FC<UpdateModalProps> = ({
  currentVersion,
  release,
  onClose,
}) => {
  const [updating, setUpdating] = useState(false);
  const [progressPercent, setProgressPercent] = useState<number | null>(null);
  const [updateError, setUpdateError] = useState<string | null>(null);

  const handleUpdateAction = async () => {
    if (release.tauriUpdate) {
      setUpdating(true);
      setUpdateError(null);
      try {
        await performAutoUpdate(release, (downloaded, total) => {
          if (total && total > 0) {
            setProgressPercent(Math.round((downloaded / total) * 100));
          }
        });
      } catch (err: any) {
        setUpdateError(err?.message || "Auto-update failed. Opening release link instead.");
        setUpdating(false);
        openReleasePage(release.downloadUrl || release.htmlUrl);
      }
    } else {
      openReleasePage(release.downloadUrl || release.htmlUrl);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md animate-fade-in">
      <div className="relative w-full max-w-sm glass-card border border-amber-500/30 shadow-2xl shadow-orange-500/20 rounded-2xl overflow-hidden flex flex-col max-h-[85vh] text-slate-200">
        {/* Top Glow & Header */}
        <div className="bg-gradient-to-r from-amber-500/20 via-red-500/20 to-orange-500/20 p-4 border-b border-white/10 flex items-start justify-between">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-gradient-to-tr from-amber-500 to-orange-500 text-white shadow-lg shadow-amber-500/30">
              <Sparkles className="w-5 h-5 animate-pulse" />
            </div>
            <div>
              <div className="flex items-center gap-1.5">
                <h3 className="text-sm font-bold text-white tracking-wide">Update Available</h3>
                <span className="px-1.5 py-0.5 rounded text-[9px] font-bold bg-amber-500/30 text-amber-300 border border-amber-500/40">
                  New
                </span>
              </div>
              <p className="text-[10px] text-slate-400 font-medium">SuperFan for macOS</p>
            </div>
          </div>

          <button
            onClick={onClose}
            disabled={updating}
            className="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-white/10 transition-colors disabled:opacity-50"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content Body */}
        <div className="p-4 flex flex-col gap-3 overflow-y-auto">
          {/* Version Badge Transition */}
          <div className="glass-card p-3 rounded-xl flex items-center justify-between bg-slate-900/50">
            <div className="text-center">
              <div className="text-[10px] text-slate-400 font-medium">Current</div>
              <div className="text-xs font-mono font-bold text-slate-300">v{currentVersion}</div>
            </div>

            <ArrowRight className="w-4 h-4 text-amber-400" />

            <div className="text-center">
              <div className="text-[10px] text-slate-400 font-medium">Latest</div>
              <div className="text-xs font-mono font-bold text-amber-400">v{release.version}</div>
            </div>

            <div className="border-l border-white/10 pl-3 text-right">
              <div className="flex items-center gap-1 text-[10px] text-slate-400">
                <Calendar className="w-3 h-3 text-slate-500" />
                {release.publishedAt}
              </div>
            </div>
          </div>

          {/* Release Title & Notes */}
          <div>
            <div className="text-xs font-semibold text-white mb-1">{release.name}</div>
            <div className="text-[11px] text-slate-300 glass-card p-3 rounded-xl max-h-36 overflow-y-auto whitespace-pre-wrap font-mono leading-relaxed bg-slate-950/60 border border-white/5">
              {release.body}
            </div>
          </div>

          {/* Download Progress Bar */}
          {updating && (
            <div className="glass-card p-3 rounded-xl flex flex-col gap-2 bg-slate-900/80 border border-amber-500/30">
              <div className="flex items-center justify-between text-xs text-amber-300 font-medium">
                <span className="flex items-center gap-1.5">
                  <RefreshCw className="w-3.5 h-3.5 animate-spin text-amber-400" />
                  Downloading & Installing...
                </span>
                <span className="font-mono">{progressPercent !== null ? `${progressPercent}%` : ""}</span>
              </div>
              <div className="w-full h-1.5 bg-slate-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-amber-500 to-orange-500 transition-all duration-300"
                  style={{ width: `${progressPercent ?? 100}%` }}
                />
              </div>
            </div>
          )}

          {updateError && (
            <div className="p-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-[10px] font-mono text-rose-200">
              {updateError}
            </div>
          )}
        </div>

        {/* Action Buttons */}
        <div className="p-3 bg-slate-950/80 border-t border-white/10 flex items-center justify-end gap-2">
          <button
            onClick={onClose}
            disabled={updating}
            className="px-3 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-white hover:bg-white/10 transition-all disabled:opacity-50"
          >
            Later
          </button>

          <button
            onClick={handleUpdateAction}
            disabled={updating}
            className="px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-gradient-to-r from-amber-500 to-orange-500 text-white hover:from-amber-600 hover:to-orange-600 shadow-md shadow-orange-500/20 flex items-center gap-1.5 transition-all disabled:opacity-50"
          >
            {updating ? (
              <RefreshCw className="w-3.5 h-3.5 animate-spin" />
            ) : release.tauriUpdate ? (
              <Download className="w-3.5 h-3.5" />
            ) : (
              <ExternalLink className="w-3.5 h-3.5" />
            )}
            {updating
              ? "Updating..."
              : release.tauriUpdate
              ? `Install & Relaunch v${release.version}`
              : `Download v${release.version}`}
          </button>
        </div>
      </div>
    </div>
  );
};
