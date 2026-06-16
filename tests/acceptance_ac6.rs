//! AC6: careen-guard never selects a dir whose binary is stale/uninstalled.
//!
//! Fixture: survey-mixed.sh returns one is_live=false (ballast's domain) and
//! one is_live=true. Only the live one must appear in candidates.
#![allow(unsafe_code)]

use careen_guard::event::Event;
use careen_guard::guard::RunArgs;
use tempfile::NamedTempFile;

/// 100 GB total, 2 GB free → 98% used → breach.
const TOTAL: u64 = 100 * 1024 * 1024 * 1024;
const FREE: u64 = 2 * 1024 * 1024 * 1024;

#[test]
fn acceptance_ac6_only_live_dirs_swept() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
        // survey-mixed.sh: one ballast-eligible (is_live=false), one careen-eligible
        std::env::set_var("CAREEN_SURVEY_BIN", format!("{fixtures}/survey-mixed.sh"));
        std::env::set_var("CAREEN_SWEEP_BIN", format!("{fixtures}/sweep-ok.sh"));
    }
    let _cleanup = EnvCleanup;

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

    // The ballast-eligible dir (old-crate, is_live=false) must NOT be in candidates
    let has_ballast_dir = ev.candidates.iter().any(|p| p.contains("old-crate"));
    assert!(!has_ballast_dir, "ballast-eligible dir must not be swept");

    // The live dir must be present (it was attempted)
    let has_live_dir = ev.candidates.iter().any(|p| p.contains("recall"));
    assert!(has_live_dir, "live dir must appear in candidates");
}

struct EnvCleanup;
impl Drop for EnvCleanup {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var("BG_MOCK_DISK_TOTAL");
            std::env::remove_var("BG_MOCK_DISK_FREE");
            std::env::remove_var("CAREEN_SURVEY_BIN");
            std::env::remove_var("CAREEN_SWEEP_BIN");
        }
    }
}
