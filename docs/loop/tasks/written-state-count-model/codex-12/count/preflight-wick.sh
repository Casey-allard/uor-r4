#!/usr/bin/env bash
# Run after staging the repaired scorer and ft/dev launcher in count/preflight-v2.
set -euo pipefail
top=$HOME/uor-codex
root="$top/20261118-paired-role-fence"
historic="$root/count"
work="$historic/preflight-v2"
inputs="$top/20261118-diagnostics/inputs"
fixture="$historic/primary-1-v2/finite"
out="$work/${1:?expected fresh evidence directory}"
test ! -e "$out"
mkdir "$out" "$out/malformed"
cd "$work"
sha256sum "$historic/context-score-device" context-score-device src/{context-score,count-score}.cu ft-run-wick.sh > "$out/source-and-binary.sha256"
cases="$out/malformed"
state="$fixture/depth1-full.reload.words"
header=$(( (4096 + 5) * 4 ))
for name in weights-gap weights-sum count-zero invalid-key overflow duplicate missing-order bad-unigram; do
  cp "$state" "$cases/$name.words"
done
# Each plant is on an independent copy of the finite standalone input.
printf '\000\000\000\000\370\000\000\000' | dd of="$cases/weights-gap.words" bs=1 seek=4 conv=notrunc status=none
printf '\001\000\000\000' | dd of="$cases/weights-sum.words" bs=1 seek=4 conv=notrunc status=none
printf '\000\000\000\000' | dd of="$cases/count-zero.words" bs=1 seek=$((header + 8)) conv=notrunc status=none
printf '\000' | dd of="$cases/invalid-key.words" bs=1 seek=$((header + 7)) conv=notrunc status=none
printf '\377\377\377\377' | dd of="$cases/overflow.words" bs=1 seek=$((header + 8)) conv=notrunc status=none
dd if="$state" bs=1 skip="$header" count=12 status=none | dd of="$cases/duplicate.words" bs=1 seek=$((header + 12)) conv=notrunc status=none
printf '\174\000\000\000\174\000\000\000' | dd of="$cases/missing-order.words" bs=1 seek=4 conv=notrunc status=none
printf '\001' | dd of="$cases/bad-unigram.words" bs=1 seek=20 conv=notrunc status=none
printf '\000\000\000\020\001\000' > "$cases/out-of-vocabulary.u16"
reject() {
  local name=$1 expected=$2 state=$3 ids=$4
  if CUDA_VISIBLE_DEVICES="" ./context-score-device score "$state" "$ids" "$cases/$name.gpu" 1 > "$cases/$name.log" 2>&1; then
    printf 'unexpected acceptance %s\n' "$name" >&2
    return 1
  fi
  test ! -e "$cases/$name.gpu"
  grep -Fx "$expected" "$cases/$name.log" >/dev/null
  printf 'PASS %s: %s\n' "$name" "$expected" >> "$out/refusals.txt"
}
query="$fixture/query.u16"
reject out-of-vocabulary "token ID outside context-score vocabulary" "$state" "$cases/out-of-vocabulary.u16"
reject weights-gap "discontinuous context-score weights" "$cases/weights-gap.words" "$query"
reject weights-sum "bad context-score weights" "$cases/weights-sum.words" "$query"
reject count-zero "invalid or unordered context-score row" "$cases/count-zero.words" "$query"
reject invalid-key "invalid context-score context key" "$cases/invalid-key.words" "$query"
reject overflow "context-score row total overflows" "$cases/overflow.words" "$query"
reject duplicate "invalid or unordered context-score row" "$cases/duplicate.words" "$query"
reject missing-order "missing configured context-score order" "$cases/missing-order.words" "$query"
reject bad-unigram "invalid context-score unigram" "$cases/bad-unigram.words" "$query"
bad_ft="$out/corrupt-ft"
mkdir "$bad_ft"
cp "$root/ft-dev/"{FIT.md,rows.json,tokens.u16,ft.raw.u16,response_mask.u8} "$bad_ft/"
printf '\001' | dd of="$bad_ft/ft.raw.u16" bs=1 count=1 conv=notrunc status=none
if CUDA_VISIBLE_DEVICES="" bash ./ft-run-wick.sh "$bad_ft" > "$out/reject-ft-raw.log" 2>&1; then
  echo "corrupted raw ft/dev tape succeeded" >&2
  exit 1
fi
grep -Fx 'ft.raw.u16: FAILED' "$out/reject-ft-raw.log" >/dev/null
echo 'PASS corrupted raw ft/dev tape rejected before launch' >> "$out/refusals.txt"
cmp "$root/ft-dev/ft.raw.u16" <(tail -c +65 "$root/ft-dev/tokens.u16")
export LD_LIBRARY_PATH=/usr/local/cuda/lib64
exec 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
flock 8
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 2048
"$historic/target/release/context-score" fixture "$inputs/c420" "$work/context-score-device" "$inputs/seed1.d11" "$out/finite" > "$out/finite.json" 2> "$out/finite.log"
for raw in "$fixture"/{*.gpu,*.d11,*.u32,*.reload.words}; do
  cmp "$raw" "$out/finite/${raw##*/}"
  printf 'PASS %s\n' "${raw##*/}" >> "$out/finite-identical.txt"
done
full="$historic/primary-100/run/depth4-equal.reload.words"
raw="$root/ft-dev/ft.raw.u16"
"$historic/context-score-device" score "$full" "$raw" "$out/historical-ft.gpu" 1
./context-score-device score "$full" "$raw" "$out/repaired-ft.gpu" 1
dev="$historic/primary-100/run/query.u16"
./context-score-device score "$full" "$dev" "$out/repaired-dev.gpu" 2639
flock -u 8
cmp "$out/historical-ft.gpu" "$out/repaired-ft.gpu"
cmp "$root/ft-dev/100-depth4-equal.gpu" "$out/repaired-ft.gpu"
sha256sum "$full" "$raw" "$root/ft-dev/100-depth4-equal.gpu" "$out/"{historical,repaired}-ft.gpu > "$out/ft-sha256.txt"
cmp "$historic/primary-100/run/depth4-equal.gpu" "$out/repaired-dev.gpu"
sha256sum "$dev" "$historic/primary-100/run/depth4-equal.gpu" "$out/repaired-dev.gpu" > "$out/dev-sha256.txt"
