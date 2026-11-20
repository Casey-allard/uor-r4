#!/usr/bin/env bash
set -euo pipefail
umask 077
stage=$(realpath "${1:?stage}")
out=$(realpath -m "${2:?new output directory}")
cd "$stage"
sha256sum --check --strict GO.sha256
grep -q '^TEST FITNESS: FIT' FIT.md
build_pin=$(sha256sum build-and-input.sha256 | cut -d' ' -f1)
source_pin=$(sha256sum source.sha256 | cut -d' ' -f1)
fit_pin=$(sha256sum FIT.md | cut -d' ' -f1)
jq -e --arg build "$build_pin" --arg source "$source_pin" --arg fit "$fit_pin" \
  '.build_and_input_sha256 == $build and .source_sha256 == $source and .fit_sha256 == $fit' GO.json >/dev/null
uuid=$(jq -er '.device_uuid | select(test("^GPU-[0-9a-f-]+$"))' GO.json)
test "$(nvidia-smi -i "$uuid" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
export CUDA_VISIBLE_DEVICES="$uuid"
sha256sum --check --strict source.sha256
sha256sum --check --strict build-and-input.sha256
mkdir "$out"
jq --slurpfile wrong prod-c128-all.json '.candidate_sha256 = $wrong[0].candidate_sha256' \
  prod-c128-8m.json >"$out/stale-witness.json"
if target/release/d36-device-witness prepare "$out/stale-witness.json" \
  ../pruner/final/c128-8m.counts "$out/stale-packet" >"$out/stale.stdout" 2>"$out/stale.stderr"; then
  exit 1
fi
test ! -e "$out/stale-packet"
grep -q 'wrong witness compact identity' "$out/stale.stderr"
nvidia-smi -i "$uuid" --query-gpu=uuid,name,driver_version,compute_cap --format=csv >"$out/device.csv"
for label in all 8m floor; do
  artifact="../pruner/final/c128-$label.counts"
  for pass in primary cold; do
    result="$out/$label-$pass"
    /usr/bin/time -v -o "$result.time" ./d36-count-device "$artifact" \
      "packet-$label/orders.bin" "packet-$label/probes.bin" "$result.words"
    target/release/d36-device-witness verify "prod-c128-$label.json" "$artifact" \
      "$result.words" >"$result.json"
  done
  cmp "$out/$label-primary.words" "$out/$label-cold.words"
  cmp "$out/$label-primary.json" "$out/$label-cold.json"
done
cp "$out/all-primary.words" "$out/all-altered.words"
printf '\000' | dd of="$out/all-altered.words" bs=1 count=1 conv=notrunc status=none
if target/release/d36-device-witness verify prod-c128-all.json ../pruner/final/c128-all.counts \
  "$out/all-altered.words" >"$out/altered.stdout" 2>"$out/altered.stderr"; then
  exit 1
fi
grep -q 'CUDA probe 0 order 0' "$out/altered.stderr"
sha256sum "$out"/* >"$out/receipts.sha256"
sha256sum --check --strict "$out/receipts.sha256"
