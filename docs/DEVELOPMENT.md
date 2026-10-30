# Development

## Prerequisites (Fedora KDE 44)

- Rust + Cargo (via Fedora packages or rustup)
- Node.js 22 + npm
- Git + GitHub CLI
- Tauri system deps: webkit2gtk-4.1, gtk3, librsvg2 (runtime + headers)
- SQLite

## Setup

```bash
git clone https://github.com/koromerzhin/retroverse.git
cd retroverse
npm install
```

## Run (dev)

```bash
export PKG_CONFIG_PATH=$HOME/.local/share/pkgconfig:$PKG_CONFIG_PATH  # if needed
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Checks

```bash
cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test
cd .. && npm run build
```

## Environment notes

- 8 GB RAM: prefer `cargo build -j 2` if memory pressure.
- librsvg2-devel not installed system-wide; a local .pc file is used (see README dev notes). Install via `sudo dnf install librsvg2-devel` when possible.