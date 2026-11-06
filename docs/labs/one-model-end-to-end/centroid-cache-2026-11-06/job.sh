#!/usr/bin/env bash
# Capture one n3S centroid-query cache on a bootstrapped UOR-R4 Vast pod.
#
# The reviewed script is copied to the pod separately from the pinned source checkout:
#   bash /workspace/centroid-cache-job.sh SEED
#
# The caller owns the pod lease, GPU assignment and teardown. This script makes
# no pod or store mutation other than uploading its completed evidence.
set -euo pipefail

SEED=${1:?usage: job.sh SEED}
case $SEED in
  1|2) ;;
  *) echo "SEED must be 1 or 2" >&2; exit 2 ;;
esac

PIN=3a9c2f11d26991a70a22f88758cfb4589a1cf14d
REPO=${UOR_REPO:-/root/build/src-$PIN}
RUN=${UOR_CENTROID_RUN:-/workspace/uor-r4/runs/deepseek-centroid-cache-s$SEED}
RUN_ID=${UOR_CENTROID_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')}
STORE=/root/uor-store
RUN_STORE=results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s$SEED
ARCHIVE=steps-data-s$SEED.tar.gz
OUT_STORE=results/deepseek/one-model-end-to-end-centroid-2026-11-06/s$SEED-$RUN_ID
MIN_FREE_GIB=${UOR_CENTROID_MIN_FREE_GIB:-128}

die() {
  if [ -d "$RUN" ]; then
    printf 'centroid-cache: %s\n' "$*" >&2
  else
    echo "centroid-cache: $*" >&2
  fi
  exit 1
}
check_file() {
  local want=$1 path=$2
  local got
  got=$(sha256sum "$path" | cut -d' ' -f1)
  [ "$got" = "$want" ] || die "$path hashes to $got, not $want"
}
need_free_gib() {
  local want=$1 available
  available=$(df -Pk /workspace | awk 'NR==2 {print int($4 / 1048576)}')
  [ "$available" -ge "$want" ] ||
    die "/workspace has ${available} GiB free, below ${want} GiB"
}
need_readback_room() {
  local cache_archive=$1 normal_archive=$2 archive_kib available_kib required_kib
  archive_kib=$((($(stat -c %s "$cache_archive") +
    $(stat -c %s "$normal_archive") + 1023) / 1024))
  available_kib=$(df -Pk /workspace | awk 'NR==2 {print $4}')
  required_kib=$((archive_kib + 31 * 1048576))
  [ "$required_kib" -lt $((40 * 1048576)) ] &&
    required_kib=$((40 * 1048576))
  [ "$available_kib" -ge "$required_kib" ] ||
    die "/workspace has $((available_kib / 1048576)) GiB free; readback needs $((required_kib / 1048576)) GiB (archive plus reserve)"
}
log() {
  printf '%s\n' "$*"
}

case $RUN_ID in
  ''|*..*|*[!A-Za-z0-9._-]*) die "RUN_ID contains unsafe characters: $RUN_ID" ;;
