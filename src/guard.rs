//! Core SLO enforcement logic for `careen-guard run`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::disk::{self, DiskUsage};
use crate::event::{Event, Level};
use crate::survey;
use crate::sweep::{self, SweepResult};

/// Arguments for the `run` subcommand (mirrored from CLI layer for lib use).
#[derive(Debug)]
pub struct RunArgs {
    /// Path to guard.toml.
    pub config: Option<PathBuf>,
    /// Filesystem path to monitor.
    pub mount: PathBuf,
    /// Optional event sink path.
    pub event_sink: Option<PathBuf>,
}

/// Outcome of a single guard run (maps to exit codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardOutcome {
    /// Below advisory; no action.
    Ok,
    /// Advisory band; alerted only.
    Warn,
    /// Breach: sweep ran successfully.
    BreachActed,
    /// Breach: sweep could not resolve.
    BreachUnresolved,
}

impl From<GuardOutcome> for ExitCode {
    fn from(o: GuardOutcome) -> Self {
        match o {
            GuardOutcome::Ok => Self::from(0_u8),
            GuardOutcome::Warn => Self::from(2_u8),
            GuardOutcome::BreachActed => Self::from(3_u8),
            GuardOutcome::BreachUnresolved => Self::from(4_u8),
        }
    }
}

/// Run the SLO evaluation pass.
///
/// # Errors
///
/// Returns an error if config loading fails or required binaries are absent.
pub fn run(args: &RunArgs) -> Result<ExitCode> {
    let config_path = args.config.clone().unwrap_or_else(Config::default_path);
    let config = Config::load(&config_path)
        .with_context(|| format!("loading config from {}", config_path.display()))?;

    let sink = args.event_sink.as_deref();
    let usage = disk::query(&args.mount)?;
    let used_pct = usage.used_pct();

    let outcome = evaluate(&config, usage, used_pct, sink, &args.mount)?;
    Ok(ExitCode::from(outcome))
}

fn evaluate(
    config: &Config,
    usage: DiskUsage,
    used_pct: u8,
    sink: Option<&Path>,
    _mount: &Path,
) -> Result<GuardOutcome> {
    if used_pct < config.advisory_pct {
        // Below advisory — ok, no action (AC1)
        let ev = Event::new(Level::Ok, used_pct, used_pct, 0, 0, vec![]);
        ev.emit(sink)?;
        return Ok(GuardOutcome::Ok);
    }

    if used_pct < config.high_water_pct {
        // Advisory band — warn only, estimate reclaimable (AC2)
        let reclaimable = survey::estimate_reclaimable(&config.roots);
        let ev = Event::new(Level::Warn, used_pct, used_pct, 0, reclaimable, vec![]);
        ev.emit(sink)?;
        return Ok(GuardOutcome::Warn);
    }

    // High-water breach — sweep live target dirs (AC3, AC4, AC8)
    sweep_to_low_water(config, usage, used_pct, sink)
}

/// Log a "target locked, skipping" notice to stderr.
///
/// Separated into its own fn so the `#[allow]` applies to a function
/// rather than to the macro invocation (which silently no-ops the allow).
#[allow(clippy::print_stderr)]
fn log_locked(path: &str) {
    eprintln!("careen-guard: {path} is build-locked, skipping");
}

/// Run the sweep loop until usage drops below `low_water_pct` or candidates exhaust.
///
/// Candidates are sorted descending by `reclaimable_bytes` (AC3).
/// Build-locked targets are skipped (AC8).
fn sweep_to_low_water(
    config: &Config,
    usage: DiskUsage,
    used_pct_before: u8,
    sink: Option<&Path>,
) -> Result<GuardOutcome> {
    let candidates = survey::gather_candidates(&config.roots)?;

    if candidates.is_empty() {
        // No live candidates → breach-unresolved immediately (AC4)
        let ev = Event::new(
            Level::BreachUnresolved,
            used_pct_before,
            used_pct_before,
            0,
            0,
            vec![],
        );
        ev.emit(sink)?;
        return Ok(GuardOutcome::BreachUnresolved);
    }

    let mut bytes_reclaimed: u64 = 0;
    let mut swept_paths: Vec<String> = Vec::new();
    let mut projected = used_pct_before;

    for candidate in &candidates {
        // Stop if projected usage is already below low-water
        if projected < config.low_water_pct {
            break;
        }

        match sweep::sweep_one(&candidate.path)? {
            SweepResult::Ok => {
                bytes_reclaimed = bytes_reclaimed.saturating_add(candidate.reclaimable_bytes);
                swept_paths.push(candidate.path.clone());
                projected = disk::projected_used_pct(usage, bytes_reclaimed);
            }
            SweepResult::Locked => {
                // Build-locked: skip this candidate and continue (AC8).
                // Intentional stderr: operational status for a daemon-style
                // tool where stderr is the audit trail.
                log_locked(&candidate.path);
            }
        }
    }

    // Re-read actual disk usage to compare with projection
    let used_pct_actual = disk::query(
        // Re-read from the first root or / as fallback
        config
            .roots
            .first()
            .map_or_else(|| Path::new("/"), PathBuf::as_path),
    )
    .map(disk::DiskUsage::used_pct)
    .unwrap_or(projected);
    let used_pct_after = projected.min(used_pct_actual);

    if projected < config.high_water_pct {
        // Resolved (AC3)
        let ev = Event::new(
            Level::Breach,
            used_pct_before,
            used_pct_after,
            bytes_reclaimed,
            0,
            swept_paths,
        );
        ev.emit(sink)?;
        Ok(GuardOutcome::BreachActed)
    } else {
        // Still breached after exhausting all candidates (AC4)
        let ev = Event::new(
            Level::BreachUnresolved,
            used_pct_before,
            used_pct_after,
            bytes_reclaimed,
            0,
            swept_paths,
        );
        ev.emit(sink)?;
        Ok(GuardOutcome::BreachUnresolved)
    }
}
