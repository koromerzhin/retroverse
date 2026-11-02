use retroverse_lib::commands::backup;
use retroverse_lib::state::AppState;
#[tokio::main]
async fn main() {
    let state = AppState::new().unwrap();
    // find game with Musicbox
    let gid = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT id FROM games WHERE title LIKE '%Musicbox%' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default()
    };
    println!("game id: {}", gid);
    let n = backup::compute_hashes_for_game_inner(&state, gid.clone())
        .await
        .unwrap();
    println!("computed: {}", n);
    let conn = state.db.lock().unwrap();
    let (c, m, s): (Option<String>, Option<String>, Option<String>) = conn.query_row("SELECT crc32, md5, sha1 FROM game_files gf JOIN game_file_links l ON l.file_id=gf.id WHERE l.game_id=?1 LIMIT 1", [gid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap_or((None,None,None));
    println!(
        "crc={:?} md5={:?} sha1={:?}",
        c,
        m,
        s.map(|x| x[..12].to_string())
    );
}
