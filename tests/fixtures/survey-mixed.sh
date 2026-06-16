#!/usr/bin/env bash
# Mock careen-survey for AC6: returns both a ballast-eligible (is_live=false)
# and a careen-eligible (is_live=true) dir. Only the live one should be touched.
cat <<'EOF'
{
  "summary": {"reclaimable_bytes": 10737418240},
  "candidates": [
    {"path": "/home/jsy/wintermute/old-crate/target", "reclaimable_bytes": 5368709120, "is_live": false},
    {"path": "/home/jsy/wintermute/recall/target", "reclaimable_bytes": 5368709120, "is_live": true}
  ]
}
EOF
