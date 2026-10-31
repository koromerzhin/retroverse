use retroverse_lib::state::AppState;
use retroverse_lib::commands::remote;
#[tokio::main]
async fn main() {
    let state = AppState::new().unwrap();
    let sid = {
        let conn = state.db.lock().unwrap();
        conn.query_row("SELECT id FROM game_sources WHERE kind='remote' LIMIT 1", [], |r| r.get::<_,String>(0)).unwrap_or_default()
    };
    if sid.is_empty() { println!("no source"); return; }
    // clear one file's hash to force recompute
    {
        let conn = state.db.lock().unwrap();
        conn.execute("UPDATE game_files SET hash=NULL,crc32=NULL,md5=NULL,sha1=NULL WHERE id=(SELECT id FROM game_files WHERE source_id=?1 LIMIT 1)", rusqlite::params![sid]).unwrap();
    }
    let r = remote::scan_remote_source_inner(&state, sid.clone(), None, state.cancel_scan.clone(), Some("/run/media/deck/WDC1/Emulation/roms/nes/".to_string())).await.unwrap();
    println!("SCAN: files={} created={} updated={}", r.files_found, r.games_created, r.games_updated);
    // check hashes
    let conn = state.db.lock().unwrap();
    let (h, c): (Option<String>, Option<String>) = conn.query_row("SELECT hash, crc32 FROM game_files WHERE source_id=?1 AND hash IS NOT NULL LIMIT 1", [sid], |r| Ok((r.get(0)?, r.get(1)?))).unwrap_or((None,None));
    println!("sample hash: {:?} crc: {:?}", h.map(|x|x[..12].to_string()), c);
}
