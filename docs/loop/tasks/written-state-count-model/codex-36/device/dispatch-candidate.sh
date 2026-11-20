#!/usr/bin/env bash
set -euo pipefail
umask 077
stage=$(realpath "${1:?stage}")
artifact=$(realpath "${2:?compact artifact}")
out=$(realpath -m "${3:?new output directory}")
cd "$stage"
witness=candidate-witness.json
sha256sum --check --strict GO.sha256
grep -q '^TEST FITNESS: FIT' FIT.md
pin() { sha256sum "$1" | cut -d' ' -f1; }
jq -e --arg build "$(pin build-and-input.sha256)" --arg source "$(pin source.sha256)" \
  --arg fit "$(pin FIT.md)" --arg artifact "$(pin "$artifact")" --arg witness "$(pin "$witness")" \
  '.build_and_input_sha256 == $build and .source_sha256 == $source and .fit_sha256 == $fit
    and .candidate_sha256 == $artifact and .witness_sha256 == $witness' GO.json >/dev/null
sha256sum --check --strict source.sha256
sha256sum --check --strict build-and-input.sha256
test "$(jq '.paired_native_lookups | length' "$witness")" -eq 18
test "$(wc -c < packet/probes.bin)" -eq 432
test "$(wc -c < packet/orders.bin)" -eq 528
uuid=$(jq -er '.device_uuid | select(test("^GPU-[0-9a-f-]+$"))' GO.json)
test "$(nvidia-smi -i "$uuid" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
export CUDA_VISIBLE_DEVICES="$uuid" CUDA_DISABLE_PTX_JIT=1
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -n 8 || { echo "shared GPU lock busy"; exit 75; }
foreign=$(nvidia-smi -i "$uuid" --query-compute-apps=pid --format=csv,noheader)
if [[ -n "$foreign" ]]; then echo "GPU occupied by $foreign"; exit 75; fi
mkdir "$out"
nvidia-smi -i "$uuid" --query-gpu=uuid,name,driver_version,compute_cap --format=csv >"$out/device.csv"
for pass in primary cold; do
  /usr/bin/time -v -o "$out/$pass.device.time" ./d36-count-device "$artifact" \
    packet/orders.bin packet/probes.bin "$out/$pass.words"
done
flock -u 8
for pass in primary cold; do
  /usr/bin/time -v -o "$out/$pass.native.time" ./d36-device-witness verify "$witness" "$artifact" \
    "$out/$pass.words" >"$out/$pass.json"
done
cmp "$out/primary.words" "$out/cold.words"
cmp "$out/primary.json" "$out/cold.json"
sha256sum "$out"/* >"$out/receipts.sha256"
sha256sum --check --strict "$out/receipts.sha256"
