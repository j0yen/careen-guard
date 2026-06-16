//! Interface to careen-sweep (intra-target reclaimer).
//!
//! careen-sweep is invoked as a subprocess. The binary name can be overridden
//! via the `CAREEN_SWEEP_BIN` environment variable (used in tests to point
//! at a fixture script).

use anyhow::{bail, Result};

/// Outcome of invoking careen-sweep on a single target dir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepResult {
    /// Sweep ran successfully and reclaimed bytes.
    Ok,
    /// Target dir was build-locked; sweep skipped it safely (AC8).
    Locked,
}

/// The name of the sweep binary, overridable in tests.
fn sweep_bin() -> String {
    std::env::var("CAREEN_SWEEP_BIN").unwrap_or_else(|_| "careen-sweep".to_owned())
}

/// Invoke `careen-sweep --apply` on a single target dir.
///
/// Returns `SweepResult::Locked` if the target is build-locked (careen-sweep
/// exits with code 2 by convention). Returns `SweepResult::Ok` on success.
///
/// # Errors
///
/// Returns an error if the binary is not on PATH or exits with an unexpected
/// non-zero code other than the lock-busy code.
pub fn sweep_one(path: &str) -> Result<SweepResult> {
    let bin = sweep_bin();
    let output = std::process::Command::new(&bin)
        .arg("--apply")
        .arg(path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "careen-sweep not on PATH — install careen-sweep first (or set CAREEN_SWEEP_BIN)"
                )
            } else {
                anyhow::anyhow!("careen-sweep failed: {e}")
            }
        })?;

    if output.status.success() {
        return Ok(SweepResult::Ok);
    }

    // Exit code 2 = target busy / build-locked (AC8: skip and continue)
    if output.status.code() == Some(2) {
        return Ok(SweepResult::Locked);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!(
        "careen-sweep exited {} on {path}: {}",
        output.status,
        stderr.trim()
    );
}
