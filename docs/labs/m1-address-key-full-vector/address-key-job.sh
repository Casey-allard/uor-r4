#!/bin/bash
# claude-110 (#2834): the address-metric zero-update pre-flight, sheet
# docs/harness/prep/m1-address-key-full-vector-2026-10-25.md, Steps 4-5.
# Order is the sheet's: pins, G-Q, then the arms; A8-Outer's G-Exact numbers are
# read by the grader before any treatment. Each eval is capped at 10 minutes.
# Terminal markers: $OUT/EXIT (0 or 1), and $OUT/FAULT on any failure.
set -uo pipefail
SHA=77123dd995be44d5b0a5f482c8b82e270dd9aa42
SRC=/root/build/src-$SHA
OUT=/workspace/address-key-2026-10-25
DATA=/workspace/uor-r4/local/driver/data
DL=/workspace/dl
mkdir -p "$OUT" "$DATA/address-cells" "$DL"
exec > >(tee -a "$OUT/job.log") 2>&1
cp "$0" "$OUT/job.sh"
fail() { echo "FAULT: $*" | tee "$OUT/FAULT"; echo 1 > "$OUT/EXIT"; exit 1; }
free_gib=$(df -BG --output=avail /workspace | tail -1 | tr -dc '0-9')
[ "${free_gib:-0}" -ge 30 ] || fail "free disk ${free_gib} GiB is under 30"
export PATH=/usr/local/cuda/bin:$HOME/.cargo/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export UOR_NVCC_ARCH=compute_120
if [ ! -d "$SRC/.git" ]; then
  git init -q "$SRC" || fail "git init"
  git -C "$SRC" fetch -q --depth 1 https://github.com/UOR-Foundation/uor-r4.git "$SHA" || fail "fetch $SHA"
  git -C "$SRC" checkout -q FETCH_HEAD || fail "checkout"
fi
[ "$(git -C "$SRC" rev-parse HEAD)" = "$SHA" ] || fail "source is not $SHA"
cd "$SRC" || fail "no source"
nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv > "$OUT/gpu.csv" || fail "nvidia-smi"
grep -q "RTX 5090" "$OUT/gpu.csv" || fail "not an RTX 5090 (UOR_NVCC_ARCH=compute_120 assumes one)"
[ -x /root/uor-store ] || fail "no /root/uor-store on this pod"
echo "== data and tables (sheet pins)"
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
/root/uor-store get results/claude/address-key-2026-10-25/tables/ "$DL" || fail "store get tables"
T="$DL/results/claude/address-key-2026-10-25/tables"
(cd "$T" && sha256sum -c SHA256SUMS) || fail "tables SHA256SUMS"
cp "$T/structured.json" "$DATA/address-cells/address-structured-v1.json"
cp "$T/full-vector.json" "$DATA/address-cells/address-full-vector-v1.json"
cp "$T/full-vector-shuffled-20261019.json" "$DATA/address-cells/address-full-vector-shuffled-v1-seed-20261019.json"
cat > "$OUT/tables.sha256" <<EOF
6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1  $DATA/address-cells/address-structured-v1.json
2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2  $DATA/address-cells/address-full-vector-v1.json
5e5abf336bfb039d6c3bde336cf5d03f207ae9ff7bc8cfb21e71691c622313c4  $DATA/address-cells/address-full-vector-shuffled-v1-seed-20261019.json
EOF
sha256sum -c "$OUT/tables.sha256" || fail "table pins"
C=docs/labs/m1-address-key-full-vector/configs
cat > "$OUT/configs.sha256" <<EOF
469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e  docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json
56c3fafe0b8cdb7800f57d0a6ba577b2964356de5765b350a701546cace6e534  $C/fv8-env.json
1cdb80615d1467bbe062bc1a70804630b3f69af5c580ce3b0d2b5d7407a89a28  $C/fv8-two-scale.json
b3b5fb60a904eaecb17d36f3ff5263ff4f83d6846612380df5ce8a5aef20830e  $C/fvs8-env.json
05d907d0d6e2d3526642b32ead7edf7a9db422c02334ff3ed9aa03939950c834  $C/fvs8-two-scale.json
1590e52d9d663ef27725bdeec55be7e3968c65060becd5bd02191e24fd2220a5  $C/fv16-two-scale.json
EOF
sha256sum -c "$OUT/configs.sha256" || fail "config pins"
echo "== build (cuda)"
export CARGO_TARGET_DIR=/root/build/target-address-key
cargo build --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack --bin coverage-census || fail "build"
B=$CARGO_TARGET_DIR/release
echo "== G-Q: the finite CUDA control"
cargo test --release --locked -p uor-r4-discrete --features cuda --lib address_metric_device_tests \
  -- --ignored --test-threads=1 > "$OUT/g-q.log" 2>&1 || fail "G-Q control"
