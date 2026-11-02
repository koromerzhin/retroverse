use retroverse_lib::identify;
fn main() {
    let path = "/run/media/koromerzhin/WDC1/Emulation/roms/nes/Super Mario Bros. 3 (Europe).zip";
    let h = identify::compute_hashes(path, &[]);
    println!("content: {:?}", h);
    let data = std::fs::read(path).unwrap();
    let a = identify::compute_hashes_from_bytes(&data, "archive.bin", &[]);
    println!("archive: {:?}", a);
}
