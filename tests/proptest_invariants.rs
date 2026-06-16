//! Proptest invariants for careen-guard types.
//!
//! Read-only harness — the edit-agent must not modify this file.

use careen_guard::disk::DiskUsage;
use careen_guard::event::{Event, Level};
use proptest::prelude::*;

proptest! {
    #[test]
    fn disk_usage_pct_in_range(total in 1u64..=1_000_000_000_000u64, free in 0u64..=1_000_000_000_000u64) {
        let free = free.min(total);
        let usage = DiskUsage { total_bytes: total, free_bytes: free };
        let pct = usage.used_pct();
        prop_assert!(pct <= 100, "used_pct must be in [0,100]");
    }

    #[test]
    fn projected_pct_never_exceeds_current(
        total in 1u64..=1_000_000_000_000u64,
        free in 0u64..=1_000_000_000_000u64,
        reclaim in 0u64..=1_000_000_000_000u64
    ) {
        let free = free.min(total);
        let usage = DiskUsage { total_bytes: total, free_bytes: free };
        let before = usage.used_pct();
        let after = careen_guard::disk::projected_used_pct(usage, reclaim);
        prop_assert!(after <= before, "projected_used_pct must not exceed current");
    }

    #[test]
    fn event_serializes_and_deserializes(
        pct_before in 0u8..=100u8,
        pct_after in 0u8..=100u8,
        bytes_reclaimed in 0u64..=u64::MAX,
        reclaimable_bytes in 0u64..=u64::MAX,
    ) {
        let ev = Event::new(Level::Ok, pct_before, pct_after, bytes_reclaimed, reclaimable_bytes, vec![]);
        let json = serde_json::to_string(&ev).expect("serialize");
        let ev2: Event = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(ev2.used_pct_before, pct_before);
        prop_assert_eq!(ev2.used_pct_after, pct_after);
        prop_assert_eq!(ev2.bytes_reclaimed, bytes_reclaimed);
        prop_assert_eq!(ev2.reclaimable_bytes, reclaimable_bytes);
    }
}
