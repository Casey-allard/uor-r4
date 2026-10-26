#!/bin/bash
# [antigravity] antigravity-92 (#2847): wider-code-carrier (B in {6, 8}) finite
# CUDA qualification and zero-training capacity pre-flight, sheet
# docs/harness/prep/m1-wider-code-carrier-2026-10-26.md.
# Order: pins -> two-build CUDA compilation -> G-Q (4 finite CUDA controls +
# timing) -> G-Exact & G-D11 (A8-Outer, FV8-B4-Env, FV8-B4-TwoScale, WB6-Outer,
# WB8-Outer) -> wider-carrier treatments & controls with G-Occ -> grade -> archive.
# Terminal markers: $OUT/EXIT (0 or 1), and $OUT/FAULT on any failure.
set -uo pipefail

OUT=/workspace/m1-wider-code-carrier-2026-10-26
BIN_DIR=/workspace/wider-carrier-bin
DATA=/workspace/uor-r4/local/driver/data
DL=/workspace/dl
mkdir -p "$OUT/arms" "$BIN_DIR" "$DATA/address-cells" "$DL"
exec > >(tee -a "$OUT/job.log") 2>&1
cp "$0" "$OUT/job.sh"

fail() {
  echo "FAULT: $*" | tee "$OUT/FAULT"
  echo 1 > "$OUT/EXIT"
  exit 1
}

free_gib=$(df -BG --output=avail /workspace | tail -1 | tr -dc '0-9')
[ "${free_gib:-0}" -ge 30 ] || fail "free disk ${free_gib} GiB is under 30"

export PATH=/usr/local/cuda/bin:$HOME/.cargo/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export UOR_NVCC_ARCH=compute_120

SRC=$(ls -d /root/build/src-* 2>/dev/null | head -n 1 || true)
[ -n "$SRC" ] && [ -d "$SRC/.git" ] || fail "no /root/build/src-<sha> checkout found"
SHA=$(git -C "$SRC" rev-parse HEAD)
echo "source_commit=$SHA" | tee "$OUT/source_commit.txt"
cd "$SRC" || fail "cd $SRC"

nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv > "$OUT/gpu.csv" || fail "nvidia-smi"
grep -q "RTX 5090" "$OUT/gpu.csv" || fail "not an RTX 5090 (UOR_NVCC_ARCH=compute_120 assumes one)"
[ -x /root/uor-store ] || fail "no /root/uor-store on this pod"

echo "== Stage 1: corpus and pinned table SHA-256 verification"
for f in data/ft-balp.tar data/ft-dev.tar data/tokenizer.json.tar; do
  /root/uor-store get "$f" "$DL" || fail "store get $f"
done
tar -xf "$DL/data/ft-balp.tar" -C "$DATA" mw-balp/train || fail "untar TRAIN"
tar -xf "$DL/data/ft-dev.tar" -C "$DATA" ft/dev || fail "untar DEV"
tar -xf "$DL/data/tokenizer.json.tar" -C "$DATA" tokenizer.json || fail "untar tokenizer"

cat > "$OUT/data.sha256" <<EOF
3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8  $DATA/mw-balp/train/tokens.u16
b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3  $DATA/mw-balp/train/response_mask.u8
0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  $DATA/ft/dev/tokens.u16
a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  $DATA/ft/dev/response_mask.u8
d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89  $DATA/tokenizer.json
EOF
sha256sum -c "$OUT/data.sha256" || fail "data pins"

/root/uor-store get results/antigravity/m1-wider-code-carrier-2026-10-26/tables/ "$DL" || fail "store get tables"
T="$DL/results/antigravity/m1-wider-code-carrier-2026-10-26/tables"
(cd "$T" && sha256sum -c SHA256SUMS) || fail "tables SHA256SUMS"
for f in \
  address-structured-v1.json \
  address-full-vector-v1.json \
  address-full-vector-shuffled-v1-seed-20261019.json \
  b6-wider-blocks-v1.json \
  b6-wider-blocks-shuffled-v1-seed-20261019.json \
  b8-wider-blocks-v1.json \
  b8-wider-blocks-shuffled-v1-seed-20261019.json \
  b8-full-vector-pad-v1.json; do
  cp "$T/$f" "$DATA/address-cells/$f" || fail "cp $f"
