# RETROVERSE

**The Universal Video Game Collection Companion**

RETROVERSE is an open-source desktop application that centralizes, organizes, enriches, documents, discovers and preserves video game collections.

It works independently of emulators, frontends, EmuDeck, distribution platforms, metadata providers and operating systems. RETROVERSE does not replace existing software — it complements it.

## Status

Version 0.1 — Foundation (in development).

## Features (v0.1)

- Desktop application (Tauri + Rust + React/TypeScript)
- Local SQLite database with migrations
- Platform profiles with extension-based detection
- Local source management (add/remove folders)
- Recursive folder scanning
- Initial game identification by filename
- Central library view with search and platform filter
- Statistics dashboard
- Settings page

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md).

## Tech stack

| Domain | Technology |
|---|---|
| Backend | Rust |
| Desktop | Tauri 2 |
| Frontend | React + TypeScript |
| Database | SQLite (rusqlite bundled) |
| Search | SQLite FTS5 (planned) |
| Async | Tokio |

## Development

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).