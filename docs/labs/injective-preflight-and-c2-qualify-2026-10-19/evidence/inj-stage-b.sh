#!/bin/bash
# Injective attention pre-flight, stage B (zero updates): B3_init via attention-dev-b3, panel,
# GPU token-cell export + coverage census (G-Occ, G-Cap raw), G-Init table equality, G-Init negative (a).
set -u
B=/root/build/target-sm120/release
I=/workspace/uor-r4/runs/claude/injective-preflight-inputs
R=/workspace/uor-r4/runs/claude/injective-preflight-20261019
TABLE=/workspace/uor-r4/local/driver/data/injective/prime-zeta-v4096-b4-c192-k8.json
mark() { echo "## $(date -u +%FT%TZ) $*"; }
for A in relation frozen-zero; do
  C="$I/injective-$A-seed1.json"; D="$R/$A"
  mark "$A: attention-dev-b3"; "$B/discrete-stack" attention-dev-b3 "$C" "$D/init/report.json" "$D/eval/report.json" "$D/b3" > "$D/b3.stdout" 2>&1; echo "rc=$?"
  mark "$A: attention-panel"; "$B/discrete-stack" attention-panel "$C" "$D/panel" > "$D/panel.stdout" 2>&1; echo "rc=$?"
done
D="$R/relation"; C="$I/injective-relation-seed1.json"
mark "relation: attention-export-token-cells (GPU, zero updates)"; "$B/discrete-stack" attention-export-token-cells "$C" "$D/init/report.json" "$D/cells" > "$D/cells.stdout" 2>&1; echo "rc=$?"
mark "relation: coverage-census --token-cells"; "$B/coverage-census" "$C" "$D/init/report.json" "$D/census.json" --token-cells "$D/cells/report.json" --bar-rows 500 > "$D/census.stdout" 2>&1; echo "rc=$?"
mark "G-Init: checkpoint-0 table equals the pinned table word for word"
a=$(jq -c '.model.table' "$D/init/report.json" | sha256sum | cut -c1-64); b=$(jq -c '.table' "$TABLE" | sha256sum | cut -c1-64)
echo "cp0_table_json_sha=$a pinned_table_json_sha=$b"; [ "$a" = "$b" ] && echo "GINIT_TABLE: PASS" || echo "GINIT_TABLE: FAIL"
mark "G-Init negative (a): an edited table file with stale hashes is refused on the host"
N="$R/neg"; mkdir -p "$N"
jq -c '.table[0] = (if .table[0] == 0 then 2 else 0 end)' "$TABLE" > "$N/a.json"
jq --arg p "$N/a.json" '.codebook_initialization.table.path = $p' "$C" > "$N/a-stale-pin-cfg.json"
"$B/discrete-stack" init-corpus "$N/a-stale-pin-cfg.json" "$N/a-run" > "$N/a.stdout" 2>&1; rc=$?; echo "rc=$rc"
[ $rc -ne 0 ] && echo "NEG_A: PASS (refused)" || echo "NEG_A: FAIL (accepted an edited table)"
tail -c 600 "$N/a.stdout"
mark "done"; echo STAGE_B_DONE
