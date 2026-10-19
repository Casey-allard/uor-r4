#!/bin/bash
# A8 address-cells dose: pre-run gates (zero updates). Runbook
# docs/labs/a8-dose-runbook-2026-10-19/ (#2756/#2757/#2760/#2761/#2762), sheet 3e622beb (#2755),
# Kimi PM dispatch #2029 6259364299. No training, no apply, no owner phrase.
set -u
SRC=$(ls -d /root/build/src-* | head -1)
export CARGO_TARGET_DIR=/root/build/target-sm120 UOR_NVCC_ARCH=compute_120 PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
B=$CARGO_TARGET_DIR/release
D=/workspace/uor-r4/local/driver/data
CFG=$SRC/docs/labs/a8-training-configs-2026-10-19/evidence
RB=$SRC/docs/labs/injective-attention-runbook-2026-10-19/evidence
NEGB=$SRC/docs/labs/a8-dose-runbook-2026-10-19/evidence/neg-b-a8-nonroot-table.json
P8=$SRC/docs/labs/address-cells-preflight-configs-2026-10-19/evidence/p8.json
A8T=$D/address-cells/address-structured-v1.json
R=/workspace/uor-r4/runs/claude/a8-dose-gates-20261019
mark() { echo "## $(date -u +%FT%TZ) $*"; }
gate() { if eval "$2"; then echo "GATE $1: PASS"; else echo "GATE $1: FAIL"; FAILED=1; fi; }
FAILED=0
mkdir -p "$(dirname "$R")"; mkdir "$R" || { echo "results dir not created exclusively: FAIL"; exit 1; }
cd "$SRC"; git rev-parse HEAD | tee "$R/source-head.txt"
nvidia-smi --query-gpu=index,name,driver_version,memory.used --format=csv,noheader | tee "$R/gpu.txt"

mark "build all uor-r4-discrete CUDA binaries"
cargo build -q --release --locked -p uor-r4-discrete --features cuda 2>&1 | grep -E '^error' | head; echo "build_rc=${PIPESTATUS[0]}"
"$B/discrete-stack" build-identity > "$R/build-identity.json" 2>&1; echo "build_identity_rc=$?"
sha256sum "$B/discrete-stack" | tee "$R/discrete-stack.sha256"

mark "inputs: config pins"
jq -r '[paths(objects) as $p | getpath($p) | select(has("path") and has("sha256")) | "\(.sha256)  \(.path)"] | unique[]' "$CFG/a8-relation-seed1.json" | grep -v address-cells > "$R/data-pins.txt"
sha256sum -c "$R/data-pins.txt" > "$R/data-pins.check" 2>&1; prc=$?; cat "$R/data-pins.check"; gate DATA_PINS "[ $prc -eq 0 ]"

mark "table rebuild (address-cells-codebook, as #2751)"
T=$R/tables; mkdir "$T"
"$B/address-cells-codebook" "$P8" "$T/a8-rebuilt.json" structured > "$T/a8.stdout" 2> "$T/a8.stderr"; echo "a8_rc=$?"; cat "$T/a8.stdout"
a8=$(sha256sum "$T/a8-rebuilt.json" | cut -c1-64)
gate TABLE_REBUILD '[ "$a8" = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1 ] && [ "$(jq -r .table_sha256 "$T/a8.stdout")" = 4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74 ]'
[ "$FAILED" = 0 ] || { echo "STOP: instrument fault before model work (D31)"; exit 2; }
mkdir -p "$D/address-cells"; [ -e "$A8T" ] || cp "$T/a8-rebuilt.json" "$A8T"; sha256sum "$A8T" | tee -a "$T/rebuilt-sha.txt"

