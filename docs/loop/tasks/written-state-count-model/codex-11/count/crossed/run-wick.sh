#!/usr/bin/env bash
set -euo pipefail
export LD_LIBRARY_PATH=/usr/local/cuda/lib64
input="$HOME/uor-codex/20261118-diagnostics/inputs"
bin=target/release/count-reweight
score=target/release/count-score
out=crossed-run
mkdir "$out"
exec 8>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock 8
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 2048
test "$(awk '/MemAvailable/{print $2}' /proc/meminfo)" -ge 4194304
test "$(df -Pk . | awk 'NR==2 {print $4}')" -ge 4194304
{ hostname; date -u; nvidia-smi --query-gpu=name,uuid,driver_version,memory.free --format=csv; sha256sum "$bin" "$score" count-score-device src/docs/labs/kn-reference/instrument/src/bin/count-reweight.rs run/{full,pruned}.d11; } > "$out/identity.txt"
sha256sum --check receipt/output.sha256 > "$out/base-check.txt"
"$bin" --fixture-donor "$input/seed1.d11" finite/full.d11 "$out/donor128.d11"
"$bin" "$input/seed1.d11" finite/full.d11 "$out/donor128.d11" "$out/finite.d11" > "$out/finite.json"
if "$bin" "$input/seed1.d11" finite/full.d11 "$input/seed1.d11" "$out/bad.d11" >"$out/reject.txt" 2>&1; then exit 1; fi
test ! -e "$out/bad.d11"
date -u '+GO: independent FIT, shared lock, source digests and finite guard passed %FT%TZ' > "$out/release.txt"
"$bin" "$input/seed1.d11" run/full.d11 run/pruned.d11 "$out/full102.d11" > "$out/model.json"
/usr/bin/time -v -o "$out/time.txt" "$score" eval "$input/c420" ./count-score-device "$out/full102.d11" "$out/eval" > "$out/result.json"
sha256sum "$out/full102.d11" "$out/result.json" > "$out/output.sha256"
