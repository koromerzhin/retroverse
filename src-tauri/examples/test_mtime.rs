use retroverse_lib::state::AppState;
#[tokio::main]
async fn main() {
    let state = AppState::new().unwrap();
    let sid = {
        let conn = state.db.lock().unwrap();
        conn.query_row("SELECT id FROM game_sources LIMIT 1", [], |r| r.get::<_,String>(0)).unwrap_or_default()
    };
    println!("source: {}", sid);
    let conn = state.db.lock().unwrap();
    let (path,): (String,) = conn.query_row("SELECT path FROM game_sources WHERE id=?1", [sid.clone()], |r| Ok((r.get(0)?,))).unwrap();
    drop(conn);
    let root = std::path::Path::new(&path);
    let mt = std::fs::metadata(root).ok().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64).unwrap_or(0);
    println!("path: {} exists: {} mtime: {}", path, root.exists(), mt);
    let conn = state.db.lock().unwrap();
    let stored: i64 = conn.query_row("SELECT COALESCE(root_mtime,0) FROM game_sources WHERE id=?1", [sid.clone()], |r| r.get(0)).unwrap_or(0);
    println!("stored mtime: {}", stored);
    println!("match: {}", mt != 0 && mt == stored);
}
