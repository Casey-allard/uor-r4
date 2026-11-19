#!/usr/bin/env bash
set -euo pipefail
umask 077
top="$HOME/uor-codex"
root="$top/20261118-query-region-pick/pick"
out="$root/${1:?required new incumbent replay directory}"
fixture="$top/20261118-paired-role-fence/ft-dev"
seed="$top/20261118-value-connectors/inputs/seed1.d11"
bin="$top/20261118-role-boundary-count/bin/action-writer"
mkdir "$out"
cd "$fixture"
sha256sum "$bin" config.json "$seed" > "$out/inputs.sha256"
(
  flock 8
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
  /usr/bin/time -v -o "$out/time.txt" "$bin" score "$seed" config.json "$out/info.json" "$out/rows.jsonl" --reference
) 8>"$top/20261118-diagnostics/wick-codex-gpu.lock" > "$out/stdout.log" 2>&1
(cd "$out"; sha256sum rows.jsonl > rows.sha256)
cmp "$out/rows.jsonl" baseline.jsonl
