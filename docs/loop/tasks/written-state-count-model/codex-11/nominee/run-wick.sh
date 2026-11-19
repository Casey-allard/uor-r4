#!/usr/bin/env bash
set -euo pipefail
# Run from an unused results root on Wick. Input locations are immutable predecessor runs.
unit=$HOME/uor-codex/20261118-content-support
old=$HOME/uor-codex/20261118-role-boundary-count
value=$HOME/uor-codex/20261118-value-connectors
span=$HOME/uor-codex/20261118-span-connectors
content=$HOME/uor-codex/20261118-content-selector
panel=$unit/panels
suffix=${1:-primary}
here=$PWD
if [[ "$suffix" == primary ]];then out=results;else out=results-repeat;fi
test -s FIT.md; test -s RELEASE.md
(cd "$panel"; sha256sum -c SHA256SUMS; cd results; sha256sum -c frozen.sha256)
(cd "$here"; sha256sum -c source.sha256)
mkdir -p "$out"
exec 9>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
device(){ flock 9; test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096; "$@"; flock -u 9; }
resolve(){
 case "$p" in
 held) bare="$panel/held-blind.json"; grade="$panel/held-grade.json"; role="$panel/results/role/seed$n-held.json"; prev="$panel/results/old/seed$n-held.json";;
 mixed) bare="$old/panels/action-held-blind.json";grade="$old/panels/action-held-grade.json";role="$old/results/action-role/seed$n-held.json";prev="$old/results/action/seed$n-held.json";;
 unseen) bare="$old/panels/unseen-blind.json";grade="$old/panels/unseen-grade.json";role="$old/results/selector/seed$n-unseen-role.json";prev="$old/results/action/seed$n-unseen.json";;
 transfer) pre=$HOME/uor-codex/20261118-continuation-eos;bare="$pre/panels/transfer-blind.json";grade="$pre/panels/transfer-grade.json";role="$old/results/selector/seed$n-transfer-role.json";prev="$old/results/action/seed$n-transfer.json";;
 esac
}
seed(){
 model="$value/inputs/seed$n.d11"; tokenizer="$value/inputs/tokenizer.json"; [[ "$n" == 1 ]] || tokenizer="$value/inputs/tokenizer-seed-2.json"
 cs="$content/results/selector-run-01/seed$n/trained.json"; ss="$span/results/span-01/seed$n-state.json"; vs="$value/results/value-01/seed$n-state.json"
 base=(bin/occurrence-device "$model" "$ss" "$old/results/fit/seed$n.json" "$tokenizer" "$old/results/action-fit/seed$n.json")
}
for n in 1 2;do seed
 device bin/occurrence fit "${base[@]}" "$panel/train-blind.json" "$panel/results/role/seed$n-train.json" "$panel/train-grade.json" - "$out/seed$n-bank.json"
done
for op in blind oracle;do
 if [[ "$op" == oracle ]];then test -s "$out/blind.sha256"; (cd "$out"; sha256sum -c blind.sha256 >/dev/null); fi
 for p in held mixed unseen transfer;do for n in 1 2;do
  date -u -Iseconds; echo "$suffix $op $p seed$n";seed;resolve; key=seed$n-$p; dest="$out/$key";mkdir -p "$dest"
  args=(-); arms=(nominee support both)
  if [[ "$op" == oracle ]];then args=("$grade");arms=(query);fi
  if [[ "$op" == oracle && "$p" == mixed ]];then jq -ce --slurpfile tok "$tokenizer" -f mixed-oracle.jq "$grade" > "$dest/oracle-grade.json"; args=("$dest/oracle-grade.json"); fi
  device bin/occurrence "$op" "${base[@]}" "$bare" "$role" "$out/seed$n-bank.json" "${args[@]}" "$dest"
  if [[ "$op" == blind ]];then cp "$prev" "$out/$key-old.json";fi
  for arm in "${arms[@]}";do
    device "$old/bin/chain" cross role "$model" "$cs" "$ss" "$vs" "$bare" "$dest/$arm-override.json" "$dest/$arm-continuation.json"
    jq -nec --arg arm "$arm" --slurpfile d "$dest/$arm-diagnostic.json" --slurpfile c "$dest/$arm-continuation.json" '
      $d[0] as $d|$c[0] as $c|if $c.bare_sha256!=$d.bare or $c.artifact_sha256!=$d.artifact or ($d.rows|length)!=($c.rows|length) then error("continuation identity") else
      {arm:$arm,artifact_sha256:$d.artifact,bare_sha256:$d.bare,state_sha256:$d.bank,rows: [range($c.rows|length) as $i | $d.rows[$i] as $r | $c.rows[$i] as $cr |
       if $r.prefix_sha256!=$cr.prefix_sha256 then error("row identity") else $r+{source:$cr.continuation.source} end]} end' > "$out/$key-$arm.json"
  done
 done;done
 (cd "$out"; find . -type f -name '*.json' -print0 | sort -z | xargs -0 sha256sum > "$op.sha256")
done
