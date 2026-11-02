use retroverse_lib::scanner;
use retroverse_lib::state::AppState;
#[tokio::main]
async fn main() {
    let state = AppState::new().unwrap();
    let sid = {
        let conn = state.db.lock().unwrap();
        conn.query_row("SELECT id FROM game_sources LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default()
    };
    if sid.is_empty() {
        println!("no source");
        return;
    }
    println!("=== First scan ===");
    let t0 = std::time::Instant::now();
    let r1 =
        scanner::scan_source(&state, &sid, None, state.cancel_scan.clone(), None, None).unwrap();
    println!(
        "files={} created={} updated={} time={:?}",
        r1.files_found,
        r1.games_created,
        r1.games_updated,
        t0.elapsed()
    );
    println!("=== Second scan (should skip all) ===");
    let t1 = std::time::Instant::now();
    let r2 =
        scanner::scan_source(&state, &sid, None, state.cancel_scan.clone(), None, None).unwrap();
    println!(
        "files={} created={} updated={} time={:?}",
        r2.files_found,
        r2.games_created,
        r2.games_updated,
        t1.elapsed()
    );
}
