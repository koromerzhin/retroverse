use retroverse_lib::identify;
fn main() {
    println!("{:?}", identify::scummvm_title("dw"));
    // simulate clean_title via identify_file?
    let gf = retroverse_lib::models::GameFile {
        id: "x".into(),
        source_id: "s".into(),
        path: "/tmp/dw.scummvm".into(),
        filename: "dw.scummvm".into(),
        extension: "scummvm".into(),
        size: 10,
        modified_at: 0,
        hash: None,
        crc32: None,
        md5: None,
        sha1: None,
        sha512: None,
        zip_inner: None,
        archive_hash: None,
        archive_crc32: None,
        archive_md5: None,
        archive_sha1: None,
        archive_sha512: None,
        platform_id: Some("scummvm".into()),
        created_at: chrono::Utc::now(),
        is_dir: false,
    };
    let (t, c) = identify::identify_file(&gf, &[]);
    println!("title={:?} conf={}", t, c);
}
