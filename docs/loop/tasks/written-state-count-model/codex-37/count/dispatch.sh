#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C

die() { echo "$*" >&2; exit 1; }
check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg key "$1" '.[$key] | strings | select(length > 0)' GO.json; }
available_host_bytes() { awk '/^MemAvailable:/ {printf "%.0f\n", $2 * 1024}' /proc/meminfo; }

test "$#" -eq 2 || die "usage: dispatch.sh STAGE GO_SHA256"
test "$(hostname -s)" = wick || die "count capture requires Wick"
stage=$(realpath -e "$1")
test "$0" -ef "$stage/dispatch.sh" || die "running script differs from staged dispatcher"
cd "$stage"
check GO.json "$2"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 real-R resident count capture'
test "$(field host)" = wick
check dispatch.sh "$(field dispatch_sha256)"
check SHEET.md "$(field sheet_sha256)"
check FIT.md "$(field fit_sha256)"
check d37-count-replay "$(field replay_sha256)"

runs=/mnt/plexpool/uor-r4-corpus/runs
d36="$runs/20261120-whole-context-36"
run_root="$runs/20261120-pruned-r-fit-37"
dest=$(field output)
test "$(realpath -e "$(dirname "$dest")")" = "$run_root"
test "$dest" = "$run_root/$(basename "$dest")"
[[ "$(basename "$dest")" =~ ^count-[a-z0-9]+(-[a-z0-9]+)*$ ]] ||
    die "choose a fresh count-* directory in the D37 run root"
test ! -e "$dest" && test ! -L "$dest" || die "output already exists"

r="$HOME/uor-codex/20261119-rival-admission-29/count/cr-v1/r.tokens.u16"
compact="$d36/pruned-full-v1/compact.counts"
batch="$d36/batch/target/release/d36-count-batch"
train_sha=cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a
compact_sha=064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8
r_sha=5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a
test "$(stat -c %s "$r")" -eq 31903352
test "$(stat -c %s "$compact")" -eq 3912334481
check "$r" "$r_sha"
check "$compact" "$compact_sha"
check "$batch" d7aeeaf1859e5853bf961ab1bde8f64ba37d61ff4e6923a241518281dc2f9fd3
check "$d36/batch/aot/d36-count-device.2.sm_89.cubin" a29613d9a0c6e3a6fcda13e09c489685b4e70c4e9a11c190f4b8ca389443f6b4
check "$d36/pruned-full-v1/verify.json" 16b7925328f392d085bb5297f2e93131068669b70787b8c5afc1d1a87739c0dd
(
    cd "$d36/batch/gpu-finite"
    check receipts.sha256 ef3ff3a0d377ce25072fcfd41ee8073e5530ceab29d3fa64e08f4758a21b0097
    sha256sum --check --strict receipts.sha256
)

gpu=$(field device_uuid)
test "$gpu" = GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
test -f "$lock"
exec 8<>"$lock"
flock -n 8 || { echo "shared Wick GPU lock busy" >&2; exit 75; }
foreign=$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)
if [[ -n "$foreign" ]]; then
    echo "Wick GPU has active compute: $foreign" >&2
    exit 75
fi
disk_available=$(df --output=avail -B1 "$run_root" | awk 'NR == 2 {print $1}')
host_available=$(available_host_bytes)
gpu_available=$(nvidia-smi -i "$gpu" --query-gpu=memory.free --format=csv,noheader,nounits)
test "$disk_available" -ge 8589934592 || die "less than 8 GiB free output capacity"
test "$host_available" -ge 8589934592 || die "less than 8 GiB available host memory"
test "$gpu_available" -ge 5120 || die "less than 5 GiB free GPU memory"

mkdir "$dest"
cp GO.json SHEET.md FIT.md "$dest/"
printf 'disk_bytes=%s\nhost_bytes=%s\ngpu_mib=%s\n' \
    "$disk_available" "$host_available" "$gpu_available" > "$dest/admission.txt"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free \
    --format=csv > "$dest/device.csv"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1

capture() {
    local name=$1 width=$2 required=$3
    /usr/bin/time -v -o "$dest/$name.time" "$batch" cuda \
        "$compact" "$train_sha" "$compact_sha" "$r" "$r_sha" \
        "$dest/$name.features" "$width" > "$dest/$name.stdout" 2> "$dest/$name.stderr"
    test ! -s "$dest/$name.stdout"
    test ! -e "$dest/$name.features.partial"
    test "$(stat -c %s "$dest/$name.features")" -eq 2232661872
    awk -v required="$required" '
        /^rows=15947584 batches=244 uploads=1 upload_bytes=3912334481 upload_ms=[0-9]+ total_ms=[0-9]+ device_free_bytes=[0-9]+ required_device_bytes=[0-9]+$/ {
            split($7, free, "=")
            split($8, need, "=")
            if ((need[2] + 0) == required && (free[2] + 0) >= required) pass = 1
        }
        END { exit (NR == 1 && pass ? 0 : 1) }
    ' "$dest/$name.stderr"
}
capture primary 65536 4004347553
capture cold 65521 4004341853
flock -u 8
exec 8>&-

if ! cmp "$dest/primary.features" "$dest/cold.features" > "$dest/compare.txt"; then
    cat "$dest/compare.txt" >&2
    awk 'match($0, /byte [0-9]+/) {
        byte = substr($0, RSTART + 5, RLENGTH - 5)
        if (byte > 112) printf "first differing zero-based ordinal %d\n", (byte - 113) / 140
        else print "first difference occurs in the header"
    }' "$dest/compare.txt" >&2
    die "cold count capture differs"
fi

test "$(available_host_bytes)" -ge 8589934592 || die "native readback needs 8 GiB available host memory"
check d37-count-replay "$(field replay_sha256)"
/usr/bin/time -v -o "$dest/native.time" "$stage/d37-count-replay" \
    "$compact" "$train_sha" "$compact_sha" "$r" "$r_sha" "$dest/primary.features" 65536 \
    > "$dest/native.json" 2> "$dest/native.stderr"
primary_sha=$(sha256sum "$dest/primary.features" | cut -d ' ' -f1)
jq -e --arg sha "$primary_sha" '
    .rows == 15947584 and .records == 4060 and .width == 65536 and
    .bytes == 2232661872 and .native_rows == 12500 and
    .words_per_native_row == 23 and .sha256 == $sha
' "$dest/native.json" > /dev/null
(
    cd "$dest"
    sha256sum GO.json FIT.md SHEET.md admission.txt device.csv compare.txt \
        primary.{features,stdout,stderr,time} cold.{features,stdout,stderr,time} \
        native.{json,stderr,time} > capture.sha256.partial
    sha256sum --check --strict capture.sha256.partial
    mv -T capture.sha256.partial capture.sha256
)
