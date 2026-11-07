#!/usr/bin/env bash
# shellcheck disable=SC2016
# Capture one reviewed checkpoint's centroid-query cache on a bootstrapped UOR-R4 Vast pod.
#
# The reviewed script and input specification are copied to the pod separately from the
# pinned source checkout:
#   UOR_CENTROID_SPEC=/workspace/centroid-checkpoint-inputs.json \
#     bash /workspace/centroid-cache-job.sh SEED
#
# The caller owns the pod lease, GPU assignment and teardown. This script makes
# no pod or store mutation other than uploading its completed evidence.
set -euo pipefail

SEED=${1:?usage: job.sh SEED}
case $SEED in
  1|2) ;;
  *) echo "SEED must be 1 or 2" >&2; exit 2 ;;
esac
[ $# -le 2 ] || { echo "usage: job.sh SEED [--check-spec]" >&2; exit 2; }
case ${2:-} in
  ''|--check-spec) ;;
  *) echo "unknown argument: $2" >&2; exit 2 ;;
esac

RUN=${UOR_CENTROID_RUN:-/workspace/uor-r4/runs/deepseek-centroid-cache-s$SEED}
RUN_ID=${UOR_CENTROID_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')}
STORE=/root/uor-store
OUT_STORE=results/deepseek/one-model-end-to-end-centroid-2026-11-06/s$SEED-$RUN_ID
MIN_FREE_GIB=${UOR_CENTROID_MIN_FREE_GIB:-128}
SPEC_INPUT=${UOR_CENTROID_SPEC:-}

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
check_cache_archive() {
  local archive=$1 members=$2 member relative
  tar --zstd -tf "$archive" > "$members"
  [ -s "$members" ] || die "cache archive is empty"
  while IFS= read -r member; do
    relative=${member%/}
    case "/$relative/" in
      *//*|*/../*) die "cache archive path is not safely relative: $member" ;;
    esac
    case $relative in
      cache|cache/*) ;;
      *) die "cache archive path is outside cache/: $member" ;;
    esac
  done < "$members"
  tar --zstd -tvf "$archive" |
    awk 'substr($0, 1, 1) != "-" && substr($0, 1, 1) != "d" {
      print "cache archive contains a non-regular entry: " $0 > "/dev/stderr";
      bad = 1
    } END { exit bad }' ||
    die "cache archive contains a link or special file"
}
log() {
  printf '%s\n' "$*"
}
spec_get() {
  jq -er --arg seed "$SEED" "$1" "$SPEC"
}

case $RUN_ID in
  ''|*..*|*[!A-Za-z0-9._-]*) die "RUN_ID contains unsafe characters: $RUN_ID" ;;
esac
case $RUN in
  /workspace/*) ;;
  *) die "UOR_CENTROID_RUN must stay under /workspace: $RUN" ;;
esac
[ -n "$SPEC_INPUT" ] || die "UOR_CENTROID_SPEC must name the reviewed input specification"
[ -f "$SPEC_INPUT" ] || die "input specification does not exist: $SPEC_INPUT"
jq -e --arg seed "$SEED" '
  .schema == "uor-r4.centroid-checkpoint-inputs/1" and
  (.source_pin | test("^[0-9a-f]{40}$")) and
  .evaluation.profile == "n3s-registered-v1" and
  .evaluation.profile_role == "frozen-evaluation-protocol" and
  (.evaluation.target_id | test("^[A-Za-z0-9._-]+$")) and
  (.expected.run.objective | type == "string") and
  (.expected.run.train_as_served | type == "boolean") and
  (.expected.run.width | type == "number" and . > 0 and floor == .) and
  (.expected.run.hamming_read | type == "number" and . >= 0 and floor == .) and
  (.expected.run.batch | type == "number" and . > 0 and floor == .) and
  (.expected.run.micro_batch | type == "number" and . > 0 and floor == .) and
  (.expected.run.steps | type == "number" and . > 0 and floor == .) and
  (.expected.model.arch | type == "string") and
  (.expected.model.gate | type == "string") and
  (.expected.model.head | type == "string") and
  (.expected.model.pattern | type == "string") and
  (.expected.model.rotation | type == "boolean") and
  (.expected.model.width | type == "number" and . > 0 and floor == .) and
  (.expected.model.context | type == "number" and . > 1 and floor == .) and
  (.expected.model.heads | type == "number" and . > 0 and floor == .) and
  (.expected.run.width == .expected.model.width) and
  ((.expected.run.batch % .expected.run.micro_batch) == 0) and
  (.seeds[$seed] as $s |
    ($s.run_store | type == "string" and
      test("^results/claude/[A-Za-z0-9._/-]+$") and (contains("..") | not)) and
    ($s.archive | test("^[A-Za-z0-9._-]+$")) and
    ($s.archive_sha256 | test("^[0-9a-f]{64}$")) and
    ($s.run_case | test("^[A-Za-z0-9._-]+$")) and
    ($s.checkpoint | test("^oh/run/[A-Za-z0-9._/-]+$") and
      (contains("..") | not)) and
    ($s.checkpoint_manifest | test("^oh/run/[A-Za-z0-9._/-]+$") and
      (contains("..") | not)) and
    ($s.run_config.sha256 | test("^[0-9a-f]{64}$")) and
    ($s.run_config.path | test("^oh/run/[A-Za-z0-9._/-]+$") and
      (contains("..") | not)) and
    ($s.checkpoint_files | keys | sort) ==
      ["config.json", "hamming_read.safetensors", "model.safetensors"] and
    all($s.checkpoint_files[];
      (.sha256 | test("^[0-9a-f]{64}$")) and
      (.path | test("^oh/run/[A-Za-z0-9._/-]+$") and (contains("..") | not))))
' "$SPEC_INPUT" >/dev/null || die "input specification is not the registered schema"
if [ "${2:-}" = --check-spec ]; then
  printf 'checkpoint-inputs: seed %s is valid\n' "$SEED"
  exit 0
fi
[ ! -e "$RUN" ] || die "refusing an existing run directory: $RUN"
install -d "$RUN/download" "$RUN/seed" "$RUN/out" "$RUN/results"
cp "$SPEC_INPUT" "$RUN/checkpoint-inputs.json"
SPEC="$RUN/checkpoint-inputs.json"
exec >> "$RUN/job.log" 2>&1
OUTPUT_CLAIMED=0
CAPTURE_STARTED=0
on_exit() {
  local rc=$?
  if [ "$rc" -ne 0 ]; then
    printf '%s\n' "$rc" > "$RUN/exit.code"
    failure_complete=1
    if [ "$OUTPUT_CLAIMED" = 1 ] && [ -x "$STORE" ]; then
      for file in job.log transfer.log store-list.err capture.stderr capture.stdout.jsonl \
        binaries.sha256 gpu.txt selected-checkpoint.sha256 checkpoint-inputs.json status.json \
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
    for file in job.log checkpoint-inputs.json exit.code; do
      [ -f "$RUN/$file" ] || failure_complete=0
    done
    if [ "$CAPTURE_STARTED" = 1 ]; then
      for file in capture.stdout.jsonl capture.stderr binaries.sha256 gpu.txt; do
        [ -f "$RUN/$file" ] || failure_complete=0
      done
    fi
    if [ "$failure_complete" != 1 ]; then
      printf 'failure diagnostics were not fully uploaded and read back\n' \
        > "$RUN/FAILURE_EVIDENCE_INCOMPLETE"
    fi
  fi
  printf '%s\n' "$rc" > "$RUN/EXIT"
}
trap on_exit EXIT

# The jq `$seed` arguments below are intentionally single-quoted.
# shellcheck disable=SC2016
PIN=$(spec_get '.source_pin')
RUN_STORE=$(spec_get '.seeds[$seed].run_store')
ARCHIVE=$(spec_get '.seeds[$seed].archive')
ARCHIVE_SHA256=$(spec_get '.seeds[$seed].archive_sha256')
RUN_CASE=$(spec_get '.seeds[$seed].run_case')
CHECKPOINT_REL=$(spec_get '.seeds[$seed].checkpoint')
CHECKPOINT_MANIFEST_REL=$(spec_get '.seeds[$seed].checkpoint_manifest')
RUN_CONFIG_REL=$(spec_get '.seeds[$seed].run_config.path')
RUN_CONFIG_SHA256=$(spec_get '.seeds[$seed].run_config.sha256')
CHECKPOINT_STEP=${CHECKPOINT_REL##*/}
REPO=${UOR_REPO:-/root/build/src-$PIN}
SPEC_SHA256=$(sha256sum "$SPEC" | cut -d' ' -f1)
EXPECTED=$(jq -c '.expected' "$SPEC")
CONTEXT=$(spec_get '.expected.model.context')
WIDTH=$(spec_get '.expected.model.width')
case "$RUN_CONFIG_REL" in
  "oh/run/$RUN_CASE/config.json") ;;
  *) die "run config path does not match run case $RUN_CASE: $RUN_CONFIG_REL" ;;
