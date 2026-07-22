#!/bin/bash
set -euo pipefail

app=${1:?usage: verify-fan-actuation-bundle.sh /path/to/SuperFan.app}
resources="$app/Contents/Resources/fan-actuation"
plist="$resources/com.superfan.fan-actuation.plist"
helper="$resources/fan-actuation-daemon"
installer="$resources/install-fan-actuation.sh"
authorizer="$resources/authorize-install.applescript"

fail() {
  echo "fan actuation bundle verification failed: $*" >&2
  exit 1
}

[ ! -e "$app/Contents/Library/LaunchDaemons/com.superfan.fan-actuation.plist" ] || fail "SMAppService plist must not be bundled"
[ ! -e "$app/Contents/MacOS/fan-actuation-daemon" ] || fail "helper must not be a second app executable"
[ -x "$helper" ] || fail "missing executable helper resource"
[ -x "$installer" ] || fail "missing executable installer resource"
[ -r "$authorizer" ] || fail "missing AppleScript authorization resource"
[ -r "$plist" ] || fail "missing traditional launchd plist resource"
plutil -lint "$plist" >/dev/null

program=$(plutil -extract ProgramArguments.0 raw "$plist")
[ "$program" = "/Library/PrivilegedHelperTools/com.superfan.fan-actuation" ] || fail "unexpected installed helper path"
if plutil -extract BundleProgram raw "$plist" >/dev/null 2>&1; then
  fail "traditional plist must not contain BundleProgram"
fi

grep -Fq 'with administrator privileges' "$authorizer" || fail "authorizer does not request administrator privileges"
grep -Fq 'quoted form of (item 1 of argv)' "$authorizer" || fail "installer path is not safely quoted"
grep -Fq 'destination_helper="/Library/PrivilegedHelperTools/$label"' "$installer" || fail "installer destination is missing"
grep -Fq 'destination_plist="/Library/LaunchDaemons/$label.plist"' "$installer" || fail "plist destination is missing"

codesign --verify --deep --strict --verbose=2 "$app"
echo "fan actuation installer bundle verified: $app"
