#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
root=$HOME/uor-codex
prev=$root/20261118-paired-role-fence
store=${1:?fresh finite root}
mkdir "$store"
sha256sum --status -c PREDECESSORS.sha256
lock=$root/20261118-diagnostics/wick-codex-gpu.lock
flock "$lock" bash run-wick.sh gpu "$prev/fence/bin/assertion-fence-final" fixture "$prev/fence/bin/fence-device" > "$store/fence.txt"
ag=$root/20261118-content-support/nominee/agreement/bin
flock "$lock" bash run-wick.sh gpu "$ag/agree" fixture "$ag/agree-device" > "$store/consensus.txt" 2>&1
p=$prev/writer/results/held/none/seed1
m=$root/20261118-value-connectors/inputs/seed1.d11
b=$prev/role/panels/held-blind.json
writer=$root/20261118-role-boundary-count/bin/action-writer
jq '.rows[0].prefix_sha256="bad"' "$p/picks.json" > "$store/corrupt-picks.json"
jq -nc --arg m "$(sha256sum "$m"|cut -d' ' -f1)" --arg b "$(sha256sum "$b"|cut -d' ' -f1)" \
    --arg p "$(sha256sum "$store/corrupt-picks.json"|cut -d' ' -f1)" '[{model:$m,bare:$b,picks:$p}]' > "$store/corrupt-manifest.json"
if flock "$lock" bash run-wick.sh gpu "$writer" entity-writer-action "$store/corrupt-manifest.json" \
    "$(sha256sum "$store/corrupt-manifest.json"|cut -d' ' -f1)" "$m" "$b" "$store/corrupt-picks.json" "$store/invalid-writer.json" \
    > "$store/corrupt.log" 2>&1; then
    exit 1
fi
grep -F 'different bare prefix or source beyond prompt' "$store/corrupt.log"
test ! -e "$store/invalid-writer.json"
