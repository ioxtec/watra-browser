# Watra Browser

A Rust desktop browser built with GPUI and prebuilt Chromium Embedded Framework (CEF) binaries.

## Workspace

- `watra-engine-api`: engine-neutral browser session contract.
- `watra-core`: application logic and runtime engine selection.
- `watra-engine-cef`: CEF adapter for navigation and session state.
- `watra-desktop`: GPUI application and platform-specific browser host.

The shared engine API and application core are designed to work across operating systems and rendering engines. The current desktop host is macOS-only; Windows and Linux hosts have not been implemented. AppKit integration lives under `apps/watra-desktop/src/platform/macos`.

The desktop shell currently supports multiple CEF-backed tabs, tab switching, tab closing, and a local new-tab page with a small set of shortcuts. Each tab owns an independent CEF browser so its page state remains available when switching tabs.

## Development

Run the macOS application with:

```sh
./script/run-watra.sh
```

Run workspace checks with:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

CI tests the engine API and core on macOS, Linux, and Windows. The desktop host currently builds only on macOS.

## CEF

The adapter pins `cef 156.2.0`, which provides bindings for CEF 156.0.14. CEF binaries are fetched by the Rust package as needed; Chromium source code is not required. The desktop app embeds CEF as a native child view and bundles the framework, resources, and helper applications.
