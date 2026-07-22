#!/bin/bash
set -euo pipefail

label="com.superfan.fan-actuation"
source_dir=$(cd "$(dirname "$0")" && pwd -P)
source_helper="$source_dir/fan-actuation-daemon"
source_plist="$source_dir/$label.plist"
destination_helper="/Library/PrivilegedHelperTools/$label"
destination_plist="/Library/LaunchDaemons/$label.plist"
staging_dir=$(mktemp -d /private/tmp/superfan-fan-actuation.XXXXXX)
backup_dir="$staging_dir/backup"
mkdir -p "$backup_dir"

cleanup() {
  rm -rf "$staging_dir"
}
trap cleanup EXIT

[ "$(id -u)" -eq 0 ] || { echo "installer must run as root" >&2; exit 1; }
[ -f "$source_helper" ] && [ ! -L "$source_helper" ] && [ -x "$source_helper" ] || {
  echo "bundled fan actuation helper is missing or unsafe" >&2
  exit 1
}
[ -f "$source_plist" ] && [ ! -L "$source_plist" ] || {
  echo "bundled fan actuation plist is missing or unsafe" >&2
  exit 1
}
/usr/bin/plutil -lint "$source_plist" >/dev/null

/usr/bin/install -o root -g wheel -m 0755 "$source_helper" "$staging_dir/helper"
/usr/bin/install -o root -g wheel -m 0644 "$source_plist" "$staging_dir/service.plist"
/usr/bin/plutil -lint "$staging_dir/service.plist" >/dev/null

/bin/launchctl bootout "system/$label" 2>/dev/null || true
mkdir -p /Library/PrivilegedHelperTools /Library/LaunchDaemons

had_helper=false
had_plist=false
if [ -e "$destination_helper" ]; then
  [ ! -L "$destination_helper" ] || { echo "installed helper path is a symlink" >&2; exit 1; }
  /bin/mv "$destination_helper" "$backup_dir/helper"
  had_helper=true
fi
if [ -e "$destination_plist" ]; then
  [ ! -L "$destination_plist" ] || { echo "installed plist path is a symlink" >&2; exit 1; }
  /bin/mv "$destination_plist" "$backup_dir/service.plist"
  had_plist=true
fi

rollback() {
  /bin/launchctl bootout "system/$label" 2>/dev/null || true
  /bin/rm -f "$destination_helper" "$destination_plist"
  if $had_helper; then /bin/mv "$backup_dir/helper" "$destination_helper"; fi
  if $had_plist; then /bin/mv "$backup_dir/service.plist" "$destination_plist"; fi
  if $had_plist; then
    /bin/launchctl bootstrap system "$destination_plist" 2>/dev/null || true
    /bin/launchctl kickstart -k "system/$label" 2>/dev/null || true
  fi
}

/bin/mv "$staging_dir/helper" "$destination_helper"
/bin/mv "$staging_dir/service.plist" "$destination_plist"
if ! /bin/launchctl bootstrap system "$destination_plist"; then
  rollback
  echo "could not bootstrap Fan actuation helper; previous installation was restored" >&2
  exit 1
fi
if ! /bin/launchctl kickstart -k "system/$label"; then
  rollback
  echo "could not start Fan actuation helper; previous installation was restored" >&2
  exit 1
fi

echo "Fan actuation helper installed"
