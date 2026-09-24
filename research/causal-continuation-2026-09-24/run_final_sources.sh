#!/bin/zsh
set -eu
R=/Users/casey.allard/uor-r4-investigations/causal-20260924T000007
W=/Users/casey.allard/uor-r4-worktrees/observer-transport-20260923
D=/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs
P=/Users/casey.allard/uor-r4/.uor-models/principal-continuation-2026-09-23/warm-decay-candidate.tlx
cd "$W"
for ARM in parent ordinary intervened; do
 if [[ $ARM == parent ]]; then M=$P; else M="$R/$ARM-1/selected.tlx"; fi
 /usr/bin/time -l env UOR_PRINCIPAL_RUN=causal UOR_CAUSAL_MODE=evaluate UOR_CAUSAL_DATA="$R/data" RAYON_NUM_THREADS=1 "$R/causal-lexical" --state-probe "$R/final-$ARM-1" --artifact "$M" --docs "$D" --source-rev 4af04cf56f213df40b78c73a7c51ed5274517857 > "$R/final-$ARM-1.log" 2>&1
 echo "evaluated=$ARM"
done
