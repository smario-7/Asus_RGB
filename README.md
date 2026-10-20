# a-rgb

Małe narzędzie CLI (Rust, tylko biblioteka standardowa) do sterowania podświetleniem klawiatury w ASUS TUF Gaming F15 FX506LHB na Ubuntu. Bez demona, bez GUI: uruchamia się, ustawia kolor i kończy (około 2 ms).

## Zgodność

| Zakres | Status |
|---|---|
| ASUS TUF Gaming F15 FX506LHB, Ubuntu 26.04, kernel 7.0, GNOME 50 / Wayland | **przetestowane** |
| Inne laptopy TUF z interfejsem `kbd_rgb_mode` w sysfs | nieprzetestowane; powinno działać, jeśli plik z kroku niżej istnieje |
| Inne wersje Ubuntu | nieprzetestowane; binarka wymaga glibc ≥ 2.34 (Ubuntu 22.04+), budowanie wymaga Rusta ≥ 1.85 |
| Inne środowiska niż GNOME | polecenia działają, skróty trzeba przypisać ręcznie |
| ROG z klawiaturą Aura (strefy / pojedyncze klawisze, USB HID) | **nieobsługiwane** |

Sprawdź, czy Twój laptop ma potrzebny interfejs:

```bash
ls /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode
```

Jeśli plik nie istnieje, `a-rgb` zgłosi, że laptop nie obsługuje koloru (jasność może działać). Klawiatura ma jeden kolor na całość. Nie uruchamiaj `a-rgb` razem z `asusd`: oba sterują tym samym LED-em.

## Instalacja

Wymagania: `build-essential`, Rust ≥ 1.85 (przez `rustup`), grupa `plugdev` dla Twojego użytkownika.

```bash
cargo install --path .
```

Binarka trafia do `~/.cargo/bin/a-rgb`.

### Dostęp bez roota (reguła udev)

Zapis do sysfs wymaga roota, więc reguła nadaje grupie `plugdev` prawo zapisu do `brightness` i `kbd_rgb_mode`:

```bash
sudo install -m 644 packaging/99-a-rgb.rules /etc/udev/rules.d/99-a-rgb.rules
sudo udevadm control --reload
sudo udevadm trigger --action=change --subsystem-match=leds --sysname-match='asus::kbd_backlight'
```

Sprawdzenie: `ls -l /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode` powinno pokazać grupę `plugdev` i prawo zapisu dla grupy. Cofnięcie: usuń plik reguły i wykonaj `sudo udevadm control --reload`.

### Przywracanie koloru po zalogowaniu

`restore` stosuje ostatnie ustawienie (jednorazowo, bez procesu w tle):

```bash
mkdir -p ~/.config/systemd/user
install -m 644 packaging/a-rgb-restore.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable a-rgb-restore.service
```

Jednostka wywołuje `~/.cargo/bin/a-rgb restore`. Przed zalogowaniem (start systemu, ekran logowania) świeci kolor ustawiony przez firmware.

## Użycie

```
a-rgb [KOLOR] [TRYB] [PRĘDKOŚĆ]     kolejność dowolna; pominięte pola z ostatniego ustawienia
a-rgb POLECENIE [argumenty]
```

| Polecenie | Działanie |
|---|---|
| `a-rgb red`, `a-rgb ff8800 breathe fast` | ustaw kolor / tryb / prędkość |
| `a-rgb b`, `b 2`, `b +`, `b -` | pokaż / ustaw / zmień o 1 jasność (0 = wyłączone) |
| `a-rgb preset gaming\|work\|night\|off` | gotowe ustawienia |
| `a-rgb off` | wyłącz podświetlenie (jasność 0) |
| `a-rgb next-color`, `prev-color` | kolejny / poprzedni kolor z palety |
| `a-rgb next-mode` | kolejny tryb |
| `a-rgb next-preset`, `prev-preset` | kolejny / poprzedni preset (gaming → work → night) |
| `a-rgb get` | jasność i ostatnie ustawienie |
| `a-rgb restore` | przywróć ostatnie ustawienie |
| `a-rgb -h`, `a-rgb help POLECENIE` | pomoc |

Kolory: `red orange yellow green cyan blue purple pink white` albo hex `RRGGBB` / `#RRGGBB`. Prędkości: `slow mid fast`.

### Tryby (sprawdzone na FX506LHB)

| Tryb | Efekt | Prędkość |
|---|---|---|
| `static` | stały kolor | – |
| `breathe` | oddychanie wybranego koloru | działa |
| `cycle` | płynna zmiana wszystkich kolorów (kolor ignorowany) | działa |
| `pulse` | szybkie oddychanie wybranego koloru | bez wpływu |

