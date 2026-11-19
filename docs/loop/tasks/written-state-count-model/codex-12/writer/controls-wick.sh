#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-paired-role-fence/writer"
p="$HOME/uor-codex"
old=$p/20261118-role-boundary-count
base=$p/20261118-value-connectors
span=$p/20261118-span-connectors
content=$p/20261118-content-selector
panel=../role/panels
test -s FIT.md
(cd "$panel"; sha256sum -c SHA256SUMS)
sha256sum -c executable.sha256
mkdir controls
mkdir -p controls/{initial,cross,role,old}
exec 8>"$p/20261118-diagnostics/wick-codex-gpu.lock"
for split in held fresh; do
  for n in 1 2; do
    key=seed$n-$split
    model=$base/inputs/seed$n.d11
    cs=$content/results/selector-run-01/seed$n/trained.json
    ss=$span/results/span-01/seed$n-state.json
    vs=$base/results/value-01/seed$n-state.json
    fit=$old/results/fit/seed$n.json
    role=controls/role/$key.json
    bare=$panel/$split-blind.json
    mkdir "controls/cross/$key"
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$p/20261118-entity-abstention/bin/entity-pair-chain" repair "$model" "$cs" "$ss" "$vs" "$bare" "controls/initial/$key.json"
    "$old/bin/cross-input" blind "$old/bin/cross-device" "$model" "$ss" "$fit" "$bare" "controls/initial/$key.json" "controls/cross/$key"
    "$old/bin/chain" cross role "$model" "$cs" "$ss" "$vs" "$bare" "controls/cross/$key/role.json" "$role"
    "$old/bin/action-final" blind "$old/bin/action-device" "$model" "$ss" "$fit" "$bare" "$role" "$old/results/action-fit/seed$n.json" "controls/old/$key.json"
    flock -u 8
  done
done
(cd controls; sha256sum {role,old}/*.json > frozen.sha256)
