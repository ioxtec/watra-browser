# CEF

Watra zaczyna od gotowej dystrybucji Chromium Embedded Framework zamiast źródeł Chromium.

## Przypięta wersja

- crate Rust: `cef 156.2.0+156.0.14`
- CEF: `156.0.14+g4fa4ab5+chromium-156.0.8078.48`
- kanał oficjalnego buildera: `stable`
- Chromium: `156.0.8078.48`

Crate `cef` pobiera odpowiadającą mu oficjalną dystrybucję binarną dopiero po włączeniu feature'a `cef-runtime`. Zwykłe `cargo test --workspace` nie pobiera CEF.

## macOS ARM64

Minimalne oficjalne archiwum ma nazwę:

`cef_binary_156.0.14+g4fa4ab5+chromium-156.0.8078.48_macosarm64_minimal.tar.bz2`

Indeks Spotify CEF Builds podaje dla niego SHA-1 `7359a9d293dd5c480bec3f18b762731b483b0e13` i rozmiar 134887274 bajtów. Downloader `cef-rs` sprawdza metadane dystrybucji i dopasowanie wersji.

## Pierwsze sprawdzenie runtime'u

Po zainstalowaniu CMake i Ninja:

```sh
cargo check -p watra-engine-cef --features cef-runtime
```

Samo sprawdzenie crate'a nie tworzy jeszcze uruchamialnego bundle'a aplikacji. Na macOS potrzebne będą framework, zasoby i helper apps CEF; zostaną dodane razem z pierwszym pionowym prototypem okna.

