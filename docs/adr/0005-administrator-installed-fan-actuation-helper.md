---
status: accepted
supersedes: 0001
---

# Install the Fan actuation helper with administrator authorization

SuperFan will preserve the existing Fan actuation runtime interface and Unix socket seam while replacing the `SMAppService` bootstrap adapter with an administrator-authorized installer. The app invokes a fixed AppleScript with the bundled installer path as an argument; the root installer validates regular non-symlinked resources, stages root-owned files, stops the existing daemon so it restores System Auto, atomically installs the helper in `/Library/PrivilegedHelperTools` and its plist in `/Library/LaunchDaemons`, and rolls back if launchd bootstrap or startup fails. This removes the Apple signing identity requirement for helper installation, but every install or update requires an administrator password prompt, and System Settings may temporarily retain a stale entry from older `SMAppService` versions.
