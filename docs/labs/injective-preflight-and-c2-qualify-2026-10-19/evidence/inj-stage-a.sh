#!/bin/bash
# Injective attention pre-flight, stage A (zero updates): table check, identities, checkpoint 0, DEV eval.
# Kimi PM dispatch #2029 6257564516, Job 1 (GPU0). No training, no owner release.
set -u
SRC=$(ls -d /root/build/src-* | head -1)
B=/root/build/target-sm120/release
I=/workspace/uor-r4/runs/claude/injective-preflight-inputs
R=/workspace/uor-r4/runs/claude/injective-preflight-20261019
TABLE=/workspace/uor-r4/local/driver/data/injective/prime-zeta-v4096-b4-c192-k8.json
mark() { echo "## $(date -u +%FT%TZ) $*"; }
mkdir "$R" || { echo "results dir exists: FAIL"; exit 1; }
cd "$SRC"; git rev-parse HEAD > "$R/source-head.txt"
mark "help"; "$B/discrete-stack" 2>&1 | head -60 > "$R/discrete-stack-help.txt"
mark "build-identity"; "$B/discrete-stack" build-identity > "$R/build-identity.json" 2>&1; echo "rc=$?"
mark "table rebuild check (create_new to a fresh path, compare with the pinned file)"
jq -c '.spec' "$I/injective-relation-seed1.json" > "$R/spec.json" 2>/dev/null || jq -n '{vocabulary:4096,blocks:4,context:192,kept_cells:8}' > "$R/spec.json"
"$B/prime-zeta-codebook" "$R/spec.json" 1 "$R/rebuilt-table.json" prime_zeta > "$R/rebuilt-table.stdout" 2>&1; echo "rc=$?"
sha256sum "$R/rebuilt-table.json" "$TABLE" | tee "$R/table-sha.txt"
for A in relation frozen-zero; do
  C="$I/injective-$A-seed1.json"; D="$R/$A"; mkdir "$D"
  mark "$A: attention-build-identity"; "$B/discrete-stack" attention-build-identity "$C" > "$D/identity.json" 2>&1; echo "rc=$?"
  mark "$A: init-corpus (checkpoint 0, new_model(0))"; "$B/discrete-stack" init-corpus "$C" "$D/init" > "$D/init.stdout" 2>&1; echo "rc=$?"
  mark "$A: eval-corpus (zero-update DEV evaluation)"; "$B/discrete-stack" eval-corpus "$C" "$D/init/report.json" "$D/eval" > "$D/eval.stdout" 2>&1; echo "rc=$?"
done
mark "done"; echo STAGE_A_DONE
