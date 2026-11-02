use crate::error::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub const SCHEMA_VERSION: i32 = 13;

pub fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("retroverse");
    std::fs::create_dir_all(&dir).ok();
    dir.join("retroverse.db")
}

pub fn open() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "cache_size", -8000)?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    migrate(&conn)?;
    let _ = cleanup_orphan_games(&conn);
    Ok(conn)
}

pub fn cleanup_orphan_games(conn: &Connection) -> Result<usize> {
    // Delete games that have no linked files.
    let n = conn.execute(
        "DELETE FROM games WHERE id NOT IN (SELECT DISTINCT game_id FROM game_file_links)",
        [],
    )?;
    Ok(n)
}

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);")?;

    let current: i32 = conn
        .query_row(
            "SELECT COALESCE((SELECT version FROM schema_version LIMIT 1), 0)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        apply_v1(conn)?;
    }
    if current < 2 {
        apply_v2(conn)?;
    }
    if current < 3 {
        apply_v3(conn)?;
    }
    if current < 4 {
        apply_v4(conn)?;
    }
    if current < 5 {
        apply_v5(conn)?;
    }
    if current < 6 {
        apply_v6(conn)?;
    }
    if current < 7 {
        apply_v7(conn)?;
    }
    if current < 8 {
        apply_v8(conn)?;
    }
    if current < 9 {
        apply_v9(conn)?;
    }
    if current < 10 {
        apply_v10(conn)?;
    }
    if current < 11 {
        apply_v11(conn)?;
    }
    if current < 12 {
        apply_v12(conn)?;
    }
    if current < 13 {
        apply_v13(conn)?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (rowid, version) VALUES (1, ?1)",
        params![SCHEMA_VERSION],
    )?;
    Ok(())
}

fn apply_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS platforms (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            manufacturer TEXT,
            generation INTEGER,
            release_year INTEGER,
            extensions TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS game_sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_scan_at TEXT
        );

        CREATE TABLE IF NOT EXISTS game_files (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL REFERENCES game_sources(id) ON DELETE CASCADE,
            path TEXT NOT NULL,
            filename TEXT NOT NULL,
            extension TEXT NOT NULL,
            size INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            hash TEXT,
            platform_id TEXT REFERENCES platforms(id),
            created_at TEXT NOT NULL,
            UNIQUE(source_id, path)
        );

        CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            platform_id TEXT REFERENCES platforms(id),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS game_file_links (
            file_id TEXT NOT NULL REFERENCES game_files(id) ON DELETE CASCADE,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            confidence REAL NOT NULL DEFAULT 0.5,
            manual INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (file_id, game_id)
        );

        CREATE TABLE IF NOT EXISTS user_corrections (
            id TEXT PRIMARY KEY,
            file_id TEXT NOT NULL REFERENCES game_files(id) ON DELETE CASCADE,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS metadata_records (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            provider TEXT NOT NULL,
            field TEXT NOT NULL,
            value TEXT NOT NULL,
            provenance TEXT,
            fetched_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS media_assets (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            kind TEXT NOT NULL,
            path TEXT,
            url TEXT,
            cached_at TEXT
        );

        CREATE TABLE IF NOT EXISTS game_sessions (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            duration_seconds INTEGER
        );

        CREATE TABLE IF NOT EXISTS collections (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS collection_games (
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            PRIMARY KEY (collection_id, game_id)
        );

        CREATE TABLE IF NOT EXISTS sync_jobs (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL REFERENCES game_sources(id) ON DELETE CASCADE,
            status TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            error TEXT,
            files_found INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS transfer_jobs (
            id TEXT PRIMARY KEY,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            transferred_bytes INTEGER NOT NULL DEFAULT 0,
            error TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_game_files_source ON game_files(source_id);
        CREATE INDEX IF NOT EXISTS idx_game_files_platform ON game_files(platform_id);
        CREATE INDEX IF NOT EXISTS idx_games_platform ON games(platform_id);
        CREATE INDEX IF NOT EXISTS idx_game_file_links_game ON game_file_links(game_id);

        -- FTS5 for game search
        CREATE VIRTUAL TABLE IF NOT EXISTS games_fts USING fts5(title, content='games', content_rowid='rowid');
        "#,
    )?;

    seed_platforms(conn)?;
    Ok(())
}

type PlatformSeed = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<i32>,
    Option<i32>,
    &'static str,
);

fn apply_v2(conn: &Connection) -> Result<()> {
    // Remote connections
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS remote_connections (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL,
            key_path TEXT NOT NULL,
            host_key_fingerprint TEXT,
            created_at TEXT NOT NULL
        );
        "#,
    )?;

    // Extend game_sources
    let alter_stmts = [
        "ALTER TABLE game_sources ADD COLUMN connection_id TEXT REFERENCES remote_connections(id)",
        "ALTER TABLE game_sources ADD COLUMN remote_path TEXT",
        "ALTER TABLE game_sources ADD COLUMN available INTEGER NOT NULL DEFAULT 1",
        "ALTER TABLE game_sources ADD COLUMN last_error TEXT",
    ];
    for s in alter_stmts {
        let _ = conn.execute(s, []);
    }

    // Extend game_files with missing flag
    let _ = conn.execute(
        "ALTER TABLE game_files ADD COLUMN missing INTEGER NOT NULL DEFAULT 0",
        [],
    );

    // Transfer job items
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS transfer_items (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL REFERENCES transfer_jobs(id) ON DELETE CASCADE,
            file_id TEXT,
            remote_path TEXT NOT NULL,
            local_path TEXT NOT NULL,
            size INTEGER NOT NULL DEFAULT 0,
            transferred INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending',
            error TEXT
        );
        "#,
    )?;

    Ok(())
}

