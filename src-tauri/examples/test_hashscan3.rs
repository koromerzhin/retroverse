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
    // clear hashes for one nes file
    {
        let conn = state.db.lock().unwrap();
        let id: Option<String> = conn.query_row("SELECT id FROM game_files WHERE source_id=?1 AND path LIKE '%/nes/%' LIMIT 1", [sid.clone()], |r| r.get(0)).ok();
        if let Some(i) = id {
            conn.execute("UPDATE game_files SET hash=NULL,crc32=NULL,md5=NULL,sha1=NULL,zip_inner=NULL WHERE id=?1", rusqlite::params![i]).unwrap();
            println!("cleared hash for {}", i);
        }
    }
    let r = remote::scan_remote_source_inner(&state, sid.clone(), None, state.cancel_scan.clone(), Some("/run/media/deck/WDC1/Emulation/roms/nes/".to_string())).await.unwrap();
    println!("SCAN: files={} created={} updated={}", r.files_found, r.games_created, r.games_updated);
    let conn = state.db.lock().unwrap();
    let h: Option<String> = conn.query_row("SELECT crc32 FROM game_files WHERE source_id=?1 AND crc32 IS NOT NULL LIMIT 1", [sid], |r| r.get(0)).ok();
    println!("sample crc after scan: {:?}", h);
}
