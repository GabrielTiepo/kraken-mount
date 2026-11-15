//! Lightweight metadata inventory for a PKG's backport overlay.
//! This does not read game-file contents, and is NOT a cryptographic fingerprint.
//! Used only to prove that the set, sizes and mtimes of non-library files have
//! NOT changed before choosing a fakelib-only refresh over a full PKG reindex.
//! If an inventory cannot be taken (symlink, invalid filename, I/O error), callers
//! must fall back to the existing full-rebuild behavior.
//!
//! Native host tests: rustc --edition=2021 --deny warnings --test
//! src/backport_inventory.rs -o /tmp/backport-inventory && /tmp/backport-inventory

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Component, Path};
use std::time::UNIX_EPOCH;

/// (path relative to scanned root, file size, mtime seconds, mtime nanos).
/// File paths are sorted for a stable representation independent of read_dir order.
pub type Inventory = Vec<(String, u64, u64, u32)>;

/// Game data: all regular files except both top-level backport library directories.
pub fn game_files(root: &Path) -> io::Result<Inventory> {
    inventory(root, true)
}

/// Library data: all regular files under the currently selected fakelib/fakelib2.
pub fn lib_files(root: &Path) -> io::Result<Inventory> {
    inventory(root, false)
}

fn inventory(root: &Path, exclude_libs: bool) -> io::Result<Inventory> {
    let mut files = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for e in fs::read_dir(&dir)? {
            let e = e?;
            let path = e.path();
            let rel = path.strip_prefix(root)
                .map_err(|_| io::Error::new(ErrorKind::InvalidData, "path escapes backport root"))?;
            let first = match rel.components().next() {
                Some(Component::Normal(c)) => c.to_str()
                    .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "non-UTF8 backport path"))?,
                _ => return Err(io::Error::new(ErrorKind::InvalidData, "invalid backport path")),
            };
            if exclude_libs && (first.eq_ignore_ascii_case("fakelib") || first.eq_ignore_ascii_case("fakelib2")) {
                continue;
            }
            let meta = fs::symlink_metadata(&path)?;
            // Never classify a symlink as unchanged by looking only at its link's
            // mtime: the target can change independently. Disallow fast path.
            if meta.file_type().is_symlink() {
                return Err(io::Error::new(ErrorKind::InvalidData, "symlink in backport; incremental refresh disabled"));
            }
            if meta.is_dir() {
                dirs.push(path);
            } else if meta.is_file() {
                let modified = meta.modified()?.duration_since(UNIX_EPOCH)
                    .map_err(|_| io::Error::new(ErrorKind::InvalidData, "negative file mtime"))?;
                let name = rel.to_str()
                    .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "non-UTF8 backport path"))?;
                files.push((name.replace('\\', "/"), meta.len(), modified.as_secs(), modified.subsec_nanos()));
            }
        }
    }
    files.sort_unstable();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Temp(PathBuf);
    impl Temp {
        fn new(label: &str) -> Self {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let dir = std::env::temp_dir().join(format!("kraken-backport-{label}-{}-{now}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn put(&self, name: &str, data: &[u8]) {
            let file = self.0.join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, data).unwrap();
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn game_inventory_ignores_both_library_directories() {
        let temp = Temp::new("exclude");
        temp.put("eboot.bin", b"patched");
        temp.put("module/game.prx", b"module");
        temp.put("fakelib/libSceAmpr.sprx", b"old");
        temp.put("fakelib2/another.sprx", b"new");
        let files = game_files(&temp.0).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].0, "eboot.bin");
        assert_eq!(files[1].0, "module/game.prx");
    }

    #[test]
    fn library_change_does_not_change_game_inventory() {
        let temp = Temp::new("lib");
        temp.put("eboot.bin", b"patched");
        temp.put("fakelib/libSceAmpr.sprx", b"old");
        let game_before = game_files(&temp.0).unwrap();
        let lib_before = lib_files(&temp.0.join("fakelib")).unwrap();
        temp.put("fakelib/libSceAmpr.sprx", b"changed-library");
        assert_eq!(game_before, game_files(&temp.0).unwrap());
        assert_ne!(lib_before, lib_files(&temp.0.join("fakelib")).unwrap());
    }

    #[test]
    fn changing_game_size_or_names_invalidates_inventory() {
        let temp = Temp::new("game");
        temp.put("eboot.bin", b"old");
        let first = game_files(&temp.0).unwrap();
        temp.put("eboot.bin", b"new-larger");
        assert_ne!(first, game_files(&temp.0).unwrap());
        let second = game_files(&temp.0).unwrap();
        temp.put("module/extra.prx", b"new");
        assert_ne!(second, game_files(&temp.0).unwrap());
    }

    #[test]
    fn empty_library_inventory_is_valid() {
        let temp = Temp::new("empty");
        fs::create_dir(temp.0.join("fakelib")).unwrap();
        assert!(lib_files(&temp.0.join("fakelib")).unwrap().is_empty());
        assert!(game_files(&temp.0).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_disables_fast_path() {
        use std::os::unix::fs::symlink;
        let temp = Temp::new("link");
        temp.put("target.bin", b"data");
        symlink(temp.0.join("target.bin"), temp.0.join("eboot.bin")).unwrap();
        assert_eq!(game_files(&temp.0).unwrap_err().kind(), ErrorKind::InvalidData);
    }
}