fn apply_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS metadata_jobs (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            provider TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            priority INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            finished_at TEXT,
            error TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_metadata_jobs_status ON metadata_jobs(status);
        "#,
    )?;
    Ok(())
}

fn apply_v4(conn: &Connection) -> Result<()> {
    // FTS5 triggers + populate
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS games_ai AFTER INSERT ON games BEGIN
            INSERT INTO games_fts(rowid, title) VALUES (new.rowid, new.title);
        END;
        CREATE TRIGGER IF NOT EXISTS games_ad AFTER DELETE ON games BEGIN
            INSERT INTO games_fts(games_fts, rowid, title) VALUES('delete', old.rowid, old.title);
        END;
        CREATE TRIGGER IF NOT EXISTS games_au AFTER UPDATE ON games BEGIN
            INSERT INTO games_fts(games_fts, rowid, title) VALUES('delete', old.rowid, old.title);
            INSERT INTO games_fts(rowid, title) VALUES (new.rowid, new.title);
        END;
        "#,
    )?;
    // populate existing
    let _ = conn.execute(
        "INSERT INTO games_fts(rowid, title) SELECT rowid, title FROM games",
        [],
    );
    // collections already exist; game_relations table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS game_relations (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            related_game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            relation_type TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

fn apply_v5(conn: &Connection) -> Result<()> {
    let alters = [
        "ALTER TABLE game_files ADD COLUMN crc32 TEXT",
        "ALTER TABLE game_files ADD COLUMN md5 TEXT",
        "ALTER TABLE game_files ADD COLUMN sha1 TEXT",
        "ALTER TABLE game_files ADD COLUMN zip_inner TEXT",
    ];
    for s in alters {
        let _ = conn.execute(s, []);
    }
    Ok(())
}

fn apply_v6(conn: &Connection) -> Result<()> {
    // Deduplicate metadata_records: keep only the most recent per (game_id, field).
    conn.execute_batch(
        r#"
        DELETE FROM metadata_records
        WHERE id NOT IN (
            SELECT id FROM (
                SELECT id, ROW_NUMBER() OVER (PARTITION BY game_id, field ORDER BY fetched_at DESC) as rn
                FROM metadata_records
            ) WHERE rn = 1
        );
        DELETE FROM media_assets
        WHERE id NOT IN (
            SELECT id FROM (
                SELECT id, ROW_NUMBER() OVER (PARTITION BY game_id, kind ORDER BY COALESCE(cached_at,'') DESC) as rn
                FROM media_assets
            ) WHERE rn = 1
        );
        "#,
    )?;
    Ok(())
}