INITLOCK='.model.gates as $g | ($g|length)==45 and ([0,5,10,15]|map($g[.]))==[1,1,1,1] and ([range(0;20)]-[0,5,10,15]|all($g[.]==0)) and ($g[20:45]|all(.==0))'
for A in relation frozen-zero; do
  C="$CFG/a8-$A-seed1.json"; U="$RB/parent-rows-$A-seed1.json"; O="$R/$A"; mkdir "$O"
  mark "$A: attention-build-identity"; "$B/discrete-stack" attention-build-identity "$C" > "$O/identity.json" 2>&1; irc=$?; gate "${A}_IDENTITY" "[ $irc -eq 0 ]"
  mark "$A: init-corpus"; "$B/discrete-stack" init-corpus "$C" "$O/init" > "$O/init.stdout" 2>&1; echo "$A init_rc=$?"
  mark "$A: eval-corpus"; "$B/discrete-stack" eval-corpus "$C" "$O/init/report.json" "$O/eval" > "$O/eval.stdout" 2>&1; echo "$A eval_rc=$?"
  mark "$A: attention-dev-b3"; "$B/discrete-stack" attention-dev-b3 "$C" "$O/init/report.json" "$O/eval/report.json" "$O/b3" > "$O/b3.stdout" 2>&1; echo "$A b3_rc=$?"
  mark "$A: attention-panel"; "$B/discrete-stack" attention-panel "$C" "$O/panel" > "$O/panel.stdout" 2>&1; echo "$A panel_rc=$?"
  mark "$A: export-token-cells + census"; "$B/discrete-stack" attention-export-token-cells "$C" "$O/init/report.json" "$O/cells" > "$O/cells.stdout" 2>&1; echo "$A cells_rc=$?"
  "$B/coverage-census" "$C" "$O/init/report.json" "$O/census.json" --token-cells "$O/cells/report.json" --bar-rows 500 > "$O/census.stdout" 2>&1; echo "$A census_rc=$?"
  jq -c '{occupied:.cells.occupied_cells, tie:.within_cell_tie_mass, ceiling:.dev_alias_ceiling_rows, alias:.alias_group_size_histogram}' "$O/census.json"
  gate "${A}_GOCC" '[ "$(jq ".cells.occupied_cells > 16 and .within_cell_tie_mass.dev_rows_targeting_non_representative == 0 and .within_cell_tie_mass.tokens_in_alias_groups == 0" "$O/census.json")" = true ]'
  a=$(jq -c '.model.table' "$O/init/report.json" | sha256sum | cut -c1-64); b=$(jq -c '.table' "$A8T" | sha256sum | cut -c1-64)
  gate "${A}_GINIT_TABLE" '[ "$a" = "$b" ]'
  gate "${A}_INIT_LOCK_GATES" 'jq -e "$INITLOCK" "$O/init/report.json" >/dev/null'
  mark "$A: unpinned initializer (parent rows_v1 config)"; "$B/discrete-stack" init-corpus "$U" "$O/unpinned-init" > "$O/unpinned-init.stdout" 2>&1; echo "$A unpinned_init_rc=$?"
  for k in routes gates seed epoch; do
    x=$(jq -cS ".model.$k" "$O/init/report.json" | sha256sum | cut -c1-64); y=$(jq -cS ".model.$k" "$O/unpinned-init/report.json" | sha256sum | cut -c1-64)
    echo "$A $k pinned=$x unpinned=$y"; gate "${A}_GINIT_UNPINNED_$k" '[ "$x" = "$y" ]'
  done
  b3=$(jq -c '.b3 | [.psucc.target_admitted, .full.target_admitted]' "$O/b3/report.json"); t1=$(jq '.aggregate.top1_correct' "$O/eval/report.json")
  echo "$A b3[psucc,full]=$b3 top1_full=$t1"
  gate "${A}_REPRODUCTION" '[ "$b3" = "[1351,4244]" ] && [ "$t1" = 133 ]'
  jq '.model' "$O/init/report.json" > "$O/snapshot.json"; jq '.probe_request' "$O/panel/report.json" > "$O/request.json"
  sha256sum "$O/snapshot.json" "$O/request.json" | tee "$O/probe-inputs.sha256"
  for g in 22 0; do
    mark "$A: propose gate $g"; "$B/discrete-stack" propose "$O/snapshot.json" "$O/request.json" gate $g > "$O/propose-gate$g.json" 2> "$O/propose-gate$g.stderr"; echo "$A propose_gate${g}_rc=$?"
    jq -c '{candidate, before, after, any_candidate_cost_differs_from_identity, epoch, candidate_losses}' "$O/propose-gate$g.json"
    gate "${A}_GATE${g}_SENSITIVE" '[ "$(jq ".any_candidate_cost_differs_from_identity == true and .epoch == 0" "$O/propose-gate$g.json")" = true ]'
  done
  # Bar (i) inputs: P_succ top-1 at checkpoint 0 (host grading of stored rows).
  jq -c '[.rows[] | {key:"\(.target_position):\(.target_token)", value:.is_psucc}] | from_entries' "$O/panel/report.json" > "$O/panel-psucc.json"
  jq --slurpfile P "$O/panel-psucc.json" '[.batches[].rows[] | select($P[0]["\(.target_position):\(.target_token)"] == true) | select(.top8[0] == .target_token)] | length' "$O/eval/report.json" > "$O/psucc-top1.txt"
  echo "$A psucc_top1_cp0=$(cat "$O/psucc-top1.txt")"
