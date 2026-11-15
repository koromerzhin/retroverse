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

## Release

Installers for Linux, Windows and macOS are built by GitHub Actions
(`.github/workflows/release.yml`), never on a developer machine.

1. Merge `develop` into `main`, bump the version in `package.json`,
   `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` (the three must match).
2. Update `CHANGELOG.md`.
3. Tag and push:

```bash
git checkout main && git pull
git tag -a vX.Y.Z -m "RETROVERSE X.Y.Z"
git push origin main --follow-tags
```

The tag starts the `Release` workflow: a `verify` job (fmt, clippy, tests,
frontend build, tag/version consistency), then one build per platform —
`ubuntu-22.04` (`.AppImage`, `.deb`, `.rpm`), `windows-latest` (`.msi`, NSIS
`.exe`), `macos-latest` (universal `.dmg`, Intel + Apple Silicon). Bundles are
attached to a **draft** release and also kept as workflow artifacts for 14 days.

Nothing is published until the draft is reviewed and released on GitHub. A
manual run of the workflow (`workflow_dispatch`) builds the same artifacts
without creating a release, which is the quickest way to check a platform fix.

Builds are unsigned: Windows raises SmartScreen, macOS requires
right-click → Open (or `xattr -dr com.apple.quarantine`) the first time.

## Environment notes

- 8 GB RAM: prefer `cargo build -j 2` if memory pressure.
- librsvg2-devel not installed system-wide; a local .pc file is used (see README dev notes). Install via `sudo dnf install librsvg2-devel` when possible.