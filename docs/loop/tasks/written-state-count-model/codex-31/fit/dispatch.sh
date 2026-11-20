#!/usr/bin/env bash
set -euo pipefail
umask 077

if [[ $# != 7 ]]; then
    echo "usage: dispatch.sh FIT_HOST FIT_DEVICE SEALED_CR_DIR TRUNK COMPACT_COUNTS COMPACT_SHA256 NEW_OUT" >&2
    exit 2
fi
host=$1 device=$2 cr=$3 trunk=$4 counts=$5 count_sha=$6 out=$7
build=$(cd "$(dirname "$host")/../.." && pwd -P)
sha() { sha256sum "$1" | cut -d ' ' -f1; }

test ! -e "$out"
test ! -e "$out-resources"
sha256sum --strict -c GO.sha256
(cd "$build" && sha256sum --strict -c stage.sha256 build.sha256 finite.sha256)
grep -qx 'TEST FITNESS: FIT' FIT.md
gpu=$(nvidia-smi --query-gpu=uuid --format=csv,noheader)
jq -e --arg gpu "$gpu" --arg count "$count_sha" --arg c1 "$(sha "$cr/c1.tokens.u16")" \
    --arg r "$(sha "$cr/r.tokens.u16")" --arg trunk "$(sha "$trunk")" \
    --arg host "$(sha "$host")" --arg device "$(sha "$device")" \
    --arg stage "$(sha "$build/stage.sha256")" --arg build "$(sha "$build/build.sha256")" \
    --arg machine "$(sha "$build/machine.sha256")" --arg finite "$(sha "$build/finite.sha256")" \
    --arg fit "$(sha FIT.md)" --arg dispatch "$(sha "$0")" '
    .schema == "uor-r4.d31.a1-go/1" and .decision == "GO" and .lab == "codex"
    and (.pod | test("^vast-[0-9]+$")) and .lease == "codex/count31-repaired-chain"
    and .gpu == $gpu and .off_pod_c1_verified == true and .compact_sha256 == $count
    and .c1_sha256 == $c1 and .r_sha256 == $r and .trunk_sha256 == $trunk
    and .host_sha256 == $host and .device_sha256 == $device
    and .stage_sha256 == $stage and .build_sha256 == $build
    and .machine_sha256 == $machine and .finite_sha256 == $finite
    and .fit_sha256 == $fit and .dispatch_sha256 == $dispatch
' GO-A1.json >/dev/null
test "$(sha "$counts")" = "$count_sha"
test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 12.0
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 20480
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 25165824
test "$(df -Pk "$(dirname "$out")" | awk 'NR == 2 {print $4}')" -ge 6291456
mkdir "$out-resources"
/usr/bin/time -v -o "$out-resources/prepare.resources.txt" \
    "$host" prepare "$cr" "$trunk" "$counts" "$count_sha" "$out"
printf '%s  %s\n' "$(jq -er '.rows_sha256' "$out/prepare.json")" "$out/train.rows" | sha256sum --strict -c
printf '%s  %s\n' "$(jq -er '.probe_ordinals_sha256' "$out/prepare.json")" "$out/probe-ordinals.u64" | sha256sum --strict -c
/usr/bin/time -v -o "$out-resources/fit.resources.txt" \
    "$device" "$out/train.rows" "$out/fit.trace" "$out/probe-ordinals.u64" "$out/probe-masses.u64"
"$host" seal "$out" | tee "$out-resources/seal.json"
sha256sum "$out"/* "$out-resources"/{prepare.resources.txt,fit.resources.txt,seal.json} > "$out-resources/results.sha256"
