use retroverse_lib::config::load;
use retroverse_lib::metadata::MetadataProvider;
use retroverse_lib::metadata::ScreenScraperProvider;
fn main() {
    let cfg = load();
    let prov = ScreenScraperProvider { cfg };
    // hashes none
    match prov.fetch("Super Mario Bros 3", Some("nes"), None) {
        Ok(r) => println!(
            "OK title={:?} pub={:?} dev={:?} genres={:?} synopsis_len={}",
            r.title,
            r.publisher,
            r.developer,
            r.genres,
            r.description.as_ref().map(|s| s.len()).unwrap_or(0)
        ),
        Err(e) => println!("ERR: {:?}", e),
    }
}
