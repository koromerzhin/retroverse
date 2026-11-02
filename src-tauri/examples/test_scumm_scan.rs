use retroverse_lib::scanner;
use retroverse_lib::state::AppState;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new().unwrap());
    let sid = "faff7a65-3ded-4010-a794-a4aa8bd5aec3".to_string();
    // reset one title to short code
    {
        let conn = state.db.lock().unwrap();
        let _ = conn.execute(
            "UPDATE games SET title='dw' WHERE title='Discworld' AND platform_id='scummvm'",
            [],
        );
    }
    let r = scanner::scan_source(
        &state,
        &sid,
        None,
        state.cancel_scan.clone(),
        None,
        Some("scummvm".to_string()),
    );
    println!("scan result: {:?}", r);
    let conn = state.db.lock().unwrap();
    let title: String = conn.query_row("SELECT title FROM games WHERE platform_id='scummvm' AND LOWER(title) IN ('dw','discworld')", [], |r| r.get(0)).unwrap_or_default();
    println!("dw/discworld title now: {}", title);
}
