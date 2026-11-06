# Source connectors

All connectors implement a common interface.

## LocalSource
- Path validation, directory traversal via walkdir.
- Metadata: size, mtime, extension.

## RemoteSource (SSH/SFTP)
- Implemented via russh + russh-sftp.
- Steam Deck priority.
- Remote browsing, scan, incremental sync.
- Optional local copy with resume + integrity.

## Interface
- `list(path) -> Vec<Entry>`
- `stat(path) -> Metadata`
- `read_range(path, offset, len) -> bytes`
- `copy_to_local(src, dst, progress)`

## Security
- No plaintext passwords; SSH keys preferred.
- Host key verification (TOFU).
- Path traversal protection.
- Non-destructive defaults.
