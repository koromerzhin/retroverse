use retroverse_lib::commands::backup::compute_hashes_inner;
use retroverse_lib::state::AppState;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new().unwrap());
    let n = compute_hashes_inner(&state, None, None).await.unwrap();
    println!("global done={}", n);
}
