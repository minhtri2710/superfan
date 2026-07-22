#!/bin/bash
# SuperFan macOS DMG Release Build Script

set -e

echo "🚀 Building SuperFan macOS App Bundle & DMG..."
echo "============================================="

# Ensure target directories exist
mkdir -p releases

# Build Frontend
echo "📦 Building Frontend..."
npm run build

# Build Tauri Desktop DMG
echo "🦀 Compiling Rust Backend & Packaging DMG..."
npm run tauri build

DMG_PATH="src-tauri/target/release/bundle/dmg"

if [ -d "$DMG_PATH" ]; then
  cp "$DMG_PATH"/*.dmg releases/
  echo ""
  echo "✅ SuperFan macOS DMG build complete!"
  echo "   Location: releases/"
  ls -lh releases/
else
  echo ""
  echo "✅ Build completed successfully! Check src-tauri/target/release/bundle/ for output."
fi
