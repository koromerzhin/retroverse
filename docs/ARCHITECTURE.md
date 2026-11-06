# Architecture

RETROVERSE follows a modular architecture with a Rust core engine independent of the UI.

## Logical components

- Desktop Application (Tauri + React)
- Core Engine (Rust)
- Source Manager (local + remote SSH/SFTP)
- Metadata Provider Manager (ScreenScraper)
- Synchronization Engine (scanning, hashing, metadata sync)
- Background Services (tray, single-instance socket, splashscreen)
- Game Identification Engine (filename, ScummVM mapping)
- Local Storage (SQLite, migrations up to v21)
- Backup & Preservation Services

## Module layout (src-tauri/src)

- `db.rs` — SQLite connection + migrations (up to v21) + seeding
- `models.rs` — domain structs (Platform, Game, GameFile, Developer, Publisher, Genre, Classification, ...)
- `error.rs` — error type
- `state.rs` — app state (db handle, cancel flags)
- `config.rs` — provider configuration (ScreenScraper credentials)
- `identify.rs` — platform detection, title cleaning
- `hashing.rs` — dedicated hashing module (streaming, traits, orchestration)
- `scanner.rs` — recursive folder scanning + game creation (incremental)
- `metadata.rs` — ScreenScraper provider (hash-first lookup)
- `tray.rs` — system tray (ksni, KDE-compatible)
- `commands/` — Tauri command handlers exposed to frontend
  - `app`, `backup` (hashes), `entities` (developers/publishers/genres/classifications sync), `genres`, `classifications`, `library`, `metadata`, `platforms`, `scans`, `sources`, `remote`, `notifications`, `settings`, `backup`

## Frontend (src)

- `pages/` — route pages
- `lib/api.ts` — typed Tauri invoke wrappers
- `types.ts` — shared TS types

## Data model

See [DATA_MODEL.md](DATA_MODEL.md).

## Principles

- Core engine does not depend on GUI.
- Connectors share a common interface (planned).
- Non-destructive by default.
- Manual corrections are never overwritten automatically.