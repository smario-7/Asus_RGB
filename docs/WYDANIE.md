# Wydawanie wersji a-rgb

Instrukcja dla opiekuna repozytorium: jak zbudować paczki lokalnie, jak je przetestować i jak opublikować Release na GitHubie.

## Co powstaje

| Plik | Skrypt | Zawartość |
|---|---|---|
| `a-rgb_WERSJA_amd64.deb` | `packaging/build-deb.sh` | binarka, reguła udev, usługa `restore` (patrz [PAKIET.md](PAKIET.md)) |
| `a-rgb-WERSJA-x86_64-linux.tar.gz` | `packaging/build-tarball.sh` | binarka, `install.sh`, reguła, jednostka, README, LICENSE |
| `SHA256SUMS` | workflow | sumy kontrolne obu plików |

Oba pliki zawierają tę samą **statyczną** binarkę zbudowaną dla celu `x86_64-unknown-linux-musl`. Dzięki temu nie zależy ona od wersji glibc w systemie. Wersja paczek pochodzi z `version` w `Cargo.toml`.

## Budowa lokalna

Wymagania: Rust ≥ 1.85, `dpkg-deb`, target musl.

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --locked --target x86_64-unknown-linux-musl
BIN=target/x86_64-unknown-linux-musl/release/a-rgb
ldd "$BIN"                                   # oczekiwane: "not a dynamic executable"
packaging/build-deb.sh     "$BIN" dist
packaging/build-tarball.sh "$BIN" dist
(cd dist && sha256sum -- *.deb *.tar.gz > SHA256SUMS)
```

Katalog `dist/` jest w `.gitignore`, więc paczki nie trafią do commita.

Zawartość i metadane paczki: `dpkg-deb -I dist/a-rgb_*.deb` oraz `dpkg-deb -c dist/a-rgb_*.deb`.

## Test instalacji w kontenerach

Instalacja w kontenerze nie dotyka Twojego systemu. **Uwaga:** kontener współdzieli `/sys` z hostem, więc nie wywołuj w nim poleceń zapisujących (`a-rgb red` itp.) — domyślnie `/sys` jest w kontenerze tylko do odczytu, ale nie polegaj na tym. Sprawdzaj tylko instalację, `--version`, `-h` i usunięcie.

```bash
for img in ubuntu:22.04 ubuntu:24.04; do
  docker run --rm -v "$PWD/dist:/dist:ro" "$img" bash -c '
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq && apt-get install -y -qq /dist/a-rgb_*_amd64.deb
    a-rgb --version && a-rgb -h | head -3
    dpkg -L a-rgb
    apt-get remove -y -qq a-rgb && ls /usr/bin/a-rgb 2>&1'
done
```

Obrazy bazowe nie mają `systemctl`, więc `systemctl --global enable` w `postinst` jest w nich pomijane (`|| true`). Dowiązanie `/etc/systemd/user/default.target.wants/a-rgb-restore.service` sprawdzisz po doinstalowaniu pakietu `systemd` przed paczką.

Archiwum `tar.gz` testuj tak samo (`alpine:3`, `debian:12`): rozpakuj, uruchom `./install.sh`, sprawdź pliki, `./install.sh --uninstall`.

## Wydanie

1. Podbij `version` w `Cargo.toml`, uruchom `cargo build` (aktualizuje `Cargo.lock`).
2. Sprawdź lokalnie:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --locked -- -D warnings
   cargo test --locked
   ```
3. Zrób commit ze zmianą wersji i wypchnij go na `main`. Poczekaj na zielone CI (`.github/workflows/ci.yml`).
4. Zrób tag, **sprawdź, że wskazuje na najnowszy commit**, i dopiero wypchnij go:
   ```bash
   git tag -a vX.Y.Z -m "a-rgb X.Y.Z"
   [ "$(git rev-parse "vX.Y.Z^{commit}")" = "$(git rev-parse HEAD)" ] && echo ZGODNE || echo RÓŻNE   # musi być ZGODNE
   git push origin vX.Y.Z
   ```
   Release powstaje z commita, na który wskazuje tag. Tag założony przed zatwierdzeniem ostatnich zmian wydaje starą wersję (tak było z `v0.1.0`, którego paczki nie zawierały poprawki z `edd3421` — dlatego powstało `v0.1.1`).
5. Workflow `.github/workflows/release.yml` sam:
   - sprawdza, że tag zgadza się z `version` w `Cargo.toml` (tag `v0.1.0-rc1` pasuje do wersji `0.1.0`),
   - uruchamia fmt, clippy i testy,
   - buduje statyczną binarkę i sprawdza, że jest statyczna,
   - buduje `.deb` i `tar.gz`, liczy `SHA256SUMS`,
   - tworzy Release z opisem z `.github/release-notes.md` (tagi z myślnikiem, np. `-rc1`, oznacza jako pre-release).
6. Zweryfikuj opublikowany Release:
   ```bash
   gh release view vX.Y.Z
   mkdir /tmp/rel && cd /tmp/rel && gh release download vX.Y.Z
   sha256sum -c SHA256SUMS
   ```
   i zainstaluj pobrany `.deb` w czystym kontenerze jak wyżej.

### Zmiany w workflowach: najpierw pre-release

Workflowy `ci.yml` i `release.yml` przeszły na GitHubie (wydania `v0.1.0-rc1` i `v0.1.0`). Po każdej zmianie w `release.yml` lub w skryptach `packaging/` wydaj najpierw kandydata, np. `vX.Y.Z-rc1`: jeśli coś zawiedzie, naprawiasz workflow bez śladu stabilnego wydania.

### Nieudane wydanie

Usunięcie Release i tagu jest nieodwracalne dla osób, które zdążyły je pobrać. Rób to tylko dla wydania, które nie powinno istnieć:

```bash
gh release delete vX.Y.Z --yes
git push --delete origin vX.Y.Z && git tag -d vX.Y.Z
```

## Bezpieczeństwo workflowów

- Jedyna zewnętrzna akcja to oficjalne `actions/checkout`, **przypięte do SHA** (komentarz przy SHA podaje wersję). Aktualizacja: `gh api repos/actions/checkout/releases/latest --jq .tag_name`, potem `gh api repos/actions/checkout/commits/TAG --jq .sha` i podmiana SHA w obu plikach.
- Uprawnienie `contents: write` ma tylko job wydania; reszta działa z `contents: read`.
- Nazwa tagu trafia do skryptów przez zmienną środowiskową `TAG`, a nie przez bezpośrednie wstawienie wyrażenia, żeby nazwa tagu nie mogła wstrzyknąć poleceń.
- Release publikuje `gh` preinstalowany na runnerze, bez akcji firm trzecich.

## Zakres zgodności w opisie Release

Szablon `.github/release-notes.md` mówi wprost, że oprogramowanie jest testowane tylko na ASUS TUF Gaming F15 FX506LHB (Ubuntu 26.04, GNOME/Wayland). Nie rozszerzaj tego stwierdzenia, dopóki ktoś nie sprawdzi innego modelu na sprzęcie.
