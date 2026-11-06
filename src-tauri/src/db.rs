use crate::error::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub const SCHEMA_VERSION: i32 = 26;

pub fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("retroverse");
    std::fs::create_dir_all(&dir).ok();
    dir.join("retroverse.db")
}

pub fn open() -> Result<Connection> {
    let path = db_path();
    // Auto-backup before migration (keep last 5).
    if path.exists() {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup = path.with_extension(format!("db.bak.{}", ts));
        let _ = std::fs::copy(&path, &backup);
        // prune old backups
        if let Some(parent) = path.parent() {
            let mut backups: Vec<_> = std::fs::read_dir(parent)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("retroverse.db.bak.")
                })
                .collect();
            backups.sort_by_key(|e| e.file_name());
            while backups.len() > 5 {
                if let Some(old) = backups.first() {
                    let _ = std::fs::remove_file(old.path());
                }
                backups.remove(0);
            }
        }
    }
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
            "SELECT COALESCE((SELECT MAX(version) FROM schema_version), 0)",
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
    if current < 14 {
        apply_v14(conn)?;
    }
    if current < 15 {
        apply_v15(conn)?;
    }
    if current < 16 {
        apply_v16(conn)?;
    }
    if current < 17 {
        apply_v17(conn)?;
    }
    if current < 18 {
        apply_v18(conn)?;
    }
    if current < 19 {
        apply_v19(conn)?;
    }
    if current < 20 {
        apply_v20(conn)?;
    }
    if current < 21 {
        apply_v21(conn)?;
    }
    if current < 22 {
        apply_v22(conn)?;
    }
    if current < 23 {
        apply_v23(conn)?;
    }
    if current < 24 {
        apply_v24(conn)?;
    }
    if current < 25 {
        apply_v25(conn)?;
    }
    if current < 26 {
        apply_v26(conn)?;
    }
    if current < 16 {
        apply_v16(conn)?;
    }
    conn.execute(
        "DELETE FROM schema_version WHERE version < ?1",
        params![SCHEMA_VERSION],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
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

fn apply_v14(conn: &Connection) -> Result<()> {
    let _ = conn.execute(
        "ALTER TABLE game_files ADD COLUMN is_dir INTEGER NOT NULL DEFAULT 0",
        [],
    );
    // Populate for local paths that are directories.
    if let Ok(mut stmt) = conn.prepare("SELECT id, path FROM game_files") {
        let rows: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for (id, path) in rows {
            if std::path::Path::new(&path).is_dir() {
                let _ = conn.execute("UPDATE game_files SET is_dir=1 WHERE id=?1", [id]);
            }
        }
    }
    Ok(())
}

fn apply_v15(conn: &Connection) -> Result<()> {
    let _ = conn.execute("ALTER TABLE games ADD COLUMN metadata_json TEXT", []);
    Ok(())
}

fn apply_v16(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS genres (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT
        );",
    )?;
    Ok(())
}

fn apply_v17(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS developers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ss_id TEXT UNIQUE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS publishers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ss_id TEXT UNIQUE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS genres (
            id TEXT PRIMARY KEY,
            name_en TEXT,
            name_fr TEXT,
            name_de TEXT,
            name_es TEXT,
            name_it TEXT,
            name_pt TEXT,
            parent_id TEXT REFERENCES genres(id),
            ss_id TEXT UNIQUE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS game_developers (
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            developer_id TEXT NOT NULL REFERENCES developers(id) ON DELETE CASCADE,
            PRIMARY KEY (game_id, developer_id)
        );
        CREATE TABLE IF NOT EXISTS game_publishers (
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            publisher_id TEXT NOT NULL REFERENCES publishers(id) ON DELETE CASCADE,
            PRIMARY KEY (game_id, publisher_id)
        );
        CREATE TABLE IF NOT EXISTS game_genres (
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            genre_id TEXT NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
            PRIMARY KEY (game_id, genre_id)
        );
        "#,
    )?;
    Ok(())
}

