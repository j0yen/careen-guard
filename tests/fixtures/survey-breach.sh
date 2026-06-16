#!/usr/bin/env bash
# Mock careen-survey for breach tests (AC3, AC4).
# Returns live candidates; large enough to resolve breach (AC3).
# 100 GB total, 2 GB free (98% used); 15 GB reclaimable brings us to ~83% (below 90).
cat <<'EOF'
{
  "summary": {"reclaimable_bytes": 16106127360},
  "candidates": [
    {"path": "/home/jsy/wintermute/recall/target", "reclaimable_bytes": 9663676416, "is_live": true},
    {"path": "/home/jsy/wintermute/wintermute-brain/target", "reclaimable_bytes": 6442450944, "is_live": true}
  ]
}
EOF
