#!/usr/bin/env bash
# Compare float-checkpoint and integer-artifact nearest decisions on identical
# content-ratio packets, seal the reports and read them back through uor-store.
set -euo pipefail

PIN=${UOR_PARITY_PIN:?UOR_PARITY_PIN must name the reviewed 40-hex commit}
case "$PIN" in
  *[!0-9a-f]*|'') echo "UOR_PARITY_PIN is not lowercase 40-hex" >&2; exit 2 ;;
esac
[ "${#PIN}" -eq 40 ] || { echo "UOR_PARITY_PIN is not 40 hex characters" >&2; exit 2; }

RUN_ID=${UOR_PARITY_RUN_ID:?UOR_PARITY_RUN_ID is required}
case "$RUN_ID" in
  ''|*..*|*[!A-Za-z0-9._-]*) echo "UOR_PARITY_RUN_ID contains unsafe characters" >&2; exit 2 ;;
esac

SOURCE_STORE=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/runs/s1-content-ratio-20261108t055107z-adb12718d6c9ea88
OUT_STORE="results/deepseek/one-model-end-to-end-packet-parity-2026-11-08/runs/$RUN_ID"
STORE=/root/uor-store
ROOT=/workspace/uor-r4/parity-runs/$RUN_ID
SOURCE=$ROOT/source
EVIDENCE=$ROOT/evidence
REPO=/workspace/uor-r4/parity-src-$PIN
TARGET=/workspace/uor-r4/parity-target-$PIN
SOURCE_CPUS=${UOR_PARITY_SOURCE_CPUS:-8}
SEAL_LIMIT_BYTES=104857600

export RAYON_NUM_THREADS=$SOURCE_CPUS
export CARGO_TARGET_DIR=$TARGET

die() {
  echo "packet-parity-job: $*" >&2
  exit 1
}
check_file() {
  local want=$1 path=$2 got
  got=$(sha256sum "$path" | cut -d' ' -f1)
  [ "$got" = "$want" ] || die "$path hashes to $got, not $want"
}

command -v git >/dev/null || die "git is required"
command -v jq >/dev/null || die "jq is required"
command -v nvidia-smi >/dev/null || die "nvidia-smi is required"
[ -x "$STORE" ] || die "uor-store is not installed"
[ ! -e "$ROOT" ] || die "refusing an existing run root: $ROOT"

store_listing=$("$STORE" ls results/deepseek/one-model-end-to-end-packet-parity-2026-11-08/runs 2>/dev/null || true)
if printf '%s\n' "$store_listing" |
  awk -v path="$OUT_STORE/" 'index($0, path) { found = 1 } END { exit(found ? 0 : 1) }'; then
  die "refusing a nonempty store output: $OUT_STORE"
fi

install -d "$SOURCE" "$EVIDENCE/run/models" "$EVIDENCE/artifacts" \
  "$EVIDENCE/inputs" "$EVIDENCE/reports" "$EVIDENCE/logs"
install -m 0755 /workspace/uor-parity-job.sh "$EVIDENCE/job.sh"
exec >> "$EVIDENCE/job.log" 2>&1

"$STORE" get "$SOURCE_STORE/" "$SOURCE"
SOURCE_RUN="$SOURCE/$SOURCE_STORE"
[ -d "$SOURCE_RUN" ] || die "store get did not create $SOURCE_RUN"
(cd "$SOURCE_RUN" && sha256sum -c SHA256SUMS) ||
  die "sealed content-ratio source readback failed its manifest"
SOURCE_MANIFEST_SHA=$(sha256sum "$SOURCE_RUN/SHA256SUMS" | cut -d' ' -f1)

for step in 000512 002048; do
  install -d "$EVIDENCE/run/models/step-$step"
  for file in model.safetensors config.json hamming_read.safetensors; do
    install -m 0644 "$SOURCE_RUN/run/models/step-$step/$file" \
      "$EVIDENCE/run/models/step-$step/$file"
  done
  install -m 0644 "$SOURCE_RUN/artifacts/attempt-1/step-$step/model.uorlut" \
    "$EVIDENCE/artifacts/step-$step.uorlut"
done
install -m 0644 "$SOURCE_RUN/run/config.json" "$EVIDENCE/run/config.json"
install -m 0644 "$SOURCE_RUN/inputs/data/tokenizer.json" "$EVIDENCE/inputs/tokenizer.json"
for file in inputs.json labels.json plan.json manifest.json; do
  install -m 0644 "$SOURCE_RUN/inputs/m2-fresh256/$file" "$EVIDENCE/inputs/$file"
done

check_file 11195de63f508a454c085cb538be33aa1d26c3c282e0248b85aa2e9b61e23133 \
  "$EVIDENCE/run/models/step-000512/model.safetensors"
check_file e4562c3aa18e3aeaa72ce646ed5d2403b5eb5394476e304d6f83ee8526817c6a \
  "$EVIDENCE/run/models/step-000512/hamming_read.safetensors"
check_file 406c9a5c05d63c8f2b6044ee6de6ea78d9fa91582dc03c4aa3e4fe877846e384 \
  "$EVIDENCE/artifacts/step-000512.uorlut"
check_file 98ad4ed7b5db03ee33cdcbcc4528c9b94dc3048e0d541e978e2895674d992e00 \
  "$EVIDENCE/run/models/step-002048/model.safetensors"
check_file 17d4a1df9465af85503db5217f7b589a352a6c0a9a06251eb48ee9887601354a \
  "$EVIDENCE/run/models/step-002048/hamming_read.safetensors"
check_file bee053c1f1507373dffa2742a444f6f64e99bd31e06df57508cbf3e821a3ec24 \
  "$EVIDENCE/artifacts/step-002048.uorlut"
