//! AC1: Below advisory threshold → single Ok event, zero sweep invocations.
//!
//! Injects mock disk usage via BG_MOCK_DISK_TOTAL / BG_MOCK_DISK_FREE.
//! Points CAREEN_SURVEY_BIN / CAREEN_SWEEP_BIN at /bin/false to prove zero
//! subprocess calls (any invocation would make the binary return non-zero).

use careen_guard::event::{Event, Level};
use careen_guard::guard::RunArgs;
use tempfile::NamedTempFile;

/// 100 GB total, 20 GB free → 80% used → below default advisory (85%).
const TOTAL: u64 = 100 * 1024 * 1024 * 1024;
const FREE: u64 = 20 * 1024 * 1024 * 1024;

#[test]
fn acceptance_ac1_ok_event_below_advisory() {
    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
        // survey and sweep must NOT be invoked below advisory
        std::env::set_var("CAREEN_SURVEY_BIN", "/bin/false");
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
    assert_eq!(ev.level, Level::Ok, "expected Ok level");
    assert_eq!(ev.bytes_reclaimed, 0);
    assert_eq!(ev.reclaimable_bytes, 0);
    assert!(ev.candidates.is_empty());
}

/// RAII guard that removes env vars after the test.
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
