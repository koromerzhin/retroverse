use retroverse_lib::identify;
#[tokio::main]
async fn main() {
    let sess = retroverse_lib::remote::connect(
        "192.168.1.125",
        22,
        "deck",
        "/home/koromerzhin/.ssh/id_ed25519",
    )
    .await
    .unwrap();
    let mut f = sess.sftp.open("/run/media/deck/WDC1/Emulation/roms/nes/Koudai Guaishou III (China) (Aftermarket) (Pirate).zip").await.unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await.unwrap();
    let tmp = std::env::temp_dir().join("test2.zip");
    std::fs::write(&tmp, &buf).unwrap();
    let allowed = vec![
        ".3dsen".to_string(),
        ".fds".to_string(),
        ".nes".to_string(),
        ".unf".to_string(),
        ".unif".to_string(),
        ".7z".to_string(),
        ".zip".to_string(),
    ];
    let r = identify::compute_hashes(tmp.to_str().unwrap(), &allowed);
    println!("result: {:?}", r);
    let _ = std::fs::remove_file(&tmp);
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
}
