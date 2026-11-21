#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
test "$(hostname -s)" = wick
stage=$(realpath -e "$1")
test "$0" -ef "$stage/dispatch.sh"
cd "$stage"

check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO.json; }
check GO.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 R-only S5 fit'
test "$(field device_uuid)" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
check dispatch.sh "$(field dispatch_sha256)"
check SHEET.md "$(field sheet_sha256)"
check FIT.md "$(field fit_sha256)"
check production-stage.sha256 "$(field stage_sha256)"
sha256sum --check --strict production-stage.sha256
finite=$(field finite_result)
test "$finite" = "$stage/$(basename "$finite")"
[[ "$(basename "$finite")" =~ ^finite-result-[a-z0-9-]+$ ]]
check "$finite/results.sha256" "$(field finite_result_sha256)"
(cd "$finite" && sha256sum --check --strict results.sha256)
sha256sum --check --strict finite-stage.sha256

runs=/mnt/plexpool/uor-r4-corpus/runs
capture="$runs/20261120-pruned-r-fit-37/count-v1"
r="$HOME/uor-codex/20261119-rival-admission-29/count/cr-v1/r.tokens.u16"
compact="$runs/20261120-whole-context-36/pruned-full-v1/compact.counts"
trunk="$runs/20261120-complete-c1-31/normal-5ea09ea47/fixtures/seed1.d11"
tape_sha=add9aa306fd70d87e02048e2a2a32d23f6f9e9be89bb8ff8011c64690042f4dd
check "$capture/capture.sha256" b7fd6424e0f13c201836c0686138aafb49231d059f35fb098e80d150a81eefd8
check "$capture/primary.features" "$tape_sha"
check "$r" 5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a
check "$compact" 064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8
check "$trunk" 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d

dest=$(field output)
test "$dest" = "$stage/$(basename "$dest")"
[[ "$(basename "$dest")" =~ ^real-[a-z0-9-]+$ ]]
test ! -e "$dest" && test ! -L "$dest"
mode=$(field pick_mode)
case "$mode" in
    identity) arms=(a1 a1s) ;;
    *) exit 2 ;;
esac
gpu=$(field device_uuid)
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
test -f "$lock"
exec 8<>"$lock"
flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
test "$(nvidia-smi -i "$gpu" --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 8192
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 10485760
test "$(df --output=avail -B1 "$stage" | awk 'NR == 2 {print $1}')" -ge 8589934592
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
mkdir "$dest"
cp GO.json SHEET.md FIT.md "$dest/"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > "$dest/device.csv"

out="$dest/$mode"
host="$stage/d37-r-fit-production"
device="$stage/d37-r-device"
/usr/bin/time -v -o "$dest/prepare.time" "$host" prepare "$mode" "$r" "$trunk" "$compact" \
    "$capture/primary.features" "$tape_sha" "$out" 2>"$dest/prepare.stderr"
check "$out/probe-ordinals.u64" "$(jq -er .probe_ordinals_sha256 "$out/prepare.json")"
check "$out/pick.rows" "$(jq -er .pick_tape_sha256 "$out/prepare.json")"
for arm in "${arms[@]}"; do
    check "$out/$arm.rows" "$(jq -er --arg arm "$arm" '.arms[$arm].rows_sha256' "$out/prepare.json")"
    /usr/bin/time -v -o "$dest/$arm.time" "$device" "$out/$arm.rows" "$out/$arm.trace" \
        "$out/probe-ordinals.u64" "$out/$arm.probe.u64" 2>"$dest/$arm.stderr"
    "$host" seal "$out" "$mode" "$arm" | tee "$out/$arm.seal.json"
done
(cd "$dest" && sha256sum GO.json SHEET.md FIT.md device.csv *.stderr *.time "$mode"/* > results.sha256)
