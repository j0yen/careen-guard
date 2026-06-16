//! AC5: Schema compatibility — careen-guard Event JSON deserializes against
//! ballast-guard's Event type (field-for-field serde parity).
//!
//! Since careen-guard cannot link against ballast-guard (separate crates, no
//! workspace), this test uses a local replica of the ballast-guard Event struct
//! with the same serde contract and verifies round-trip for all four Level variants.

use careen_guard::event::{Event, Level};
use serde::{Deserialize, Serialize};

/// Local replica of ballast-guard's Event/Level for AC5 parity check.
/// Field names, types, and serde attributes must match exactly.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BallastLevel {
    Ok,
    Warn,
    Breach,
    BreachUnresolved,
}

#[derive(Debug, Deserialize, Serialize)]
struct BallastEvent {
    level: BallastLevel,
    used_pct_before: u8,
    used_pct_after: u8,
    bytes_reclaimed: u64,
    reclaimable_bytes: u64,
    candidates: Vec<String>,
    ts: String,
}

fn round_trip_level(level: Level, expected_ballast_level: &str) {
    let ev = Event::new(level, 50, 50, 0, 0, vec![]);
    let json = serde_json::to_string(&ev).expect("serialize careen Event");

    // Deserialize into ballast-compatible struct
    let ballast_ev: BallastEvent = serde_json::from_str(&json)
        .expect("careen Event JSON must deserialize as BallastEvent");

    // Verify the level string matches ballast's expectation
    let level_json = serde_json::to_string(&ballast_ev.level).expect("serialize level");
    assert_eq!(
        level_json,
        format!("\"{expected_ballast_level}\""),
        "level serde string mismatch"
    );

    // Verify numeric fields round-trip
    assert_eq!(ballast_ev.used_pct_before, 50);
    assert_eq!(ballast_ev.used_pct_after, 50);
    assert_eq!(ballast_ev.bytes_reclaimed, 0);
    assert_eq!(ballast_ev.reclaimable_bytes, 0);
    assert!(ballast_ev.candidates.is_empty());
    assert!(!ballast_ev.ts.is_empty());
}

#[test]
fn acceptance_ac5_schema_compat_ok() {
    round_trip_level(Level::Ok, "ok");
}

#[test]
fn acceptance_ac5_schema_compat_warn() {
    round_trip_level(Level::Warn, "warn");
}

#[test]
fn acceptance_ac5_schema_compat_breach() {
    round_trip_level(Level::Breach, "breach");
}

#[test]
fn acceptance_ac5_schema_compat_breach_unresolved() {
    round_trip_level(Level::BreachUnresolved, "breach-unresolved");
}

#[test]
fn acceptance_ac5_full_event_round_trip() {
    let candidates = vec![
        "/home/jsy/wintermute/recall/target".to_owned(),
        "/home/jsy/wintermute/brain/target".to_owned(),
    ];
    let ev = Event::new(Level::Breach, 95, 82, 12_884_901_888, 0, candidates.clone());
    let json = serde_json::to_string(&ev).expect("serialize");
    let ballast: BallastEvent = serde_json::from_str(&json).expect("deserialize as ballast");
    assert_eq!(ballast.used_pct_before, 95);
    assert_eq!(ballast.used_pct_after, 82);
    assert_eq!(ballast.bytes_reclaimed, 12_884_901_888);
    assert_eq!(ballast.candidates, candidates);
}
