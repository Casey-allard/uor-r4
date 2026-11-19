#!/usr/bin/env bash
set -euo pipefail
P=${UOR_UNIT:-$HOME/uor-codex/20261118-content-support}
C=$P/count
D=$P/precision
I=$HOME/uor-codex/20261118-diagnostics
cd "$D"
test -s FIT.md
sha256sum -c release.sha256
exec 8>"$I/wick-codex-gpu.lock"
flock 8
date --utc -Iseconds
nvidia-smi --query-gpu=name,memory.free,utilization.gpu --format=csv,noheader
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
for out in primary repeat; do
    /usr/bin/time -v -o "$out.time" "$C/target/release/count-score" eval "$I/inputs/c420" "$C/count-score-device" "$D/restored.d11" "$D/$out" > "$out.json"
done
cmp primary.json repeat.json || {
    # The result includes elapsed wall time. Compare native tape and every scientific field.
    diff <(jq -S 'del(.score.wall_including_host_parity_ms)' primary.json) <(jq -S 'del(.score.wall_including_host_parity_ms)' repeat.json)
}
date --utc -Iseconds
