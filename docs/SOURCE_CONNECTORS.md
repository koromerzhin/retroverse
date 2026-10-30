# Source connectors

All connectors implement a common interface (planned for v0.2).

## LocalSource
- Path validation, directory traversal via walkdir.
- Metadata: size, mtime, extension.

## Planned connectors
- SSH/SFTP (russh or equivalent) — Steam Deck priority.
- SMB, NFS, NAS.

## Interface (target)
- `list(path) -> Vec<Entry>`
- `stat(path) -> Metadata`
- `read_range(path, offset, len) -> bytes`
- `copy_to_local(src, dst, progress)`

## Security
- No plaintext passwords; SSH keys preferred.
- Host key verification.
- Path traversal protection.
- Non-destructive defaults.