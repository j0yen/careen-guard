#!/usr/bin/env bash
# Mock careen-survey for AC8 lock test: two live candidates.
# First will be locked, second will succeed. Together they resolve the breach.
# 100 GB total, 2 GB free = 98% used; high_water=90.
# Each candidate has ~8 GB reclaimable; second alone brings to ~90% (just at boundary).
# Use 9 GB for second to definitely cross below 90% after 8 GB: 98% - 9% = 89%.
cat <<'EOF'
{
  "summary": {"reclaimable_bytes": 18253611008},
  "candidates": [
    {"path": "/home/jsy/wintermute/recall/target", "reclaimable_bytes": 9126805504, "is_live": true},
    {"path": "/home/jsy/wintermute/wintermute-brain/target", "reclaimable_bytes": 9126805504, "is_live": true}
  ]
}
EOF