fn apply_v18(conn: &Connection) -> Result<()> {
    // Ensure the genres table has the multilingual columns (old schema had only name/parent_id).
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(genres)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    let add_col = |conn: &Connection, col: &str, typ: &str| {
        if !cols.contains(&col.to_string()) {
            let _ = conn.execute(
                &format!("ALTER TABLE genres ADD COLUMN {} {}", col, typ),
                [],
            );
        }
    };
    add_col(conn, "name_en", "TEXT");
    add_col(conn, "name_fr", "TEXT");
    add_col(conn, "name_de", "TEXT");
    add_col(conn, "name_es", "TEXT");
    add_col(conn, "name_it", "TEXT");
    add_col(conn, "name_pt", "TEXT");
    add_col(conn, "ss_id", "TEXT");
    add_col(conn, "created_at", "TEXT");
    add_col(conn, "updated_at", "TEXT");
    // Migrate old single name column into name_fr/name_en if present.
    if cols.contains(&"name".to_string()) {
        let _ = conn.execute("UPDATE genres SET name_fr = COALESCE(name_fr, name), name_en = COALESCE(name_en, name) WHERE name IS NOT NULL", []);
    }
    let _ = conn.execute("UPDATE genres SET created_at = COALESCE(created_at, '2020-01-01T00:00:00Z'), updated_at = COALESCE(updated_at, '2020-01-01T00:00:00Z') WHERE created_at IS NULL OR updated_at IS NULL", []);
    Ok(())
}

fn apply_v19(conn: &Connection) -> Result<()> {
    // Safety net: ensure created_at/updated_at exist.
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(genres)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    if !cols.contains(&"created_at".to_string()) {
        let _ = conn.execute("ALTER TABLE genres ADD COLUMN created_at TEXT", []);
    }
    if !cols.contains(&"updated_at".to_string()) {
        let _ = conn.execute("ALTER TABLE genres ADD COLUMN updated_at TEXT", []);
    }
    let _ = conn.execute("UPDATE genres SET created_at = COALESCE(created_at, '2020-01-01T00:00:00Z'), updated_at = COALESCE(updated_at, '2020-01-01T00:00:00Z') WHERE created_at IS NULL OR updated_at IS NULL", []);
    Ok(())
}

fn apply_v20(conn: &Connection) -> Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(genres)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    if cols.contains(&"name".to_string()) {
        let _ = conn.execute("ALTER TABLE genres DROP COLUMN name", []);
    }
    Ok(())
}

fn apply_v21(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS classification_systems (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            ss_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS classifications (
            id TEXT PRIMARY KEY,
            system_id TEXT NOT NULL REFERENCES classification_systems(id) ON DELETE CASCADE,
            value TEXT NOT NULL,
            ss_id TEXT UNIQUE,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(system_id, value)
        );
        CREATE TABLE IF NOT EXISTS game_classifications (
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            classification_id TEXT NOT NULL REFERENCES classifications(id) ON DELETE CASCADE,
            PRIMARY KEY (game_id, classification_id)
        );
        "#,
    )?;

    let now = chrono::Utc::now().to_rfc3339();

    // Helper closure inline: migrate a (system_name, value) pair.
    // Collect games
    let mut gstmt = conn.prepare("SELECT id, metadata_json FROM games")?;
    let games: Vec<(String, Option<String>)> = gstmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    for (gid, mj) in &games {
        // Gather (system, value) pairs from metadata_json
        let mut pairs: Vec<(String, String)> = Vec::new();
        if let Some(j) = mj
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(j)
            && let Some(arr) = v.get("classifications").and_then(|c| c.as_array())
        {
            for c in arr {
                // could be {type, text} or [type, text]
                if let (Some(t), Some(val)) = (
                    c.get("type").and_then(|x| x.as_str()),
                    c.get("text").and_then(|x| x.as_str()),
                ) {
                    pairs.push((t.to_string(), val.to_string()));
                } else if let Some(arr2) = c.as_array()
                    && arr2.len() >= 2
                    && let (Some(t), Some(val)) = (arr2[0].as_str(), arr2[1].as_str())
                {
                    pairs.push((t.to_string(), val.to_string()));
                }
            }
        }
        // Also metadata_records field='classifications'
        if let Ok(val) = conn.query_row(
            "SELECT value FROM metadata_records WHERE game_id=?1 AND field='classifications' LIMIT 1",
            [gid.clone()],
            |r| r.get::<_, String>(0),
        ) && let Ok(v) = serde_json::from_str::<serde_json::Value>(&val)
            && let Some(arr) = v.as_array()
        {
            for c in arr {
                if let Some(arr2) = c.as_array()
                    && arr2.len() >= 2
                    && let (Some(t), Some(val)) = (arr2[0].as_str(), arr2[1].as_str())
                {
                    pairs.push((t.to_string(), val.to_string()));
                } else if let (Some(t), Some(val)) = (c.get(0).and_then(|x| x.as_str()), c.get(1).and_then(|x| x.as_str())) {
                    pairs.push((t.to_string(), val.to_string()));
                }
            }
        }
        // Also details record
        if let Ok(details) = conn.query_row(
            "SELECT value FROM metadata_records WHERE game_id=?1 AND field='details' LIMIT 1",
            [gid.clone()],
            |r| r.get::<_, String>(0),
        ) && let Ok(v) = serde_json::from_str::<serde_json::Value>(&details)
            && let Some(arr) = v.get("classifications").and_then(|c| c.as_array())
        {
            for c in arr {
                if let (Some(t), Some(val)) = (
                    c.get("type").and_then(|x| x.as_str()),
                    c.get("text").and_then(|x| x.as_str()),
                ) {
                    pairs.push((t.to_string(), val.to_string()));
                }
            }
        }

        for (sys_name, value) in pairs {
            if sys_name.trim().is_empty() || value.trim().is_empty() {
                continue;
            }
            // upsert system
            let sys_id: String = match conn.query_row(
                "SELECT id FROM classification_systems WHERE name=?1",
                [sys_name.clone()],
                |r| r.get(0),
            ) {
                Ok(id) => id,
                Err(_) => {
                    let id = uuid::Uuid::new_v4().to_string();
                    let _ = conn.execute(
                        "INSERT INTO classification_systems (id,name,ss_id,created_at,updated_at) VALUES (?1,?2,NULL,?3,?4)",
                        rusqlite::params![id, sys_name, now, now],
                    );
                    id
                }
            };
            // upsert classification
            let cls_id: String = match conn.query_row(
                "SELECT id FROM classifications WHERE system_id=?1 AND value=?2",
                rusqlite::params![sys_id, value],
                |r| r.get(0),
            ) {
                Ok(id) => id,
                Err(_) => {
                    let id = uuid::Uuid::new_v4().to_string();
                    let _ = conn.execute(
                        "INSERT INTO classifications (id,system_id,value,ss_id,description,created_at,updated_at) VALUES (?1,?2,?3,NULL,NULL,?4,?5)",
                        rusqlite::params![id, sys_id, value, now, now],
                    );
                    id
                }
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO game_classifications (game_id, classification_id) VALUES (?1,?2)",
                rusqlite::params![gid, cls_id],
            );
        }
    }
    Ok(())
}

