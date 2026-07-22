#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
export TS_RS_EXPORT_DIR="bindings"
RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}" cargo test --manifest-path src-tauri/Cargo.toml export_bindings -j 1
mkdir -p src/generated
cp src-tauri/bindings/*.ts src/generated/
