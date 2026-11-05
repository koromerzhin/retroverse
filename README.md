# RETROVERSE

**The Universal Video Game Collection Companion**

RETROVERSE is an open-source desktop application that centralizes, organizes, enriches, documents, discovers and preserves video game collections.

It works independently of emulators, frontends, EmuDeck, distribution platforms, metadata providers and operating systems. RETROVERSE does not replace existing software — it complements it.

## Status

Version 1.0 — in development (develop branch).

## Features

- Desktop application (Tauri 2 + Rust + React/TypeScript)
- Local SQLite database with migrations (up to v21)
- Platform profiles with extension-based detection
- Local and remote source management (SSH/SFTP)
- Recursive folder scanning (local + remote) with incremental (mtime/size) skipping
- Game identification by filename (ScummVM compatibility mapping)
- Central library view with search, platform/developer/publisher filters and sortable headers
- Platform cards/list with game/file counts, hash status and sorting
- Detailed game view with editable metadata (title, genre, publisher, developer, classifications, release dates, synopsis)
- ScreenScraper integration (hash-first lookup, raw JSON storage, developer/publisher/genre sync)
- Hash computation (content + archive) on demand, per file/game/platform/library
- Native OS notifications (no in-app toasts)
- System tray (KDE-compatible via ksni) with show/hide, scan, hash, quit
- Independent splashscreen + sequential main window display
- Single-instance enforcement (Unix socket)
- Referentials: developers, publishers, genres (multilingual), classifications
- Collections, statistics dashboard, settings

## Tech stack

| Domain | Technology |
|---|---|
| Backend | Rust |
| Desktop | Tauri 2 |
| Frontend | React + TypeScript + Vite |
| Database | SQLite (rusqlite bundled) |
| Search | SQLite FTS (planned) |
| Async | Tokio |
| Tray | ksni (KDE-compatible) |
| Notifications | tauri-plugin-notification |

## Development

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/DATA_MODEL.md](docs/DATA_MODEL.md)
- [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md)
- [docs/ROADMAP.md](docs/ROADMAP.md)
- [docs/IMPROVEMENTS.md](docs/IMPROVEMENTS.md)
- [docs/SOURCE_CONNECTORS.md](docs/SOURCE_CONNECTORS.md)

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).