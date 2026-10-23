#!/bin/bash
# Read-off vs read-on paired evaluation battery (zero updates, #2885).
# Sheet: docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md
# Ten arms: {a8, p8} x {r-on, r-off, r-perm, f-on, f-off} at checkpoint 0.
# No training, no apply, no owner phrase. Requires non-author FIT + Casey chat GO before pod execution.
set -u
SRC=$(ls -d /root/build/src-* | head -1)
export CARGO_TARGET_DIR=/root/build/target-sm120 UOR_NVCC_ARCH=compute_120 PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
B=$CARGO_TARGET_DIR/release
D=/workspace/uor-r4/local/driver/data
EV=$SRC/docs/labs/r-read-off-vs-read-on/evidence
R=/workspace/uor-r4/runs/antigravity/r-read-off-vs-read-on-20261023
PZ=$D/injective/prime-zeta-v4096-b4-c192-k8.json
A8T=$D/address-cells/address-structured-v1.json
mark() { echo "## $(date -u +%FT%TZ) $*"; }
mkdir -p "$(dirname "$R")"
mkdir "$R" || { echo "results dir not created exclusively: FAIL"; exit 1; }
cd "$SRC"; git rev-parse HEAD | tee "$R/source-head.txt"
nvidia-smi --query-gpu=name,driver_version,memory.used --format=csv,noheader | tee "$R/gpu.txt"

mark "build all uor-r4-discrete CUDA binaries"
cargo build -q --release --locked -p uor-r4-discrete --features cuda 2>&1 | grep -E '^error' | head; echo "build_rc=${PIPESTATUS[0]}"
"$B/discrete-stack" build-identity > "$R/build-identity.json" 2>&1; echo "build_identity_rc=$?"

mark "G-Qual: CUDA device test route_read_off_and_permuted_controls_run_and_refuse"
cargo test -q --release --locked -p uor-r4-discrete --features cuda --lib route_read_off_and_permuted_controls_run_and_refuse -- --ignored --exact > "$R/g-qual-device-test.stdout" 2>&1
rc=$?; echo "g_qual_rc=$rc"
[ $rc -eq 0 ] && echo "G_QUAL: PASS" || { echo "G_QUAL: FAIL"; cat "$R/g-qual-device-test.stdout"; exit 2; }

mark "verify or rebuild A8 table from TRAIN if missing"
if [ ! -e "$A8T" ]; then
  mkdir -p "$D/address-cells" "$R/tables"
  "$B/address-cells-codebook" "$EV/p8-f-on.json" "$R/tables/a8-rebuilt.json" structured > "$R/tables/a8.stdout" 2> "$R/tables/a8.stderr"
  cp "$R/tables/a8-rebuilt.json" "$A8T"
fi
sha256sum "$D/ft/dev/tokens.u16" "$D/ft/dev/response_mask.u8" "$D/mw-balp/train/tokens.u16" "$D/mw-balp/train/response_mask.u8" "$D/tokenizer.json" "$PZ" "$A8T" | tee "$R/inputs-sha.txt"
a8=$(sha256sum "$A8T" | cut -c1-64); pz=$(sha256sum "$PZ" | cut -c1-64)
[ "$a8" = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1 ] && [ "$pz" = 83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b ] \
  && echo "INPUT_TABLES: PASS" || { echo "INPUT_TABLES: FAIL (D31 stop)"; exit 3; }

