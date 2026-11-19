#!/usr/bin/env bash
set -euo pipefail
R=results
mkdir -p "$R"/{action,action-repeat,action-fit,action-fit-repeat}
test -s action-release/FIT.md && test -s action-release/PANEL-FIT.md && test -s action-release/RELEASE.md
sha256sum -c action-release/panel.sha256
(cd "$R/action-finite"; cmp parity.txt parity-repeat.txt)
exec 9>../20261118-diagnostics/wick-codex-gpu.lock
device(){
  flock 9
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
  bin/action-final "$@"
  flock -u 9
}
date -u -Is
for stage in '' -repeat; do
  for n in 1 2; do
    base=(bin/action-device "../20261118-value-connectors/inputs/seed$n.d11"       "../20261118-span-connectors/results/span-01/seed$n-state.json" "$R/fit/seed$n.json")
    bank=$R/action-fit$stage/seed$n.json
    device fit "${base[@]}" panels/action-train-blind.json "$R/action-role/seed$n-train.json"       panels/action-train-grade.json "$bank"
    if test -n "$stage"; then cmp "$bank" "$R/action-fit/seed$n.json"; fi
    for p in held transfer challenge primary fresh unseen unanswerable; do
      h="$R/selector/seed$n-$p-role.json"
      case "$p" in
        held) b=panels/action-held-blind.json; h="$R/action-role/seed$n-held.json";;
        transfer) b=../20261118-continuation-eos/panels/transfer-blind.json;;
        challenge) b=../20261118-entity-abstention/panels/challenge-blind.json;;
        primary) b=../20261118-span-connectors/panels/held-blind.json;;
        fresh) b=../20261118-value-connectors/panels/fresh-blind.json;;
        unseen|unanswerable) b=panels/$p-blind.json;;
      esac
      out="$R/action$stage/seed$n-$p.json"
      device blind "${base[@]}" "$b" "$h" "$bank" "$out"
      if test -n "$stage"; then cmp "$out" "$R/action/seed$n-$p.json"; fi
    done
  done
  (cd "$R/action$stage"; sha256sum *.json > SHA256SUMS)
done
cmp "$R"/action{,-repeat}/SHA256SUMS
date -u -Is
