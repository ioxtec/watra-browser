# Watra CEF prototype

Standalone macOS lifecycle prototype derived from tauri-apps/cef-rs examples/cefsimple. Upstream MIT and Apache licenses are included. The original cefsimple target names are retained for compatibility with upstream bundle tooling.

This prototype exercises native CEF windows and helper processes. It is separate from Watra's application core; connecting the core is a subsequent step.

Build the bundler once:

```sh
cargo install cef --version '=156.2.0' --bin bundle-cef-app --locked
```

Then from this directory:

```sh
bundle-cef-app cefsimple -o target/bundle
open target/bundle/cefsimple.app --args --url=https://example.com
```
