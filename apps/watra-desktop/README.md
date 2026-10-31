# Watra Desktop

Watra's native macOS desktop application. The interface is built with GPUI and embeds a CEF Alloy browser view.

The app uses `watra-core` and `watra-engine-api` for engine-independent navigation, with `watra-engine-cef` providing the CEF adapter. Each tab owns an independent CEF browser; switching tabs keeps their pages alive, and closing the final tab opens a fresh new tab. The new-tab page includes Google, YouTube, and Wikipedia shortcuts.

Use `⌘T` to open a tab and `⌘W` to close the active tab. The sidebar also provides controls to create, switch, and close tabs.

## Build and run

Install Xcode with the Metal Toolchain, Rust, CMake, and Ninja. Install the CEF bundler matching the pinned CEF Rust package:

```sh
cargo install cef --version '=156.2.0' --bin bundle-cef-app --locked
```

From the repository root, run:

```sh
./script/run-watra.sh
```

The script builds the app and helper, bundles the pinned CEF runtime, and launches Watra. Pass `--build-only` to create the app bundle without opening it. Set `CEF_PATH` only when the runtime is already installed in a custom location.

## Diagnostics

Watra writes lifecycle, address-editor, CEF startup, and Rust panic markers to `~/Library/Logs/Watra/watra.log`. The log does not include entered addresses or page URLs and is reset when it exceeds 2 MiB. To inspect the latest events, run:

```sh
tail -n 100 ~/Library/Logs/Watra/watra.log
```

Run the standard workspace checks from the repository root:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The helper entry point remains based on the MIT/Apache-2.0 licensed `tauri-apps/cef-rs` cefsimple example. Its upstream license texts are included. GPUI, CEF, and Chromium retain their respective licenses and notices.