ARMS="a8-r-on a8-r-off a8-r-perm a8-f-on a8-f-off p8-r-on p8-r-off p8-r-perm p8-f-on p8-f-off"
for A in $ARMS; do
  C="$EV/$A.json"; O="$R/$A"; mkdir "$O"
  mark "$A: init-corpus (checkpoint 0)"
  "$B/discrete-stack" init-corpus "$C" "$O/init" > "$O/init.stdout" 2>&1; rc=$?; echo "$A init_rc=$rc"
  [ $rc -eq 0 ] || { echo "$A init-corpus FAIL"; tail -c 400 "$O/init.stdout"; exit 4; }
  mark "$A: eval-corpus (zero-update DEV)"
  "$B/discrete-stack" eval-corpus "$C" "$O/init/report.json" "$O/eval" > "$O/eval.stdout" 2>&1; rc=$?; echo "$A eval_rc=$rc"
  [ $rc -eq 0 ] || { echo "$A eval-corpus FAIL"; tail -c 400 "$O/eval.stdout"; exit 5; }
  jq -c '.aggregate' "$O/eval/report.json" 2>/dev/null | head -c 600; echo
  mark "$A: attention-dev-b3 (B3 full/psucc/complement)"
  "$B/discrete-stack" attention-dev-b3 "$C" "$O/init/report.json" "$O/eval/report.json" "$O/b3" > "$O/b3.stdout" 2>&1; rc=$?; echo "$A b3_rc=$rc"
  [ $rc -eq 0 ] || { echo "$A attention-dev-b3 FAIL"; tail -c 400 "$O/b3.stdout"; exit 6; }
  jq -c '.b3 | {full:.full.target_admitted, psucc:.psucc.target_admitted, complement:.complement.target_admitted}' "$O/b3/report.json" 2>/dev/null
  case $A in a8-*) PIN=$A8T;; p8-*) PIN=$PZ;; esac
  a=$(jq -c '.model.table' "$O/init/report.json" | sha256sum | cut -c1-64); b=$(jq -c '.table' "$PIN" | sha256sum | cut -c1-64)
  upd=$(jq -r '.cursor.accepted_updates' "$O/init/report.json"); ep=$(jq -r '.model.epoch' "$O/init/report.json")
  [ "$a" = "$b" ] && [ "$upd" = 0 ] && [ "$ep" = 0 ] && echo "$A G_INIT_AND_ZERO_APPLY: PASS" || { echo "$A G_INIT_AND_ZERO_APPLY: FAIL"; exit 7; }
done

mark "G-Repro: verify a8-f-on (4244/1351/133) and p8-f-on (427/127/38)"
a8_full=$(jq -r '.b3.full.target_admitted' "$R/a8-f-on/b3/report.json")
a8_psucc=$(jq -r '.b3.psucc.target_admitted' "$R/a8-f-on/b3/report.json")
a8_top1=$(jq -r '.aggregate.top1_hits' "$R/a8-f-on/eval/report.json")
p8_full=$(jq -r '.b3.full.target_admitted' "$R/p8-f-on/b3/report.json")
p8_psucc=$(jq -r '.b3.psucc.target_admitted' "$R/p8-f-on/b3/report.json")
p8_top1=$(jq -r '.aggregate.top1_hits' "$R/p8-f-on/eval/report.json")
echo "a8-f-on=$a8_full/$a8_psucc/$a8_top1 p8-f-on=$p8_full/$p8_psucc/$p8_top1"
[ "$a8_full" = 4244 ] && [ "$a8_psucc" = 1351 ] && [ "$a8_top1" = 133 ] && [ "$p8_full" = 427 ] && [ "$p8_psucc" = 127 ] && [ "$p8_top1" = 38 ] \
  && echo "G_REPRO: PASS" || { echo "G_REPRO: FAIL"; exit 8; }

mark "G-CrossMode: verify r-off == f-off across all 14287 rows on a8 and p8"
for FAM in a8 p8; do
  r_agg=$(jq -c '.aggregate' "$R/$FAM-r-off/eval/report.json" | sha256sum | cut -c1-64)
  f_agg=$(jq -c '.aggregate' "$R/$FAM-f-off/eval/report.json" | sha256sum | cut -c1-64)
  r_bat=$(jq -c '.batches' "$R/$FAM-r-off/eval/report.json" | sha256sum | cut -c1-64)
  f_bat=$(jq -c '.batches' "$R/$FAM-f-off/eval/report.json" | sha256sum | cut -c1-64)
  r_b3=$(jq -c '.b3' "$R/$FAM-r-off/b3/report.json" | sha256sum | cut -c1-64)
  f_b3=$(jq -c '.b3' "$R/$FAM-f-off/b3/report.json" | sha256sum | cut -c1-64)
  echo "$FAM r_off_agg=$r_agg f_off_agg=$f_agg r_off_bat=$r_bat f_off_bat=$f_bat r_off_b3=$r_b3 f_off_b3=$f_b3"
  [ "$r_agg" = "$f_agg" ] && [ "$r_bat" = "$f_bat" ] && [ "$r_b3" = "$f_b3" ] \
    && echo "$FAM G_CROSSMODE: PASS" || { echo "$FAM G_CROSSMODE: FAIL"; exit 9; }
done

mark "done"; sha256sum $(find "$R" -type f | sort) > "$R/FILES.sha256"; echo BATTERY_DONE
