use retroverse_lib::state::AppState;
use retroverse_lib::commands::remote;

#[tokio::main]
async fn main() {
    let state = AppState::new().unwrap();
    // register a remote source manually
    {
        let conn = state.db.lock().unwrap();
        let cid = retroverse_lib::models::new_id();
        let sid = retroverse_lib::models::new_id();
        conn.execute("INSERT INTO remote_connections (id,name,host,port,username,key_path,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![cid, "deck", "192.168.1.125", 22i64, "deck", "/home/koromerzhin/.ssh/id_ed25519", chrono::Utc::now().to_rfc3339()]).unwrap();
        conn.execute("INSERT INTO game_sources (id,name,kind,path,created_at,connection_id,remote_path,available) VALUES (?1,?2,?3,?4,?5,?6,?7,1)",
            rusqlite::params![sid, "deck", "remote", "/run/media/deck/WDC1/Emulation/roms/nes/", chrono::Utc::now().to_rfc3339(), cid, "/run/media/deck/WDC1/Emulation/roms/nes/"]).unwrap();
        println!("source id: {}", sid);
        // store sid for scan
        std::fs::write("/tmp/opencode/sid.txt", &sid).unwrap();
    }
    let sid = std::fs::read_to_string("/tmp/opencode/sid.txt").unwrap();
    match remote::scan_remote_source_inner(&state, sid.clone(), None).await {
        Ok(r) => println!("SCAN RESULT: files={} created={} updated={}", r.files_found, r.games_created, r.games_updated),
        Err(e) => println!("SCAN ERROR: {}", e),
    }
}