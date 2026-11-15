# AGENTS.md

## Project

RETROVERSE — Tauri 2 desktop app (Rust backend + React/TypeScript frontend + SQLite). Manages video game collections, metadata via ScreenScraper, hashes, preservation.

## ⚠️ Git workflow (READ FIRST)

- **Never merge a branch without explicit user confirmation ("ok, merge").**
- Create the branch, push it, then **STOP and wait** for the user to say "ok, merge".
- Do NOT auto-merge, even after successful checks.
- Never commit directly to `main` or `develop`.

## Commands

- Dev server: `./dev.sh` (sets `PATH` and `PKG_CONFIG_PATH`, runs `npm run tauri dev`). Vite on port 1420 (strict).
- Build: `npm run tauri build` (runs `npm run build` first via `beforeBuildCommand`).
- Frontend only: `npm run build` (`tsc && vite build`).
- Rust checks (run from `src-tauri/`):
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo check`
  - `cargo test` (3 unit tests in `identify`)
- Full verification matches CI (`.github/workflows/ci.yml`) and the pre-push hook.

Git hooks live in `.githooks/` (`core.hooksPath=.githooks`): pre-commit runs fmt+clippy; pre-push runs fmt+clippy+check+test.

## Environment quirks

- Rust/Cargo at `~/.cargo/bin`; `PKG_CONFIG_PATH` must include `$HOME/.local/share/pkgconfig` (librsvg2-devel not installed system-wide, local `.pc` used).
- Data dir: `~/.local/share/retroverse/` (SQLite DB, logs). Config: `~/.config/retroverse/providers.json`.
- ScreenScraper credentials come from `providers.json`, never hardcode.

## Architecture

- Backend entry: `src-tauri/src/lib.rs` (`run()`). Modules: `db`, `models`, `identify`, `hashing`, `scanner`, `metadata` (ScreenScraper), `tray` (ksni), `commands/*`.
- Frontend entry: `src/main.tsx` → `src/App.tsx` (routing + sidebar + progress modals).
- DB migrations in `src-tauri/src/db.rs` (`migrate()`), version tracked in `schema_version` (current v27). Migrations must be additive and never destroy data; auto-backup before migration.
- App state: `AppState` (db handle, cancel flags). Single instance via Unix socket under data dir.
- Splashscreen is an independent Tauri window with embedded HTML (`public/splash.html`); main window hidden until `app_ready`.

## Git workflow (STRICT)

- **Git Flow**: `main` (stable releases), `develop` (integration), `feature/*`, `bugfix/*`, `release/*`, `hotfix/*`.
- **Never commit directly to `main` or `develop`.**
- **Every new modification = a new branch** created from `develop` (`feature/...` or `bugfix/...`).
- **Never merge a branch without explicit user confirmation** ("ok, merge"). Create the branch, push it, and wait.
- **Never merge `develop` into `main`** without explicit instruction (release step).
- Preserve uncommitted changes; never overwrite or reset user work.
- After merging (only when authorized), delete the feature branch locally and remotely.
- **After every modification, run a full build** (`npm run build` + `cargo build` / `cargo tauri build`) to verify nothing is broken before pushing.

## Conventions

- SQL migrations: never destroy existing data; always additive; bump `SCHEMA_VERSION`.
- Hashes computed on demand; archives hash both inner ROM and archive bytes.
- Native OS notifications only; no in-app toasts.
- All user-facing fields editable and persisted; empty values shown empty (no placeholders).
- ScreenScraper is the only metadata provider (TheGamesDB removed).
- Genres synced from ScreenScraper into multilingual `genres` table.

## Testing

- Rust tests: `cargo test` in `src-tauri/` (unit tests in `identify` module).
- Frontend: `npm run build` (tsc + vite). No unit tests yet.
- Playwright config in `playwright.config.ts` + `e2e/smoke.spec.ts` (requires network for install).

## Gotchas

- `tauri.conf.json` `devUrl` is `http://localhost:1420`; running the built binary directly (without Vite) causes "Could not connect to localhost". Use `./dev.sh` or `npm run tauri dev`.
- `beforeBuildCommand` runs `npm run build`; Tauri embeds `../dist`. Touch `lib.rs` to force re-embed after frontend-only changes.
- File paths for remote sources are stored as `sftp://user@host:port/path` (after dedup migration); hash code extracts the path via `identify::remote_path_from_url`.
- `game_files.path` for remote files must be extracted from the sftp URL before SFTP access.
- Pre-commit hook enforces `cargo fmt --check` + clippy; commits fail if formatting is off. Run `cargo fmt` first.
EOF
echo done