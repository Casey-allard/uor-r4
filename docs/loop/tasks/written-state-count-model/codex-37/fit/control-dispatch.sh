#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
test "$#" -eq 2
test "$(hostname -s)" = wick
stage=$(realpath -e "$1")
test "$0" -ef "$stage/control-dispatch.sh"
cd "$stage"

check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO-CONTROLS.json; }
check GO-CONTROLS.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 R-only S5 controls'
test "$(field device_uuid)" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
check control-dispatch.sh "$(field dispatch_sha256)"
check SHEET-CONTROLS.md "$(field sheet_sha256)"
check FIT-CONTROLS.md "$(field fit_sha256)"
check control-stage.sha256 "$(field stage_sha256)"
sha256sum --check --strict control-stage.sha256
check finite-result/results.sha256 "$(field masked_finite_result_sha256)"
(cd finite-result && sha256sum --check --strict results.sha256)
check c2-finite-result/results.sha256 "$(field c2_finite_result_sha256)"
(cd c2-finite-result && sha256sum --check --strict results.sha256)

fit=$(dirname "$stage")
identity_run=$(field identity_run)
test "$identity_run" = "$fit/$(basename "$identity_run")"
[[ "$(basename "$identity_run")" =~ ^real-[a-z0-9-]+$ ]]
check "$identity_run/results.sha256" "$(field identity_result_sha256)"
(cd "$identity_run" && sha256sum --check --strict results.sha256)
identity="$identity_run/identity"
identity_pin=$(field identity_preparation_sha256)
check "$identity/prepare.json" "$identity_pin"
test "$(jq -r '.prepared.pick_mode' "$identity/a1.manifest.json")" = identity

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
gpu=$(field device_uuid)
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
test -f "$lock"
exec 8<>"$lock"
flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
test "$(nvidia-smi -i "$gpu" --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 8192
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 10485760
test "$(df --output=avail -B1 "$stage" | awk 'NR == 2 {print $1}')" -ge 21474836480
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
mkdir "$dest"
cp GO-CONTROLS.json SHEET-CONTROLS.md FIT-CONTROLS.md "$dest/"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > "$dest/device.csv"

host="$stage/d37-r-fit-control"
for mode in disabled exact masked permuted; do
    out="$dest/$mode"
    device="$fit/d37-r-device"
    arms=(a1)
    case "$mode" in
        masked)
            device="$stage/d37-r-device-masked"
            /usr/bin/time -v -o "$dest/$mode-prepare.time" "$host" prepare-masked "$identity" "$identity_pin" "$out" 2>"$dest/$mode-prepare.stderr" ;;
        exact)
            /usr/bin/time -v -o "$dest/c2-selector.time" "$stage/d37-c2-device" "$r" "$dest/c2.picks.u32" 2>"$dest/c2-selector.stderr"
            c2_sha=$(sha256sum "$dest/c2.picks.u32" | cut -d " " -f1)
            /usr/bin/time -v -o "$dest/$mode-prepare.time" "$host" prepare-c2 "$r" "$trunk" "$compact" \
                "$capture/primary.features" "$tape_sha" "$dest/c2.picks.u32" "$c2_sha" "$out" 2>"$dest/$mode-prepare.stderr" ;;
        *)
            /usr/bin/time -v -o "$dest/$mode-prepare.time" "$host" prepare "$mode" "$r" "$trunk" "$compact" \
                "$capture/primary.features" "$tape_sha" "$out" 2>"$dest/$mode-prepare.stderr"
            if [[ "$mode" == permuted ]]; then arms+=(a1s); fi ;;
    esac
    check "$out/probe-ordinals.u64" "$(jq -er .probe_ordinals_sha256 "$out/prepare.json")"
    for arm in "${arms[@]}"; do
        check "$out/$arm.rows" "$(jq -er --arg arm "$arm" '.arms[$arm].rows_sha256' "$out/prepare.json")"
        /usr/bin/time -v -o "$dest/$mode-$arm.time" "$device" "$out/$arm.rows" "$out/$arm.trace" \
            "$out/probe-ordinals.u64" "$out/$arm.probe.u64" 2>"$dest/$mode-$arm.stderr"
        "$host" seal "$out" "$mode" "$arm" | tee "$out/$arm.seal.json"
    done
done
(cd "$dest" && sha256sum GO-CONTROLS.json SHEET-CONTROLS.md FIT-CONTROLS.md device.csv c2.picks.u32 *.stderr *.time {disabled,exact,masked,permuted}/* > results.sha256)
