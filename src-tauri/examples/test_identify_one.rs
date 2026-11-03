use retroverse_lib::identify;
fn main() {
    let path = "/run/media/koromerzhin/WDC1/Emulation/roms/gx4000/Baba's Palace (Rafael Castillo, John McKlain & Dragon).7z";
    let h = identify::compute_hashes(path, &[]);
    println!("content: {:?}", h);
}
