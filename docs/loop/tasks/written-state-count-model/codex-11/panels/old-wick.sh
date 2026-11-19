#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/panels"
p="$HOME/uor-codex"
old=$p/20261118-role-boundary-count
base=$p/20261118-value-connectors
span=$p/20261118-span-connectors
content=$p/20261118-content-selector
mkdir -p results/{initial,cross,role,old}
test -s FIT.md
sha256sum -c SHA256SUMS
exec 8>"$p/20261118-diagnostics/wick-codex-gpu.lock"
for split in train held; do
  for n in 1 2; do
    key=seed$n-$split
    m=$base/inputs/seed$n.d11
    cs=$content/results/selector-run-01/seed$n/trained.json
    ss=$span/results/span-01/seed$n-state.json
    vs=$base/results/value-01/seed$n-state.json
    fit=$old/results/fit/seed$n.json
    picks=results/role/$key.json
    b=$split-blind.json
    mkdir "results/cross/$key"
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$p/20261118-entity-abstention/bin/entity-pair-chain" repair "$m" "$cs" "$ss" "$vs" "$b" "results/initial/$key.json"
    "$old/bin/cross-input" blind "$old/bin/cross-device" "$m" "$ss" "$fit" "$b" "results/initial/$key.json" "results/cross/$key"
    "$old/bin/chain" cross role "$m" "$cs" "$ss" "$vs" "$b" "results/cross/$key/role.json" "$picks"
    if [[ $split == held ]]; then
      "$old/bin/action-final" blind "$old/bin/action-device" "$m" "$ss" "$fit" "$b" "$picks" "$old/results/action-fit/seed$n.json" "results/old/$key.json"
    fi
    flock -u 8
  done
done
(cd results; sha256sum {role,old}/*.json > frozen.sha256)
