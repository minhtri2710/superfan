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

build_args=(--bin fan-actuation-daemon --features daemon-bin)
if [ "$profile" = release ]; then
  build_args+=(--release)
fi

if [ "${TAURI_ENV_ARCH:-}" = "universal" ] || [ -d "$target_dir/universal-apple-darwin/$profile" ]; then
  arm_binary="$target_dir/aarch64-apple-darwin/$profile/fan-actuation-daemon"
  intel_binary="$target_dir/x86_64-apple-darwin/$profile/fan-actuation-daemon"
  [ -x "$arm_binary" ] || cargo build "${build_args[@]}" --target aarch64-apple-darwin
  [ -x "$intel_binary" ] || cargo build "${build_args[@]}" --target x86_64-apple-darwin
  mkdir -p "$target_dir/universal-apple-darwin/$profile"
  binary="$target_dir/universal-apple-darwin/$profile/fan-actuation-daemon"
  lipo -create "$arm_binary" "$intel_binary" -output "$binary"
elif [ -n "${TAURI_ENV_ARCH:-}" ] && [ -d "$target_dir/${TAURI_ENV_ARCH}-apple-darwin/$profile" ]; then
  binary="$target_dir/${TAURI_ENV_ARCH}-apple-darwin/$profile/fan-actuation-daemon"
  [ -x "$binary" ] || cargo build "${build_args[@]}" --target "${TAURI_ENV_ARCH}-apple-darwin"
else
  binary="$target_dir/$profile/fan-actuation-daemon"
  [ -x "$binary" ] || cargo build "${build_args[@]}"
fi

mkdir -p bundle/macos/resources
install -m 755 "$binary" bundle/macos/resources/fan-actuation-daemon
