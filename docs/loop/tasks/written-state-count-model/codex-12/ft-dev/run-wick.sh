#!/usr/bin/env bash
set -euo pipefail
cd "${1:-$HOME/uor-codex/20261118-paired-role-fence/ft-dev}"
test -s FIT.md
top=$HOME/uor-codex
count=../count
tool=$top/20261118-role-boundary-count/bin/action-writer
seed=$top/20261118-value-connectors/inputs/seed1.d11
exec 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
run() {
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$@"
    flock -u 8
}
jq -e '.rows_sha256=="d933c533b8b95b4ed6cb54cf96fe83c25a221e3e3226f9a212f52c7bfc1a7074" and .rows==14287' rows.json >/dev/null
echo '0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  tokens.u16' | sha256sum -c -
echo 'd09a2d3675385220326a0322b0e4eca9d7a1e9d8cfe6b366c3b41fa36373b2c1  ft.raw.u16' | sha256sum -c -
cmp ft.raw.u16 <(tail -c +65 tokens.u16)
echo 'a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  response_mask.u8' | sha256sum -c -
nvidia-smi --query-gpu=name,memory.free --format=csv
if test ! -e baseline.jsonl; then
  run "$tool" score "$seed" config.json baseline.json baseline.jsonl --reference
fi
echo '9c88ee8d20518a0063f1a8eabb0b37ed84861d52cce561604093404840c8caac  baseline.jsonl' | sha256sum -c -
for fraction in 1 10 100; do
  dir=$count/primary-$fraction
  [[ $fraction != 1 ]] || dir=$count/primary-1-v2
  for arm in depth1-full depth2-equal depth3-equal depth4-equal; do
    name="$fraction-$arm"
    d11=$(jq -re --arg arm "$arm" '.arms[]|select(.arm==$arm)|.d11_sha256' "$dir/result.json")
    words=$(jq -re --arg arm "$arm" '.arms[]|select(.arm==$arm)|.words_sha256' "$dir/result.json")
    printf '%s  %s\n' "$d11" "$dir/run/$arm.d11" "$words" "$dir/run/$arm.reload.words" | sha256sum -c -
    run "$count/context-score-device" score "$dir/run/$arm.reload.words" ft.raw.u16 "$name.gpu" 1
    sha256sum "$name.gpu"
  done
done
