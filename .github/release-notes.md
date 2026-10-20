A CLI tool for controlling the keyboard backlight of ASUS TUF laptops (color, mode, speed, brightness, presets). No daemon, no GUI.

**Tested only on:** ASUS TUF Gaming F15 FX506LHB, Ubuntu 26.04, GNOME/Wayland. Other TUF models with a `kbd_rgb_mode` interface may work, but have not been checked. ROG laptops with an Aura keyboard are not supported.

## Install (Ubuntu / Debian, amd64)

```bash
sha256sum -c SHA256SUMS --ignore-missing
sudo apt install ./a-rgb_@VERSION@_amd64.deb
id -nG | grep -qw plugdev || sudo usermod -aG plugdev "$USER"   # log in again after being added to the group
a-rgb get
```

## Other distributions (static binary)

```bash
tar -xzf a-rgb-@VERSION@-x86_64-linux.tar.gz
cd a-rgb-@VERSION@-x86_64-linux
sudo ./install.sh        # to uninstall: sudo ./install.sh --uninstall
```

GNOME keyboard shortcuts and the Fn+← key map for the FX506LHB are described in the README (they are not part of the package).

---

## Po polsku

CLI do sterowania podświetleniem klawiatury w laptopach ASUS TUF (kolor, tryb, prędkość, jasność, presety). Bez demona i bez GUI.

**Testowane tylko na:** ASUS TUF Gaming F15 FX506LHB, Ubuntu 26.04, GNOME/Wayland. Inne modele TUF z interfejsem `kbd_rgb_mode` mogą działać, ale nie były sprawdzane. Laptopy ROG z klawiaturą Aura nie są obsługiwane.

Instalacja jest taka sama jak powyżej (`.deb` dla Ubuntu/Debiana, `tar.gz` dla innych dystrybucji). Dokumentacja po polsku: [README.pl.md](https://github.com/smario-7/Asus_RGB/blob/main/README.pl.md).
