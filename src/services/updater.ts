import { openUrl } from "@tauri-apps/plugin-opener";
import { check as checkTauriUpdate, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface ReleaseInfo {
  version: string;
  name: string;
  body: string;
  htmlUrl: string;
  publishedAt: string;
  downloadUrl?: string;
  tauriUpdate?: Update;
}

export interface UpdateCheckResult {
  hasUpdate: boolean;
  currentVersion: string;
  latestRelease?: ReleaseInfo;
  error?: string;
}

const REPO_OWNER = "minhtri2710";
const REPO_NAME = "superfan";
const CURRENT_VERSION = "1.0.4";

export function cleanVersion(v: string): string {
  return v.replace(/^v/i, "").trim();
}

export function compareVersions(v1: string, v2: string): number {
  const n1 = cleanVersion(v1).split(".").map(Number);
  const n2 = cleanVersion(v2).split(".").map(Number);
  const maxLen = Math.max(n1.length, n2.length);

  for (let i = 0; i < maxLen; i++) {
    const num1 = n1[i] || 0;
    const num2 = n2[i] || 0;
    if (num2 > num1) return 1;
    if (num2 < num1) return -1;
  }
  return 0;
}

export async function checkForUpdates(): Promise<UpdateCheckResult> {
  // 1. Try Native Tauri Updater first
  try {
    const update = await checkTauriUpdate();
    if (update) {
      return {
        hasUpdate: true,
        currentVersion: CURRENT_VERSION,
        latestRelease: {
          version: update.version,
          name: `SuperFan v${update.version}`,
          body: update.body || "New update available with performance & stability improvements.",
          htmlUrl: `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/tag/v${update.version}`,
          publishedAt: update.date || new Date().toLocaleDateString(),
          tauriUpdate: update,
        },
      };
    }
  } catch {
    // Native updater fallback to GitHub Releases API
  }

  // 2. Fallback to GitHub Releases API
  try {
    const response = await fetch(
      `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest`,
      {
        headers: {
          Accept: "application/vnd.github.v3+json",
        },
      }
    );

    if (!response.ok) {
      return {
        hasUpdate: false,
        currentVersion: CURRENT_VERSION,
        error: response.status === 404 ? "No releases found." : `GitHub error (${response.status})`,
      };
    }

    const data = await response.json();
    const rawTag = data.tag_name || "";
    const latestVer = cleanVersion(rawTag);

    let downloadUrl: string | undefined = undefined;
    if (Array.isArray(data.assets)) {
      const asset = data.assets.find((a: any) =>
        a.name?.endsWith(".dmg") || a.name?.endsWith(".zip") || a.name?.endsWith(".tar.gz")
      );
      if (asset) {
        downloadUrl = asset.browser_download_url;
      }
    }

    const releaseInfo: ReleaseInfo = {
      version: latestVer,
      name: data.name || rawTag,
      body: data.body || "No release notes provided.",
      htmlUrl: data.html_url || `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases`,
      publishedAt: data.published_at
        ? new Date(data.published_at).toLocaleDateString()
        : "Recently",
      downloadUrl,
    };

    const hasUpdate = compareVersions(CURRENT_VERSION, latestVer) > 0;

    return {
      hasUpdate,
      currentVersion: CURRENT_VERSION,
      latestRelease: releaseInfo,
    };
  } catch (err: any) {
    return {
      hasUpdate: false,
      currentVersion: CURRENT_VERSION,
      error: err?.message || "Failed to check for updates.",
    };
  }
}

export async function performAutoUpdate(
  release: ReleaseInfo,
  onProgress?: (downloaded: number, contentLength?: number) => void
): Promise<void> {
  if (release.tauriUpdate) {
    let downloaded = 0;
    let contentLength: number | undefined;

    await release.tauriUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          contentLength = event.data.contentLength;
          if (onProgress) onProgress(0, contentLength);
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          if (onProgress) onProgress(downloaded, contentLength);
          break;
        case "Finished":
          if (onProgress) onProgress(downloaded || 100, contentLength || 100);
          break;
      }
    });

    await relaunch();
  } else {
    // If native updater package is not attached, open download URL / release page
    const targetUrl = release.downloadUrl || release.htmlUrl;
    try {
      await openUrl(targetUrl);
    } catch {
      window.open(targetUrl, "_blank");
    }
  }
}

export async function openReleasePage(url: string) {
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank");
  }
}
