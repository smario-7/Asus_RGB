#!/bin/sh
# Instalacja a-rgb z archiwum tar.gz. Uruchom jako root: sudo ./install.sh
# Odinstalowanie:                                        sudo ./install.sh --uninstall
# Zmienna PREFIX (domyślnie /usr/local) wskazuje katalog, w którym ląduje bin/a-rgb.
set -eu

PREFIX=${PREFIX:-/usr/local}
HERE=$(cd "$(dirname "$0")" && pwd)
RULE=/etc/udev/rules.d/99-a-rgb.rules
UNIT=/etc/systemd/user/a-rgb-restore.service

[ "$(id -u)" -eq 0 ] || { echo "install.sh: uruchom jako root (sudo ./install.sh)" >&2; exit 1; }

reload_udev() {
    command -v udevadm >/dev/null 2>&1 || return 0
    udevadm control --reload || true
    udevadm trigger --action=change --subsystem-match=leds --sysname-match='asus::kbd_backlight' || true
}

if [ "${1:-}" = "--uninstall" ]; then
    command -v systemctl >/dev/null 2>&1 && systemctl --global disable a-rgb-restore.service >/dev/null 2>&1 || true
    rm -f "$PREFIX/bin/a-rgb" "$RULE" "$UNIT"
    reload_udev
    echo "a-rgb odinstalowany."
    exit 0
fi

install -Dm755 "$HERE/a-rgb" "$PREFIX/bin/a-rgb"
install -Dm644 "$HERE/99-a-rgb.rules" "$RULE"
install -d /etc/systemd/user
sed "s|^ExecStart=.*|ExecStart=$PREFIX/bin/a-rgb restore|" "$HERE/a-rgb-restore.service" > "$UNIT"
chmod 644 "$UNIT"
reload_udev
command -v systemctl >/dev/null 2>&1 && systemctl --global enable a-rgb-restore.service >/dev/null 2>&1 || true

echo "a-rgb zainstalowany w $PREFIX/bin."
echo "Zapis koloru bez sudo wymaga grupy plugdev. Jako zwykły użytkownik uruchom:"
echo "  id -nG | grep -qw plugdev || sudo usermod -aG plugdev \"\$USER\"   # po dodaniu zaloguj się ponownie"
