#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="aarch64-apple-darwin"
BIN_NAME="treefold-gui"
APP_NAME="treefold-gui"
APP_DIR="${ROOT_DIR}/dist/${APP_NAME}.app"
BIN_PATH="${ROOT_DIR}/target/${TARGET}/release/${BIN_NAME}"
RES_DIR="${APP_DIR}/Contents/Resources"
MACOS_DIR="${APP_DIR}/Contents/MacOS"

cd "${ROOT_DIR}"
cargo build --release --target "${TARGET}" --bin "${BIN_NAME}"

mkdir -p "${RES_DIR}" "${MACOS_DIR}"
cp "${BIN_PATH}" "${MACOS_DIR}/${BIN_NAME}"

if [[ -f "${ROOT_DIR}/assets/treefold.icns" ]]; then
  cp "${ROOT_DIR}/assets/treefold.icns" "${RES_DIR}/treefold.icns"
fi

cat > "${APP_DIR}/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>treefold-gui</string>
  <key>CFBundleDisplayName</key>
  <string>treefold-gui</string>
  <key>CFBundleIdentifier</key>
  <string>com.treefold.gui</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleExecutable</key>
  <string>treefold-gui</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleIconFile</key>
  <string>treefold.icns</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
</dict>
</plist>
PLIST

chmod +x "${MACOS_DIR}/${BIN_NAME}"
echo "Built ${APP_DIR}"
echo "Binary: ${BIN_PATH}"