fn apply_v22(conn: &Connection) -> Result<()> {
    // Add hashed_mtime and hashed_size to track when hashes were computed.
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(game_files)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    if !cols.contains(&"hashed_mtime".to_string()) {
        let _ = conn.execute("ALTER TABLE game_files ADD COLUMN hashed_mtime INTEGER", []);
    }
    if !cols.contains(&"hashed_size".to_string()) {
        let _ = conn.execute("ALTER TABLE game_files ADD COLUMN hashed_size INTEGER", []);
    }
    // Backfill: existing hashed files get their current mtime/size as hashed values.
    let _ = conn.execute("UPDATE game_files SET hashed_mtime = COALESCE(hashed_mtime, modified_at), hashed_size = COALESCE(hashed_size, size) WHERE hash IS NOT NULL", []);
    Ok(())
}

fn apply_v23(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS audit_log (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            action TEXT NOT NULL,
            target_type TEXT,
            target_id TEXT,
            details TEXT,
            provenance TEXT
        );
        CREATE TABLE IF NOT EXISTS dat_files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            system TEXT,
            version TEXT,
            imported_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS dat_entries (
            id TEXT PRIMARY KEY,
            dat_id TEXT NOT NULL REFERENCES dat_files(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            size INTEGER,
            crc32 TEXT,
            md5 TEXT,
            sha1 TEXT,
            sha256 TEXT
        );
        CREATE TABLE IF NOT EXISTS game_preservation (
            game_id TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
            status TEXT NOT NULL DEFAULT 'unknown',
            dat_entry_id TEXT REFERENCES dat_entries(id),
            verified_at TEXT,
            notes TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_dat_entries_dat ON dat_entries(dat_id);
        CREATE INDEX IF NOT EXISTS idx_dat_entries_sha1 ON dat_entries(sha1);
        CREATE INDEX IF NOT EXISTS idx_dat_entries_md5 ON dat_entries(md5);
        CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
        "#,
    )?;
    Ok(())
}

fn apply_v24(conn: &Connection) -> Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(dat_entries)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    if !cols.contains(&"bad".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE dat_entries ADD COLUMN bad INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }
    Ok(())
}

fn apply_v25(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS game_favorites (
            game_id TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS game_tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS game_tag_links (
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            tag_id TEXT NOT NULL REFERENCES game_tags(id) ON DELETE CASCADE,
            PRIMARY KEY (game_id, tag_id)
        );
        "#,
    )?;
    Ok(())
}

fn apply_v26(conn: &Connection) -> Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(game_sources)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    if !cols.contains(&"active".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE game_sources ADD COLUMN active INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM game_sources WHERE active=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if count == 0 {
        let _ = conn.execute("UPDATE game_sources SET active=1 WHERE id=(SELECT id FROM game_sources ORDER BY created_at LIMIT 1)", []);
    }
    Ok(())
}
