use retroverse_lib::remote;
use retroverse_lib::error::Result;

#[tokio::main]
async fn main() {
    let host = "192.168.1.125";
    let port = 22u16;
    let user = "deck";
    let key = "/home/koromerzhin/.ssh/id_ed25519";
    match remote::connect(host, port, user, key).await {
        Ok(sess) => {
            println!("Connected. canonicalize . = {:?}", sess.sftp.canonicalize(".").await);
            match sess.sftp.read_dir("/run/media/deck/WDC1/Emulation/").await {
                Ok(rd) => {
                    let entries: Vec<_> = rd.collect();
                    println!("entries: {}", entries.len());
                    for e in entries.iter().take(10) {
                        println!("  {}", e.file_name());
                    }
                }
                Err(e) => println!("read_dir err: {}", e),
            }
            let _ = sess.session.disconnect(russh::Disconnect::ByApplication, "", "").await;
        }
        Err(e) => println!("connect err: {}", e),
    }
}