fn apply_v7(conn: &Connection) -> Result<()> {
    // Deduplicate platforms by name: keep the one with the smallest id, reassign references.
    let mut stmt = conn.prepare("SELECT name FROM platforms GROUP BY name HAVING COUNT(*) > 1")?;
    let dup_names: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for name in dup_names {
        // find all ids for this name
        let mut stmt = conn.prepare("SELECT id FROM platforms WHERE name=?1 ORDER BY id")?;
        let ids: Vec<String> = stmt
            .query_map([name.clone()], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if ids.len() < 2 {
            continue;
        }
        let kept = ids[0].clone();
        for dup in &ids[1..] {
            conn.execute(
                "UPDATE games SET platform_id=?1 WHERE platform_id=?2",
                params![kept, dup],
            )?;
            conn.execute(
                "UPDATE game_files SET platform_id=?1 WHERE platform_id=?2",
                params![kept, dup],
            )?;
            conn.execute("DELETE FROM platforms WHERE id=?1", [dup.clone()])?;
        }
    }
    Ok(())
}

fn apply_v8(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_metadata_records_game_field ON metadata_records(game_id, field);
        CREATE INDEX IF NOT EXISTS idx_game_files_source_path ON game_files(source_id, path);
        CREATE INDEX IF NOT EXISTS idx_games_title_platform ON games(title, platform_id);
        "#,
    )?;
    Ok(())
}

fn apply_v9(conn: &Connection) -> Result<()> {
    let _ = conn.execute("ALTER TABLE game_sources ADD COLUMN root_mtime INTEGER", []);
    Ok(())
}

