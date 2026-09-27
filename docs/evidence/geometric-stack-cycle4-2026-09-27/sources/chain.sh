#!/usr/bin/env bash
# Cycle 4 pipeline after the learning-rate pilot. Rule fixed before any pilot result was complete:
# each arm's learning rate is the pilot rate with the lowest final development NLL (512 windows).
# Then the ablations (geometric rate), then the main comparison.
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c
S=$S/scratchpad
until grep -q "pilot done" $S/c4/runs/pilot/launch.log 2>/dev/null; do sleep 60; done
choose() {
  python3 - "$S/c4/runs/pilot" "$1" <<'PY'
import json, os, sys
root, arch = sys.argv[1], sys.argv[2]
best = None
for name in sorted(os.listdir(root)):
    report = os.path.join(root, name, "report.json")
    if name.startswith(arch + "_lr") and os.path.exists(report):
        r = json.load(open(report))
        nll = r["final"]["nll"]
        lr = r["settings"]["lr"]
        if best is None or nll < best[0]:
            best = (nll, lr)
print(best[1])
PY
}
LT=$(choose transformer)
LG=$(choose geometric)
echo "chosen lr transformer $LT geometric $LG $(date -u +%FT%TZ)" >> $S/c4/chain.log
$S/c4/launch_ablations.sh "$LG"
$S/c4/launch_main.sh "$LT" "$LG" 1
echo "chain done $(date -u +%FT%TZ)" >> $S/c4/chain.log
