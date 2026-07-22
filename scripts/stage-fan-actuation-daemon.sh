#!/bin/bash
set -euo pipefail

if [ "${TAURI_ENV_PLATFORM:-}" != "darwin" ]; then
  exit 0
fi

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root/src-tauri"

profile=release
if [ "${TAURI_ENV_DEBUG:-false}" = "true" ]; then
  profile=debug
fi

target_dir=${CARGO_TARGET_DIR:-target}
case "$target_dir" in
  /*) ;;
  *) target_dir="$PWD/$target_dir" ;;
esac

if [ -n "${TAURI_ENV_ARCH:-}" ] && [ -d "$target_dir/${TAURI_ENV_ARCH}-apple-darwin/$profile" ]; then
  binary="$target_dir/${TAURI_ENV_ARCH}-apple-darwin/$profile/fan-actuation-daemon"
elif [ -d "$target_dir/universal-apple-darwin/$profile" ]; then
  binary="$target_dir/universal-apple-darwin/$profile/fan-actuation-daemon"
else
  binary="$target_dir/$profile/fan-actuation-daemon"
fi

if [ ! -x "$binary" ]; then
  build_args=(--bin fan-actuation-daemon --features daemon-bin)
  if [ "$profile" = release ]; then
    build_args+=(--release)
  fi
  cargo build "${build_args[@]}"
fi

mkdir -p bundle/macos/resources
install -m 755 "$binary" bundle/macos/resources/fan-actuation-daemon
