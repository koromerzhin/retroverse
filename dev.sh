#!/usr/bin/env bash
# Lance RETROVERSE en mode développement (hot-reload)
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
export PKG_CONFIG_PATH="$HOME/.local/share/pkgconfig:$PKG_CONFIG_PATH"
exec npm run tauri dev