done

cat > "$OUT/tables.sha256" <<EOF
6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1  $DATA/address-cells/address-structured-v1.json
2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2  $DATA/address-cells/address-full-vector-v1.json
5e5abf336bfb039d6c3bde336cf5d03f207ae9ff7bc8cfb21e71691c622313c4  $DATA/address-cells/address-full-vector-shuffled-v1-seed-20261019.json
fd2c79277ef20c7fa1cac136c5d8f6fcb2586f5563aecfd889afb5c4b6748374  $DATA/address-cells/b6-wider-blocks-v1.json
67031b7f9da75e6672aced861b299b9f0301ba7894d2571d6b81525bec873df5  $DATA/address-cells/b6-wider-blocks-shuffled-v1-seed-20261019.json
d5c3cb97f02210942778fe26fd874aaabf967d605438715925f501f2214a3222  $DATA/address-cells/b8-wider-blocks-v1.json
1490c1b8c8878221d65f39bd975259ccaadc2a338b94bdc91c2061f0fc535dd2  $DATA/address-cells/b8-wider-blocks-shuffled-v1-seed-20261019.json
2b933bc5d4f0da4eb602e5490876f80cf8ce21bdbe47306a8f27da7e90df5620  $DATA/address-cells/b8-full-vector-pad-v1.json
EOF
sha256sum -c "$OUT/tables.sha256" || fail "table pins"

C=docs/labs/m1-wider-code-carrier/configs
cat > "$OUT/configs.sha256" <<EOF
469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e  docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json
56c3fafe0b8cdb7800f57d0a6ba577b2964356de5765b350a701546cace6e534  docs/labs/m1-address-key-full-vector/configs/fv8-env.json
1cdb80615d1467bbe062bc1a70804630b3f69af5c580ce3b0d2b5d7407a89a28  docs/labs/m1-address-key-full-vector/configs/fv8-two-scale.json
4e1545948a9cb4d3560590f7d5e3db64acf0efbe063eef48dd32d0c978c21c3a  $C/wb6-outer.json
e86d0f157310d928bb9c515291566a563d0acffe7f1e1c6089c63aacf98160b2  $C/wb6-env.json
49a2822bb65e5d895fe8e9820bf3a450a27b552c80e7f9365d5424a6e33593f7  $C/wb6-two-scale.json
ec86db0fe1c537cdda2401df525cce84c81a6b68d4192f7a3c63851818e1582f  $C/wbs6-env.json
c3499a619b21a9264792d8f2aed9177de302b7bf5d8d5e02a48e0d5263ab0d2f  $C/wbs6-two-scale.json
6a72404fef373dc2f3c59d3141819ccbdd168fa5004bbef2b4803221a3f19af0  $C/wb8-outer.json
840431c40a0ea25567727a2ff1f6c026f70b9bcf906291ace9982941cb8628ba  $C/wb8-env.json
c31490a11dd9525ba702534511ef92621790f8e138913df6a343267896fbc16e  $C/wb8-two-scale.json
cdcd9d102172d6f47e13ad7293f51f447cc00c756b7b0ab78bde4ee09bc38edc  $C/wbs8-env.json
9992e290a7940bc377057faade753bd7d8448e603c7a03ffc9e076e2f44e7733  $C/wbs8-two-scale.json
b67157b8dbf39ff3a240eb3ab0cb1f9c2a3654e720d51b09b2f6eb813a337a7e  $C/fv8-b8-pad-env.json
d025c109a30dfce732c6e846c9f8947a53b17eca5459a9d3e4bf138b09d321d2  $C/fv8-b8-pad-two-scale.json
EOF
sha256sum -c "$OUT/configs.sha256" || fail "config pins"