grep -q "device_admission_matches_the_host_oracles_for_each_metric ... ok" "$OUT/g-q.log" || fail "G-Q not run"
init() {  # name config
  mkdir -p "$OUT/arm-$1"
  "$B/discrete-stack" init-corpus "$2" "$OUT/arm-$1/init" || fail "$1 init-corpus"
}
arm() {  # name config dev_b3(yes|no)
  local name=$1 config=$2 b3=$3 d="$OUT/arm-$1"
  local start=$(date +%s)
  timeout 600 "$B/discrete-stack" eval-corpus "$config" "$d/init/report.json" "$d/eval" || fail "$name eval-corpus (or past 10 minutes)"
  echo "$name eval seconds $(( $(date +%s) - start ))"
  if [ "$b3" = yes ]; then
    "$B/discrete-stack" attention-dev-b3 "$config" "$d/init/report.json" "$d/eval/report.json" "$d/devb3" || fail "$name dev-b3"
  fi
}
census() {  # name config
  local name=$1 config=$2 d="$OUT/census-$1"
  mkdir -p "$d"
  "$B/discrete-stack" attention-export-token-cells "$config" "$OUT/arm-$name/init/report.json" "$d/cells" || fail "$name export"
  "$B/coverage-census" "$config" "$OUT/arm-$name/init/report.json" "$d/census.json" \
    --token-cells "$d/cells/report.json" --bar-rows 1941 || fail "$name census"
}
A8=docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json
echo "== init-corpus, every arm"
init a8-outer "$A8"
for a in fv8-env fv8-two-scale fvs8-env fvs8-two-scale fv16-two-scale; do init "$a" "$C/$a.json"; done
echo "== G-Exact: A8-Outer first"
arm a8-outer "$A8" yes
R="$OUT/arm-a8-outer"
exact="$(jq -r '[.b3.full.target_admitted, .b3.psucc.target_admitted] | @tsv' "$R/devb3/report.json") $(jq -r '.aggregate.top1_correct' "$R/eval/report.json")"
echo "G-Exact read: $exact"
[ "$exact" = "$(printf '4244\t1351') 133" ] || fail "G-Exact: A8-Outer gives $exact, not 4244 1351 133"
echo "== G-Occ: both pinned tables"
census fv8-two-scale "$C/fv8-two-scale.json"
census fvs8-two-scale "$C/fvs8-two-scale.json"
for t in fv8-two-scale fvs8-two-scale; do
  occ=$(jq -r '[.distinct_vectors, .cells.occupied_cells, .within_cell_tie_mass.tokens_in_alias_groups, .within_cell_tie_mass.dev_rows_targeting_non_representative, .dev_alias_ceiling_rows] | @tsv' "$OUT/census-$t/census.json")
  echo "G-Occ $t: $occ"
  [ "$occ" = "$(printf '4096\t256\t0\t0\t14287')" ] || fail "G-Occ: $t census gives $occ"
done
echo "== treatments (every instrument check passed)"
arm fv8-env "$C/fv8-env.json" yes
arm fv8-two-scale "$C/fv8-two-scale.json" yes
arm fvs8-env "$C/fvs8-env.json" yes
arm fvs8-two-scale "$C/fvs8-two-scale.json" yes
arm fv16-two-scale "$C/fv16-two-scale.json" no
cat > "$OUT/manifest.json" <<EOF
{"g_q": "g-q.log",
 "a8": {"dev_b3": "arm-a8-outer/devb3/report.json", "eval": "arm-a8-outer/eval/report.json"},
 "fv8_env": {"dev_b3": "arm-fv8-env/devb3/report.json", "eval": "arm-fv8-env/eval/report.json"},
 "fv8_two_scale": {"dev_b3": "arm-fv8-two-scale/devb3/report.json", "eval": "arm-fv8-two-scale/eval/report.json"},
 "fvs8_env": {"dev_b3": "arm-fvs8-env/devb3/report.json", "eval": "arm-fvs8-env/eval/report.json"},
 "fvs8_two_scale": {"dev_b3": "arm-fvs8-two-scale/devb3/report.json", "eval": "arm-fvs8-two-scale/eval/report.json"},
 "fv_census": "census-fv8-two-scale/census.json", "fvs_census": "census-fvs8-two-scale/census.json"}
EOF
(cd "$OUT" && find . -type f ! -name SHA256SUMS ! -name job.log -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
# Saved off the pod by the job itself, before anyone brings the pod down.
/root/uor-store put "$OUT" results/claude/address-key-2026-10-25/run || fail "store put run"
echo 0 > "$OUT/EXIT"
echo "PRE-FLIGHT DONE"
