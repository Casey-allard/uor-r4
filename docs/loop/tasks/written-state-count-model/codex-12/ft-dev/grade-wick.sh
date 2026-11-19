#!/usr/bin/env bash
# Re-open the twelve scored artifacts and compare the previously captured CUDA tapes.
set -euo pipefail
cd "$HOME/uor-codex/20261118-paired-role-fence/ft-dev"
grade="${1:?expected fresh verification receipt directory}"
test ! -e "$grade"
mkdir "$grade"
count=../count
tool="$count/target/release/count-ft-replay"
source="$count/src/docs/labs/kn-reference/instrument/src/bin/count-ft-replay.rs"
sha256sum "$tool" "$source" "$count/context-score-device" {config,tokens,ft.raw,response_mask,rows,baseline,device,time}.* *.gpu > "$grade/inputs.sha256"
awk '/^[[:xdigit:]]{64}  .*\.gpu$/ { print }' device.log > "$grade/device-tapes.sha256"
test "$(wc -l < "$grade/device-tapes.sha256")" -eq 12
sha256sum -c "$grade/device-tapes.sha256" > "$grade/checked-tapes.txt"
model="$count/primary-1-v2/run/depth1-full.d11"
right="$count/primary-1-v2/run/depth1-full.reload.words"
wrong="$count/primary-1-v2/run/depth2-equal.reload.words"
if "$tool" "$model" . "$wrong" 1-depth1-full.gpu > "$grade/reject-wrong-native.log" 2>&1; then
  echo "wrong native state succeeded" >&2
  exit 1
fi
grep -Fq "device input differs from D11 native state" "$grade/reject-wrong-native.log"
bad="$grade/corrupt-fixture"
mkdir "$bad"
cp {tokens.u16,ft.raw.u16,response_mask.u8,rows.json,baseline.jsonl} "$bad/"
printf '\002' | dd of="$bad/response_mask.u8" bs=1 count=1 conv=notrunc status=none
if "$tool" "$model" "$bad" "$right" 1-depth1-full.gpu > "$grade/reject-corrupt-mask.log" 2>&1; then
  echo "corrupted response mask succeeded" >&2
  exit 1
fi
grep -Fq "ft/dev or baseline identity mismatch" "$grade/reject-corrupt-mask.log"
for fraction in 1 10 100; do
  dir="$count/primary-$fraction"
  [[ $fraction != 1 ]] || dir="$count/primary-1-v2"
  for arm in depth1-full depth2-equal depth3-equal depth4-equal; do
    name="$fraction-$arm"
    observed=$("$tool" "$dir/run/$arm.d11" . "$dir/run/$arm.reload.words" "$name.gpu")
    expected=$(jq -re --arg arm "$arm" '.arms[]|select(.arm==$arm)|.d11_sha256' "$dir/result.json")
    jq -ce --arg arm "$arm" --argjson fraction "$fraction" --arg expected "$expected" \
      'select(.model==$expected) | . + {arm:$arm,train_fraction_percent:$fraction}' \
      <<< "$observed" >> "$grade/arms.jsonl"
  done
done
jq -se '
  {
    eligible_targets:14287,
    cuda_d11_arm_rows:(map(.exact_cuda_d11_rows)|add),
    base_correct:7002,
    arms: .
  }
  | select(.arms|length==12)
  | select(.cuda_d11_arm_rows==12*14287)
' "$grade/arms.jsonl" > "$grade/result.json"
