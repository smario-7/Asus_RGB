# Paczka .deb — co instaluje i czego nie robi

## Zawartość

| Ścieżka w systemie | Źródło w repo | Rola |
|---|---|---|
| `/usr/bin/a-rgb` | binarka musl (statyczna) | polecenie `a-rgb` |
| `/usr/lib/udev/rules.d/99-a-rgb.rules` | `packaging/99-a-rgb.rules` | nadaje grupie `plugdev` prawo zapisu do `brightness` i `kbd_rgb_mode` |
| `/usr/lib/systemd/user/a-rgb-restore.service` | `packaging/a-rgb-restore.service` | jednorazowo po zalogowaniu wykonuje `a-rgb restore`; `build-deb.sh` zamienia w niej `ExecStart` na `/usr/bin/a-rgb restore` |
| `/usr/share/doc/a-rgb/{README.md,copyright}` | `README.md`, `LICENSE` | dokumentacja i licencja (MIT, format DEP-5) |

Metadane (`control`): pakiet `a-rgb`, architektura `amd64`, zależność `udev`. Binarka jest statyczna, więc paczka nie zależy od wersji glibc.

## Skrypty instalacyjne

| Skrypt | Kiedy | Co robi |
|---|---|---|
| `postinst` | po instalacji i aktualizacji (`configure`) | `udevadm control --reload`, `udevadm trigger` dla `asus::kbd_backlight` (reguła działa od razu, bez restartu), `systemctl --global enable a-rgb-restore.service` (usługa `restore` dla każdego użytkownika) |
| `prerm` | przed usunięciem (`remove`) | `systemctl --global disable a-rgb-restore.service` |
| `postrm` | po usunięciu i `purge` | `udevadm control --reload` |

Każde polecenie ma `|| true`, więc instalacja nie padnie w środowisku bez systemd lub udev (np. w kontenerze).

## Czego paczka NIE robi

Tych rzeczy nie da się bezpiecznie zrobić ze skryptów pakietu:

1. **Nie dodaje użytkownika do grupy `plugdev`.** Bez tego zapis do sysfs się nie uda. `postinst` jest wykonywany jako root i nie wie, który użytkownik instaluje, więc tylko wypisuje przypomnienie. Polecenie, które dodaje do grupy tylko wtedy, gdy trzeba: `id -nG | grep -qw plugdev || sudo usermod -aG plugdev "$USER"`, potem ponowne zalogowanie (grupa obowiązuje dopiero w nowej sesji). Pierwszy użytkownik z instalatora Ubuntu zwykle już należy do `plugdev`, konta dodane później prawdopodobnie nie (niezweryfikowane).
2. **Nie rejestruje skrótów GNOME** (Super+F5/F6/F7, Fn+←/→). Skróty są ustawieniami użytkownika; opis w [README](../README.md#skróty-klawiszowe-gnome).
3. **Nie zmienia mapy klawiszy xkb** dla Fn+← na FX506LHB — to konfiguracja użytkownika, opisana w README.
4. **Nie zmienia koloru na ekranie logowania ani przy starcie systemu.** Ustawia go firmware; `restore` działa dopiero po zalogowaniu.

## Weryfikacja po instalacji

```bash
dpkg -L a-rgb                                   # lista plików
a-rgb --version
ls -l /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode   # grupa plugdev, prawo zapisu dla grupy
systemctl --user is-enabled a-rgb-restore.service        # po ponownym zalogowaniu
```

## Odinstalowanie

```bash
sudo apt remove a-rgb
```

Usuwa wszystkie pliki z tabeli i wyłącza usługę. Plik `~/.config/a-rgb/last` (ostatnie ustawienie) zostaje w katalogu użytkownika. Uprawnienia plików w `/sys` nadane przez regułę wracają do domyślnych najpewniej dopiero po restarcie (nie weryfikowano).

## Konflikty

Jeśli w systemie działa `asusd` (asusctl), oba programy sterują tym samym LED-em. Nie używaj ich razem.
