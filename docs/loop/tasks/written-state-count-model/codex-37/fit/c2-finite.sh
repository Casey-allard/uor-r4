#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
test "$(hostname -s)" = wick
stage=$(realpath -e "$1")
test "$0" -ef "$stage/c2-finite.sh"
cd "$stage"
check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO-C2-FINITE.json; }
check GO-C2-FINITE.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 finite C2 exact pair'
test "$(field device_uuid)" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
check c2-finite.sh "$(field dispatch_sha256)"
check FIT-C2-FINITE.md "$(field fit_sha256)"
check SHEET-CONTROLS.md "$(field sheet_sha256)"
check control-stage.sha256 "$(field stage_sha256)"
sha256sum --check --strict control-stage.sha256
test ! -e c2-finite-result && test ! -L c2-finite-result
gpu=$(field device_uuid)
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
test -f "$lock"
exec 8<>"$lock"
flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
mkdir c2-finite-result
cp GO-C2-FINITE.json SHEET-CONTROLS.md FIT-C2-FINITE.md c2-finite-result/
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > c2-finite-result/device.csv
/usr/bin/time -v -o c2-finite-result/selector.time ./d37-c2-device c2-input/fixture.uort \
    c2-finite-result/fixture.picks.u32 2>c2-finite-result/selector.stderr
./d37-r-fit-control verify-finite-c2 c2-input/fixture.uort c2-finite-result/fixture.picks.u32 | tee c2-finite-result/parity.txt
(cd c2-finite-result && sha256sum GO-C2-FINITE.json SHEET-CONTROLS.md FIT-C2-FINITE.md device.csv fixture.picks.u32 selector.* parity.txt > results.sha256)
