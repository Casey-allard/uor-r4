#!/usr/bin/env bash
# Reuse exactly the previous action writer binary; the sealed filename records each research arm.
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/panels"
top="$HOME/uor-codex"
prev="$top/20261118-role-boundary-count"
new=../nominee/results
case ${1:-} in agree|query-agree) new=../nominee/agreement/results ;; esac
arm=${1:?role,old,none,right,wrong,nominee,support,both,query}
split=${2:?held,mixed,unseen,transfer}
case "$split" in
  held) pref=held; role=results/role/seedNUM-held.json ;;
  mixed) pref=$prev/panels/action-held; role=$prev/results/action-role/seedNUM-held.json ;;
  unseen) pref=$prev/panels/unseen; role=$prev/results/selector/seedNUM-unseen-role.json ;;
  transfer) pref=$top/20261118-continuation-eos/panels/transfer; role=$prev/results/selector/seedNUM-transfer-role.json ;;
  *) exit 2 ;;
esac
dst=results/writer/$split/$arm
mkdir -p "$dst"
mkdir "$dst/seal"
test -s FIT.md
exec 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
for n in 1 2; do
  model=$top/20261118-value-connectors/inputs/seed$n.d11
  bare=$pref-blind.json
  picks=$dst/seed$n-picks.json
  readrole=${role/NUM/$n}
  args=(--argjson p '[]' --argjson grade '[]')
  case "$arm" in
    right|wrong) args=(--argjson p '[]' --slurpfile grade "$pref-grade.json") ;;
    role|none) ;;
    old)
       case $split in
        held) old=results/old/seed$n-held.json ;;
        mixed) old=$prev/results/action/seed$n-held.json ;;
        *) old=$prev/results/action/seed$n-$split.json ;;
       esac
       args=(--slurpfile p "$old" --argjson grade '[]') ;;
    *) args=(--slurpfile p "$new/seed$n-$split-$arm.json" --argjson grade '[]') ;;
  esac
  jq -ce --arg arm "$arm" "${args[@]}" -f picks.jq "$readrole" > "$picks"
  jq -n --arg m "$(sha256sum "$model"|cut -d' ' -f1)" --arg b "$(sha256sum "$bare"|cut -d' ' -f1)" \
    --arg p "$(sha256sum "$picks"| cut -d' ' -f1)" '[{model:$m,bare:$b,picks:$p}]' > "$dst/seal/seed$n.json"
  manifest=$dst/seal/seed$n.json
  sum=$(sha256sum "$manifest"|cut -d' ' -f1)
  # One launch per seed and repeat. The fixed writer also compares every GPU step to the reference.
  for stage in primary repeat; do
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$prev/bin/action-writer" entity-writer-action "$manifest" "$sum" "$model" "$bare" "$picks" "$dst/seed$n-$stage.json"
    flock -u 8
  done
  cmp "$dst/seed$n-primary.json" "$dst/seed$n-repeat.json"
done
(cd "$dst"; sha256sum *.json seal/*.json > SHA256SUMS)
