#!/usr/bin/env bash
# Exercise the index/native join independently of device arithmetic on an exposed row.
set -euo pipefail
parity=$(realpath "$1")
src=$(realpath "$2")
out=$3
mkdir "$out"
cd "$out"
mkdir forward reverse
cp "$src/candidate-forward.json" forward/30110.json
cp "$src/candidate-reversed.json" reverse/30110.json
jq '[.role_replay as $r | $r.pair_rank_sources[] as $s |
  {packet_row:0,row:30110,fact:$s.fact_span,candidates:[$r.pair_rank_candidates[]|select(.fact_span==$s.fact_span)|.candidates[]|select(.span.start as $x|$s.eligible_sources|index($x)!=null)|[.span.start,.span.end]]}]' forward/30110.json > index.json
jq --slurpfile rev reverse/30110.json --slurpfile idx index.json '[.role_replay.pair_rank_sources[] |
  {packet_row:0,winner_forward:.selection.source,winner_reversed:$rev[0].role_replay.pair_rank_sources[0].selection.source,candidates:[$idx[0][0].candidates[]|{start:.[0],end:.[1]}],pairs:.pairs}]' forward/30110.json > joined-control.json
"$parity" index.json joined-control.json forward reverse > pass.json
jq -e '.prompts==1 and .ordered_pairs_each_orientation==20 and .orientations==2' pass.json > /dev/null
jq '.[0].pairs[0].nearest_forward+=1' joined-control.json > altered.json
if "$parity" index.json altered.json forward reverse > negative.out 2>negative.log; then exit 4; fi
grep -Fq 'native pair addresses, distances or margin differ' negative.log
