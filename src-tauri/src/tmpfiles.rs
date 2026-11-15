//! Scratch files used while streaming remote content (hashing, archive checks).
//!
//! Remote ROMs are downloaded to a scratch file before hashing. Two rules keep
//! that safe on every platform:
//!
//! - scratch files live in the application data directory, not in the system
//!   temporary directory, which is often a small RAM-backed filesystem
//!   (`tmpfs` on Linux). A single Nintendo Switch dump can exceed 3 GB and
//!   filling `/tmp` breaks unrelated software;
//! - the file is removed by [`TempFile`]'s destructor, so errors, early returns
//!   and panics cannot leave it behind. Crash leftovers are swept at startup by
//!   [`sweep_orphans`].

use crate::error::Result;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Prefix shared by every scratch file, so orphans are easy to spot.
pub const PREFIX: &str = "retroverse_";
/// Scratch files younger than this are assumed to belong to a running job.
const ORPHAN_AGE: Duration = Duration::from_secs(3600);

/// Directory holding scratch files, created on demand.
///
/// Falls back to the system temporary directory only when the data directory
/// cannot be resolved.
pub fn scratch_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("retroverse")
        .join("scratch");
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// A scratch file removed when the guard is dropped.
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    /// Reserve a scratch path. The caller still has to create the file.
    pub fn new(tag: &str) -> Self {
        let path = scratch_dir().join(format!("{}{}", PREFIX, tag));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Path as `&str`, which is what the hashing helpers expect.
    pub fn as_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn is_stale(path: &Path) -> bool {
    match std::fs::metadata(path).and_then(|m| m.modified()) {
        Ok(modified) => SystemTime::now()
            .duration_since(modified)
            .map(|age| age > ORPHAN_AGE)
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn sweep_dir(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let named = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with(PREFIX))
            .unwrap_or(false);
        if named && path.is_file() && is_stale(&path) && std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Remove scratch files left behind by an earlier crashed or killed run.
///
/// Only files older than [`ORPHAN_AGE`] are touched, so a job started by a
/// previous instance that is still shutting down is never deleted underneath
/// it. The system temporary directory is swept too: older releases kept their
/// scratch files there.
pub fn sweep_orphans() -> usize {
    sweep_dir(&scratch_dir()) + sweep_dir(&std::env::temp_dir())
}

/// Cheap guard against filling the disk with a dump larger than the free space.
pub fn ensure_room_for(bytes: u64) -> Result<()> {
    // Leave headroom for the database, logs and hashing itself.
    const MARGIN: u64 = 256 * 1024 * 1024;
    let needed = bytes.saturating_add(MARGIN);
    if let Some(free) = available_space(&scratch_dir())
        && free < needed
    {
        return Err(crate::error::RetroError::Other(format!(
            "espace insuffisant pour le fichier temporaire : {} requis, {} disponibles",
            human_size(needed),
            human_size(free)
        )));
    }
    Ok(())
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["o", "Ko", "Mo", "Go"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", value, UNITS[unit])
}

/// Free bytes available to the current user, when the platform exposes it.
#[cfg(unix)]
fn available_space(dir: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c_path = CString::new(dir.as_os_str().as_bytes()).ok()?;
    // SAFETY: the struct is zero-initialised and only read after `statvfs`
    // returned 0, i.e. after the kernel filled it.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        Some(stat.f_bavail as u64 * stat.f_frsize as u64)
    }
}

/// Free-space check is skipped on Windows: a full disk surfaces as a write
/// error instead, which [`TempFile`] still cleans up.
#[cfg(not(unix))]
fn available_space(_dir: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn scratch_files_live_outside_the_system_temp_dir() {
        let file = TempFile::new("unit_test");
        assert!(file.as_str().starts_with(scratch_dir().to_str().unwrap()));
        assert!(
            !file
                .as_str()
                .starts_with(std::env::temp_dir().to_str().unwrap())
        );
    }

    #[test]
    fn drop_removes_the_file() {
        let file = TempFile::new("unit_test_drop");
        let path = file.path().to_path_buf();
        let mut handle = std::fs::File::create(&path).unwrap();
        handle.write_all(b"payload").unwrap();
        drop(handle);
        assert!(path.exists());
        drop(file);
        assert!(!path.exists());
    }

    #[test]
    fn sweep_keeps_fresh_files() {
        let file = TempFile::new("unit_test_sweep");
        std::fs::write(file.path(), b"still in use").unwrap();
        sweep_dir(&scratch_dir());
        assert!(file.path().exists(), "a fresh file must not be swept");
    }

    #[test]
    fn human_size_is_readable() {
        assert_eq!(human_size(512), "512.0 o");
        assert_eq!(human_size(3 * 1024 * 1024 * 1024), "3.0 Go");
    }
}
