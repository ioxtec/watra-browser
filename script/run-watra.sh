#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="$ROOT_DIR/target"
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
find_cef_runtime() {
    if [ -n "${CEF_PATH:-}" ]; then
        return
    fi
    for candidate in "$ROOT_DIR"/target/debug/build/cef-dll-sys-*/out/cef_macos_aarch64; do
        if [ -d "$candidate/Chromium Embedded Framework.framework" ]; then
            export CEF_PATH="$candidate"
            return
        fi
    done
}
find_cef_runtime
cd "$ROOT_DIR"
cargo build --locked --package watra-desktop
find_cef_runtime
if [ -z "${CEF_PATH:-}" ]; then
    echo "CEF runtime not found. Build the CEF runtime first or set CEF_PATH." >&2
    exit 1
fi
bundle-cef-app watra-desktop -o "$ROOT_DIR/target/watra" -i com.ioxtec.watra -d 'Watra'
APP_BUNDLE="$ROOT_DIR/target/watra/watra-desktop.app"
codesign --force --deep --sign - "$APP_BUNDLE"
if [ "${1:-}" != --build-only ]; then
    open "$APP_BUNDLE"
fi
