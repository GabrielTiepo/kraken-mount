//! Lightweight baseline diagnostics for the Kraken Mount management process.
//!
//! These timings are **not** per-block AMPR runtime I/O timings and must not be
//! interpreted as USB throughput or game FPS. The most recent measurements live
//! in RAM and are reset when the payload restarts.
//!
//! Pure Rust module: host check with
//! `rustc --edition=2021 --deny warnings --test src/diagnostics.rs -o /tmp/kraken-diag-tests && /tmp/kraken-diag-tests`.

use std::time::Duration;

const BUILD_HISTORY: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanSample {
    pub elapsed_ms: u64,
    pub games: usize,
    pub duplicates: usize,
    pub errors: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildSample {
    pub title_id: String,
    pub drive: String,
    pub kind: &'static str,
    pub elapsed_ms: u64,
    pub success: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub scan: Option<ScanSample>,
    pub builds: Vec<BuildSample>,
}

fn milliseconds(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

impl Diagnostics {
    pub fn record_scan(&mut self, elapsed: Duration, games: usize, duplicates: usize, errors: usize) {
        self.scan = Some(ScanSample {
            elapsed_ms: milliseconds(elapsed),
            games,
            duplicates,
            errors,
        });
    }

    pub fn record_build(&mut self, title_id: &str, drive: &str, kind: &'static str, elapsed: Duration, success: bool) {
        // Bound memory even when a console mounts many games during one session.
        if self.builds.len() == BUILD_HISTORY {
            self.builds.remove(0);
        }
        self.builds.push(BuildSample {
            title_id: title_id.to_owned(),
            drive: drive.to_owned(),
            kind,
            elapsed_ms: milliseconds(elapsed),
            success,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_reports_unavailable_metrics() {
        let state = Diagnostics::default();
        assert!(state.scan.is_none());
        assert!(state.builds.is_empty());
    }

    #[test]
    fn scan_counts_are_replaced_by_latest_pass() {
        let mut state = Diagnostics::default();
        state.record_scan(Duration::from_millis(250), 4, 1, 0);
        assert_eq!(state.scan.as_ref().unwrap().elapsed_ms, 250);
        state.record_scan(Duration::from_micros(19_400), 3, 0, 2);
        assert_eq!(state.scan, Some(ScanSample { elapsed_ms: 19, games: 3, duplicates: 0, errors: 2 }));
    }

    #[test]
    fn build_timings_preserve_failure_state_and_source_type() {
        let mut state = Diagnostics::default();
        state.record_build("PPSA35889", "/mnt/usb0", "pkg", Duration::from_secs(2), false);
        assert_eq!(state.builds[0], BuildSample {
            title_id: "PPSA35889".into(),
            drive: "/mnt/usb0".into(),
            kind: "pkg",
            elapsed_ms: 2000,
            success: false,
        });
    }

    #[test]
    fn rolling_history_keeps_only_eight_most_recent_builds() {
        let mut state = Diagnostics::default();
        for i in 0..12 {
            state.record_build(&format!("PPSA{i:05}"), "/data", "folder", Duration::from_millis(i), true);
        }
        assert_eq!(state.builds.len(), 8);
        assert_eq!(state.builds[0].title_id, "PPSA00004");
        assert_eq!(state.builds[7].title_id, "PPSA00011");
    }
}
