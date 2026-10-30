use retroverse_lib::remote;
use retroverse_lib::identify;

#[tokio::main]
async fn main() {
    let host = "192.168.1.125";
    let port = 22u16;
    let user = "deck";
    let key = "/home/koromerzhin/.ssh/id_ed25519";
    let sess = remote::connect(host, port, user, key).await.unwrap();
    let base = "/run/media/deck/WDC1/Emulation/roms/nes/";
    let mut collected: Vec<(String, i64, i64)> = Vec::new();
    // replicate walk
    async fn walk(sftp: &russh_sftp::client::SftpSession, dir: &str, exts: &[String], out: &mut Vec<(String,i64,i64)>) {
        if let Ok(entries) = sftp.read_dir(dir).await {
            for e in entries {
                let name = e.file_name();
                if name == "." || name == ".." { continue; }
                let meta = e.metadata();
                let is_dir = meta.is_dir();
                let full = if dir.ends_with('/') { format!("{}{}", dir, name) } else { format!("{}/{}", dir, name) };
                if is_dir {
                    Box::pin(walk(sftp, &full, exts, out)).await;
                } else {
                    let ext = std::path::Path::new(&name).extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
                    let ext_dot = format!(".{}", ext);
                    if exts.iter().any(|x| x.eq_ignore_ascii_case(&ext_dot)) {
                        let size = meta.size.unwrap_or(0) as i64;
                        let mtime = meta.mtime.unwrap_or(0) as i64;
                        out.push((full, size, mtime));
                    }
                }
            }
        } else {
            println!("read_dir failed for {}", dir);
        }
    }
    // load platforms from real db
    let conn = retroverse_lib::db::open().unwrap();
    let platforms = identify::load_platforms(&conn).unwrap();
    let exts: Vec<String> = platforms.iter().flat_map(|p| p.extensions.clone()).collect();
    println!("platform exts count: {}", exts.len());
    // debug: list first entries
    if let Ok(rd) = sess.sftp.read_dir(base).await {
        let v: Vec<_> = rd.collect();
        println!("nes entries: {}", v.len());
        for e in v.iter().take(5) {
            println!("  name='{}' is_dir={}", e.file_name(), e.metadata().is_dir());
        }
    } else {
        println!("read_dir failed");
    }
    walk(&sess.sftp, base, &exts, &mut collected).await;
    println!("collected: {}", collected.len());
    for c in collected.iter().take(5) { println!("  {}", c.0); }
    let _ = sess.session.disconnect(russh::Disconnect::ByApplication, "", "").await;
}