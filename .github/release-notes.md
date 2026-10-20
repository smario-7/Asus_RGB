CLI do sterowania podświetleniem klawiatury ASUS TUF (kolor, tryb, prędkość, jasność, presety). Bez demona i bez GUI.

**Testowane tylko na:** ASUS TUF Gaming F15 FX506LHB, Ubuntu 26.04, GNOME/Wayland. Inne modele TUF z interfejsem `kbd_rgb_mode` mogą działać, ale nie były sprawdzane. Laptopy ROG z klawiaturą Aura nie są obsługiwane.

## Instalacja (Ubuntu / Debian, amd64)

```bash
sha256sum -c SHA256SUMS --ignore-missing
sudo apt install ./a-rgb_@VERSION@_amd64.deb
sudo usermod -aG plugdev "$USER"   # potem wyloguj się i zaloguj ponownie
a-rgb get
```

## Inne dystrybucje (statyczna binarka)

```bash
tar -xzf a-rgb-@VERSION@-x86_64-linux.tar.gz
cd a-rgb-@VERSION@-x86_64-linux
sudo ./install.sh        # odinstalowanie: sudo ./install.sh --uninstall
```

Skróty klawiszowe GNOME i mapa Fn+← dla FX506LHB są opisane w README (nie są częścią paczki).
