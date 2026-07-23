import { openUrl } from "@tauri-apps/plugin-opener";

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
const CURRENT_VERSION = "1.0.6";

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

export async function openReleasePage(url: string) {
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank");
  }
}
