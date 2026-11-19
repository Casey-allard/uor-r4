#!/usr/bin/env bash
# Run from a prepared count directory. Each invocation writes one fresh receipt.
set -euo pipefail
dose="${1:?expected 1, 10, or 100}"
receipt="${2:?expected new receipt directory}"
case "$dose" in
  1) device_min=2048; host_min=12582912 ;;
  10) device_min=4096; host_min=12582912 ;;
  100) device_min=8192; host_min=18874368 ;;
  *) exit 2 ;;
esac
test ! -e "$receipt"
mkdir "$receipt"
export LD_LIBRARY_PATH=/usr/local/cuda/lib64
inputs="$HOME/uor-codex/20261118-diagnostics/inputs"
exec 8>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock 8
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge "$device_min"
test "$(awk '/MemAvailable/{print $2}' /proc/meminfo)" -ge "$host_min"
test "$(df -Pk . | awk 'NR==2 {print $4}')" -ge 12582912
{
  hostname
  date -u
  nvidia-smi --query-gpu=name,uuid,driver_version,memory.free --format=csv
  free -m
  df -Pk .
  sha256sum target/release/context-score context-score-device "$inputs/"{seed1.d11,tokenizer.json} "$inputs/c420/"{tokenizer.json,train.tokens.u16,dev-tune.tokens.u16,dev-tune.novel.u8,id-bytes.u8}
} > "$receipt/identity.txt"
sha256sum src/crates/uor-r4-integer/src/{count_mixer.rs,discrete_d11.rs} src/docs/labs/kn-reference/instrument/{Cargo.toml,Cargo.lock,src/bin/context-score.rs,src/bin/context-score.cu,src/bin/count-score.cu} > "$receipt/source.sha256"
target/release/context-score fixture "$inputs/c420" ./context-score-device "$inputs/seed1.d11" "$receipt/finite" > "$receipt/finite.json" 2> "$receipt/finite.log"
date -u '+GO: finite GPU/D11/mass/route/fixture controls passed under FIT and Wick lock %FT%TZ' > "$receipt/release.txt"
( exec 8>&-
  exec nvidia-smi --query-gpu=timestamp,memory.used,utilization.gpu --format=csv --loop-ms=500
) > "$receipt/device-wide.csv" &
monitor_pid=$!
stop_monitor() {
  kill "$monitor_pid" 2>/dev/null || true
  wait "$monitor_pid" 2>/dev/null || true
}
trap stop_monitor EXIT
/usr/bin/time -v -o "$receipt/time.txt" target/release/context-score "$dose" "$inputs/c420" ./context-score-device "$inputs/seed1.d11" "$receipt/run" > "$receipt/result.json" 2> "$receipt/run.log"
stop_monitor
trap - EXIT
sha256sum "$receipt/run/"{*.d11,*.gpu,*.u32,*.reload.words,fit.u16,query.u16} > "$receipt/output.sha256"
