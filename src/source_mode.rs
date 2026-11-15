//! Which game sources the scanner should discover.
//!
//! This module deliberately has no FreeBSD/PS5 dependencies. Run its tests on the host:
//! `rustc --test src/source_mode.rs -o /tmp/kraken-source-mode-tests && /tmp/kraken-source-mode-tests`.
//! Backports are associated with an accepted game independently of this discovery filter.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceMode {
    #[default]
    All,
    Pkg,
    Folder,
}

impl SourceMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Pkg => "pkg",
            Self::Folder => "folder",
        }
    }

    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "all" => Ok(Self::All),
            "pkg" => Ok(Self::Pkg),
            "folder" => Ok(Self::Folder),
            _ => Err("expected all, pkg or folder"),
        }
    }

    pub const fn allows_pkg(self) -> bool {
        matches!(self, Self::All | Self::Pkg)
    }

    pub const fn allows_folder(self) -> bool {
        matches!(self, Self::All | Self::Folder)
    }
}

#[cfg(test)]
mod tests {
    use super::SourceMode;

    #[test]
    fn default_preserves_previous_scanner_behavior() {
        assert_eq!(SourceMode::default(), SourceMode::All);
        assert!(SourceMode::All.allows_pkg());
        assert!(SourceMode::All.allows_folder());
    }

    #[test]
    fn pkg_mode_does_not_discover_game_folders() {
        assert!(SourceMode::Pkg.allows_pkg());
        assert!(!SourceMode::Pkg.allows_folder());
    }

    #[test]
    fn folder_mode_does_not_discover_pkgs() {
        assert!(!SourceMode::Folder.allows_pkg());
        assert!(SourceMode::Folder.allows_folder());
    }

    #[test]
    fn roundtrip_supported_modes() {
        for mode in [SourceMode::All, SourceMode::Pkg, SourceMode::Folder] {
            assert_eq!(SourceMode::parse(mode.as_str()), Ok(mode));
        }
    }

    #[test]
    fn invalid_mode_is_rejected_not_silently_changed() {
        for invalid in ["", "PKG", "folders", "all, pkg", "../pkg", "true"] {
            assert!(SourceMode::parse(invalid).is_err());
        }
    }
}
