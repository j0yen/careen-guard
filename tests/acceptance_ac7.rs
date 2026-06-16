//! AC7: --event-sink appends exactly one JSON line per pass to the named file.

use careen_guard::event::Event;
use careen_guard::guard::RunArgs;
use tempfile::NamedTempFile;

/// Below advisory (80% used) → Ok event written to sink.
const TOTAL: u64 = 100 * 1024 * 1024 * 1024;
const FREE: u64 = 20 * 1024 * 1024 * 1024;

#[test]
fn acceptance_ac7_event_sink_appends_json_line() {
    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
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

    careen_guard::guard::run(&args).expect("guard run");

    let content = std::fs::read_to_string(&sink_path).expect("read sink");
    let lines: Vec<&str> = content.lines().collect();

    // Exactly one JSON line (AC7)
    assert_eq!(lines.len(), 1, "exactly one line in sink");
    // The line must parse as a valid Event
    let _ev: Event = serde_json::from_str(lines[0]).expect("valid Event JSON in sink");
}

#[test]
fn acceptance_ac7_event_sink_appends_not_overwrites() {
    unsafe {
        std::env::set_var("BG_MOCK_DISK_TOTAL", TOTAL.to_string());
        std::env::set_var("BG_MOCK_DISK_FREE", FREE.to_string());
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

    // Run twice — sink should have 2 lines (append, not overwrite)
    careen_guard::guard::run(&args).expect("first run");
    careen_guard::guard::run(&args).expect("second run");

    let content = std::fs::read_to_string(&sink_path).expect("read sink");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "two runs → two lines in sink");
    for line in &lines {
        let _ev: Event = serde_json::from_str(line).expect("valid Event JSON");
    }
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
