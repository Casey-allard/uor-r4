#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
test "$(hostname -s)" = wick
stage=$(realpath -e "$1")
test "$0" -ef "$stage/finite-dispatch.sh"
cd "$stage"

check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO-FINITE.json; }
check GO-FINITE.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 finite two-shape optimizer'
test "$(field device_uuid)" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
check finite-dispatch.sh "$(field dispatch_sha256)"
check SHEET-FINITE.md "$(field sheet_sha256)"
check FIT-FINITE.md "$(field fit_sha256)"
check finite-stage.sha256 "$(field stage_sha256)"
sha256sum --check --strict finite-stage.sha256

dest=$(field output)
test "$dest" = "$stage/$(basename "$dest")"
[[ "$(basename "$dest")" =~ ^finite-result-[a-z0-9-]+$ ]]
test ! -e "$dest" && test ! -L "$dest"
gpu=$(field device_uuid)
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
test -f "$lock"
exec 8<>"$lock"
flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
mkdir "$dest"
cp GO-FINITE.json SHEET-FINITE.md FIT-FINITE.md "$dest/"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > "$dest/device.csv"

if "$stage/d37-r-device" finite-input/a1s-truncated.rows "$dest/truncated.trace" >"$dest/truncated.stdout" 2>"$dest/truncated.stderr"; then
    echo 'truncated tape unexpectedly passed' >&2
    exit 1
fi
grep -qx 'invalid row tape length' "$dest/truncated.stderr"
test ! -e "$dest/truncated.trace"
for arm in a1 a1s; do
    /usr/bin/time -v -o "$dest/$arm.time" "$stage/d37-r-device" "finite-input/$arm.rows" \
        "$dest/$arm.trace" finite-input/finite-ordinal.u64 "$dest/$arm.probe.u64" 2>"$dest/$arm.stderr"
done
"$stage/d37-r-fit-finite-host" verify-finite "$dest" | tee "$dest/parity.txt"
(cd "$dest" && sha256sum GO-FINITE.json SHEET-FINITE.md FIT-FINITE.md device.csv a1* truncated.* parity.txt > results.sha256)
