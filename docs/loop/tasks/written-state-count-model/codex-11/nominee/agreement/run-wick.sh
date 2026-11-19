#!/usr/bin/env bash
set -euo pipefail
top=$HOME/uor-codex
prior=$top/20261118-role-boundary-count
panel=$top/20261118-content-support/panels
test -s FIT.md; test -s RELEASE.md
sha256sum -c source.sha256
(cd ../results;sha256sum -c oracle.sha256 >/dev/null)
(cd "$panel";sha256sum -c SHA256SUMS)
stage=${1:?primary or repeat}
case "$stage" in primary)out=results;;repeat)out=results-repeat;;*)exit 2;;esac
mkdir -p "$out"
exec 9>"$top/20261118-diagnostics/wick-codex-gpu.lock"
dev(){ test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096; "$@"; }
for input in both query; do
 arm=agree; [[ "$input" == both ]] || arm=query-agree
 for n in 1 2; do
  key=seed$n-held
  bare=$panel/held-blind.json
  model=$top/20261118-value-connectors/inputs/seed$n.d11
  cs=$top/20261118-content-selector/results/selector-run-01/seed$n/trained.json
  ss=$top/20261118-span-connectors/results/span-01/seed$n-state.json
  vs=$top/20261118-value-connectors/results/value-01/seed$n-state.json
  d=$out/$key-$arm
  mkdir "$d";prev=../results/$key-$input.json
  flock 9
  date -u -Iseconds
  dev bin/agree enumerate bin/agree-device "$ss" ../results/seed$n-bank.json "$bare" "$prev" "$d"
  max=$(jq -re .max "$d/sites.json")
  for ((k=0;k<max;k++)); do
    dev "$prior/bin/chain" cross role "$model" "$cs" "$ss" "$vs" "$bare" "$d/rank-$k-override.json" "$d/rank-$k-cont.json"
    jq -ce '.arm="action" | .rows=[.rows[] | {prefix_sha256,continuation}]' "$d/rank-$k-cont.json" > "$d/rank-$k-picks.json"
    # The archive can re-run the unmodified writer from the exact three sealed digests.
    jq -nc --arg m "$(sha256sum "$model"|cut -d' ' -f1)" --arg b "$(sha256sum "$bare"|cut -d' ' -f1)" --arg p "$(sha256sum "$d/rank-$k-picks.json"|cut -d' ' -f1)" '[{model:$m,bare:$b,picks:$p}]' > "$d/rank-$k-manifest.json"
    m="$d/rank-$k-manifest.json"; sum=$(sha256sum "$m"|cut -d' ' -f1)
    dev "$prior/bin/action-writer" entity-writer-action "$m" "$sum" "$model" "$bare" "$d/rank-$k-picks.json" "$d/rank-$k-writer.json"
  done
  dev bin/agree adjudicate bin/agree-device "$bare" "$prev" "$d" "$arm" "$out/$key-$arm.json"
  flock -u 9
 done
done
(cd "$out";find . -type f -name '*.json' -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
