#!/bin/bash
# claude-112 (#2881): the prime-zeta zero-update pre-flight against six injective
# nulls, sheet docs/harness/prep/r-prime-zeta-vs-nulls-2026-10-25.md, Steps.
# Order is the decision card's (C): G-Rebuild, G-Init, G-Occ, then P8's G-Exact
# before any null is evaluated. Nothing trains. Each eval is capped at 10 minutes.
# Terminal markers: $OUT/EXIT (0 or 1), and $OUT/FAULT on any failure.
set -uo pipefail
SHA=d1eea395d64f1d95e753f6800bfaae9c69c2f212
SRC=/root/build/src-$SHA
OUT=/workspace/prime-zeta-nulls-2026-10-25
DATA=/workspace/uor-r4/local/driver/data
DL=/workspace/dl
# A rerun never reads an earlier run's markers: the earlier folder is kept, aside.
[ -e "$OUT" ] && mv "$OUT" "$OUT.prev-$(date +%s)"
mkdir -p "$OUT" "$DATA/injective" "$DL"
exec > >(tee -a "$OUT/job.log") 2>&1
# A fault saves what ran to the store too, before anyone brings the pod down.
fail() {
  echo "FAULT: $*" | tee "$OUT/FAULT"; echo 1 > "$OUT/EXIT"
  [ -x /root/uor-store ] && /root/uor-store put "$OUT" "results/claude/prime-zeta-nulls-2026-10-25/fault-$(date +%s)"
  exit 1
}
cp "$0" "$OUT/job.sh" || fail "job bytes not saved"
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
echo "== data (sheet pins)"
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
E=docs/labs/r-prime-zeta-vs-nulls/evidence
cat > "$OUT/configs.sha256" <<EOF
7927b1ddde3025d947ce56166c0a30e3ad8d310211aaab1df80851a4ae611bda  $E/p8.json
51986b8c0773a1a003c6a8cde2d63ea7068968beeff19f44e689f7fce66ec6d7  $E/q0.json
94c5634d71a25484c682cfe8606b976eaa33c22577432ecd2fc9e5160431973b  $E/q1.json
6f8b70dff8e692de6cdff9133de01f26c8897bd9d5b8b0034400cbd35512f945  $E/q2.json
6a3a457aae0e8f94b7a095f8e9a9e9081bd91a22f77b37e5ae67d8e99093ed1b  $E/r0.json
dc188404fd1eb2d5e94266dbdee5e5478deeef2a1be39d8826f8d85800fcdb94  $E/r1.json
9c78bfa93d60412f9d86d5f4c19830be8c79797e824458ae00a2c249ab04b144  $E/r2.json
EOF
sha256sum -c "$OUT/configs.sha256" || fail "config pins"
echo "== build (cuda)"
export CARGO_TARGET_DIR=/root/build/target-pz-nulls
cargo build --release --locked -p uor-r4-discrete --features cuda \
  --bin discrete-stack --bin coverage-census --bin prime-zeta-codebook || fail "build"
B=$CARGO_TARGET_DIR/release
ARMS="p8 q0 q1 q2 r0 r1 r2"
mode() { case $1 in p8) echo prime_zeta;; q*) echo "quota_only_injective:${1#q}";; r*) echo "random_injective:${1#r}";; esac; }
echo "== G-Rebuild: every table rebuilt on the pod equals its pin"
mkdir -p "$OUT/tables"
for a in $ARMS; do
  "$B/prime-zeta-codebook" "$E/spec-v4096-b4-c192-k8.json" 0 "$OUT/tables/$a.json" "$(mode $a)" > "$OUT/tables/$a.stdout" \
    || fail "$a prime-zeta-codebook"
  want=$(jq -r '.codebook_initialization.table.sha256' "$E/$a.json")
  got=$(sha256sum "$OUT/tables/$a.json" | cut -c1-64)
  echo "$a $(mode $a) $got"
  [ -n "$want" ] && [ "$got" = "$want" ] || fail "G-Rebuild: $a gives $got, not $want"
  cp "$OUT/tables/$a.json" "$(jq -r '.codebook_initialization.table.path' "$E/$a.json")" || fail "$a table copy"
