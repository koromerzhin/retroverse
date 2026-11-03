use retroverse_lib::commands::backup::compute_hashes_inner;
use retroverse_lib::state::AppState;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new().unwrap());
    // Clear archive_hash to simulate migration
    {
        let conn = state.db.lock().unwrap();
        conn.execute(
            "UPDATE game_files SET archive_hash=NULL WHERE platform_id='gx4000'",
            [],
        )
        .unwrap();
    }
    let n = compute_hashes_inner(
        &state,
        None,
        Some("faff7a65-3ded-4010-a794-a4aa8bd5aec3".to_string()),
    )
    .await
    .unwrap();
    println!("source done={}", n);
    let conn = state.db.lock().unwrap();
    let (ah,): (Option<String>,) = conn
        .query_row(
            "SELECT archive_hash FROM game_files WHERE platform_id='gx4000' LIMIT 1",
            [],
            |r| Ok((r.get(0)?,)),
        )
        .unwrap();
    println!("archive_hash after={:?}", ah);
}
