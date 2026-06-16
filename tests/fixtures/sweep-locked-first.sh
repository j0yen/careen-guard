#!/usr/bin/env bash
# Mock careen-sweep for AC8: first call returns locked (exit 2), subsequent calls succeed.
# Uses a state file in /tmp to track call count.
STATE_FILE="/tmp/careen-guard-sweep-locked-state-$$"
# We need a stable state file across calls in the same test process.
# Use a fixed name based on the test session.
STATE_FILE="${CAREEN_SWEEP_STATE_FILE:-/tmp/careen-guard-sweep-locked-state}"

if [ ! -f "$STATE_FILE" ]; then
    echo "0" > "$STATE_FILE"
fi

COUNT=$(cat "$STATE_FILE")
NEW_COUNT=$((COUNT + 1))
echo "$NEW_COUNT" > "$STATE_FILE"

if [ "$COUNT" -eq 0 ]; then
    # First call: report locked
    echo "target busy: build lock held" >&2
    exit 2
fi

# Subsequent calls: succeed
exit 0
