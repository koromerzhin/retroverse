use retroverse_lib::identify;
fn main() {
    let content = "System name:\nnes\n\nFull system name:\nNintendo Entertainment System\n\nSupported file extensions:\n.3dsen .3DSEN .fds .FDS .nes .NES .unf .UNF .unif .UNIF .7z .7Z .zip .ZIP\n\nLaunch command:\n%EMULATOR_RETROARCH%\n";
    let r = identify::parse_systeminfo_txt(content);
    println!("{:?}", r);
}
