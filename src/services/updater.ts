import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";

export interface ReleaseInfo {
  version: string;
  name: string;
  body: string;
  htmlUrl: string;
  publishedAt: string;
  downloadUrl?: string;
}

export interface UpdateCheckResult {
  hasUpdate: boolean;
  currentVersion: string;
  latestRelease?: ReleaseInfo;
  error?: string;
}

const REPO_OWNER = "minhtri2710";
const REPO_NAME = "superfan";
export const CURRENT_VERSION = "1.2.3";

export async function getAppVersion(): Promise<string> {
  try {
    const v = await getVersion();
    if (v) return cleanVersion(v);
  } catch {
    // fallback
  }
  return CURRENT_VERSION;
}

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
  const currentVersion = await getAppVersion();
  // Strategy 1: GitHub REST API
  try {
    const response = await fetch(
      `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest`,
      {
        headers: {
          Accept: "application/vnd.github.v3+json",
        },
      }
    );

    if (response.ok) {
      const data = await response.json();
      const rawTag = data.tag_name || "";
      const latestVer = cleanVersion(rawTag);

      let downloadUrl: string | undefined = undefined;
      if (Array.isArray(data.assets)) {
        const dmgAsset = data.assets.find((a: any) =>
          a.name?.toLowerCase().endsWith(".dmg")
        );
        const fallbackAsset = data.assets.find((a: any) =>
          a.name?.toLowerCase().endsWith(".zip") || a.name?.toLowerCase().endsWith(".tar.gz")
        );
        const asset = dmgAsset || fallbackAsset;
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

      const hasUpdate = compareVersions(currentVersion, latestVer) > 0;

      return {
        hasUpdate,
        currentVersion,
        latestRelease: releaseInfo,
      };
    }
  } catch {
    // Ignore and proceed to Fallback Strategy
  }

  // Strategy 2: Fallback to Raw GitHub CDN (bypass 403 API rate limit)
  try {
    const rawResponse = await fetch(
      `https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/main/package.json`
    );

    if (rawResponse.ok) {
      const pkg = await rawResponse.json();
      const latestVer = cleanVersion(pkg.version || currentVersion);
      const hasUpdate = compareVersions(currentVersion, latestVer) > 0;

      return {
        hasUpdate,
        currentVersion,
        latestRelease: hasUpdate
          ? {
              version: latestVer,
              name: `SuperFan v${latestVer}`,
              body: "A new version of SuperFan is available on GitHub Releases.",
              htmlUrl: `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/tag/v${latestVer}`,
              publishedAt: "Latest",
              downloadUrl: `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest`,
            }
          : undefined,
      };
    }
  } catch (rawErr: any) {
    return {
      hasUpdate: false,
      currentVersion,
      error: rawErr?.message || "Failed to check for updates.",
    };
  }

  return {
    hasUpdate: false,
    currentVersion,
    error: "GitHub API rate limit reached (HTTP 403). Try again later.",
  };
}

export async function performAutoInstall(downloadUrl: string): Promise<void> {
  await invoke("install_app_update", { downloadUrl });
}

export async function openReleasePage(url: string) {
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank");
  }
}
