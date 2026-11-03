use retroverse_lib::config::load;
use retroverse_lib::metadata::MetadataProvider;
use retroverse_lib::metadata::TheGamesDbProvider;
fn main() {
    let cfg = load();
    let prov = TheGamesDbProvider { cfg };
    match prov.fetch("Super Mario Bros", Some("nes")) {
        Ok(r) => println!("OK: {:?}", r),
        Err(e) => println!("ERR: {:?}", e),
    }
}
