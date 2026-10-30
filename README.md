# Watra Browser

Minimalny szkielet przeglądarki napisanej w Rust. Repozytorium jest workspace'em Cargo i na tym etapie świadomie nie wybiera biblioteki UI ani silnika renderującego.

## Struktura

- `crates/watra-engine-api` — niewielki kontrakt pomiędzy aplikacją a przyszłym adapterem silnika.
- `crates/watra-engine-cef` — adapter CEF; zależność od runtime'u jest opcjonalna i przypięta do konkretnej wersji.
- `crates/watra-core` — logika aplikacyjna niezależna od UI i konkretnego silnika.
- `apps/watra-desktop` — punkt składania aplikacji desktopowej; obecnie używa pustego adaptera demonstracyjnego.
- `engines` — manifesty źródeł silników; bez samych źródeł i binariów.
- `tools` — narzędzia do kontrolowanego pobierania zależności deweloperskich.
- `docs/adr` — decyzje architektoniczne.

## Wymagania

- Rust 1.85 lub nowszy (edycja 2024).

## Uruchomienie

```sh
cargo run -p watra-desktop
```

Program potwierdzi uruchomienie szkieletu. Nie otwiera jeszcze okna ani strony internetowej.

## Źródła silników

Repozytorium Watry nie przechowuje kopii silników. Dla zwykłego repozytorium Git można dodać mały manifest na podstawie `engines/_template/source` i pobrać tylko wskazane ścieżki:

```sh
./tools/fetch-engine-source.sh nazwa-silnika
```

Narzędzie używa partial clone, sparse checkout i przypiętego pełnego identyfikatora commita. Pliki trafiają do ignorowanego katalogu `.watra/engine-sources`, więc nie zwiększają rozmiaru repozytorium Watry.

Chromium jest wyjątkiem: jego oficjalne repozytorium wymaga `depot_tools` i `gclient`, aby odtworzyć zależności z `DEPS`. Nie należy pobierać go tym ogólnym skryptem. Gdy wybierzemy konkretną integrację, dodamy osobny bootstrap pobierający minimalny obsługiwany SDK/runtime albo checkout bez historii wymagany przez oficjalne narzędzia.

Pierwszym wybranym adapterem jest CEF. Zwykłe testy workspace'u nie pobierają jego binariów. Pierwsze sprawdzenie adaptera wraz z przypiętym runtime'em wykonuje się świadomie:

```sh
cargo check -p watra-engine-cef --features cef-runtime
```

CEF wymaga CMake, Ninja oraz narzędzi kompilacyjnych platformy. Na macOS gotowa aplikacja będzie dodatkowo wymagać poprawnego bundle'a z frameworkiem i procesami pomocniczymi; nie uruchamiamy CEF jako nieopakowanego pliku wykonywalnego.

## Testy i kontrola jakości

```sh
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Następne decyzje

Dobór UI (np. GPUI) powinien nastąpić osobno, po przygotowaniu małego eksperymentu technicznego. Granice opisuje [ADR-0001](docs/adr/0001-architecture-boundaries.md), zasady pobierania zewnętrznych źródeł [ADR-0002](docs/adr/0002-engine-source-acquisition.md), a wybór pierwszego adaptera [ADR-0003](docs/adr/0003-cef-first-engine.md).


