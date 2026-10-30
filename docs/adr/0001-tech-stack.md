# ADR 0001 — Technology stack

## Status
Accepted.

## Decision
- Backend: Rust
- Desktop: Tauri 2
- Frontend: React + TypeScript (Vite)
- Database: SQLite (rusqlite, bundled)
- Async runtime: Tokio
- SSH/SFTP: russh (planned, v0.2)

## Rationale
- Performance and low memory footprint (8 GB target).
- Cross-platform (Linux/Windows/macOS/SteamOS).
- Strong typing and safety.
- GPL-compatible dependencies.