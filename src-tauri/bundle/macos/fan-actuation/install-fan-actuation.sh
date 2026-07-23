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

mutation_started=false
committed=false
had_helper=false
had_plist=false

rollback() {
  /bin/launchctl bootout "system/$label" 2>/dev/null || true
  /bin/rm -f "$destination_helper" "$destination_plist"
  if $had_helper && [ -e "$backup_dir/helper" ]; then /bin/mv "$backup_dir/helper" "$destination_helper"; fi
  if $had_plist && [ -e "$backup_dir/service.plist" ]; then /bin/mv "$backup_dir/service.plist" "$destination_plist"; fi
  if $had_plist; then
    /bin/launchctl enable "system/$label" 2>/dev/null || true
    /bin/launchctl bootstrap system "$destination_plist" 2>/dev/null || true
    /bin/launchctl kickstart -k "system/$label" 2>/dev/null || true
  fi
}

finalize() {
  status=$?
  if $mutation_started && ! $committed; then rollback; fi
  rm -rf "$staging_dir"
  exit "$status"
}
trap finalize EXIT

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
[ "$(/usr/bin/plutil -extract Label raw "$source_plist")" = "$label" ] || {
  echo "bundled fan actuation plist has the wrong Label" >&2
  exit 1
}
[ "$(/usr/bin/plutil -extract ProgramArguments.0 raw "$source_plist")" = "$destination_helper" ] || {
  echo "bundled fan actuation plist has the wrong helper path" >&2
  exit 1
}
if /usr/bin/plutil -extract BundleProgram raw "$source_plist" >/dev/null 2>&1; then
  echo "bundled fan actuation plist must not contain BundleProgram" >&2
  exit 1
fi

if [ -e "$destination_helper" ] && [ -L "$destination_helper" ]; then
  echo "installed helper path is a symlink" >&2
  exit 1
fi
if [ -e "$destination_plist" ] && [ -L "$destination_plist" ]; then
  echo "installed plist path is a symlink" >&2
  exit 1
fi

/usr/bin/install -o root -g wheel -m 0755 "$source_helper" "$staging_dir/helper"
/usr/bin/install -o root -g wheel -m 0644 "$source_plist" "$staging_dir/service.plist"
/usr/bin/plutil -lint "$staging_dir/service.plist" >/dev/null

mutation_started=true
/bin/launchctl bootout "system/$label" 2>/dev/null || true
/bin/launchctl disable "system/$label" 2>/dev/null || true

# Wait up to 2 seconds for launchd to finish tearing down any previous instance
for i in 1 2 3 4 5 6 7 8 9 10; do
  if ! /bin/launchctl print "system/$label" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

mkdir -p /Library/PrivilegedHelperTools /Library/LaunchDaemons

if [ -e "$destination_helper" ]; then
  /bin/mv "$destination_helper" "$backup_dir/helper"
  had_helper=true
fi
if [ -e "$destination_plist" ]; then
  /bin/mv "$destination_plist" "$backup_dir/service.plist"
  had_plist=true
fi

/bin/mv "$staging_dir/helper" "$destination_helper"
/bin/mv "$staging_dir/service.plist" "$destination_plist"

/bin/launchctl enable "system/$label" 2>/dev/null || true

if ! /bin/launchctl bootstrap system "$destination_plist"; then
  # If launchd considers the service registered, attempt kickstart or retry
  if /bin/launchctl print "system/$label" >/dev/null 2>&1; then
    /bin/launchctl kickstart -k "system/$label" 2>/dev/null || true
  else
    sleep 1
    if ! /bin/launchctl bootstrap system "$destination_plist"; then
      echo "could not bootstrap Fan actuation helper; previous installation was restored" >&2
      exit 1
    fi
  fi
fi

if ! /bin/launchctl kickstart -k "system/$label"; then
  if ! /bin/launchctl print "system/$label" >/dev/null 2>&1; then
    echo "could not start Fan actuation helper; previous installation was restored" >&2
    exit 1
  fi
fi

committed=true
echo "Fan actuation helper installed"
