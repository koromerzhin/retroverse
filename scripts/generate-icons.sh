#!/usr/bin/env bash
set -euo pipefail

# Configurable source image
SRC="${1:-/home/koromerzhin/Téléchargements/logo.png}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PUBLIC="$ROOT/public"
ICONS="$ROOT/src-tauri/icons"

if [ ! -f "$SRC" ]; then
  echo "Erreur : fichier source introuvable : $SRC" >&2
  exit 1
fi

# Detect dependencies
for cmd in magick convert; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Erreur : '$cmd' est requis. Installez ImageMagick (sudo dnf install ImageMagick)." >&2
    exit 1
  fi
done
MAGICK="$(command -v magick || command -v convert)"

echo "Source : $SRC"
echo "Génération des favicons dans $PUBLIC"
mkdir -p "$PUBLIC"
"$MAGICK" "$SRC" -resize 16x16  "$PUBLIC/favicon-16x16.png"
"$MAGICK" "$SRC" -resize 32x32  "$PUBLIC/favicon-32x32.png"
"$MAGICK" "$SRC" -resize 180x180 "$PUBLIC/apple-touch-icon.png"
"$MAGICK" "$SRC" -resize 192x192 "$PUBLIC/favicon.png"
"$MAGICK" "$SRC" -resize 256x256 -define icon:auto-resize=256,128,64,48,32,16 "$PUBLIC/favicon.ico"

echo "Génération des icônes Tauri dans $ICONS"
mkdir -p "$ICONS"
if command -v cargo >/dev/null 2>&1 && cargo tauri --version >/dev/null 2>&1; then
  cargo tauri icon "$SRC" --output "$ICONS" 2>&1 | tail -5
else
  echo "cargo tauri non disponible : génération manuelle des icônes PNG/ICO."
  "$MAGICK" "$SRC" -resize 32x32   "$ICONS/32x32.png"
  "$MAGICK" "$SRC" -resize 128x128 "$ICONS/128x128.png"
  "$MAGICK" "$SRC" -resize 256x256 "$ICONS/128x128@2x.png"
  "$MAGICK" "$SRC" -resize 256x256 -define icon:auto-resize=256,128,64,48,32,16 "$ICONS/icon.ico"
  cp "$PUBLIC/favicon.png" "$ICONS/icon.png"
fi

echo "Terminé."
