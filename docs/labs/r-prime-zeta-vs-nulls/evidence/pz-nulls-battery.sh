#!/bin/bash
# PrimeZeta vs QuotaOnlyInjective (Q0..Q2) and RandomInjective (R0..R2) zero-update
# pre-flight battery (`r-prime-zeta-vs-nulls`, #2881).
# Sheet: docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md
# Requires non-author TEST FITNESS: FIT and owner/PM bar + pod-spend approval before execution.
# Seven arms P8, Q0..Q2, R0..R2 at checkpoint 0 (accepted_updates = 0). No training, no apply.
set -euo pipefail
SRC=$(ls -d /root/build/src-* | head -1)
export CARGO_TARGET_DIR=/root/build/target-sm120 UOR_NVCC_ARCH=compute_120 PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
B=$CARGO_TARGET_DIR/release
D=/workspace/uor-r4/local/driver/data
EV=$SRC/docs/labs/r-prime-zeta-vs-nulls/evidence
R=/workspace/uor-r4/runs/antigravity/r-prime-zeta-vs-nulls-20261022
SPEC=$EV/spec-v4096-b4-c192-k8.json

mark() { echo "## $(date -u +%FT%TZ) $*"; }
mkdir -p "$(dirname "$R")"
mkdir "$R" || { echo "results dir not created exclusively: FAIL"; exit 1; }
cd "$SRC"
git rev-parse HEAD | tee "$R/source-head.txt"
nvidia-smi --query-gpu=name,driver_version,memory.used --format=csv,noheader | tee "$R/gpu.txt"

mark "build uor-r4-discrete CUDA binaries"
cargo build -q --release --locked -p uor-r4-discrete --features cuda
"$B/discrete-stack" build-identity > "$R/build-identity.json" 2>&1

mark "verify pinned corpus data SHAs"
sha256sum "$D/ft/dev/tokens.u16" "$D/ft/dev/response_mask.u8" "$D/mw-balp/train/tokens.u16" "$D/mw-balp/train/response_mask.u8" "$D/tokenizer.json" | tee "$R/inputs-sha.txt"

mark "rebuild all 7 codebook tables on the pod and verify exact file_sha256 pins"
T=$R/tables
mkdir "$T"
mkdir -p "$D/injective"

rebuild_and_check() {
  local arm=$1 mode=$2 expected_sha=$3 dest=$4
  "$B/prime-zeta-codebook" "$SPEC" 0 "$T/$arm.json" "$mode" > "$T/$arm.stdout" 2> "$T/$arm.stderr"
  local actual_sha
  actual_sha=$(sha256sum "$T/$arm.json" | cut -c1-64)
  echo "$arm mode=$mode actual=$actual_sha expected=$expected_sha" | tee -a "$T/rebuilt-sha.txt"
  if [ "$actual_sha" != "$expected_sha" ]; then
    echo "TABLE_REBUILD_FAIL ($arm): D31 stop"
    exit 2
  fi
  [ -e "$dest" ] || cp "$T/$arm.json" "$dest"
}

rebuild_and_check p8 "prime_zeta" "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b" "$D/injective/prime-zeta-v4096-b4-c192-k8.json"
rebuild_and_check q0 "quota_only_injective:0" "33b4e459cef8303f9d502f6f3e66327ec9e045026e7c8de896ad3ff350c2c415" "$D/injective/quota-only-injective-seed0-v4096-b4-c192-k8.json"
rebuild_and_check q1 "quota_only_injective:1" "57225bbb63387d9a6f39e7cb6415aa0e317d19534633bb731a3d7559efd4c3b9" "$D/injective/quota-only-injective-seed1-v4096-b4-c192-k8.json"
rebuild_and_check q2 "quota_only_injective:2" "e03d6aef42608e1db3f4d33a0b034fd2c113cd9c48f366e0b6319d20d4f77175" "$D/injective/quota-only-injective-seed2-v4096-b4-c192-k8.json"
rebuild_and_check r0 "random_injective:0" "229e93f58ad36cb3f1fc2fa4ab194ab8ce06798a67e39537352b9817cc5df345" "$D/injective/random-injective-seed0-v4096-b4-c192-k8.json"
rebuild_and_check r1 "random_injective:1" "36fec6323ae1daf15d965ff5cd1161f66b7a4a8a8b0a3261e9094e9c125a9468" "$D/injective/random-injective-seed1-v4096-b4-c192-k8.json"
rebuild_and_check r2 "random_injective:2" "25ec9bd29ce7fec20721826bab93f46caed8720cda6387cd43c5bddc62f0edea" "$D/injective/random-injective-seed2-v4096-b4-c192-k8.json"
echo "TABLE_REBUILD: PASS"

for A in p8 q0 q1 q2 r0 r1 r2; do
  C="$EV/$A.json"
  O="$R/$A"
  mkdir "$O"
  mark "$A: attention-build-identity"
  "$B/discrete-stack" attention-build-identity "$C" > "$O/identity.json" 2>&1
  mark "$A: init-corpus (checkpoint 0)"
  "$B/discrete-stack" init-corpus "$C" "$O/init" > "$O/init.stdout" 2>&1
  mark "$A: eval-corpus (zero-update DEV)"
  "$B/discrete-stack" eval-corpus "$C" "$O/init/report.json" "$O/eval" > "$O/eval.stdout" 2>&1
  mark "$A: attention-dev-b3"
  "$B/discrete-stack" attention-dev-b3 "$C" "$O/init/report.json" "$O/eval/report.json" "$O/b3" > "$O/b3.stdout" 2>&1
  mark "$A: attention-export-token-cells"
  "$B/discrete-stack" attention-export-token-cells "$C" "$O/init/report.json" "$O/cells" > "$O/cells.stdout" 2>&1
  mark "$A: coverage-census --token-cells"
  "$B/coverage-census" "$C" "$O/init/report.json" "$O/census.json" --token-cells "$O/cells/report.json" --bar-rows 500 > "$O/census.stdout" 2>&1
  jq -c '{arm:"'"$A"'", b3_full:.b3.full.target_admitted, b3_psucc:.b3.psucc.target_admitted, psucc_rows:.b3.psucc.rows}' "$O/b3/report.json" | tee -a "$R/summary.jsonl"
  jq -c '{arm:"'"$A"'", distinct:.distinct_vectors, ceiling:.dev_alias_ceiling_rows, tie_mass:.within_cell_tie_mass.tokens_in_alias_groups, occupied:.cells.occupied_cells, max_cell:.cells.max_cell_occupancy, collisions:.cells.cell_collision_pairs}' "$O/census.json" | tee -a "$R/summary.jsonl"
done

mark "done"
sha256sum $(find "$R" -type f | sort) > "$R/FILES.sha256"
echo "BATTERY_DONE"
