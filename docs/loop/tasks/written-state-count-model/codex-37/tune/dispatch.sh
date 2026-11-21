#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
stage=$(realpath -e "$1")
test "$0" -ef "$stage/dispatch.sh"
cd "$stage"

check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO.json; }
check GO.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'DEV-tune frozen S5 selection'
check dispatch.sh "$(field dispatch_sha256)"
check SHEET.md "$(field sheet_sha256)"
check FIT.md "$(field fit_sha256)"
check finite.sha256 "$(field finite_sha256)"
sha256sum --check --strict finite.sha256
for binary in d37-dev-tune d37-tune-device; do test -x "$binary"; done
check d37-dev-tune "$(field scorer_sha256)"
check d37-tune-device "$(field device_scorer_sha256)"
check "$(field compact)" "$(field compact_sha256)"
check "$(field trunk)" 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d
check "$(field count_tape)" "$(field dev_tune_tape_sha256)"
check "$(field c2_tape)" "$(field dev_tune_c2_tape_sha256)"

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
test "$(nvidia-smi -i "$gpu" --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 8192
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 10485760
test "$(df --output=avail -B1 "$stage" | awk 'NR == 2 {print $1}')" -ge 2147483648

dest=$(field out)
test "$dest" = "$stage/$(basename "$dest")"
[[ "$(basename "$dest")" =~ ^real-[a-z0-9-]+$ ]]
test ! -e "$dest" && test ! -L "$dest"
logs="$dest.logs"
mkdir "$logs"
cp GO.json SHEET.md FIT.md "$logs/"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > "$logs/device.csv"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
/usr/bin/time -v -o "$logs/score.time" ./d37-dev-tune score "$(field split_root)" \
    "$stage/GO.json" "$2" "$(field compact)" "$(field trunk)" "$(field count_tape)" \
    "$(field c2_tape)" "$stage/d37-tune-device" "$(field fit_root)" "$dest" \
    > "$logs/score.stdout" 2> "$logs/score.stderr"
(cd "$dest" && sha256sum selection.json per-record-q12.json > selection.sha256)
(cd "$logs" && sha256sum GO.json SHEET.md FIT.md device.csv score.{time,stdout,stderr} > logs.sha256)
