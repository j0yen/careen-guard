//! AC2: Advisory band → Warn event with non-zero reclaimable_bytes, zero deletions.

use careen_guard::event::{Event, Level};
use careen_guard::guard::RunArgs;
use tempfile::NamedTempFile;

/// 100 GB total, 13 GB free → 87% used → in advisory band (85%–90%).
const TOTAL: u64 = 100 * 1024 * 1024 * 1024;
const FREE: u64 = 13 * 1024 * 1024 * 1024; // 87% used

#[test]
fn acceptance_ac2_warn_event_advisory_band() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
        std::env::set_var(
            "CAREEN_SURVEY_BIN",
            format!("{fixtures}/survey-warn.sh"),
        );
        // sweep must NOT be invoked in advisory band
        std::env::set_var("CAREEN_SWEEP_BIN", "/bin/false");
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
    assert_eq!(lines.len(), 1, "exactly one event line");

    let ev: Event = serde_json::from_str(lines[0]).expect("parse event JSON");
    assert_eq!(ev.level, Level::Warn, "expected Warn level");
    assert!(ev.reclaimable_bytes > 0, "reclaimable_bytes must be non-zero");
    assert_eq!(ev.bytes_reclaimed, 0, "no bytes actually reclaimed in warn band");
    assert!(ev.candidates.is_empty(), "no candidates swept in warn band");
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
