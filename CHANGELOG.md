# Changelog

All notable changes to RETROVERSE are documented here.

## [1.0.0] - Universal Library (stable)

### Added
- Stabilization pass, documentation refresh.
- Multi-version roadmap delivered (0.1 → 0.6).
- Full feature set: local + remote sources, RetroData enrichment, encyclopedia, discovery, preservation, companion.

## [0.6.0] - Companion

### Added
- EmuDeck installation detection.
- Running emulator process detection (Linux /proc).
- Game session tracking (start/stop, duration).
- Session statistics.
- Companion page.

## [0.5.0] - Transfer & Preservation

### Added
- SHA-256 hash computation for local files.
- Full backup export/import (JSON).
- Transfer jobs history UI.
- Preservation page.

## [0.4.0] - Game Encyclopedia & Discovery

### Added
- FTS5 full-text search with triggers and population (v4 migration).
- Duplicate detection by title + platform.
- Collections CRUD and game membership.
- Discovery page (duplicates, collections, forgotten games).
- game_relations table.

## [0.3.0] - RetroData Engine

### Added
- Metadata provider trait architecture.
- Wikipedia provider (free, no API key) for description + cover.
- Enrichment command with provenance tracking.
- metadata_jobs queue table (v3 migration).
- Game detail enrichment UI with provider selection and metadata display.
- ureq HTTP client with rustls + json.

## [0.2.0] - Remote Sources

### Added
- SSH/SFTP connector (russh + russh-sftp) with key-based auth.
- Remote source management (test connection, add source).
- Remote directory browsing.
- Remote folder scanning (catalog without downloading ROMs).
- Incremental sync with missing-file detection.
- Optional local copy with resume (skip existing same-size) and progress events.
- Transfer jobs history.
- v2 DB migration (remote_connections, transfer_items, source availability).
- Game detail copy action for remote games.

## [0.1.0] - Foundation

### Added
- Initial Tauri 2 desktop application (Rust backend + React/TypeScript frontend).
- SQLite database with versioned migrations.
- Platform profiles seeded for common consoles.
- Local source management (add/remove).
- Recursive folder scanning with extension-based platform detection.
- Initial game identification by cleaned filename.
- Central library view with search and platform filter.
- Statistics dashboard.
- Settings page with app info.
- Git repository with Git Flow (main/develop).
- GitHub repository (public).
- Documentation: architecture, roadmap, data model, development, git workflow, source connectors.
- CI workflow for tests/format.

### Planned
- Remote sources (SSH/SFTP) — v0.2
- RetroData Engine (metadata providers) — v0.3
- Game encyclopedia — v0.4