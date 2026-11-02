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
    let mut f = sess.sftp.open("/run/media/deck/WDC1/Emulation/roms/nes/!Clik! (World) (Proto 1) (Aftermarket) (Unl).zip").await.unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await.unwrap();
    let tmp = std::env::temp_dir().join("test1.zip");
    std::fs::write(&tmp, &buf).unwrap();
    let allowed = vec![".nes".to_string()];
    let r = identify::compute_hashes(tmp.to_str().unwrap(), &allowed);
    println!("result: {:?}", r);
    let _ = std::fs::remove_file(&tmp);
    let _ = sess
        .session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await;
}
