#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 3
mode=$1
stage=$(realpath -e "$2")
test "$0" -ef "$stage/tape-dispatch.sh"
cd "$stage"

case "$mode" in
    finite) phase='D37 finite frozen scorer'; suffix=FINITE ;;
    c2) phase='D37 DEV-tune C2 tape'; suffix=C2 ;;
    *) echo 'expected finite or c2' >&2; exit 2 ;;
esac
go="GO-$suffix.json"
sheet="SHEET-$suffix.md"
fit="FIT-$suffix.md"
check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' "$go"; }
check "$go" "$3"
test "$(field decision)" = GO
test "$(field phase)" = "$phase"
check tape-dispatch.sh "$(field dispatch_sha256)"
check "$sheet" "$(field sheet_sha256)"
check "$fit" "$(field fit_sha256)"
check d37-dev-tune "$(field scorer_sha256)"
if [[ "$mode" == finite ]]; then
    check d37-tune-device "$(field device_scorer_sha256)"
else
    check d37-c2-device "$(field c2_selector_sha256)"
    c2_finite=$(field c2_finite_result)
    check "$c2_finite/results.sha256" "$(field c2_finite_result_sha256)"
    (cd "$c2_finite" && sha256sum --check --strict results.sha256)
    split=$(field split_root)
    check "$split/dev-tune.tokens.u16" 75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a
fi
dest=$(field out)
test "$dest" = "$stage/$(basename "$dest")"
[[ "$(basename "$dest")" =~ ^(finite|c2)-[a-z0-9-]+$ ]]
test ! -e "$dest" && test ! -L "$dest"
gpu=$(field device_uuid)
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
case "$(field host_kind)" in
    wick)
        test "$(hostname -s)" = wick
        test "$gpu" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
        lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
        test -f "$lock"
        exec 8<>"$lock"
        flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
        ;;
    vast)
        test "${CUDA_VISIBLE_DEVICES:-}" = 0
        test "$(hostname -s)" = "$(field host_name)"
        test "$(nvidia-smi -i 0 --query-gpu=uuid --format=csv,noheader)" = "$gpu"
        test "$(nvidia-smi -i 0 --query-gpu=name --format=csv,noheader)" = 'NVIDIA GeForce RTX 4090'
        test /proc/self/fd/20 -ef /root/gpu0.lock
        exec 8</root/gpu0.lock
        if flock -n 8; then
            echo 'uor-pod GPU lease lock is not held' >&2
            exit 75
        fi
        ;;
    *) echo 'unadmitted execution host' >&2; exit 2 ;;
esac
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
test "$(nvidia-smi -i "$gpu" --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 4194304
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1

mkdir "$dest"
cp "$go" "$sheet" "$fit" "$dest/"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > "$dest/device.csv"
if [[ "$mode" == finite ]]; then
    /usr/bin/time -v -o "$dest/device.time" ./d37-dev-tune finite "$stage/d37-tune-device" \
        > "$dest/parity.json" 2> "$dest/device.stderr"
else
    /usr/bin/time -v -o "$dest/device.time" ./d37-c2-device "$split/dev-tune.tokens.u16" \
        "$dest/c2.picks.u32" > "$dest/device.stdout" 2> "$dest/device.stderr"
    c2_sha=$(sha256sum "$dest/c2.picks.u32" | cut -d ' ' -f1)
    ./d37-dev-tune verify-c2 "$split" "$dest/c2.picks.u32" "$c2_sha" > "$dest/parity.json"
fi
(cd "$dest" && sha256sum -- * > results.sha256.partial && mv -T results.sha256.partial results.sha256)