esac
case $RUN in
  /workspace/*) ;;
  *) die "UOR_CENTROID_RUN must stay under /workspace: $RUN" ;;
esac
[ ! -e "$RUN" ] || die "refusing an existing run directory: $RUN"
install -d "$RUN/download" "$RUN/seed" "$RUN/out" "$RUN/results"
exec >> "$RUN/job.log" 2>&1
OUTPUT_CLAIMED=0
on_exit() {
  local rc=$?
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$rc" > "$RUN/exit.code"
    failure_complete=1
    if [ "$OUTPUT_CLAIMED" = 1 ] && [ -x "$STORE" ]; then
      for file in job.log transfer.log store-list.err capture.stderr capture.stdout.jsonl \
        binaries.sha256 gpu.txt selected-checkpoint.sha256 status.json \
        CACHE_TRANSFER_VERIFIED results/cache.tar.zst.sha256 \
        results/cache.SHA256SUMS results/normal-outputs.tar.zst.sha256 exit.code; do
        [ -f "$RUN/$file" ] || continue
        destination=$file
        [ "$file" = exit.code ] && destination=EXIT
        if ! timeout 20s "$STORE" put "$RUN/$file" "$OUT_STORE/failure/$destination" \
            >/dev/null 2>&1 ||
          ! timeout 20s "$STORE" get "$OUT_STORE/failure/$destination" \
            "$RUN/failure-readback" >/dev/null 2>&1 ||
          ! cmp -s "$RUN/$file" "$RUN/failure-readback/$OUT_STORE/failure/$destination"; then
          failure_complete=0
        fi
      done
    else
      failure_complete=0
    fi
    if [ "$failure_complete" != 1 ]; then
      printf 'failure diagnostics were not fully uploaded and read back\n' \
        > "$RUN/FAILURE_EVIDENCE_INCOMPLETE"
    fi
  fi
  printf '%s\n' "$rc" > "$RUN/EXIT"
}
trap on_exit EXIT

case $MIN_FREE_GIB in ''|*[!0-9]*) die "MIN_FREE_GIB must be an integer" ;; esac
[ "$MIN_FREE_GIB" -ge 128 ] || die "MIN_FREE_GIB may not be below 128"
command -v timeout >/dev/null || die "timeout is not installed"
[ -x "$STORE" ] || die "uor-store is not installed"
store_listing=$("$STORE" ls results/deepseek/ 2>"$RUN/store-list.err") ||
  die "could not list the DeepSeek store namespace"
if grep -F "  $OUT_STORE/" <<<"$store_listing" >/dev/null; then
  die "refusing an existing store output: $OUT_STORE"
fi
OUTPUT_CLAIMED=1

if [ ! -d "$REPO/.git" ]; then
  command -v git >/dev/null || die "git is not installed and repository $REPO is absent"
  REPO="$RUN/source-$PIN"
  install -d "$REPO"
  git -C "$REPO" init --quiet
  git -C "$REPO" remote add origin https://github.com/UOR-Foundation/uor-r4.git
  git -C "$REPO" fetch --quiet --depth 1 origin "$PIN" ||
    die "could not fetch source pin $PIN"
  git -C "$REPO" checkout --quiet --detach FETCH_HEAD ||
    die "could not check out source pin $PIN"
  log "source checkout created at $REPO"
fi

[ -d "$REPO/.git" ] || die "missing repository at $REPO"
command -v zstd >/dev/null || die "zstd is not installed"
command -v jq >/dev/null || die "jq is not installed"
command -v /usr/bin/time >/dev/null || die "/usr/bin/time is not installed"
command -v nvidia-smi >/dev/null || die "nvidia-smi is not installed"
printf '' | zstd -q | zstd -dq >/dev/null || die "zstd round-trip preflight failed"
need_free_gib "$MIN_FREE_GIB"

actual=$(git -C "$REPO" rev-parse HEAD)
[ "$actual" = "$PIN" ] || die "repository is $actual, expected $PIN"
[ -z "$(git -C "$REPO" status --porcelain --untracked-files=all)" ] ||
  die "repository tree is not clean"

log "[0/6] run=$RUN output=$OUT_STORE source=$PIN"
log "script_sha256=$(sha256sum "$0" | cut -d' ' -f1)"
log "[1/6] fetch pinned inputs"
"$STORE" get "results/claude/scale-probe-2026-11-05/inputs/d3/" "$RUN/download"
for panel in oasst2 wiki-00000 wiki-00020 wiki-00040; do
  "$STORE" get "results/claude/real-text-corpus-2026-11-05/$panel/heldout/" "$RUN/download"
done
"$STORE" get "w2-inputs/2026-10-25/driver-data/" "$RUN/download"
"$STORE" get "$RUN_STORE/$ARCHIVE" "$RUN/download"

DATA=$RUN/download/w2-inputs/2026-10-25/driver-data
D3=$RUN/download/results/claude/scale-probe-2026-11-05/inputs/d3
PANELS=$RUN/download/results/claude/real-text-corpus-2026-11-05
ARCHIVE_PATH=$RUN/download/$RUN_STORE/$ARCHIVE

check_file d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89 "$DATA/tokenizer.json"
check_file 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b "$DATA/ft/dev/tokens.u16"
check_file a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 "$DATA/ft/dev/response_mask.u8"
check_file ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52 "$D3/tokens.u16"
check_file ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875 "$D3/response_mask.u8"
check_file 02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e "$PANELS/oasst2/heldout/tokens.u16"
check_file 9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833 "$PANELS/oasst2/heldout/response_mask.u8"
check_file 82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7 "$PANELS/wiki-00000/heldout/tokens.u16"
check_file 34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f "$PANELS/wiki-00000/heldout/response_mask.u8"
check_file 9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640 "$PANELS/wiki-00020/heldout/tokens.u16"
check_file a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d "$PANELS/wiki-00020/heldout/response_mask.u8"
check_file c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d "$PANELS/wiki-00040/heldout/tokens.u16"
check_file 9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625 "$PANELS/wiki-00040/heldout/response_mask.u8"

case $SEED in
  1) archive_sha=1c7a09781c60475067b4fc2319db92a16ef0f589dcf355db1d09f35edcc4ec69 ;;
  2) archive_sha=dd3cc8885cf1b35aa840da9f7d029d8a8dc6097a8321d963e4abdfcc8ce5c467 ;;
esac
check_file "$archive_sha" "$ARCHIVE_PATH"

log "[2/6] extract and verify n3S checkpoint"
runcase=n3S-s$SEED
tar -xzf "$ARCHIVE_PATH" -C "$RUN/seed" \
  "oh/run/$runcase/config.json" \
  "oh/run/$runcase/models/step-008192/model.safetensors" \
  "oh/run/$runcase/models/step-008192/config.json" \
  "oh/run/$runcase/models/step-008192/hamming_read.safetensors" \
  "oh/run/SHA256SUMS.files"

case $SEED in
  1)
    check_file 30e9113c05f266a11df6e2baa302e7a569280efc0d0d7b1bfb80b06245a71631 "$RUN/seed/oh/run/$runcase/config.json"
    check_file 1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2 "$RUN/seed/oh/run/$runcase/models/step-008192/config.json"
    check_file c64894f3648e2663bfe1f323836c9fc22c6989c9a6f2917e8ce1f25c6a686fb2 "$RUN/seed/oh/run/$runcase/models/step-008192/model.safetensors"
    check_file e2347c4ef266c2166e870926a44e740edbd5c27065ed6a0ccda5ab11da5998b5 "$RUN/seed/oh/run/$runcase/models/step-008192/hamming_read.safetensors"
    ;;
  2)
    check_file 3a3f9e0d3b428625270e6393e42a2444e14f61912e81a7499c96228329c74ca2 "$RUN/seed/oh/run/$runcase/config.json"
    check_file 778145167a18ff3b1006d4f609b7f608cc528b0dd753279b3ff38c8c63a88b95 "$RUN/seed/oh/run/$runcase/models/step-008192/config.json"
    check_file 30bf5db7db6600211ee14a17557bbf54f24c86de8b7c16f93b795085f0bfc9e8 "$RUN/seed/oh/run/$runcase/models/step-008192/model.safetensors"
    check_file 25805079be3bcfe5917cf0b80a362e7a774aac931c5f4cdaf5351c272684a5bd "$RUN/seed/oh/run/$runcase/models/step-008192/hamming_read.safetensors"
    ;;
esac
selected="$RUN/selected-checkpoint.sha256"
grep -E "  \\./($runcase/(config\\.json|models/step-008192/(config\\.json|model\\.safetensors|hamming_read\\.safetensors)))$" \
  "$RUN/seed/oh/run/SHA256SUMS.files" > "$selected"
[ "$(wc -l < "$selected")" -eq 4 ] || die "checkpoint SHA manifest did not select four files"
(cd "$RUN/seed/oh/run" && sha256sum -c "$selected")

log "[3/6] build and test pinned CUDA scorer"
export CARGO_TARGET_DIR="$RUN/target"
(cd "$REPO" && cargo test --locked --offline -p uor-r4-float-train --features cuda --lib)
(cd "$REPO" && cargo build --release --locked --offline -p uor-r4-float-train --features cuda \
  --bin train-in-space)
sha256sum "$CARGO_TARGET_DIR/release/train-in-space" > "$RUN/binaries.sha256"
need_free_gib 118

log "[4/6] capture TRAIN and panel queries"
VISIBLE_GPU=${CUDA_VISIBLE_DEVICES:-0}
nvidia-smi --id="$VISIBLE_GPU" --query-gpu=name,driver_version,memory.total,memory.used \
  --format=csv > "$RUN/gpu.txt"
gpu_name=$(nvidia-smi --id="$VISIBLE_GPU" --query-gpu=name --format=csv,noheader | sed -n '1p')
case $gpu_name in
  *"RTX 5090"*) ;;
  *) die "expected a visible RTX 5090, found: $gpu_name" ;;
esac
started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
set +e
/usr/bin/time -v "$CARGO_TARGET_DIR/release/train-in-space" \
  --data-root "$DATA" \
  --score-rows "$RUN/seed/oh/run/$runcase/models/step-008192" \
  --out "$RUN/out/score" \
  --train-split "$D3" \
  --train-sha256 ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52 \
  --train-mask-sha256 ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875 \
  --panel "oasst2=$PANELS/oasst2/heldout" \
  --panel "wiki-00000=$PANELS/wiki-00000/heldout" \
  --panel "wiki-00020=$PANELS/wiki-00020/heldout" \
  --panel "wiki-00040=$PANELS/wiki-00040/heldout" \
  --samples 0 \
  --retain-rates 1 \
  --fold-scores 0 \
  --reflect-floor 0 \
  --seed "$SEED" \
  --context 192 \
  --served-forward 1 \
  --query-cache-out "$RUN/results/cache" \
  --query-cache-memory 0 \
  --query-cache-dev 0 \
  > "$RUN/capture.stdout.jsonl" \
  2> "$RUN/capture.stderr"
status=$?
set -e
finished=$(date -u +%Y-%m-%dT%H:%M:%SZ)
[ "$status" = 0 ] || die "capture exited $status"
need_free_gib 80

log "[5/6] seal cache"
[ -f "$RUN/results/cache/manifest.json" ] || die "capture wrote no sealed cache manifest"
jq -e '
  .schema == "uor-r4.fbs-query-cache/1" and
  .width == 256 and
  .query_space == "q12-f32" and
  .tokenizer_sha256 == "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89" and
  (.checkpoint_files | keys | sort) == ["config.json", "hamming_read.safetensors", "model.safetensors"] and
  (.splits | length) == 5 and
  all(.splits[]; .memory == false and .label == "no-memory" and .context == 192) and
  ([.splits[].split] | sort) == ["oasst2", "train", "wiki-00000", "wiki-00020", "wiki-00040"] and
  any(.splits[]; .split == "train" and .rows == 27416554) and
  ([.splits[] | select(.split != "train") | .rows] | add) == 1460697 and
  any(.splits[];
    .split == "train" and
    .tokens_sha256 == "ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52" and
    .mask_sha256 == "ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875") and
  any(.splits[];
    .split == "oasst2" and
    .tokens_sha256 == "02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e" and
    .mask_sha256 == "9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833") and
  any(.splits[];
    .split == "wiki-00000" and
    .tokens_sha256 == "82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7" and
    .mask_sha256 == "34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f") and
  any(.splits[];
    .split == "wiki-00020" and
    .tokens_sha256 == "9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640" and
    .mask_sha256 == "a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d") and
  any(.splits[];
    .split == "wiki-00040" and
    .tokens_sha256 == "c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d" and
    .mask_sha256 == "9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625")
' "$RUN/results/cache/manifest.json" >/dev/null ||
  die "cache manifest does not contain the registered five-split inventory"
tar -I 'zstd -3 -T0' -cf "$RUN/results/normal-outputs.tar.zst" -C "$RUN" out/score
zstd -q -t "$RUN/results/normal-outputs.tar.zst"
(cd "$RUN/results" && sha256sum normal-outputs.tar.zst) \
  > "$RUN/results/normal-outputs.tar.zst.sha256"
(cd "$RUN/results" && find cache -type f -print0 | sort -z | xargs -0 sha256sum) \
  > "$RUN/results/cache.SHA256SUMS"
tar -I 'zstd -3 -T0' -cf "$RUN/results/cache.tar.zst" -C "$RUN/results" cache
zstd -q -t "$RUN/results/cache.tar.zst"
(cd "$RUN/results" && sha256sum cache.tar.zst) \
  > "$RUN/results/cache.tar.zst.sha256"
need_readback_room "$RUN/results/cache.tar.zst" "$RUN/results/normal-outputs.tar.zst"

log "[6/6] upload and read back sealed evidence"
for file in results/cache.tar.zst results/cache.tar.zst.sha256 results/cache.SHA256SUMS \
  results/normal-outputs.tar.zst results/normal-outputs.tar.zst.sha256 \
  capture.stderr capture.stdout.jsonl binaries.sha256 gpu.txt selected-checkpoint.sha256; do
  "$STORE" put "$RUN/$file" "$OUT_STORE/$file" >> "$RUN/transfer.log" 2>&1
done
for file in results/cache.tar.zst.sha256 results/cache.SHA256SUMS capture.stderr \
  capture.stdout.jsonl results/normal-outputs.tar.zst \
  results/normal-outputs.tar.zst.sha256 binaries.sha256 gpu.txt \
  selected-checkpoint.sha256; do
  "$STORE" get "$OUT_STORE/$file" "$RUN/readback" >> "$RUN/transfer.log" 2>&1
  cmp "$RUN/$file" "$RUN/readback/$OUT_STORE/$file" ||
    die "uploaded $file did not read back"
done
"$STORE" get "$OUT_STORE/results/cache.tar.zst" "$RUN/readback" >> "$RUN/transfer.log" 2>&1
readback_sha=$(sha256sum "$RUN/readback/$OUT_STORE/results/cache.tar.zst" | cut -d' ' -f1)
[ "$readback_sha" = "$(cut -d' ' -f1 "$RUN/results/cache.tar.zst.sha256")" ] ||
  die "uploaded cache archive did not read back"
touch "$RUN/CACHE_TRANSFER_VERIFIED"

normal_sha=$(cut -d' ' -f1 "$RUN/results/normal-outputs.tar.zst.sha256")
printf '{"seed":%s,"run_id":"%s","status":"CAPTURED","source_pin":"%s","started":"%s","finished":"%s","cache_manifest_sha256":"%s","cache_tar_sha256":"%s","normal_outputs_tar_sha256":"%s","capture_binary_sha256":"%s"}\n' \
  "$SEED" "$RUN_ID" "$PIN" "$started" "$finished" \
  "$(sha256sum "$RUN/results/cache/manifest.json" | cut -d' ' -f1)" \
  "$(cut -d' ' -f1 "$RUN/results/cache.tar.zst.sha256")" \
  "$normal_sha" \
  "$(cut -d' ' -f1 "$RUN/binaries.sha256")" \
  > "$RUN/status.json"
for file in status.json job.log; do
  "$STORE" put "$RUN/$file" "$OUT_STORE/$file" >> "$RUN/transfer.log" 2>&1
  "$STORE" get "$OUT_STORE/$file" "$RUN/readback" >> "$RUN/transfer.log" 2>&1
  cmp "$RUN/$file" "$RUN/readback/$OUT_STORE/$file" ||
    die "uploaded $file did not read back"
done
"$STORE" put "$RUN/transfer.log" "$OUT_STORE/store.log" >/dev/null
"$STORE" get "$OUT_STORE/store.log" "$RUN/readback" >/dev/null
cmp "$RUN/transfer.log" "$RUN/readback/$OUT_STORE/store.log" ||
  die "uploaded transfer log did not read back"
