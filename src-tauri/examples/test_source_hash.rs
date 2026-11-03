use retroverse_lib::commands::backup::compute_hashes_inner;
use retroverse_lib::state::AppState;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new().unwrap());
    let n = compute_hashes_inner(&state, None, None).await.unwrap();
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
