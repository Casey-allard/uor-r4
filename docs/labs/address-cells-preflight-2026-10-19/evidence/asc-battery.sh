#!/bin/bash
# Address-structured cells pre-flight battery (zero updates). Sheet
# m1-attention-address-structured-cells-DRAFT-2026-10-19.md (FIT #2746, bindings #2748),
# Kimi PM dispatch #2029 6258511411, owner GO re-confirmed to Claude in chat 03:06Z.
# Five arms P8/A8/S8/P16/A16 at checkpoint 0. No training, no apply, no owner phrase.
set -u
SRC=$(ls -d /root/build/src-* | head -1)
export CARGO_TARGET_DIR=/root/build/target-sm120 UOR_NVCC_ARCH=compute_120 PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
B=$CARGO_TARGET_DIR/release
D=/workspace/uor-r4/local/driver/data
EV=$SRC/docs/labs/address-cells-preflight-configs-2026-10-19/evidence
R=/workspace/uor-r4/runs/claude/address-cells-preflight-20261019
NEGB=/root/neg-b-nonroot-table.json
PZ=$D/injective/prime-zeta-v4096-b4-c192-k8.json
A8T=$D/address-cells/address-structured-v1.json
S8T=$D/address-cells/address-shuffled-v1-seed-20261019.json
mark() { echo "## $(date -u +%FT%TZ) $*"; }
mkdir -p "$(dirname "$R")"
mkdir "$R" || { echo "results dir not created exclusively: FAIL"; exit 1; }
cd "$SRC"; git rev-parse HEAD | tee "$R/source-head.txt"
nvidia-smi --query-gpu=name,driver_version,memory.used --format=csv,noheader | tee "$R/gpu.txt"

mark "build all uor-r4-discrete CUDA binaries"
cargo build -q --release --locked -p uor-r4-discrete --features cuda 2>&1 | grep -E '^error' | head; echo "build_rc=${PIPESTATUS[0]}"
"$B/discrete-stack" build-identity > "$R/build-identity.json" 2>&1; echo "build_identity_rc=$?"

mark "inputs: pinned data and the PrimeZeta table"
sha256sum $D/ft/dev/tokens.u16 $D/ft/dev/response_mask.u8 $D/mw-balp/train/tokens.u16 $D/mw-balp/train/response_mask.u8 $D/tokenizer.json "$PZ" | tee "$R/inputs-sha.txt"

mark "rebuild the A8 and S8 tables from TRAIN on the pod (address-cells-codebook)"
T=$R/tables; mkdir "$T"
"$B/address-cells-codebook" "$EV/p8.json" "$T/a8-rebuilt.json" structured > "$T/a8.stdout" 2> "$T/a8.stderr"; echo "a8_rc=$?"
"$B/address-cells-codebook" "$EV/p8.json" "$T/s8-rebuilt.json" shuffled:20261019 > "$T/s8.stdout" 2> "$T/s8.stderr"; echo "s8_rc=$?"
cat "$T/a8.stdout" "$T/s8.stdout"
a8=$(sha256sum "$T/a8-rebuilt.json" | cut -c1-64); s8=$(sha256sum "$T/s8-rebuilt.json" | cut -c1-64)
echo "a8_file=$a8 s8_file=$s8" | tee "$T/rebuilt-sha.txt"
[ "$a8" = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1 ] && [ "$s8" = a185bdca07077fe7157e947ff1fa80919e6f99165a4dd5a04061a97873c92718 ] \
  && echo "TABLE_REBUILD: PASS" || { echo "TABLE_REBUILD: FAIL (D31 stop)"; exit 2; }
mkdir -p "$D/address-cells"
[ -e "$A8T" ] || cp "$T/a8-rebuilt.json" "$A8T"; [ -e "$S8T" ] || cp "$T/s8-rebuilt.json" "$S8T"
sha256sum "$A8T" "$S8T" | tee -a "$T/rebuilt-sha.txt"

