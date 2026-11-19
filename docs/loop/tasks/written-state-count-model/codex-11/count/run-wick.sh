#!/usr/bin/env bash
# Run from a new prepared Wick count work directory. Do not overwrite any prior receipt.
set -euo pipefail
export LD_LIBRARY_PATH=/usr/local/cuda/lib64
input="$HOME/uor-codex/20261118-diagnostics/inputs"
exec 8>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock 8
test ! -e finite && test ! -e run
mkdir receipt
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 2048
test "$(awk '/MemAvailable/{print $2}' /proc/meminfo)" -ge 4194304
test "$(df -Pk . | awk 'NR==2 {print $4}')" -ge 4194304
{ hostname; date -u; nvidia-smi --query-gpu=name,uuid,driver_version,memory.free --format=csv; free -m; df -Pk .; sha256sum target/release/count-score count-score-device "$input/seed1.d11" "$input/c420/"{tokenizer.json,train.tokens.u16,dev-tune.tokens.u16,dev-tune.novel.u8,id-bytes.u8}; } > receipt/identity.txt
sha256sum src/crates/uor-r4-integer/src/{count_mixer.rs,discrete_d11.rs,native_geometry.rs,serving_root.rs} src/docs/labs/kn-reference/instrument/src/bin/count-score.{rs,cu} > receipt/source.sha256
target/release/count-score fixture "$input/c420" ./count-score-device "$input/seed1.d11" finite > receipt/finite.json
date -u '+GO: finite device/loader/refusal controls passed, independent COUNT-FIT, shared Wick lock %FT%TZ' > receipt/release.txt
/usr/bin/time -v -o receipt/time.txt target/release/count-score run "$input/c420" ./count-score-device "$input/seed1.d11" run > receipt/result.json
sha256sum run/{full,pruned}.d11 receipt/{result,finite}.json > receipt/output.sha256
