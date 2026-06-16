//! Disk usage reporting via `statvfs`.
//!
//! Tests inject mock values via the `BG_MOCK_DISK_TOTAL` and `BG_MOCK_DISK_FREE`
//! environment variables (both in bytes), which override the real `statvfs` call.

use anyhow::{Context, Result};
use std::path::Path;

/// Disk usage snapshot for a filesystem.
#[derive(Debug, Clone, Copy)]
pub struct DiskUsage {
    /// Total bytes on the filesystem.
    pub total_bytes: u64,
    /// Free bytes available to unprivileged users.
    pub free_bytes: u64,
}

impl DiskUsage {
    /// Used percentage, rounded down (0–100).
    #[must_use]
    pub fn used_pct(self) -> u8 {
        if self.total_bytes == 0 {
            return 0;
        }
        let used = self.total_bytes.saturating_sub(self.free_bytes);
        u8::try_from((used * 100) / self.total_bytes).unwrap_or(100)
    }

    /// Bytes currently used.
    #[must_use]
    pub const fn used_bytes(self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }
}

/// Read disk usage for the filesystem containing `path`.
///
/// If `BG_MOCK_DISK_TOTAL` and `BG_MOCK_DISK_FREE` are set, returns a
/// synthetic usage snapshot (used in tests to avoid real-disk dependency).
///
/// # Errors
///
/// Returns an error if `statvfs` fails (and no mock env vars are set).
pub fn query(path: &Path) -> Result<DiskUsage> {
    if let (Ok(total_str), Ok(free_str)) = (
        std::env::var("BG_MOCK_DISK_TOTAL"),
        std::env::var("BG_MOCK_DISK_FREE"),
    ) {
        let total_bytes: u64 = total_str
            .trim()
            .parse()
            .with_context(|| "parsing BG_MOCK_DISK_TOTAL")?;
        let free_bytes: u64 = free_str
            .trim()
            .parse()
            .with_context(|| "parsing BG_MOCK_DISK_FREE")?;
        return Ok(DiskUsage {
            total_bytes,
            free_bytes,
        });
    }

    let stat = nix::sys::statvfs::statvfs(path)
        .with_context(|| format!("statvfs({})", path.display()))?;

    let block_size = stat.block_size();
    let total_bytes = stat.blocks() * block_size;
    let free_bytes = stat.blocks_available() * block_size;

    Ok(DiskUsage {
        total_bytes,
        free_bytes,
    })
}

/// Compute what `used_pct` would be after reclaiming `reclaim_bytes` from the
/// current `usage`. Saturates at 0 used if reclaim exceeds used.
#[must_use]
pub fn projected_used_pct(usage: DiskUsage, reclaim_bytes: u64) -> u8 {
    if usage.total_bytes == 0 {
        return 0;
    }
    let new_used = usage.used_bytes().saturating_sub(reclaim_bytes);
    u8::try_from((new_used * 100) / usage.total_bytes).unwrap_or(100)
}
