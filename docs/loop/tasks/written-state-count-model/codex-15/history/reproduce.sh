#!/usr/bin/env bash
# Retrospective joins only, over recorded CUDA outputs.
set -euo pipefail
[[ $# == 4 ]]
here=$1
lab=$2
repo=$3
out=$4
archive="$lab/continuation-eos-20261118"
receipt="$repo/docs/loop/tasks/written-state-count-model/codex-7"
mkdir "$out"
digest() { shasum -a 256 "$1" | awk '{print $1}'; }
record() { shasum -a 256 "$@" >> "$out/INPUTS.sha256"; }
git -C "$repo" rev-parse HEAD > "$out/source-head.txt"
jq --version > "$out/jq-version.txt"
record "$receipt/BLIND-INPUTS.json" "$receipt/selector/continuation-01.sha256" "$receipt/writer/SHA256SUMS"
for seed in seed1 seed2; do
  for split in primary fresh transfer prior; do
    key="$seed-$split"
    case "$split" in
      primary) grade="$lab/span-connectors-20261118/panels/held-grade.json" ;;
      fresh) grade="$lab/value-connectors-20261118/panels/fresh-grade.json" ;;
      transfer) grade="$archive/transfer-01/transfer-grade.json" ;;
      prior) grade="$lab/span-connectors-20261118/panels/prior-grade.json" ;;
    esac
    if [[ "$split" == transfer ]]; then
      value="$archive/results/transfer-baseline/$seed-value.json"
    else
      value="$lab/value-connectors-20261118/results/value-01/$key.json"
    fi
    cont="$archive/results/continuation-01/$key.json"
    cold="$archive/results/continuation-repeat/$key.json"
    writer="$archive/results/writer-01/$key.json"
    writer_cold="$archive/results/writer-01/$key-repeat.json"
    seal="$archive/results/writer-inputs/$key-seal.json"
    [[ "$(digest "$cont")" == "$(awk -v f="$key.json" '$2==f {print $1}' "$receipt/selector/continuation-01.sha256")" ]]
    [[ "$(digest "$writer")" == "$(awk -v f="$key.json" '$2==f {print $1}' "$receipt/writer/SHA256SUMS")" ]]
    [[ "$(digest "$grade")" == "$(jq -r '.grade' "$seal")" ]]
    [[ "$(digest "$value")" == "$(jq -r '.value_picks' "$seal")" ]]
    jq -e --slurpfile seal "$seal" '.inputs==$seal[0]' "$writer" >/dev/null
    cmp "$cont" "$cold"
    cmp "$writer" "$writer_cold"
    record "$grade" "$value" "$cont" "$cold" "$writer" "$writer_cold" "$seal"
    jq --arg split "$split" --arg seed "$seed" --slurpfile grade "$grade" --slurpfile value "$value"       --slurpfile writer "$writer" -f "$here/history.jq" "$cont" > "$out/$key.json"
  done
  f="$here/source/4104"
  grade="$f/$seed/oracle-grade.json"
  sel="$f/$seed/oracle.json"
  blind="$f/held-blind.json"
  labels="$f/held-grades.json"
  [[ "$(digest "$grade")" == "$(awk -v f="$seed/oracle-grade.json" '$2==f {print $1}' "$f/cold.sha256")" ]]
  [[ "$(digest "$sel")" == "$(jq -r '.selection' "$grade")" ]]
  [[ "$(digest "$labels")" == "$(jq -r '.grades' "$grade")" ]]
  [[ "$(digest "$blind")" == "$(jq -r '.blind' "$sel")" ]]
  record "$grade" "$sel" "$blind" "$labels" "$f/cold.sha256"
  jq -s --arg seed "$seed" --slurpfile blind "$blind" -f "$here/4104.jq" "$grade" "$sel" "$labels" > "$out/$seed-4104-exposed-24.json"
done
jq -s -f "$here/summarize.jq" "$out"/seed*-{primary,fresh,transfer,prior,4104-exposed-24}.json > "$out/RESULT.json"
for seed in seed1 seed2; do
  tables="$here/source/tables"
  roots="$tables/$seed-roots.json"
  classes="$tables/$seed-classes.json"
  state="$lab/span-connectors-20261118/results/span-01/$seed-state.json"
  (cd "$here" && shasum -a 256 -c SOURCE-PINS.sha256) >/dev/null
  diff -q "$classes" <(jq -e '.bank.classes' "$state")
  record "$roots" "$classes" "$state"
  jq -s --slurpfile roots "$roots" --slurpfile classes "$classes" -f "$here/geometry.jq" \
    "$out/$seed"-{primary,fresh,transfer,prior,4104-exposed-24}.json > "$out/$seed-geometry.json"
  d="$here/source/4104/$seed/adjudication"
  cmp "$d/cold.sha256" "$d/cold-repeat.sha256"
  for k in current literal-start word-start; do
    p="$d/$k-grade.json"
    [[ "$(digest "$p")" == "$(awk -v f="$k-grade.json" '$2==f {print $1}' "$d/cold.sha256")" ]]
    [[ "$(digest "$here/source/4104/held-grades.json")" == "$(jq -r '.grades' "$p")" ]]
    record "$p"
  done
  record "$d/cold.sha256" "$d/cold-repeat.sha256"
  jq -s --slurpfile exposed "$out/$seed-4104-exposed-24.json" -f "$here/adjudication.jq" \
    "$d"/{current,literal-start,word-start}-grade.json > "$out/$seed-adjudication.json"
done
cmp "$out/seed1-geometry.json" "$out/seed2-geometry.json"
cmp "$out/seed1-adjudication.json" "$out/seed2-adjudication.json"
record "$here/source/tables/orbit.json"
jq -n --slurpfile orbit "$here/source/tables/orbit.json" --slurpfile roots "$here/source/tables/seed1-roots.json" \
  -f "$here/signed-root.jq" > "$out/signed-root-examples.json"
shasum -a 256 "$here"/{history,4104,summarize,geometry,adjudication,signed-root}.jq "$here"/{table,orbit}.rs "$here/reproduce.sh" \
  > "$out/ANALYSIS.sha256"