esac
case "$CHECKPOINT_REL" in
  "oh/run/$RUN_CASE/models/step-"*) ;;
  *) die "checkpoint path does not match run case $RUN_CASE: $CHECKPOINT_REL" ;;
esac
while IFS=$'\t' read -r name rel; do
  case "$rel" in
    "$CHECKPOINT_REL/"*) ;;
    *) die "$name path is outside $CHECKPOINT_REL: $rel" ;;
  esac
done < <(jq -r --arg seed "$SEED" \
  '.seeds[$seed].checkpoint_files | to_entries[] | [.key, .value.path] | @tsv' "$SPEC")

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

log "[0/6] run=$RUN output=$OUT_STORE source=$PIN spec=$SPEC_SHA256"
log "spec_sha256=$SPEC_SHA256"
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

check_file "$ARCHIVE_SHA256" "$ARCHIVE_PATH"

log "[2/6] extract and verify $RUN_CASE checkpoint $CHECKPOINT_STEP"
mapfile -t extract_paths < <(jq -r --arg seed "$SEED" \
  '.seeds[$seed] |
    [.checkpoint_manifest, .run_config.path, (.checkpoint_files[].path)] | .[]' "$SPEC")
tar -xzf "$ARCHIVE_PATH" -C "$RUN/seed" "${extract_paths[@]}"

selected="$RUN/selected-checkpoint.sha256"
: > "$selected"
{
  printf 'run-config\t%s\t%s\n' "$RUN_CONFIG_REL" "$RUN_CONFIG_SHA256"
  jq -r --arg seed "$SEED" \
    '.seeds[$seed].checkpoint_files | to_entries[] |
      [.key, .value.path, .value.sha256] | @tsv' "$SPEC"
} | while IFS=$'\t' read -r name path sha; do
  check_file "$sha" "$RUN/seed/$path"
  manifest_line="$sha  ./${path#oh/run/}"
  grep -Fxq "$manifest_line" "$RUN/seed/$CHECKPOINT_MANIFEST_REL" ||
    die "$path differs from its inner SHA-256 manifest"
  printf '%s\n' "$manifest_line" >> "$selected"
done
[ "$(wc -l < "$selected")" -eq 4 ] || die "checkpoint SHA manifest did not select four files"
(cd "$RUN/seed/oh/run" && sha256sum -c "$selected")
CHECKPOINT_ROOT="$RUN/seed/$CHECKPOINT_REL"
jq -e --argjson seed "$SEED" --argjson expected "$EXPECTED" '
  .seed == $seed and
  .objective == $expected.run.objective and
  .train_as_served == $expected.run.train_as_served and
  .width == $expected.run.width and
  .hamming_read == $expected.run.hamming_read and
  .batch == $expected.run.batch and
  .micro_batch == $expected.run.micro_batch and
  .steps == $expected.run.steps and
  .train_split.tokens_sha256 ==
    "ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52" and
  .train_split.mask_sha256 ==
    "ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875"
' "$RUN/seed/$RUN_CONFIG_REL" >/dev/null ||
  die "$RUN_CASE run config is not the registered b64L recipe"
jq -e --argjson expected "$EXPECTED" '
  .arch == $expected.model.arch and
  .gate == $expected.model.gate and
  .head == $expected.model.head and
  .pattern == $expected.model.pattern and
  .rotation == $expected.model.rotation and
  .width == $expected.model.width and
  .context == $expected.model.context and
  .heads == $expected.model.heads
' "$CHECKPOINT_ROOT/config.json" >/dev/null ||
  die "$RUN_CASE model config is not the registered width-256 six-layer checkpoint"

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
CAPTURE_STARTED=1
set +e
/usr/bin/time -v "$CARGO_TARGET_DIR/release/train-in-space" \
  --data-root "$DATA" \
  --score-rows "$CHECKPOINT_ROOT" \
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
  --context "$CONTEXT" \
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
jq -e --argjson width "$WIDTH" --argjson context "$CONTEXT" '
  .schema == "uor-r4.fbs-query-cache/1" and
  .width == $width and
  .query_space == "q12-f32" and
  .tokenizer_sha256 == "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89" and
  (.checkpoint_files | keys | sort) == ["config.json", "hamming_read.safetensors", "model.safetensors"] and
  (.splits | length) == 5 and
  all(.splits[]; .memory == false and .label == "no-memory" and .context == $context) and
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
while IFS=$'\t' read -r name sha; do
  actual=$(jq -er --arg name "$name" '.checkpoint_files[$name]' \
    "$RUN/results/cache/manifest.json")
  [ "$actual" = "$sha" ] ||
    die "cache manifest checkpoint file $name hashes to $actual, not $sha"
done < <(jq -r --arg seed "$SEED" \
  '.seeds[$seed].checkpoint_files | to_entries[] |
    [.key, .value.sha256] | @tsv' "$SPEC")
jq -e --arg sha "$RUN_CONFIG_SHA256" '.run_config_sha256 == $sha' \
  "$RUN/results/cache/manifest.json" >/dev/null ||
  die "cache manifest does not name the registered run config"
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
check_cache_archive "$RUN/results/cache.tar.zst" "$RUN/results/cache.members.txt"
need_readback_room "$RUN/results/cache.tar.zst" "$RUN/results/normal-outputs.tar.zst"

log "[6/6] upload and read back sealed evidence"
for file in results/cache.tar.zst results/cache.tar.zst.sha256 results/cache.SHA256SUMS \
  results/normal-outputs.tar.zst results/normal-outputs.tar.zst.sha256 \
  capture.stderr capture.stdout.jsonl binaries.sha256 gpu.txt selected-checkpoint.sha256 \
  checkpoint-inputs.json; do
  "$STORE" put "$RUN/$file" "$OUT_STORE/$file" >> "$RUN/transfer.log" 2>&1
done
for file in results/cache.tar.zst.sha256 results/cache.SHA256SUMS capture.stderr \
  capture.stdout.jsonl results/normal-outputs.tar.zst \
  results/normal-outputs.tar.zst.sha256 binaries.sha256 gpu.txt \
  selected-checkpoint.sha256 checkpoint-inputs.json; do
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
printf '{"seed":%s,"run_id":"%s","status":"CAPTURED","source_pin":"%s","checkpoint":"%s","checkpoint_inputs_sha256":"%s","run_config_sha256":"%s","started":"%s","finished":"%s","cache_manifest_sha256":"%s","cache_tar_sha256":"%s","normal_outputs_tar_sha256":"%s","capture_binary_sha256":"%s"}\n' \
  "$SEED" "$RUN_ID" "$PIN" "$CHECKPOINT_REL" "$SPEC_SHA256" "$RUN_CONFIG_SHA256" \
  "$started" "$finished" \
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
