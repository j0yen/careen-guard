//! Structured JSON event emission for careen-guard.
//!
//! The serde contract here is intentionally field-for-field identical to
//! `ballast_guard::event::Event` so that a careen-guard event JSON can be
//! deserialized by ballast-guard's type without modification (AC5).

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Event level — same names and kebab-case serialization as ballast-guard's `Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// Usage below advisory threshold; no action needed.
    Ok,
    /// Usage in advisory band; alert only, no sweep.
    Warn,
    /// Breach: sweep ran and brought usage below high-water.
    Breach,
    /// Breach: sweep exhausted safe candidates; still above high-water.
    BreachUnresolved,
}

/// A single guard event emitted at each SLO transition.
///
/// Field names and serde attributes match ballast-guard's `Event` exactly
/// (AC5 schema compatibility requirement).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event level.
    pub level: Level,
    /// Disk usage percentage before any action (0–100).
    pub used_pct_before: u8,
    /// Disk usage percentage after reclamation (same as before if no sweep).
    pub used_pct_after: u8,
    /// Bytes reclaimed this pass (0 if no sweep ran).
    pub bytes_reclaimed: u64,
    /// Reclaimable bytes estimate (advisory band only; 0 otherwise).
    pub reclaimable_bytes: u64,
    /// Candidates swept (paths as strings).
    pub candidates: Vec<String>,
    /// ISO 8601 timestamp.
    pub ts: String,
}

impl Event {
    /// Construct a new event with the current UTC timestamp.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        level: Level,
        used_pct_before: u8,
        used_pct_after: u8,
        bytes_reclaimed: u64,
        reclaimable_bytes: u64,
        candidates: Vec<String>,
    ) -> Self {
        Self {
            level,
            used_pct_before,
            used_pct_after,
            bytes_reclaimed,
            reclaimable_bytes,
            candidates,
            ts: Utc::now().to_rfc3339(),
        }
    }

    /// Emit this event as a JSON line to stdout and optionally append to `sink`.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails or the sink file cannot be
    /// opened/written.
    pub fn emit(&self, sink: Option<&Path>) -> Result<()> {
        let line = serde_json::to_string(self).context("serializing event")?;
        #[allow(clippy::print_stdout)]
        {
            println!("{line}");
        }
        if let Some(path) = sink {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .with_context(|| format!("opening event sink {}", path.display()))?;
            writeln!(f, "{line}")
                .with_context(|| format!("writing to sink {}", path.display()))?;
        }
        Ok(())
    }
}
