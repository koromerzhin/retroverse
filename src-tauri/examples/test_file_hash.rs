use retroverse_lib::commands::backup::compute_hash_for_file_inner;
use retroverse_lib::state::AppState;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new().unwrap());
    // find file id
    let fid: String = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT id FROM game_files WHERE platform_id='gx4000' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap()
    };
    println!("file_id={}", fid);
    let n = compute_hash_for_file_inner(&state, fid.clone())
        .await
        .unwrap();
    println!("done={}", n);
    let conn = state.db.lock().unwrap();
    let (h, ah, sh512, ash512): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT hash, archive_hash, sha512, archive_sha512 FROM game_files WHERE id=?1",
            [fid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    println!(
        "hash={:?} archive_hash={:?} sha512={:?} archive_sha512={:?}",
        h, ah, sh512, ash512
    );
}
