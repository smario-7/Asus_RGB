#!/usr/bin/env bash
# Buduje a-rgb_WERSJA_amd64.deb z gotowej binarki (dpkg-deb, bez dodatkowych narzędzi).
# Użycie: packaging/build-deb.sh <binarka> <katalog-wyjściowy>
set -euo pipefail

BIN=${1:?podaj ścieżkę do binarki}
OUT=${2:?podaj katalog wyjściowy}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
[ -n "$VERSION" ] || { echo "nie znaleziono wersji w Cargo.toml" >&2; exit 1; }

STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT

install -Dm755 "$BIN" "$STAGE/usr/bin/a-rgb"
install -Dm644 "$ROOT/packaging/99-a-rgb.rules" "$STAGE/usr/lib/udev/rules.d/99-a-rgb.rules"
install -d "$STAGE/usr/lib/systemd/user"
sed 's|^ExecStart=.*|ExecStart=/usr/bin/a-rgb restore|' "$ROOT/packaging/a-rgb-restore.service" \
    > "$STAGE/usr/lib/systemd/user/a-rgb-restore.service"
chmod 644 "$STAGE/usr/lib/systemd/user/a-rgb-restore.service"
install -Dm644 "$ROOT/README.md" "$STAGE/usr/share/doc/a-rgb/README.md"

# copyright w formacie DEP-5; tekst licencji z LICENSE (puste linie -> " .")
{
    printf 'Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\n'
    printf 'Upstream-Name: a-rgb\nSource: https://github.com/smario-7/Asus_RGB\n\n'
    printf 'Files: *\nCopyright: 2026 smario-7\nLicense: MIT\n'
    tail -n +4 "$ROOT/LICENSE" | sed -e 's/^$/./' -e 's/^/ /'
} > "$STAGE/usr/share/doc/a-rgb/copyright"
chmod 644 "$STAGE/usr/share/doc/a-rgb/copyright"

install -d "$STAGE/DEBIAN"
SIZE=$(du -sk --exclude=DEBIAN "$STAGE" | cut -f1)
sed -e "s/@VERSION@/$VERSION/" -e "s/@SIZE@/$SIZE/" "$ROOT/packaging/deb/control.in" > "$STAGE/DEBIAN/control"
for s in postinst prerm postrm; do
    install -m755 "$ROOT/packaging/deb/$s" "$STAGE/DEBIAN/$s"
done

find "$STAGE" -type d -exec chmod 755 {} +

mkdir -p "$OUT"
dpkg-deb --root-owner-group --build "$STAGE" "$OUT/a-rgb_${VERSION}_amd64.deb" >/dev/null
echo "$OUT/a-rgb_${VERSION}_amd64.deb"
