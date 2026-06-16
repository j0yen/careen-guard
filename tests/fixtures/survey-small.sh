#!/usr/bin/env bash
# Mock careen-survey for AC4 (BreachUnresolved): candidate too small to resolve.
# Disk at 98% (2 GB free of 100 GB). Only 500 MB reclaimable — not enough.
cat <<'EOF'
{
  "summary": {"reclaimable_bytes": 524288000},
  "candidates": [
    {"path": "/home/jsy/wintermute/small-crate/target", "reclaimable_bytes": 524288000, "is_live": true}
  ]
}
EOF
