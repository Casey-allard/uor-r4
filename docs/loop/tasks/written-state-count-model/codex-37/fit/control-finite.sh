#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
test "$(hostname -s)" = wick
stage=$(realpath -e "$1")
test "$0" -ef "$stage/control-finite.sh"
cd "$stage"

check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO-C1-FINITE.json; }
check GO-C1-FINITE.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 finite masked optimizer'
test "$(field device_uuid)" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
check control-finite.sh "$(field dispatch_sha256)"
check FIT-C1-FINITE.md "$(field fit_sha256)"
check SHEET-CONTROLS.md "$(field sheet_sha256)"
check control-stage.sha256 "$(field stage_sha256)"
sha256sum --check --strict control-stage.sha256

test ! -e finite-result && test ! -L finite-result
gpu=$(field device_uuid)
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
test -f "$lock"
exec 8<>"$lock"
flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
mkdir finite-result
cp GO-C1-FINITE.json SHEET-CONTROLS.md FIT-C1-FINITE.md finite-result/
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > finite-result/device.csv
/usr/bin/time -v -o finite-result/masked.time ./d37-r-device-masked finite-input/a1-masked.rows \
    finite-result/a1-masked.trace finite-input/finite-ordinal.u64 finite-result/a1-masked.probe.u64 2>finite-result/masked.stderr
./d37-r-fit-control verify-finite-masked finite-result | tee finite-result/parity.txt
(cd finite-result && sha256sum GO-C1-FINITE.json SHEET-CONTROLS.md FIT-C1-FINITE.md device.csv a1-masked.* masked.* parity.txt > results.sha256)
