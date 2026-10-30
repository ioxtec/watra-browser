# Watra desktop prototype

Experimental macOS Apple Silicon shell combining GPUI with a native CEF Alloy child view.

The prototype includes integrated macOS window controls, a compact navigation toolbar, a collapsible sidebar, a constrained resize divider, and a local CEF test page. Toolbar navigation calls CEF; the address display is not an editable input yet. The settings button opens a placeholder window.

## Build

Install Xcode with the Metal Toolchain, Rust, CMake, and Ninja. Install the matching upstream bundler:

```sh
cargo install cef --version '=156.2.0' --bin bundle-cef-app --locked
```

From this directory, build the prototype first so the pinned CEF binary distribution is available in Cargo's build directory:

```sh
export CARGO_TARGET_DIR="$(cd ../.. && pwd)/target"
cargo build --locked
```

Set `CEF_PATH` to the downloaded `cef_macos_aarch64` directory under `target/debug/build/cef-dll-sys-*/out/`, containing `Chromium Embedded Framework.framework`. Then package and launch:

```sh
bundle-cef-app watra-gpui-prototype -o "$CARGO_TARGET_DIR/gpui-sidebar" -i com.ioxtec.watra.sidebar -d 'Watra Sidebar'
open "$CARGO_TARGET_DIR/gpui-sidebar/watra-gpui-prototype.app"
```

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

This is a UI checkpoint, not a release-ready browser. Native input integration and orderly CEF shutdown still need work. The prototype uses a separate temporary CEF profile and a simple periodic external message pump. Runtime tests must cover keyboard focus, repeated sidebar drags, window resizing, and helper-process termination before production use.

The helper entry point is derived from the MIT/Apache-2.0 licensed `tauri-apps/cef-rs` cefsimple example. The upstream license texts are included. GPUI, CEF, and Chromium retain their respective licenses and notices.
