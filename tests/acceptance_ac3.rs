//! AC3: Breach → selects live target dirs descending by size, emits Breach event
//! with bytes_reclaimed > 0 and swept paths in candidates.

use careen_guard::event::{Event, Level};
use careen_guard::guard::RunArgs;
use tempfile::NamedTempFile;

/// 100 GB total, 2 GB free → 98% used → breach (above 90%).
const TOTAL: u64 = 100 * 1024 * 1024 * 1024;
const FREE: u64 = 2 * 1024 * 1024 * 1024; // 98% used

#[test]
fn acceptance_ac3_breach_selects_descending_and_sweeps() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
        // survey-breach.sh returns two live candidates; combined they resolve breach
        std::env::set_var(
            "CAREEN_SURVEY_BIN",
            format!("{fixtures}/survey-breach.sh"),
        );
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
    assert_eq!(lines.len(), 1, "exactly one event line");

    let ev: Event = serde_json::from_str(lines[0]).expect("parse event JSON");
    assert_eq!(ev.level, Level::Breach, "expected Breach level");
    assert!(ev.bytes_reclaimed > 0, "bytes_reclaimed must be > 0");
    assert!(!ev.candidates.is_empty(), "candidates must be non-empty");

    // Verify descending-size order: first path should be the larger one from fixture
    // survey-breach.sh returns recall (9.6G) before brain (6.4G)
    assert_eq!(
        ev.candidates[0],
        "/home/jsy/wintermute/recall/target",
        "largest candidate first"
    );
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