check_file 1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2 \
  "$EVIDENCE/run/models/step-000512/config.json"
check_file 1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2 \
  "$EVIDENCE/run/models/step-002048/config.json"
check_file 2eda0a4c212fd56d186b05fabbca5bf9cc93cc6a5085bbab26009e22e6089389 \
  "$EVIDENCE/run/config.json"
check_file d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89 \
  "$EVIDENCE/inputs/tokenizer.json"
check_file 1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a \
  "$EVIDENCE/inputs/inputs.json"
check_file 3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be \
  "$EVIDENCE/inputs/labels.json"
check_file 0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715 \
  "$EVIDENCE/inputs/plan.json"
check_file 7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964 \
  "$EVIDENCE/inputs/manifest.json"

nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader \
  > "$EVIDENCE/gpu.txt"
lscpu > "$EVIDENCE/cpu.txt"
jq -n \
  --arg model "$(sed -n 's/^model name[[:space:]]*:[[:space:]]*//p' /proc/cpuinfo | head -n 1)" \
  --arg arch "$(uname -m)" \
  --argjson logical "$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 0)" \
  '{schema:"uor-r4.cpu-hardware/1",model:$model,architecture:$arch,logical_cpus:$logical}' \
  > "$EVIDENCE/cpu.json"
jq -e '(.model | length) > 0 and (.architecture | length) > 0 and .logical_cpus > 0' \
  "$EVIDENCE/cpu.json" >/dev/null || die "could not identify CPU hardware"

if [ ! -d "$REPO/.git" ]; then
  install -d "$REPO"
  git -C "$REPO" init --quiet
  git -C "$REPO" remote add origin https://github.com/UOR-Foundation/uor-r4.git
fi
git -C "$REPO" fetch --quiet --depth 1 origin "$PIN" || die "could not fetch $PIN"
git -C "$REPO" checkout --quiet --detach FETCH_HEAD || die "could not checkout $PIN"
[ "$(git -C "$REPO" rev-parse HEAD)" = "$PIN" ] || die "parity source is not $PIN"
[ -z "$(git -C "$REPO" status --porcelain)" ] || die "parity source checkout is dirty"
cargo build --manifest-path "$REPO/Cargo.toml" --release --locked \
  -p uor-r4-float-train --features cuda,packet-parity --bin packet-parity
BIN="$TARGET/release/packet-parity"
[ -x "$BIN" ] || die "build did not produce $BIN"
sha256sum "$BIN" > "$EVIDENCE/binary.sha256"

for step in 000512 002048; do
  for rendering in per-turn/v1 one-user-turn/v1; do
    slug=${rendering%/v1}
    out="$EVIDENCE/reports/step-$step-$slug.json"
    log="$EVIDENCE/logs/step-$step-$slug.log"
    "$BIN" \
      --checkpoint "$EVIDENCE/run/models/step-$step" \
      --artifact "$EVIDENCE/artifacts/step-$step.uorlut" \
      --tokenizer "$EVIDENCE/inputs/tokenizer.json" \
      --inputs "$EVIDENCE/inputs/inputs.json" \
      --labels "$EVIDENCE/inputs/labels.json" \
      --plan "$EVIDENCE/inputs/plan.json" \
      --manifest "$EVIDENCE/inputs/manifest.json" \
      --out "$out" --rendering "$rendering" > "$log" 2>&1 ||
      die "packet parity failed at step $step, rendering $rendering"
    jq -e '.totals.rows == 256 and .totals.eos_rows == 256' "$out" >/dev/null ||
      die "packet parity report at step $step, rendering $rendering is incomplete"
  done
done

jq -n --arg pin "$PIN" --arg source "$SOURCE_STORE" \
  --arg source_manifest "$SOURCE_MANIFEST_SHA" \
  --arg binary "$(cut -d' ' -f1 "$EVIDENCE/binary.sha256")" \
  '{schema:"uor-r4.packet-parity-run/1",pin:$pin,source_store:$source,
    source_manifest_sha256:$source_manifest,binary_sha256:$binary,
    checkpoints:[512,2048],renderings:["per-turn/v1","one-user-turn/v1"]}' \
  > "$EVIDENCE/run.json"

printf '0\n' > "$EVIDENCE/EXIT"
exec >> "$ROOT.seal.log" 2>&1
(
  cd "$EVIDENCE"
  find . -type f ! -name 'SHA256SUMS*' -print0 |
    sort -z | xargs -0 -r sha256sum > SHA256SUMS
  sha256sum -c SHA256SUMS
)
SEAL_BYTES=$(find "$EVIDENCE" -type f -printf '%s\n' | awk '{ total += $1 } END { print total + 0 }')
[ "$SEAL_BYTES" -le "$SEAL_LIMIT_BYTES" ] ||
  die "seal is $SEAL_BYTES bytes, over the $SEAL_LIMIT_BYTES-byte limit"
printf '%s\n' "$SEAL_BYTES" > "$EVIDENCE/BYTES"

"$STORE" put "$EVIDENCE" "$OUT_STORE" || die "store put failed"
install -d "$ROOT/readback"
"$STORE" get "$OUT_STORE/" "$ROOT/readback" || die "store readback failed"
SEALED="$ROOT/readback/$OUT_STORE"
[ -d "$SEALED" ] || die "store readback did not create $SEALED"
(cd "$SEALED" && sha256sum -c SHA256SUMS) || die "store readback checksum failed"
printf 'verified\n' > "$ROOT/readback-status"
"$STORE" put "$ROOT/readback-status" "$OUT_STORE/READBACK" ||
  die "could not upload READBACK marker"

echo "packet parity sealed and read back: $OUT_STORE"
