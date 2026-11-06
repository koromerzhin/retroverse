# Data model

SQLite is the primary store. Schema version tracked in `schema_version` (current: v24).

## Core entities

- `platforms` — supported platforms with extension lists.
- `game_sources` — registered sources (local or remote/SSH).
- `game_files` — physical files discovered (with hashes, sizes, mtimes, hashed_mtime, hashed_size).
- `games` — works (logical games), with `metadata_json`.
- `game_file_links` — links files to games with confidence + manual flag.
- `user_corrections` — manual corrections (protected).
- `metadata_records` — metadata from providers (description, developer, publisher, alt_title, details, classifications).
- `media_assets` — media (covers, etc.).
- `game_sessions` — play sessions.
- `collections`, `collection_games` — custom collections.
- `sync_jobs`, `transfer_jobs`, `metadata_jobs` — job history.
- `games_fts` — FTS5 virtual table.

## Referentials

- `developers` — developer referential (name, ss_id).
- `publishers` — publisher referential (name, ss_id).
- `genres` — multilingual genres (name_en/fr/de/es/it/pt, parent_id, ss_id).
- `classification_systems` — classification organisms (PEGI, ESRB, CERO, ...).
- `classifications` — classification values (system_id, value, ss_id).
- `game_developers`, `game_publishers`, `game_genres`, `game_classifications` — N-N links.
- `audit_log` — audit trail (timestamp, action, target, details, provenance).
- `dat_files`, `dat_entries` — imported DAT files and their entries.
- `game_preservation` — preservation status per game (verified/unknown/bad_dump).

## Distinction

- Work (`games`) ≠ Edition ≠ physical file (`game_files`) ≠ source (`game_sources`).

A single work can have multiple editions and files across multiple sources. Manual links are never overwritten by automatic scans.
