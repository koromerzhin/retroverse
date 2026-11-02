use retroverse_lib::identify;
use retroverse_lib::remote;
#[tokio::main]
async fn main() {
    let sess = remote::connect(
        "192.168.1.125",
        22,
        "deck",
        "/home/koromerzhin/.ssh/id_ed25519",
    )
    .await
    .unwrap();
    let path = "/run/media/deck/WDC1/Emulation/roms/c64/Renegade (Europe).zip";
    println!("computing hashes...");
    let r = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        remote::compute_hashes(&sess.sftp, path, &[]),
    )
    .await;
    match r {
        Ok(h) => println!("result: {:?}", h),
        Err(_) => println!("TIMEOUT"),
    }
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
}
