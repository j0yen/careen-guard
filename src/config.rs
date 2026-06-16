//! Configuration types for careen-guard.
//!
//! Uses the same TOML field names as ballast-guard's `guard.toml` so a single
//! config file can serve both guards with the same thresholds.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// careen-guard configuration read from `guard.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Percentage at/above which guard sweeps. Default 90.
    #[serde(default = "default_high_water")]
    pub high_water_pct: u8,

    /// Percentage below which sweep stops. Default 80.
    #[serde(default = "default_low_water")]
    pub low_water_pct: u8,

    /// Percentage at/above which guard alerts (but doesn't sweep). Default 85.
    #[serde(default = "default_advisory")]
    pub advisory_pct: u8,

    /// Scan roots passed through to careen-survey.
    #[serde(default)]
    pub roots: Vec<PathBuf>,
}

const fn default_high_water() -> u8 {
    90
}
const fn default_low_water() -> u8 {
    80
}
const fn default_advisory() -> u8 {
    85
}

impl Default for Config {
    fn default() -> Self {
        Self {
            high_water_pct: default_high_water(),
            low_water_pct: default_low_water(),
            advisory_pct: default_advisory(),
            roots: Vec::new(),
        }
    }
}

impl Config {
    /// Load config from a TOML file, falling back to defaults if the file is absent.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed, or if
    /// the resulting thresholds are logically invalid.
    pub fn load(path: &Path) -> Result<Self> {
        let cfg: Self = if path.exists() {
            let raw = std::fs::read_to_string(path)
                .with_context(|| format!("reading config {}", path.display()))?;
            toml::from_str(&raw)
                .with_context(|| format!("parsing config {}", path.display()))?
        } else {
            Self::default()
        };
        cfg.validate()?;
        Ok(cfg)
    }

    /// Default config path: `~/.config/careen/guard.toml`.
    #[must_use]
    pub fn default_path() -> PathBuf {
        home_config().join("careen").join("guard.toml")
    }

    /// Validate threshold ordering.
    ///
    /// # Errors
    ///
    /// Returns an error if `low_water_pct >= high_water_pct` or other invalid orderings.
    pub fn validate(&self) -> Result<()> {
        if self.low_water_pct >= self.high_water_pct {
            bail!(
                "invalid config: low_water_pct ({}) must be strictly less than high_water_pct ({})",
                self.low_water_pct,
                self.high_water_pct
            );
        }
        if self.advisory_pct >= self.high_water_pct {
            bail!(
                "invalid config: advisory_pct ({}) must be strictly less than high_water_pct ({})",
                self.advisory_pct,
                self.high_water_pct
            );
        }
        if self.low_water_pct >= self.advisory_pct {
            bail!(
                "invalid config: low_water_pct ({}) must be strictly less than advisory_pct ({})",
                self.low_water_pct,
                self.advisory_pct
            );
        }
        Ok(())
    }
}

fn home_config() -> PathBuf {
    std::env::var_os("HOME")
        .map_or_else(|| PathBuf::from("/root"), PathBuf::from)
        .join(".config")
}
