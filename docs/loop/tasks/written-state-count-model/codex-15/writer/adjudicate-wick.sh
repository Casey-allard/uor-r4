#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-ordered-role-transfer"
stage=${1:?primary or repeat}
seed=${2:?1 or 2}
[[ $stage == primary || $stage == repeat ]]
[[ $seed == 1 || $seed == 2 ]]
sha256sum -c driver/adjudication-pins.sha256
sha256sum -c driver/oracle-pins.sha256
(cd assets && sha256sum -c SHA256SUMS)
(cd panel && sha256sum -c panel-inputs.sha256)
(cd "oracle-$stage" && sha256sum -c cold.sha256)
cmp oracle-primary/cold.sha256 oracle-repeat/cold.sha256
out=$PWD/diagnose-$stage/seed$seed
mkdir -p "$PWD/diagnose-$stage"
mkdir "$out"
f=$PWD/oracle-$stage/seed$seed
w=$PWD/bins/writer-c
tok=$PWD/assets/tokenizer-seed$seed.json
m=$PWD/assets/seed$seed-paired.d11
blind=$PWD/panel/held-blind.json
labels=$PWD/grading/held-grades.json
# Bound the source readout to independent CUDA intermediates from the same
# original endpoint, not merely to the ordinary client's proposed value start.
jq -ecs --slurpfile original "$f/oracle.json" '
  (map(.rows[])|group_by(.prefix_sha256)|map({key:.[0].prefix_sha256,value:.})|from_entries) as $index |
  [$original[0].rows[] as $r | $r.assertions[] as $a |
    if any($index[$r.prefix_sha256][]; (.frozen.query_start==$r.paired.role_words[0] and .frozen.query_end+1==$r.paired.role_words[1] and .frozen.fact_start==$a.fact_span[0] and .bounded==$a.assertion))
    then 1 else error("assertion differs between client and CUDA") end] | {assertion_matches:add}' "$f"/device/level-*-chunk-*/nominees/rank-*-cont.json >"$out/assertion-parity.json"
for arm in literal-start current word-start; do
    jq -enc --arg arm "$arm" --slurpfile grades "$f/oracle-grade.json" --slurpfile labels "$labels" --slurpfile original "$f/oracle.json" -f driver/onset.jq > "$out/$arm.json"
done
flock "$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock" bash "$HOME/uor-codex/20261118-query-region-pick/writer/run-wick.sh" gpu "$w" replay "$HOME/uor-codex/20261118-role-boundary-count/bin/action-writer" "$HOME/uor-codex/20261118-value-connectors/inputs/seed$seed.d11" "$blind" "$out/forced" "$out/literal-start.json" "$out/current.json" "$out/word-start.json"
mkdir "$out/forced/nominees"
ln "$out/forced/gpu.json" "$out/forced/nominees/chosen.json"
for arm in literal-start current word-start; do
    "$w" grade "$out/$arm.json" "$out/forced" "$labels" - "$out/$arm-grade.json"
done
# Public original-text controls on every row, with the frozen role bank only.
declare -a clients=()
for arm in public no-read; do
    "$w" select "$m" "$blind" "$arm" - - "$out/$arm.json"
    "$w" public "$PWD/native-finite/source/target/release/uor-chat" "$m" "$tok" "$blind" "$arm" "$out/say-$arm" >"$out/say-$arm.log" 2>&1 & clients+=("$!")
done
"$w" chain "$HOME/uor-codex" "$seed" "$blind" "$out/ordinary" "$out/public.json" "$out/no-read.json"
fail=0
for job in "${clients[@]}"; do wait "$job" || fail=1; done
[[ $fail == 0 ]]
for arm in public no-read; do
    "$w" grade "$out/$arm.json" "$out/ordinary" "$labels" "$out/say-$arm" "$out/$arm-grade.json"
done
(cd "$out" && sha256sum *-grade.json assertion-parity.json forced/gpu.json ordinary/nominees/chosen.json > cold.sha256)
