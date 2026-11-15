//! Fail-closed preflight for purge when the cached source lives on a removed
//! PS5 USB or extended volume. This does not identify individual USB devices and
//! does not guarantee immunity to a drive being unplugged *during* the purge.
//! It prevents the common error of treating an unmounted source as deleted.
//!
//! Host tests: rustc --edition=2021 --deny warnings --test src/removable_guard.rs -o /tmp/kraken-removable-tests && /tmp/kraken-removable-tests

/// Returns the mount root when the source is on a currently absent removable drive.
/// Exact /mnt/usb0..usb7 and /mnt/ext0..ext1 only; reject prefix ambiguity like usb01.
pub fn unavailable_mount(source: &str, mounted: &[String]) -> Option<String> {
    let rest = source.strip_prefix("/mnt/")?;
    let label = rest.split('/').next()?;
    let recognized = if let Some(n) = label.strip_prefix("usb") {
        n.len() == 1 && matches!(n.as_bytes()[0], b'0'..=b'7')
    } else if let Some(n) = label.strip_prefix("ext") {
        n.len() == 1 && matches!(n.as_bytes()[0], b'0'..=b'1')
    } else {
        false
    };
    if !recognized {
        return None;
    }
    let root = format!("/mnt/{label}");
    (!mounted.iter().any(|m| m == &root)).then_some(root)
}

#[cfg(test)]
mod tests {
    use super::unavailable_mount;

    fn mounted() -> Vec<String> {
        vec!["/mnt/usb0".into(), "/mnt/ext1".into()]
    }

    #[test]
    fn unavailable_usb_blocks_purge() {
        assert_eq!(unavailable_mount("/mnt/usb1/homebrew/game.pkg", &mounted()), Some("/mnt/usb1".into()));
    }

    #[test]
    fn mounted_removable_sources_are_accepted() {
        assert_eq!(unavailable_mount("/mnt/usb0/game.pkg", &mounted()), None);
        assert_eq!(unavailable_mount("/mnt/ext1/homebrew/game", &mounted()), None);
    }

    #[test]
    fn absent_extended_drive_blocks_purge() {
        assert_eq!(unavailable_mount("/mnt/ext0/homebrew/game.pkg", &mounted()), Some("/mnt/ext0".into()));
    }

    #[test]
    fn internal_sources_are_not_removable() {
        for path in ["/data/homebrew/game.pkg", "/user/app/PPSA12345", "/mnt", "E:/games/game.pkg"] {
            assert_eq!(unavailable_mount(path, &mounted()), None);
        }
    }

    #[test]
    fn prefix_spoofing_and_unsupported_numbers_are_ignored() {
        for path in [
            "/mnt/usb01/homebrew/game.pkg", "/mnt/usb8/game.pkg",
            "/mnt/ext2/game.pkg", "/mnt/usb0-other/game.pkg", "/mnt/usb0evil/game.pkg",
        ] {
            assert_eq!(unavailable_mount(path, &mounted()), None);
        }
    }

    #[test]
    fn empty_mount_list_blocks_recognized_removable_sources() {
        assert_eq!(unavailable_mount("/mnt/usb0/game.pkg", &[]), Some("/mnt/usb0".into()));
        assert_eq!(unavailable_mount("/mnt/ext1/game", &[]), Some("/mnt/ext1".into()));
    }
}
