#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-role-boundary-count"
base=../20261118-value-connectors
span=../20261118-span-connectors
content=../20261118-content-selector
prior=../20261118-entity-abstention
mkdir -p results/{action-initial,action-overrides,action-role}
exec 8>../20261118-diagnostics/wick-codex-gpu.lock
for p in train held; do
  for n in 1 2; do
    key=seed$n-$p
    model="$base/inputs/seed$n.d11"
    cs="$content/results/selector-run-01/seed$n/trained.json"
    ss="$span/results/span-01/seed$n-state.json"
    vs="$base/results/value-01/seed$n-state.json"
    bare=panels/action-$p-blind.json
    old=results/action-initial/$key.json
    hand=results/action-overrides/$key
    mkdir -p "$hand"
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$prior/bin/entity-pair-chain" repair "$model" "$cs" "$ss" "$vs" "$bare" "$old"
    bin/cross-input blind bin/cross-device "$model" "$ss" "results/fit/seed$n.json" "$bare" "$old" "$hand"
    bin/chain cross role "$model" "$cs" "$ss" "$vs" "$bare" "$hand/role.json" "results/action-role/$key.json"
    flock -u 8
  done
done
(cd results/action-role; sha256sum seed*.json > SHA256SUMS)