fn seed_platforms(conn: &Connection) -> Result<()> {
    let platforms: Vec<PlatformSeed> = vec![
        (
            "pc",
            "PC",
            "pc",
            Some("Various"),
            None,
            None,
            r#"[".exe",".bat",".lnk"]"#,
        ),
        (
            "nes",
            "Nintendo Entertainment System",
            "nes",
            Some("Nintendo"),
            Some(3),
            Some(1983),
            r#"[".nes",".fds",".unf",".unif"]"#,
        ),
        (
            "snes",
            "Super Nintendo",
            "snes",
            Some("Nintendo"),
            Some(4),
            Some(1990),
            r#"[".sfc",".smc",".fig"]"#,
        ),
        (
            "n64",
            "Nintendo 64",
            "n64",
            Some("Nintendo"),
            Some(5),
            Some(1996),
            r#"[".n64",".z64",".v64"]"#,
        ),
        (
            "gb",
            "Game Boy",
            "gb",
            Some("Nintendo"),
            Some(2),
            Some(1989),
            r#"[".gb",".gbc"]"#,
        ),
        (
            "gba",
            "Game Boy Advance",
            "gba",
            Some("Nintendo"),
            Some(6),
            Some(2001),
            r#"[".gba"]"#,
        ),
        (
            "nds",
            "Nintendo DS",
            "nds",
            Some("Nintendo"),
            Some(7),
            Some(2004),
            r#"[".nds"]"#,
        ),
        (
            "gc",
            "Nintendo GameCube",
            "gc",
            Some("Nintendo"),
            Some(6),
            Some(2001),
            r#"[".gcm",".iso",".gcz"]"#,
        ),
        (
            "wii",
            "Nintendo Wii",
            "wii",
            Some("Nintendo"),
            Some(7),
            Some(2006),
            r#"[".wbfs",".iso"]"#,
        ),
        (
            "switch",
            "Nintendo Switch",
            "switch",
            Some("Nintendo"),
            Some(8),
            Some(2017),
            r#"[".nsp",".xci"]"#,
        ),
        (
            "genesis",
            "Sega Genesis / Mega Drive",
            "genesis",
            Some("Sega"),
            Some(4),
            Some(1988),
            r#"[".bin",".gen",".md",".smd"]"#,
        ),
        (
            "saturn",
            "Sega Saturn",
            "saturn",
            Some("Sega"),
            Some(5),
            Some(1994),
            r#"[".cue",".iso",".bin"]"#,
        ),
        (
            "dreamcast",
            "Sega Dreamcast",
            "dreamcast",
            Some("Sega"),
            Some(6),
            Some(1998),
            r#"[".cdi",".gdi"]"#,
        ),
        (
            "ps1",
            "PlayStation",
            "ps1",
            Some("Sony"),
            Some(5),
            Some(1994),
            r#"[".bin",".cue",".img",".iso",".pbp",".chd"]"#,
        ),
        (
            "ps2",
            "PlayStation 2",
            "ps2",
            Some("Sony"),
            Some(6),
            Some(2000),
            r#"[".iso",".bin",".cue",".chd"]"#,
        ),
        (
            "ps3",
            "PlayStation 3",
            "ps3",
            Some("Sony"),
            Some(7),
            Some(2006),
            r#"[".iso"]"#,
        ),
        (
            "psp",
            "PlayStation Portable",
            "psp",
            Some("Sony"),
            Some(7),
            Some(2004),
            r#"[".iso",".cso",".pbp"]"#,
        ),
        (
            "psvita",
            "PlayStation Vita",
            "psvita",
            Some("Sony"),
            Some(8),
            Some(2011),
            r#"[".vpk"]"#,
        ),
        (
            "atari2600",
            "Atari 2600",
            "atari2600",
            Some("Atari"),
            Some(2),
            Some(1977),
            r#"[".a26",".bin"]"#,
        ),
        (
            "arcade",
            "Arcade",
            "arcade",
            Some("Various"),
            None,
            None,
            r#"[".zip",".7z"]"#,
        ),
        (
            "msx",
            "MSX",
            "msx",
            Some("Microsoft"),
            None,
            Some(1983),
            r#"[".rom",".mx1",".mx2",".dsk"]"#,
        ),
        (
            "amiga",
            "Commodore Amiga",
            "amiga",
            Some("Commodore"),
            None,
            Some(1985),
            r#"[".adf",".hdf",".lha"]"#,
        ),
        (
            "c64",
            "Commodore 64",
            "c64",
            Some("Commodore"),
            None,
            Some(1982),
            r#"[".d64",".t64",".prg"]"#,
        ),
    ];

    for (id, name, slug, mfr, generation, year, exts) in platforms {
        conn.execute(
            "INSERT OR IGNORE INTO platforms (id,name,slug,manufacturer,generation,release_year,extensions) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![id, name, slug, mfr, generation, year, exts],
        )?;
    }
    let _ = Utc::now();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_platforms() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM platforms", [], |r| r.get(0))
            .unwrap();
        assert!(count > 0);
        // idempotent
        migrate(&conn).unwrap();
        let count2: i64 = conn
            .query_row("SELECT COUNT(*) FROM platforms", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, count2);
    }
}

fn apply_v10(conn: &Connection) -> Result<()> {
    let _ = conn.execute("ALTER TABLE game_files ADD COLUMN archive_hash TEXT", []);
    let _ = conn.execute("ALTER TABLE game_files ADD COLUMN archive_crc32 TEXT", []);
    let _ = conn.execute("ALTER TABLE game_files ADD COLUMN archive_md5 TEXT", []);
    let _ = conn.execute("ALTER TABLE game_files ADD COLUMN archive_sha1 TEXT", []);
    Ok(())
}

fn apply_v11(conn: &Connection) -> Result<()> {
    let _ = conn.execute("ALTER TABLE game_files ADD COLUMN sha512 TEXT", []);
    Ok(())
}

fn apply_v12(conn: &Connection) -> Result<()> {
    let _ = conn.execute("ALTER TABLE game_files ADD COLUMN archive_sha512 TEXT", []);
    Ok(())
}

fn apply_v13(conn: &Connection) -> Result<()> {
    // Clear wrong archive hashes (previously stored inner hash) for archives.
    let _ = conn.execute(
        "UPDATE game_files SET archive_hash=NULL, archive_crc32=NULL, archive_md5=NULL, archive_sha1=NULL, archive_sha512=NULL WHERE extension IN ('zip','7z')",
        [],
    );
    Ok(())
}
