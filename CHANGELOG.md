# Changelog

All notable changes to RETROVERSE are documented here.

## [0.2.0] - Remote Sources (in progress)

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