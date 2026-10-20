#!/bin/zsh
set -eu
W=/Users/casey.allard/uor-r4-worktrees/observer-transport-20260923
R=/Users/casey.allard/uor-r4-investigations/causal-20260924T000007
D=/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs
M=/Users/casey.allard/uor-r4/.uor-models/principal-continuation-2026-09-23/warm-decay-candidate.tlx
cd "$W"
REV=$(git rev-parse HEAD)
for MODE in geometry cost; do
 /usr/bin/time -l env UOR_PRINCIPAL_RUN=causal UOR_CAUSAL_MODE=$MODE RAYON_NUM_THREADS=1 "$R/causal-lexical" --state-probe "$R/$MODE-1" --artifact "$M" --docs "$D" --source-rev "$REV" > "$R/$MODE-1.log" 2>&1
 echo "finished=$MODE"
done
for ARM in ordinary intervened; do
 /usr/bin/time -l env UOR_PRINCIPAL_RUN=causal UOR_CAUSAL_MODE=train UOR_CAUSAL_ARM=$ARM UOR_CAUSAL_DATA="$R/data" UOR_CAUSAL_STEPS=256 RAYON_NUM_THREADS=1 "$R/causal-lexical" --state-probe "$R/$ARM-1" --artifact "$M" --docs "$D" --source-rev "$REV" > "$R/$ARM-1.log" 2>&1
 echo "finished=$ARM"
done
