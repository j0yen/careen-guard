//! Interface to careen-survey (live target dir inventory).
//!
//! careen-survey is invoked as a subprocess. The binary name can be overridden
//! via the `CAREEN_SURVEY_BIN` environment variable (used in tests to point
//! at a fixture script).

use anyhow::{bail, Context, Result};

/// A candidate live target dir returned by careen-survey.
#[derive(Debug, Clone)]
pub struct Candidate {
    /// Absolute path to the target directory.
    pub path: String,
    /// Estimated reclaimable bytes (conservative by default).
    pub reclaimable_bytes: u64,
    /// Whether this is a live (binary-current) dir; ballast-eligible dirs have this false.
    pub is_live: bool,
}

/// The name of the survey binary, overridable in tests.
fn survey_bin() -> String {
    std::env::var("CAREEN_SURVEY_BIN").unwrap_or_else(|_| "careen-survey".to_owned())
}

/// Estimate total reclaimable bytes across all live target dirs.
///
/// Returns 0 on any failure (advisory estimate, not load-bearing).
///
/// # Errors
///
/// Never returns an error — survey failures degrade gracefully to 0.
pub fn estimate_reclaimable(roots: &[std::path::PathBuf]) -> u64 {
    let bin = survey_bin();
    let mut cmd = std::process::Command::new(&bin);
    cmd.arg("--json");
    for root in roots {
        cmd.arg("--root").arg(root);
    }
    let Ok(output) = cmd.output() else {
        return 0;
    };
    if !output.status.success() {
        return 0;
    }
    let Ok(text) = std::str::from_utf8(&output.stdout) else {
        return 0;
    };
    let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(text) else {
        return 0;
    };
    // careen-survey JSON: {"summary": {"reclaimable_bytes": N, ...}, "candidates": [...]}
    v.get("summary")
        .and_then(|s| s.get("reclaimable_bytes"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

/// Gather live target dir candidates from careen-survey, sorted descending by `reclaimable_bytes`.
///
/// Only returns dirs classified as live (binary-current). Ballast-eligible
/// (stale/uninstalled) dirs are excluded — that is ballast-guard's domain (AC6).
///
/// # Errors
///
/// Returns an error if careen-survey is not on PATH or returns non-zero.
pub fn gather_candidates(roots: &[std::path::PathBuf]) -> Result<Vec<Candidate>> {
    let bin = survey_bin();
    let mut cmd = std::process::Command::new(&bin);
    cmd.arg("--json");
    for root in roots {
        cmd.arg("--root").arg(root);
    }

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!(
                "careen-survey not on PATH — install careen-survey first (or set CAREEN_SURVEY_BIN)"
            )
        } else {
            anyhow::anyhow!("careen-survey failed: {e}")
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("careen-survey exited {}: {}", output.status, stderr.trim());
    }

    let text = std::str::from_utf8(&output.stdout).context("careen-survey output not UTF-8")?;
    let root: serde_json::Value =
        serde_json::from_str(text).context("parsing careen-survey JSON output")?;

    let raw = root
        .get("candidates")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut candidates: Vec<Candidate> = raw
        .into_iter()
        .filter_map(|v| {
            let path = v.get("path")?.as_str()?.to_owned();
            let reclaimable_bytes = v.get("reclaimable_bytes")?.as_u64()?;
            // is_live: true = binary-current (careen's domain), false = ballast's domain
            let is_live = v.get("is_live")?.as_bool()?;
            Some(Candidate {
                path,
                reclaimable_bytes,
                is_live,
            })
        })
        .collect();

    // Only keep live dirs (AC6: never touch ballast-eligible dirs)
    candidates.retain(|c| c.is_live);

    // Sort descending by reclaimable_bytes (AC3: largest first)
    candidates.sort_by(|a, b| b.reclaimable_bytes.cmp(&a.reclaimable_bytes));

    Ok(candidates)
}
