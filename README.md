# Watra Browser

A Rust desktop browser project using prebuilt Chromium Embedded Framework (CEF) binaries.

## Workspace

- `watra-engine-api`: engine-independent browser contracts.
- `watra-core`: application state and use cases.
- `watra-engine-cef`: optional CEF adapter.
- `watra-desktop`: application entry point, currently a console scaffold.

## Development

```sh
cargo run -p watra-desktop
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The desktop scaffold does not open a browser window yet.

## CEF

The adapter uses `cef 156.2.0`, resolved in `Cargo.lock` to bindings for CEF 156.0.14. CEF binaries are downloaded on demand; Chromium source code is not required.

With CMake, Ninja, and platform build tools installed:

```sh
cargo check -p watra-engine-cef --features cef-runtime
```

On macOS, running a CEF application requires an application bundle containing the framework, resources, and helper applications. Ordinary workspace tests do not download or initialize CEF.
