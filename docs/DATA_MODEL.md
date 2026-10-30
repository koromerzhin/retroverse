# Data model

SQLite is the primary store. Schema version tracked in `schema_version`.

## Core entities

- `platforms` — supported platforms with extension lists.
- `game_sources` — registered sources (local folders).
- `game_files` — physical files discovered.
- `games` — works (logical games).
- `game_file_links` — links files to games with confidence + manual flag.
- `user_corrections` — manual corrections (protected).
- `metadata_records` — metadata from providers (planned usage).
- `media_assets` — media (planned).
- `game_sessions` — play sessions (planned).
- `collections`, `collection_games` — custom collections.
- `sync_jobs`, `transfer_jobs` — job history.
- `games_fts` — FTS5 virtual table (planned population).

## Distinction

- Work (`games`) ≠ Edition ≠ physical file (`game_files`) ≠ source (`game_sources`).

A single work can have multiple editions and files across multiple sources. Manual links are never overwritten by automatic scans.