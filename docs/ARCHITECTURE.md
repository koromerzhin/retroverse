# Architecture

RETROVERSE follows a modular architecture with a Rust core engine independent of the UI.

## Logical components

- Desktop Application (Tauri + React)
- Core Engine (Rust)
- Source Manager
- Metadata Provider Manager (planned)
- Synchronization Engine
- Background Services (planned)
- Game Identification Engine
- Local Storage (SQLite)
- Backup & Preservation Services (planned)

## Module layout (src-tauri/src)

- `db.rs` — SQLite connection + migrations + seeding
- `models.rs` — domain structs
- `error.rs` — error type
- `state.rs` — app state (db handle)
- `identify.rs` — platform detection, title cleaning
- `scanner.rs` — recursive folder scanning + game creation
- `commands/` — Tauri command handlers exposed to frontend

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