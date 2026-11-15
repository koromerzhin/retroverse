Installers for every supported platform are attached below.

| Platform | Files |
| --- | --- |
| Windows | `.msi` (WiX) and `-setup.exe` (NSIS) |
| macOS | `.dmg` — universal, runs on Apple Silicon and Intel |
| Linux | `.AppImage`, `.deb` (Debian/Ubuntu), `.rpm` (Fedora) |

### Notes

- These builds are **not code signed**. Windows shows a SmartScreen warning
  (More info → Run anyway). On macOS, right-click the app and choose Open the
  first time, or run `xattr -dr com.apple.quarantine /Applications/retroverse.app`.
- The Linux builds target glibc 2.35 (Ubuntu 22.04) or newer and require
  WebKitGTK 4.1. The `.AppImage` needs `libfuse2` on distributions that no
  longer ship it (`sudo apt install libfuse2` on Ubuntu 22.04+).
- Your library, database and logs stay in the standard per-user data directory
  (`~/.local/share/retroverse/` on Linux, `%APPDATA%\retroverse\` on Windows,
  `~/Library/Application Support/retroverse/` on macOS).

Please report problems at https://github.com/koromerzhin/retroverse/issues
