use retroverse_lib::commands::gamelists;
use retroverse_lib::state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new().unwrap();
    // find a remote source
    let sid = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT id FROM game_sources WHERE kind='remote' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default()
    };
    if sid.is_empty() {
        println!("no remote source");
        return;
    }
    println!("source id: {}", sid);
    match gamelists::import_gamelists_inner(&state, sid).await {
        Ok(n) => println!("IMPORTED: {}", n),
        Err(e) => println!("ERROR: {}", e),
    }
}