done
echo "== init-corpus and G-Init, every arm"
for a in $ARMS; do
  mkdir -p "$OUT/arm-$a"
  "$B/discrete-stack" init-corpus "$E/$a.json" "$OUT/arm-$a/init" || fail "$a init-corpus"
  ck="$OUT/arm-$a/init/report.json"
  [ "$(jq -c '[.cursor.accepted_updates, .cursor.completed_batches]' "$ck")" = "[0,0]" ] || fail "G-Init: $a is not checkpoint 0"
  # Absent fields would compare equal as null: require them, and the table's exact size.
  jq -e '(.model.table | length) == 131072 and (.model.routes | type) == "array" and (.model.gates | type) == "array"' "$ck" > /dev/null \
    || fail "G-Init: $a checkpoint lacks a 4096x4x8 table, routes or gates"
  jq -e '(.table | length) == 131072' "$OUT/tables/$a.json" > /dev/null || fail "G-Init: $a rebuilt table is not 4096x4x8"
  [ "$(jq -c '.model.table' "$ck" | sha256sum)" = "$(jq -c '.table' "$OUT/tables/$a.json" | sha256sum)" ] \
    || fail "G-Init: $a checkpoint table differs from its pinned table"
  jq -c '[.model.routes, .model.gates]' "$ck" | sha256sum | cut -c1-64 > "$OUT/arm-$a/routes-gates.sha256"
done
[ "$(cat "$OUT"/arm-*/routes-gates.sha256 | sort -u | wc -l)" = 1 ] || fail "G-Init: routes or gates differ across arms"
echo "== G-Occ: every arm's census"
for a in $ARMS; do
  d="$OUT/census-$a"
  mkdir -p "$d"
  "$B/discrete-stack" attention-export-token-cells "$E/$a.json" "$OUT/arm-$a/init/report.json" "$d/cells" || fail "$a export"
  "$B/coverage-census" "$E/$a.json" "$OUT/arm-$a/init/report.json" "$d/census.json" \
    --token-cells "$d/cells/report.json" --bar-rows 500 || fail "$a census"
  occ=$(jq -r '[.distinct_vectors, .dev_alias_ceiling_rows, .within_cell_tie_mass.tokens_in_alias_groups, .within_cell_tie_mass.dev_rows_targeting_non_representative, .cells.occupied_cells] | @tsv' "$d/census.json")
  pairs=$(jq -r '.cells.cell_collision_pairs' "$d/census.json")
  maxc=$(jq -r '.cells.max_cell_occupancy' "$d/census.json")
  echo "G-Occ $a: $occ max $maxc pairs $pairs"
  [ "$occ" = "$(printf '4096\t14287\t0\t0\t256')" ] || fail "G-Occ: $a census gives $occ"
  case $a in
    p8|q*) [ "$maxc" = 16 ] && [ "$pairs" = 30720 ] || fail "G-Occ: $a max $maxc pairs $pairs";;
    r*) [ "$pairs" -gt 30720 ] || fail "G-Occ: $a pairs $pairs is not above 30720";;
  esac
done
arm() {  # name
  local a=$1 d="$OUT/arm-$1" start=$(date +%s)
  timeout 600 "$B/discrete-stack" eval-corpus "$E/$a.json" "$d/init/report.json" "$d/eval" || fail "$a eval-corpus (or past 10 minutes)"
  echo "$a eval seconds $(( $(date +%s) - start ))"
  "$B/discrete-stack" attention-dev-b3 "$E/$a.json" "$d/init/report.json" "$d/eval/report.json" "$d/devb3" || fail "$a dev-b3"
}
echo "== G-Exact: P8 first"
arm p8
exact="$(jq -r '[.b3.full.target_admitted, .b3.psucc.target_admitted] | @tsv' "$OUT/arm-p8/devb3/report.json") $(jq -r '.aggregate.top1_correct' "$OUT/arm-p8/eval/report.json")"
echo "G-Exact read: $exact"
[ "$exact" = "$(printf '427\t127') 38" ] || fail "G-Exact: P8 gives $exact, not 427 127 38"
echo "== nulls (every instrument check passed)"
for a in q0 q1 q2 r0 r1 r2; do arm "$a"; done
{
  echo "{"
  for a in $ARMS; do
    sep=,; [ "$a" = r2 ] && sep=
    echo " \"$a\": {\"dev_b3\": \"arm-$a/devb3/report.json\", \"eval\": \"arm-$a/eval/report.json\", \"census\": \"census-$a/census.json\"}$sep"
  done
  echo "}"
} > "$OUT/manifest.json"
jq -e . "$OUT/manifest.json" > /dev/null || fail "manifest is not JSON"
(cd "$OUT" && find . -type f ! -name SHA256SUMS ! -name job.log -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
# Saved off the pod by the job itself, before anyone brings the pod down.
/root/uor-store put "$OUT" results/claude/prime-zeta-nulls-2026-10-25/run || fail "store put run"
echo 0 > "$OUT/EXIT"
echo "PRE-FLIGHT DONE"
