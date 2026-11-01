use retroverse_lib::remote;
use retroverse_lib::identify;
#[tokio::main]
async fn main() {
    let sess = remote::connect("192.168.1.125", 22, "deck", "/home/koromerzhin/.ssh/id_ed25519").await.unwrap();
    let path = "/run/media/deck/WDC1/Emulation/roms/c64/Renegade (Europe).zip";
    // read systeminfo
    let mut f = sess.sftp.open("/run/media/deck/WDC1/Emulation/roms/c64/systeminfo.txt").await.unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await.unwrap();
    let content = String::from_utf8(buf).unwrap();
    let exts = identify::parse_systeminfo_txt(&content).unwrap_or_default();
    println!("allowed exts: {:?}", exts);
    println!("computing hashes (30s timeout)...");
    let r = tokio::time::timeout(std::time::Duration::from_secs(30), remote::compute_hashes(&sess.sftp, path, &exts)).await;
    match r {
        Ok(h) => println!("result: {:?}", h),
        Err(_) => println!("TIMEOUT"),
    }
    let _ = sess.session.disconnect(russh::Disconnect::ByApplication, "", "").await;
}