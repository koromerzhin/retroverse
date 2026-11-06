# Roadmap

## v1.0 — Collection Companion (current) ✅
- Repo + Git Flow + GitHub
- Tauri app, Rust backend, React frontend
- SQLite + migrations
- Platforms, local sources, scanner
- Initial identification
- Library UI, stats, settings

## v0.2 — Remote Sources
- SSH/SFTP connectors, Steam Deck
- Remote browsing + scan
- Incremental sync, disconnect handling
- Optional local copy with resume + integrity

## v0.3 — RetroData Engine
- Provider adapters (ScreenScraper, IGDB, etc.)
- Persistent queue, cache, quotas
- Improved identification, provenance, conflict resolution

## v0.4 — Game Encyclopedia
- Full game sheets, editions, regions, franchises, relations, media, advanced search

## v0.5 — Transfer & Preservation
- Advanced transfers, backups, exports, restores, duplicate detection

## v0.6 — Companion
- EmuDeck detection, emulator/frontend integrations, sessions, stats

- Tauri 2 desktop app, Rust backend, React/TypeScript frontend
- SQLite database with migrations up to v24
- Platform profiles, local + remote (SSH/SFTP) sources
- Recursive scanning with incremental (mtime/size) skipping
- Game identification (ScummVM compatibility mapping)
- Library view with search, filters (platform/developer/publisher), sorting
- Platform cards/list with game/file counts and hash status
- Detailed game view with editable metadata
- ScreenScraper integration (hash-first, raw JSON, referential sync)
- Hash computation (content + archive) on demand
- Native OS notifications, system tray, splashscreen, single-instance
- Referentials: developers, publishers, genres (multilingual), classifications
- Collections, statistics, settings
