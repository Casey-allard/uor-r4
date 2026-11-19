#!/usr/bin/env bash
# Called once per authorized cut. The fourth argument is present only for the cold recovery.
set -euo pipefail
umask 077
cut=$1
name=$2
min_free_mib=$3
recover=${4:-}
case "$cut" in 1|100|1000) ;; *) exit 2 ;; esac
base=$HOME/uor-codex/20261118-full-count-ceiling
inputs=$HOME/uor-codex/20261118-diagnostics/inputs/c420
pack=$base/dispatch/release
mkdir -p "$base/runs"
run=$base/runs/$name
mkdir "$run"
exec 9>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock 9
cd "$pack"
sha256sum -c binary.sha256 > "$run/bin-verified.log"
source_sha=$(cut -d' ' -f1 source.sha256)
export CUDA_VISIBLE_DEVICES=GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
[[ $(nvidia-smi -i 0 --query-gpu=uuid --format=csv,noheader) == "$CUDA_VISIBLE_DEVICES" ]]
hostname > "$run/host"
date -u > "$run/started"
nvidia-smi --query-gpu=name,uuid,memory.total,memory.free,utilization.gpu --format=csv > "$run/device-start.csv"
nvidia-smi --query-compute-apps=pid,name,used_gpu_memory --format=csv > "$run/other-processes-start.csv"
free -k > "$run/host-start"
cgroup=$(awk -F: '$1==0 {print $3}' /proc/self/cgroup)
gpu_free=$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits -i 0 | tr -d ' ')
ram_free=$(awk '/MemAvailable:/{print $2}' /proc/meminfo)
node=/sys/fs/cgroup${cgroup%/}
while [[ $node == /sys/fs/cgroup || $node == /sys/fs/cgroup/* ]]; do
    if [[ -r $node/memory.max ]]; then
        limit=$(cat "$node/memory.max")
        used=$(cat "$node/memory.current")
        echo "$node $limit $used" >> "$run/cgroup-memory"
        if [[ $limit != max ]] && (( (limit-used)/1024 < ram_free )); then
            ram_free=$(( (limit-used)/1024 ))
        fi
    fi
    node=${node%/*}
done
if ((gpu_free < min_free_mib || ram_free < 13 * 1024 * 1024)); then
    echo "Insufficient available capacity at lock acquisition: GPU $gpu_free MiB, RAM $ram_free KiB" | tee "$run/stop"
    exit 3
fi
nvidia-smi -i 0 --query-gpu=timestamp,power.draw,utilization.gpu,memory.used,memory.free --format=csv -l 1 > "$run/gpu.csv" & monitor=$!
nvidia-smi --query-compute-apps=timestamp,pid,name,used_gpu_memory --format=csv -l 1 > "$run/processes.csv" & process_monitor=$!
trap 'kill "$monitor" "$process_monitor" 2>/dev/null || true; wait "$monitor" "$process_monitor" 2>/dev/null || true' EXIT
collect=("$pack/kn-reference" "$inputs/train.tokens.u16" "$inputs/dev-tune.tokens.u16" "$inputs/id-bytes.u8" "$inputs/dev-tune.novel.u8" 10 16777216 "$run/frame" --fit-frame --source-sha256 "$source_sha" --per-mille "$cut" --profile-phases)
if ((cut==1)); then collect+=(--check-host); fi
/usr/bin/time -v -o "$run/collector.time" "${collect[@]}" > "$run/collector.log" 2>&1
frame=$run/frame
if [[ -n "$recover" ]]; then
    old=$base/runs/$recover/frame
    cmp "$old/frame.bin" "$frame/frame.bin"
    cmp "$old/manifest.json" "$frame/manifest.json"
    cmp "$old/seal.sha256" "$frame/seal.sha256"
    frame=$old
fi
"$pack/kn-reference" --verify-fit-frame "$frame" > "$run/frame-reopen.log" 2>&1
"$pack/kn-ceiling-verify" "$frame" "$source_sha" "$pack/kn-reference" --query "$inputs" > "$run/frame-verified.json"
n=$(jq -er '.target_count' "$frame/manifest.json")
hash=$(jq -er '.frame_sha256' "$frame/manifest.json")
manifest_hash=$(sha256sum "$frame/manifest.json" | cut -d' ' -f1)
/usr/bin/time -v -o "$run/fit.time" "$pack/kn-ceiling-fit" "$frame/frame.bin" "$n" "$run/fit" --identity "$manifest_hash" "$hash" > "$run/fit.log" 2>&1
"$pack/kn-ceiling-verify" "$frame" "$source_sha" "$pack/kn-reference" --query "$inputs" --fit "$run/fit" "$pack/kn-ceiling-fit" > "$run/verified.json"
if [[ -n "$recover" ]]; then
    cmp "$base/runs/$recover/fit/result.json" "$run/fit/result.json"
fi
date -u > "$run/finished"
jq -e '.reading.contains_kn_all_and_novel and .reading.converged' "$run/verified.json" > "$run/eligible"
