#!/bin/bash
set -euo pipefail

app=${1:?usage: verify-fan-actuation-bundle.sh /path/to/SuperFan.app [--registration-capable]}
registration_capable=${2:-}
plist="$app/Contents/Library/LaunchDaemons/com.superfan.fan-actuation.plist"

fail() {
  echo "fan actuation bundle verification failed: $*" >&2
  exit 1
}

[ -f "$plist" ] || fail "missing LaunchDaemon plist"
plutil -lint "$plist" >/dev/null
[ "$(plutil -extract Label raw "$plist")" = "com.superfan.fan-actuation" ] || fail "unexpected Label"

associated=$(plutil -extract AssociatedBundleIdentifiers.0 raw "$plist" 2>/dev/null || true)
[ "$associated" = "com.superfan.app" ] || fail "AssociatedBundleIdentifiers must contain com.superfan.app"

program=$(plutil -extract BundleProgram raw "$plist")
case "$program" in
  /*|*../*) fail "BundleProgram must stay relative and inside the app bundle" ;;
esac
[ "$program" = "Contents/Resources/fan-actuation-daemon" ] || fail "BundleProgram must use the documented auxiliary resource path"
helper="$app/$program"
[ -x "$helper" ] || fail "BundleProgram does not resolve to an executable"

codesign --verify --strict --verbose=2 "$helper"
codesign --verify --deep --strict --verbose=2 "$app"

if [ "$registration_capable" = "--registration-capable" ]; then
  app_team=$(codesign -dvv "$app" 2>&1 | awk -F= '/^TeamIdentifier=/{print $2}')
  helper_team=$(codesign -dvv "$helper" 2>&1 | awk -F= '/^TeamIdentifier=/{print $2}')
  [ -n "$app_team" ] && [ "$app_team" != "not set" ] || fail "app is ad-hoc signed; Apple signing is required for registration"
  [ "$helper_team" = "$app_team" ] || fail "app and helper TeamIdentifier values differ"
fi

echo "fan actuation bundle verified: $app"
