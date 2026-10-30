# Źródła silników

Ten katalog przechowuje wyłącznie małe, przeglądalne manifesty. Źródła i binaria są pobierane do `.watra/`, który jest ignorowany przez Git.

## Zwykłe repozytorium Git

Skopiuj `engines/_template` do `engines/<nazwa>` i uzupełnij trzy pliki w katalogu `source`:

- `repository` — oficjalny adres HTTPS repozytorium;
- `revision` — pełny, 40-znakowy identyfikator commita;
- `paths` — po jednej potrzebnej ścieżce na linię.

Następnie uruchom:

```sh
./tools/fetch-engine-source.sh <nazwa>
```

Skrypt nie pobiera historii i materializuje tylko ścieżki wymienione w manifeście. Zmiana wersji polega na aktualizacji `revision` i ponownym uruchomieniu skryptu.

## Chromium

Chromium nie jest zwykłym kandydatem do sparse checkoutu. Oficjalne repozytorium wymaga `depot_tools`/`gclient`, a plik `DEPS` przypina wiele repozytoriów i artefaktów niezbędnych do kompilacji. Dla adaptera opartego na Chromium należy przygotować osobny bootstrap, który:

1. preferuje oficjalny, gotowy SDK/runtime, jeśli wybrana integracja go udostępnia;
2. przypina wersję i sumy kontrolne pobieranych artefaktów;
3. korzysta z oficjalnego checkoutu bez historii tylko wtedy, gdy kompilacja ze źródeł jest konieczna;
4. zapisuje wszystko w `.watra/`, nigdy w historii Watry.

