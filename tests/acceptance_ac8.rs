//! AC8: Build-locked candidate is skipped; sweep continues to next candidate.
//!
//! sweep-locked-first.sh: first call returns exit 2 (locked), second returns 0.
//! survey-breach-two.sh: two live candidates, each ~9 GB reclaimable.
//! After first is locked and second is swept (~9 GB), usage drops below 90%.
#![allow(unsafe_code)]

use careen_guard::event::{Event, Level};
use careen_guard::guard::RunArgs;
use tempfile::NamedTempFile;

/// 100 GB total, 2 GB free → 98% used.
const TOTAL: u64 = 100 * 1024 * 1024 * 1024;
const FREE: u64 = 2 * 1024 * 1024 * 1024;

#[test]
fn acceptance_ac8_locked_candidate_skipped_continues_to_next() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let state_file = format!(
        "/tmp/careen-guard-sweep-locked-state-ac8-{}",
        std::process::id()
    );

    // Clean up any leftover state from previous runs
    let _ = std::fs::remove_file(&state_file);

    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
        std::env::set_var(
            "CAREEN_SURVEY_BIN",
            format!("{fixtures}/survey-breach-two.sh"),
        );
        std::env::set_var(
            "CAREEN_SWEEP_BIN",
            format!("{fixtures}/sweep-locked-first.sh"),
        );
        // Tell sweep-locked-first.sh where to put its state
        std::env::set_var("CAREEN_SWEEP_STATE_FILE", &state_file);
    }
    let _cleanup = EnvCleanup {
        state_file: state_file.clone(),
    };

    let cfg_file = NamedTempFile::new().expect("tempfile");
    std::fs::write(
        cfg_file.path(),
        "high_water_pct = 90\nlow_water_pct = 80\nadvisory_pct = 85\n",
    )
    .expect("write config");

    let sink = NamedTempFile::new().expect("tempfile");
    let sink_path = sink.path().to_path_buf();

    let args = RunArgs {
        config: Some(cfg_file.path().to_path_buf()),
        mount: std::path::PathBuf::from("/"),
        event_sink: Some(sink_path.clone()),
    };

    careen_guard::guard::run(&args).expect("guard run should succeed");

    let content = std::fs::read_to_string(&sink_path).expect("read sink");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1);

    let ev: Event = serde_json::from_str(lines[0]).expect("parse event JSON");

    // First candidate (recall) was locked → skipped; second (brain) was swept.
    assert!(
        ev.bytes_reclaimed > 0,
        "bytes_reclaimed must be > 0 (second candidate swept)"
    );

    // The locked candidate must NOT appear in swept paths
    let has_locked = ev.candidates.iter().any(|p| p.contains("recall"));
    assert!(
        !has_locked,
        "locked (first) candidate must not appear in swept paths"
    );

    // The second candidate MUST appear in swept paths
    let has_second = ev.candidates.iter().any(|p| p.contains("wintermute-brain"));
    assert!(
        has_second,
        "second (unlocked) candidate must appear in swept paths"
    );

    // Breach resolved after second candidate (~9GB of 100GB = 9% reduction: 98% → 89%)
    assert_eq!(ev.level, Level::Breach, "breach resolved after second candidate");
}

struct EnvCleanup {
    state_file: String,
}
impl Drop for EnvCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.state_file);
        unsafe {
            std::env::remove_var("BG_MOCK_DISK_TOTAL");
            std::env::remove_var("BG_MOCK_DISK_FREE");
            std::env::remove_var("CAREEN_SURVEY_BIN");
            std::env::remove_var("CAREEN_SWEEP_BIN");
            std::env::remove_var("CAREEN_SWEEP_STATE_FILE");
        }
    }
}