Inne wartości trybu przyjmowane przez kernel (3–8, 11) dawały na tym laptopie stały kolor, więc nie są wystawione.

### Presety

| Preset | Wygląd |
|---|---|
| `gaming` | czerwony, oddychanie, prędkość mid |
| `work` | biały, stały |
| `night` | ciepły pomarańczowy `ff9329`, stały, jasność 1 |
| `off` | jasność 0 |

`next-preset` / `prev-preset` przełączają same wyglądy i nie zmieniają jasności.

### Kody wyjścia

`0` sukces, `1` błąd wykonania (np. brak uprawnień), `2` błędne argumenty.

## Skróty klawiszowe (GNOME)

Skróty to zwykłe skróty niestandardowe wywołujące `a-rgb`. Można je dodać w Ustawienia → Klawiatura → Skróty, albo przez `gsettings`. Użyte na FX506LHB:

| Klawisze | Polecenie |
|---|---|
| Super+F5 / Super+F6 | `a-rgb prev-color` / `a-rgb next-color` |
| Super+F7 | `a-rgb next-mode` |
| Fn+← (`XF86Launch3`) | `a-rgb prev-preset` |
| Fn+→ (`XF86Launch4`) | `a-rgb next-preset` |

W poleceniach skrótów podaj pełną ścieżkę (`/home/UŻYTKOWNIK/.cargo/bin/a-rgb`), bo sesja GNOME może nie mieć `~/.cargo/bin` w `PATH`.

Fn+↑/↓ obsługuje GNOME (jasność) i `a-rgb` go nie zastępuje. Jasność zapamiętuje systemd (`systemd-backlight`), dlatego `restore` jej nie rusza. Suwak „Klawiatura" w menu szybkich ustawień to też element GNOME, który działa na tym samym pliku `brightness`.

### Fn+← na FX506LHB

Fn+← wysyła scancode `0xb2`, który kernel zgłasza jako `KEY_UNKNOWN` (240) i nie pozwala go przemapować przez hwdb (`EINVAL`). Klawisz można jednak przemapować na poziomie xkb, na symbol `XF86Launch3`. Nie rób tego, jeśli używasz Fn+← do czegoś innego (np. jako zastępczego Super).

`~/.config/xkb/symbols/custom`:

```
partial
xkb_symbols "fix_super" {
    key <I248> { [ XF86Launch3 ] };
};
```

`~/.config/xkb/rules/evdev`:

```
! option = symbols
  custom:fix_super = +custom(fix_super)

! include %S/evdev
```

Następnie dodaj opcję w GNOME i zaloguj się ponownie (GNOME wczytuje pliki xkb przy starcie sesji):

```bash
gsettings set org.gnome.desktop.input-sources xkb-options "['custom:fix_super']"
```

Jeśli masz już inne opcje xkb, dopisz tę do istniejącej listy zamiast ją zastępować (`gsettings get org.gnome.desktop.input-sources xkb-options`).

## Jak to działa

`a-rgb` zapisuje do `/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode` linię `cmd mode r g b speed`. Zawsze używa `cmd=0` (ustaw), nigdy `cmd=1` (zapis w BIOS), żeby nie zapisywać pamięci nieulotnej. Kernel nie waliduje wartości r, g, b, więc zakres 0–255 pilnuje `a-rgb`. Plik jest tylko do zapisu: aktualnego koloru nie da się odczytać ze sprzętu, więc `get` i `restore` korzystają z ostatniego ustawienia zapisanego w `~/.config/a-rgb/last`.

Zmienna `A_RGB_SYSFS` podmienia katalog LED (testy bez sprzętu), a `XDG_CONFIG_HOME` katalog konfiguracji.

## Znane ograniczenia

- Kolor na ekranie logowania i przy starcie systemu ustawia firmware. Zmiana wymagałaby zapisu w BIOS (`cmd=1`) albo usługi systemowej i nie jest zaimplementowana.
- Brak obsługi stanów podświetlenia boot / awake / sleep (`kbd_rgb_state`).

## Rozwój

```bash
cargo test
cargo clippy --all-targets
cargo build --release
```

Testy jednostkowe nie wymagają sprzętu. Test na urządzeniu: ustaw kolor i potwierdź go wzrokiem, bo `kbd_rgb_mode` jest write-only.

## Odinstalowanie

```bash
systemctl --user disable --now a-rgb-restore.service
rm ~/.config/systemd/user/a-rgb-restore.service ~/.cargo/bin/a-rgb
rm -r ~/.config/a-rgb
sudo rm /etc/udev/rules.d/99-a-rgb.rules && sudo udevadm control --reload
```

Skróty GNOME usuń w Ustawienia → Klawiatura → Skróty (wpisy „a-rgb: …").
