#!/bin/zsh
set -eu
R=/Users/casey.allard/uor-r4-investigations/causal-20260924T000007
W=/Users/casey.allard/uor-r4-worktrees/observer-transport-20260923
D=/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs
cd "$W"
ps -axo pid,etime,pcpu,comm > "$R/cost-idle-processes-before.txt"
for ARM in parent intervened; do
 if [[ $ARM == parent ]]; then M=/Users/casey.allard/uor-r4/.uor-models/principal-continuation-2026-09-23/warm-decay-candidate.tlx; else M="$R/intervened-1/selected.tlx"; fi
 /usr/bin/time -l env UOR_PRINCIPAL_RUN=causal UOR_CAUSAL_MODE=cost RAYON_NUM_THREADS=1 "$R/causal-lexical" --state-probe "$R/cost-idle-$ARM" --artifact "$M" --docs "$D" --source-rev 4af04cf56f213df40b78c73a7c51ed5274517857 > "$R/cost-idle-$ARM.log" 2>&1
 echo "idle_cost_complete=$ARM"
done
