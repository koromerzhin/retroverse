# Improvements

Possible evolution paths for RETROVERSE, grouped by theme. Prioritize as needed.

## Performance and memory

1. **Streaming large ISOs** (started): extend streaming to all paths (remote, archives) and add a configurable buffer.
2. **Controlled concurrency**: queue (semaphore) to limit simultaneous hash computations on large files.
3. **Result caching**: skip recomputing hashes when the file is unchanged (mtime/size).
4. **Virtual pagination** in the Library for very large collections.

## UX and interface

5. **Real hash cancellation**: implement a dedicated `cancel_hash` (the "Stop" button currently calls `cancelScan`).
6. **Granular progress**: per-file percentage and estimated time remaining.
7. **Advanced filters**: by missing hash, size, added date, metadata presence.
8. **Detail view tabs** (started): add "Media" (covers/screenshots) and "Linked files" tabs.
9. **Dark/light mode** with customizable themes.
10. **Fuzzy search** and autocomplete.

## Metadata and referentials

11. **Manual field editing**: persist edited genre/dates/classifications/synopsis (currently only publisher/developer are saved).
12. **Multi-genre per game**: Genre is a single select; switch to multi-select.
13. **Automatic referential sync** (publishers/developers) at startup, like genres.
14. **Metadata import/export** (JSON/CSV) for backup/migration.
15. **Enrichment history** with revert capability.

## Features

16. **Duplicate detection** (started): manual merge and assisted resolution.
17. **Collections/playlists** and custom tags.
18. **Statistics**: charts by platform, decade, genre.
19. **Game launching**: emulator integration (detection already present) + "Play" button.
20. **Incremental scan**: only scan new/modified files (mtime).
21. **Box art support** via ScreenScraper (medias already in metadata_json).
22. **Offline mode**: cache metadata and images.

## Quality and development

23. **Tests**: add Rust unit tests (identify, scanner) and frontend tests (React Testing Library).
24. **CI**: add frontend lint (eslint) and E2E tests (Playwright).
25. **Logging**: structured tracing with configurable levels.
26. **Error handling**: more precise user messages, error codes.
27. **Security**: path validation (path traversal), sandboxing.
28. **i18n**: complete translations (fr/en complete, others fallback to English).

## Architecture

29. **Refactoring**: extract hashing logic into a dedicated module with traits.
30. **Database**: versioned migrations with rollback, automatic backup.
31. **Configuration**: in-app management of providers (ScreenScraper credentials) ✅ (Settings page).