echo "== Stage 2: two independent CUDA release builds (sm_120)"
CARGO_TARGET_DIR=/root/build/target-wider-1 cargo build --release --locked -p uor-r4-discrete --features cuda --bin wider-carrier-qualify --bin wider-carrier-run || fail "build 1"
CARGO_TARGET_DIR=/root/build/target-wider-2 cargo build --release --locked -p uor-r4-discrete --features cuda --bin wider-carrier-qualify --bin wider-carrier-run || fail "build 2"

cp /root/build/target-wider-1/release/wider-carrier-qualify "$BIN_DIR/wider-carrier-qualify-1"
cp /root/build/target-wider-1/release/wider-carrier-run "$BIN_DIR/wider-carrier-run-1"
cp /root/build/target-wider-2/release/wider-carrier-qualify "$BIN_DIR/wider-carrier-qualify-2"
cp /root/build/target-wider-2/release/wider-carrier-run "$BIN_DIR/wider-carrier-run-2"
sha256sum "$BIN_DIR"/* | tee "$OUT/binaries.sha256"

echo "== Stage 3: G-Q finite CUDA qualification across both builds"
"$BIN_DIR/wider-carrier-qualify-1" "$OUT/qual-build1" || fail "G-Q build 1"
"$BIN_DIR/wider-carrier-qualify-2" "$OUT/qual-build2" || fail "G-Q build 2"
[ "$(jq -r '.status' "$OUT/qual-build1/wider-carrier-qualification.json")" = "PASS" ] || fail "G-Q build 1 status != PASS"
[ "$(jq -r '.status' "$OUT/qual-build2/wider-carrier-qualification.json")" = "PASS" ] || fail "G-Q build 2 status != PASS"
# Note: d11_exports[].artifact_sha256 embeds Snapshot::source_sha256, which folds in ptxas "Compile time" lines across separate target dirs (see run-antigravity.txt §4.C).
[ "$(jq -c 'del(.controls[3].detail.d11_exports[].artifact_sha256) | .controls' "$OUT/qual-build1/wider-carrier-qualification.json")" = "$(jq -c 'del(.controls[3].detail.d11_exports[].artifact_sha256) | .controls' "$OUT/qual-build2/wider-carrier-qualification.json")" ] || fail "G-Q controls differ between build 1 and build 2"
cp "$OUT/qual-build1/wider-carrier-qualification.json" "$OUT/wider-carrier-qualification.json"

run_arm() {
  local dir_name=$1 label=$2 cfg=$3
  echo "-- evaluating $label ($cfg) -> $OUT/arms/$dir_name"
  timeout 600 "$BIN_DIR/wider-carrier-run-1" arm "$label" "$cfg" "$OUT/arms/$dir_name" || fail "$label evaluation failed"
  local occ
  occ=$(jq -r '[.distinct_vectors, .occupied_cells, .within_cell_tie_tokens, .within_cell_tie_dev_rows, .dev_alias_ceiling_rows] | @tsv' "$OUT/arms/$dir_name/arm_summary.json")
  [ "$occ" = "$(printf '4096\t256\t0\t0\t14287')" ] || fail "G-Occ failed on $label: $occ"
}

echo "== Stage 4: G-Exact and G-D11 controls (A8-Outer, FV8-B4, WB6-Outer, WB8-Outer)"
run_arm a8-outer "A8-Outer" docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json
a8_nums=$(jq -r '[.full_admitted, .psucc_admitted, .top1, .top8] | @tsv' "$OUT/arms/a8-outer/arm_summary.json")
[ "$a8_nums" = "$(printf '4244\t1351\t133\t513')" ] || fail "G-Exact A8-Outer mismatch: $a8_nums"

run_arm fv8-b4-env "FV8-B4-Env" docs/labs/m1-address-key-full-vector/configs/fv8-env.json
fv_env_nums=$(jq -r '[.full_admitted, .psucc_admitted, .top1, .top8] | @tsv' "$OUT/arms/fv8-b4-env/arm_summary.json")
[ "$fv_env_nums" = "$(printf '5709\t1980\t118\t1126')" ] || fail "G-Exact FV8-B4-Env mismatch: $fv_env_nums"

run_arm fv8-b4-two-scale "FV8-B4-TwoScale" docs/labs/m1-address-key-full-vector/configs/fv8-two-scale.json
fv_ts_nums=$(jq -r '[.full_admitted, .psucc_admitted, .top1, .top8] | @tsv' "$OUT/arms/fv8-b4-two-scale/arm_summary.json")
[ "$fv_ts_nums" = "$(printf '5666\t1802\t118\t1179')" ] || fail "G-Exact FV8-B4-TwoScale mismatch: $fv_ts_nums"

run_arm wb6-outer "WB6-Outer" "$C/wb6-outer.json"
wb6_d11=$(jq -r '.d11_artifact_sha256 // empty' "$OUT/arms/wb6-outer/arm_summary.json")
[ "${#wb6_d11}" -eq 64 ] || fail "G-D11 WB6-Outer missing d11_artifact_sha256"

run_arm wb8-outer "WB8-Outer" "$C/wb8-outer.json"
wb8_d11=$(jq -r '.d11_artifact_sha256 // empty' "$OUT/arms/wb8-outer/arm_summary.json")
[ "${#wb8_d11}" -eq 64 ] || fail "G-D11 WB8-Outer missing d11_artifact_sha256"

echo "== Stage 5: wider-carrier B = 6 and B = 8 treatment and control arms"
run_arm wb6-env "WB6-Env" "$C/wb6-env.json"
run_arm wb6-two-scale "WB6-TwoScale" "$C/wb6-two-scale.json"
run_arm wbs6-env "WBS6-Env" "$C/wbs6-env.json"
run_arm wbs6-two-scale "WBS6-TwoScale" "$C/wbs6-two-scale.json"
run_arm wb8-env "WB8-Env" "$C/wb8-env.json"
run_arm wb8-two-scale "WB8-TwoScale" "$C/wb8-two-scale.json"
run_arm wbs8-env "WBS8-Env" "$C/wbs8-env.json"
run_arm wbs8-two-scale "WBS8-TwoScale" "$C/wbs8-two-scale.json"
run_arm fv8-b8-pad-env "FV8-B8-Pad-Env" "$C/fv8-b8-pad-env.json"
run_arm fv8-b8-pad-two-scale "FV8-B8-Pad-TwoScale" "$C/fv8-b8-pad-two-scale.json"

echo "== Stage 6: build-2 replication check on WB8-Env and WB8-TwoScale"
"$BIN_DIR/wider-carrier-run-2" arm "WB8-Env" "$C/wb8-env.json" "$OUT/replicate-build2/wb8-env" || fail "build 2 WB8-Env"
"$BIN_DIR/wider-carrier-run-2" arm "WB8-TwoScale" "$C/wb8-two-scale.json" "$OUT/replicate-build2/wb8-two-scale" || fail "build 2 WB8-TwoScale"
cmp -s "$OUT/arms/wb8-env/dev_b3.json" "$OUT/replicate-build2/wb8-env/dev_b3.json" || fail "build 1 vs build 2 WB8-Env dev_b3.json mismatch"
cmp -s "$OUT/arms/wb8-two-scale/dev_b3.json" "$OUT/replicate-build2/wb8-two-scale/dev_b3.json" || fail "build 1 vs build 2 WB8-TwoScale dev_b3.json mismatch"

echo "== Stage 7: deterministic grading and HF store archive"
"$BIN_DIR/wider-carrier-run-1" grade "$OUT/arms" "$OUT/wider-carrier-qualification.json" "$OUT/grade.json" || fail "wider-carrier-run grade"

(cd "$OUT" && find . -type f ! -name SHA256SUMS ! -name job.log -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
/root/uor-store put "$OUT" results/antigravity/m1-wider-code-carrier-2026-10-26/run || fail "uor-store put run"

echo 0 > "$OUT/EXIT"
echo "WIDER-CARRIER PRE-FLIGHT DONE"
