#!/usr/bin/env bash
# Buduje a-rgb-WERSJA-x86_64-linux.tar.gz z gotowej, statycznej binarki.
# Użycie: packaging/build-tarball.sh <binarka> <katalog-wyjściowy>
set -euo pipefail

BIN=${1:?podaj ścieżkę do binarki}
OUT=${2:?podaj katalog wyjściowy}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
[ -n "$VERSION" ] || { echo "nie znaleziono wersji w Cargo.toml" >&2; exit 1; }

NAME="a-rgb-${VERSION}-x86_64-linux"
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT

install -Dm755 "$BIN" "$STAGE/$NAME/a-rgb"
install -Dm755 "$ROOT/packaging/tarball/install.sh" "$STAGE/$NAME/install.sh"
install -Dm644 "$ROOT/packaging/99-a-rgb.rules" "$STAGE/$NAME/99-a-rgb.rules"
install -Dm644 "$ROOT/packaging/a-rgb-restore.service" "$STAGE/$NAME/a-rgb-restore.service"
install -Dm644 "$ROOT/README.md" "$STAGE/$NAME/README.md"
install -Dm644 "$ROOT/LICENSE" "$STAGE/$NAME/LICENSE"

mkdir -p "$OUT"
tar --sort=name --owner=0 --group=0 --numeric-owner -C "$STAGE" -czf "$OUT/$NAME.tar.gz" "$NAME"
echo "$OUT/$NAME.tar.gz"
