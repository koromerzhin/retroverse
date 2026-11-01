use retroverse_lib::state::AppState;
use retroverse_lib::commands::remote;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new().unwrap());
    let sid = {
        let conn = state.db.lock().unwrap();
        conn.query_row("SELECT id FROM game_sources WHERE kind='remote' LIMIT 1", [], |r| r.get::<_,String>(0)).unwrap_or_default()
    };
    if sid.is_empty() { println!("no source"); return; }
    // reset cancel
    state.cancel_scan.store(false, std::sync::atomic::Ordering::SeqCst);
    // spawn scan
    let state2 = state.clone();
    let sid2 = sid.clone();
    let handle = tokio::spawn(async move {
        remote::scan_remote_source_inner(&state2, sid2, None, state2.cancel_scan.clone(), Some("/run/media/deck/WDC1/Emulation/".to_string())).await
    });
    // cancel after 2s
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    state.cancel_scan.store(true, std::sync::atomic::Ordering::SeqCst);
    println!("cancelled flag set");
    match handle.await.unwrap() {
        Ok(r) => println!("RESULT: files={} (cancelled scan returns partial)", r.files_found),
        Err(e) => println!("ERR: {}", e),
    }
}