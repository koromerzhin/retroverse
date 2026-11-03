use retroverse_lib::identify;
fn main() {
    let path = "/run/media/koromerzhin/WDC1/Emulation/roms/gx4000/Baba's Palace (Rafael Castillo, John McKlain & Dragon).7z";
    let data = std::fs::read(path).unwrap();
    let (s, c, m, h, zi, sh) = identify::compute_hashes_from_bytes(&data, "archive.bin", &[]);
    println!("archive: {:?} {:?} {:?} {:?} {:?}", s, c, m, h, sh);
}