for A in p8 a8 s8 p16 a16; do
  C="$EV/$A.json"; O="$R/$A"; mkdir "$O"
  mark "$A: attention-build-identity"; "$B/discrete-stack" attention-build-identity "$C" > "$O/identity.json" 2>&1; echo "$A identity_rc=$?"
  mark "$A: init-corpus (checkpoint 0)"; "$B/discrete-stack" init-corpus "$C" "$O/init" > "$O/init.stdout" 2>&1; echo "$A init_rc=$?"
  mark "$A: eval-corpus (zero-update DEV)"; "$B/discrete-stack" eval-corpus "$C" "$O/init/report.json" "$O/eval" > "$O/eval.stdout" 2>&1; echo "$A eval_rc=$?"
  jq -c '.aggregate' "$O/eval/report.json" 2>/dev/null | head -c 600; echo
done

for A in p8 a8 s8; do
  C="$EV/$A.json"; O="$R/$A"
  mark "$A: attention-dev-b3"; "$B/discrete-stack" attention-dev-b3 "$C" "$O/init/report.json" "$O/eval/report.json" "$O/b3" > "$O/b3.stdout" 2>&1; echo "$A b3_rc=$?"
  jq -c '.b3 | {full:.full.target_admitted, psucc:.psucc.target_admitted, psucc_rows:.psucc.rows, complement:.complement.target_admitted}' "$O/b3/report.json" 2>/dev/null
  mark "$A: attention-panel"; "$B/discrete-stack" attention-panel "$C" "$O/panel" > "$O/panel.stdout" 2>&1; echo "$A panel_rc=$?"
  mark "$A: attention-export-token-cells"; "$B/discrete-stack" attention-export-token-cells "$C" "$O/init/report.json" "$O/cells" > "$O/cells.stdout" 2>&1; echo "$A cells_rc=$?"
  mark "$A: coverage-census --token-cells"; "$B/coverage-census" "$C" "$O/init/report.json" "$O/census.json" --token-cells "$O/cells/report.json" --bar-rows 500 > "$O/census.stdout" 2>&1; echo "$A census_rc=$?"
  jq -c '{occupied:.cells.occupied_cells, tie_mass:.cells.within_cell_tie_mass, ceiling:.cells.dev_alias_ceiling_rows}' "$O/census.json" 2>/dev/null
  case $A in p8) PIN=$PZ;; a8) PIN=$A8T;; s8) PIN=$S8T;; esac
  a=$(jq -c '.model.table' "$O/init/report.json" | sha256sum | cut -c1-64); b=$(jq -c '.table' "$PIN" | sha256sum | cut -c1-64)
  echo "$A cp0_table=$a pinned_table=$b"; [ "$a" = "$b" ] && echo "$A GINIT_TABLE: PASS" || echo "$A GINIT_TABLE: FAIL"
done

mark "G-Init negative (a): an edited A8 table with stale hashes is refused on the host"
N="$R/neg"; mkdir "$N"
jq -c '.table[0] = (if .table[0] == 0 then 2 else 0 end)' "$A8T" > "$N/a.json"
jq --arg p "$N/a.json" '.codebook_initialization.table.path = $p' "$EV/a8.json" > "$N/a-cfg.json"
"$B/discrete-stack" init-corpus "$N/a-cfg.json" "$N/a-run" > "$N/a.stdout" 2>&1; rc=$?; echo "neg_a_rc=$rc"
[ $rc -ne 0 ] && echo "NEG_A: PASS (refused)" || echo "NEG_A: FAIL (accepted)"; tail -c 400 "$N/a.stdout"; echo
mark "G-Init negative (b): a re-hashed, re-pinned non-root word is refused on the GPU (status 6)"
cp "$NEGB" "$N/b.json"; nb=$(sha256sum "$N/b.json" | cut -c1-64)
jq --arg p "$N/b.json" --arg s "$nb" '.codebook_initialization.table.path = $p | .codebook_initialization.table.sha256 = $s' "$EV/p8.json" > "$N/b-cfg.json"
"$B/discrete-stack" init-corpus "$N/b-cfg.json" "$N/b-run" > "$N/b.stdout" 2>&1; rc=$?; echo "neg_b_rc=$rc"
grep -qiE 'status[^0-9]*6' "$N/b.stdout" && [ $rc -ne 0 ] && echo "NEG_B: PASS (GPU status 6)" || echo "NEG_B: CHECK (rc=$rc; read b.stdout)"; tail -c 400 "$N/b.stdout"; echo

mark "done"; sha256sum $(find "$R" -type f | sort) > /root/asc-files.sha256; echo BATTERY_DONE
