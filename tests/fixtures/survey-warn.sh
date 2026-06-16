#!/usr/bin/env bash
# Mock careen-survey for advisory-band tests (AC2).
# Returns two live candidates with a non-zero reclaimable_bytes summary.
cat <<'EOF'
{
  "summary": {"reclaimable_bytes": 5368709120},
  "candidates": [
    {"path": "/home/jsy/wintermute/recall/target", "reclaimable_bytes": 3221225472, "is_live": true},
    {"path": "/home/jsy/wintermute/wintermute-brain/target", "reclaimable_bytes": 2147483648, "is_live": true}
  ]
}
EOF