done

mark "cross-arm: checkpoint-0 models byte-equal (INIT LOCK supplement)"
gate CROSS_ARM_MODEL 'cmp -s <(jq -cS .model "$R/relation/init/report.json") <(jq -cS .model "$R/frozen-zero/init/report.json")'
mark "Bar (i) and net G-Cap"
bari=$(( $(cat "$R/frozen-zero/psucc-top1.txt") + 24 )); ceil=$(jq '.dev_alias_ceiling_rows' "$R/relation/census.json"); ps=$(jq '.b3.psucc.target_admitted' "$R/relation/b3/report.json")
net=$(( ceil < ps ? ceil : ps )); thr=$(( bari > 500 ? bari : 500 ))
echo "bar_i=$bari ceiling=$ceil b3_psucc=$ps net_gcap=$net threshold=$thr"; gate NET_GCAP '[ $net -gt $thr ]'; gate B3_INIT '[ $ps -ge 500 ]'

mark "G-Init negative (a): edited A8 table, stale hashes, refused on the host"
N="$R/neg"; mkdir "$N"
jq -c '.table[0] = (if .table[0] == 0 then 2 else 0 end)' "$A8T" > "$N/a.json"
jq --arg p "$N/a.json" '.codebook_initialization.table.path = $p' "$CFG/a8-relation-seed1.json" > "$N/a-cfg.json"
"$B/discrete-stack" init-corpus "$N/a-cfg.json" "$N/a-run" > "$N/a.stdout" 2>&1; rc=$?; echo "neg_a_rc=$rc"; tail -c 300 "$N/a.stdout"; echo
gate NEG_A '[ $rc -ne 0 ]'
mark "G-Init negative (b): re-hashed re-pinned A8 non-root word, refused on the GPU (status 6)"
cp "$NEGB" "$N/b.json"; nb=$(sha256sum "$N/b.json" | cut -c1-64)
jq --arg p "$N/b.json" --arg s "$nb" '.codebook_initialization.table.path = $p | .codebook_initialization.table.sha256 = $s' "$CFG/a8-relation-seed1.json" > "$N/b-cfg.json"
"$B/discrete-stack" init-corpus "$N/b-cfg.json" "$N/b-run" > "$N/b.stdout" 2>&1; rc=$?; echo "neg_b_rc=$rc nb=$nb"; tail -c 300 "$N/b.stdout"; echo
gate NEG_B '[ $rc -ne 0 ] && grep -qiE "status[^0-9]*6" "$N/b.stdout"'

mark "qualify-relation (C2 ACTIVE/INERT, relation suite, INIT LOCK record)"
"$B/discrete-stack" qualify-relation "$R/qualify-relation" > "$R/qualify-relation.stdout" 2>&1; echo "qualify_rc=$?"
jq -c '{status, registered, passed, not_run, applied_updates}' "$R/qualify-relation/relation-controls.json"
gate QUALIFY_RELATION '[ "$(jq "(.status==\"PASS\") and .registered==9 and .passed==9 and .not_run==0 and .applied_updates==0" "$R/qualify-relation/relation-controls.json")" = true ]'
grep '"part":"INIT LOCK"' "$R/qualify-relation.stdout" | jq -c '{part, treatment_control_snapshot_bit_identical, reload_bit_identical, plus_one_combine_indices, snapshot_sha256, applied_updates}' | tee "$R/init-lock-records.jsonl"
gate INIT_LOCK_RECORD '[ -s "$R/init-lock-records.jsonl" ] && jq -se "all(.treatment_control_snapshot_bit_identical and .reload_bit_identical and .applied_updates==0)" "$R/init-lock-records.jsonl" >/dev/null'

mark "done"; (cd "$R" && find . -type f | sort | xargs sha256sum > FILES.sha256)
[ "$FAILED" = 0 ] && echo "GATES_ALL_PASS" || echo "GATES_FAILED"; echo GATES_DONE
