#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-paired-role-fence/writer"
split=${1:?held or fresh}; arm=${2:?action arm}; n=${3:?seed}
r=results/$split
d=$r/$arm/seed$n
o=$r/old/seed$n/primary.json
baseline=$r/role/seed$n/primary.json
nr=$r/none/seed$n/primary.json
grade=../role/panels/$split-grade.json
cmp "$d/primary.json" "$d/repeat.json"
for mode in rows summary; do
  jq -ce --arg mode "$mode" --arg panel "$split" --arg arm "$arm" --slurpfile grades "$grade" \
    --slurpfile none "$nr" --slurpfile old "$o" --slurpfile role "$baseline" -f grade.jq "$d/primary.json" > "$d/$mode.json"
done
jq -ne --slurpfile g "$grade" --slurpfile w "$d/rows.json" '
  if ($g[0]|length)!=($w[0]|length) then error("length mismatch") end |
  [range($g[0]|length) as $i | $g[0][$i] as $k | $w[0][$i] |
    if .id!=$k.id then error("wrong grade order") end |
    . + {case:$k.case,construction:$k.construction,evidence:$k.evidence}] as $rows |
  def by(f): group_by(f) | map({key:((.[0]|f)|tostring),value:{n:length,scorable:(map(select(.expected!=null))|length),
    exact:(map(select(.first=="exact"))|length),unexpected:(map(select(.supported==false and .copied))|length)}})|from_entries;
  {by_case:($rows|by(.case)),by_form:($rows|by(.construction)),by_evidence:($rows|by(.evidence))}' > "$d/slices.json"
