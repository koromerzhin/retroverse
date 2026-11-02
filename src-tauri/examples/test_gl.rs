use retroverse_lib::remote;
#[tokio::main]
async fn main() {
    let host = "192.168.1.125";
    let sess = remote::connect(host, 22, "deck", "/home/koromerzhin/.ssh/id_ed25519")
        .await
        .unwrap();
    let emu = "/run/media/deck/WDC1/Emulation";
    let gl_dir = format!("{}/saves/es-de/gamelists", emu);
    println!("reading {}", gl_dir);
    match sess.sftp.read_dir(&gl_dir).await {
        Ok(rd) => {
            let dirs: Vec<_> = rd.collect();
            println!("gamelists dirs: {}", dirs.len());
            for d in dirs.iter().take(5) {
                println!("  {}", d.file_name());
            }
        }
        Err(e) => println!("read_dir err: {}", e),
    }
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
}
