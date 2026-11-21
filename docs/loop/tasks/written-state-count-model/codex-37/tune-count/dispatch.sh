#!/usr/bin/env bash
set -euo pipefail
umask 077
export LC_ALL=C
base=/mnt/plexpool/uor-r4-corpus/runs
r="$base/20261120-pruned-r-fit-37"
q="$base/20261120-whole-context-36"
stage=$(realpath -e "${1:?stage}")
cd "$stage"
check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
field() { jq -er --arg k "$1" '.[$k]' GO.json; }
test "$(hostname -s)" = wick
test "$0" -ef "$stage/dispatch.sh"
check GO.json "${2:?GO digest}"
test "$(field decision)" = GO
test "$(field phase)" = 'D37 DEV-tune count capture'
check dispatch.sh "$(field dispatch_sha256)"; check SHEET.md "$(field sheet_sha256)"; check FIT.md "$(field fit_sha256)"
gpu=GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41
out=$(field output)
test "$(dirname "$out")" = "$r"
[[ $(basename "$out") =~ ^tune-count-v[0-9]+$ ]]
test ! -e "$out"
test ! -L "$out"
compact="$q/pruned-full-v1/compact.counts"; train=cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a
cs=064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8
us=75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a
uort="$r/tune/c420/dev-tune.tokens.u16"; dev="$q/batch/target/release/d36-count-batch"; native="$r/count-stage/d37-count-replay"
check "$dev" d7aeeaf1859e5853bf961ab1bde8f64ba37d61ff4e6923a241518281dc2f9fd3
check "$native" 53435175c783ab644da6e2aa7e80dd0f0efdf5c5702a1b54c4d1b6875ccb4fdd
check "$compact" "$cs"; check "$uort" "$us"; check "$r/tune/c420/dev-tune.json" 90fbc73a7b19129d39860cece7403027f71d6d952a6825e89bd1619dac3f8467
check "$r/count-v1/capture.sha256" b7fd6424e0f13c201836c0686138aafb49231d059f35fb098e80d150a81eefd8
grep -q 'native mismatch ordinal 0' "$r/replay-evidence/neg.log"
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"; test -f "$lock"; exec 8<>"$lock"; flock -n 8 || exit 75
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
test -z "$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)"
test "$(nvidia-smi -i "$gpu" --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 5120
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 8388608
test "$(df --output=avail -B1 "$r" | awk 'NR==2{print $1}')" -ge 8589934592
mkdir "$out"; cp GO.json SHEET.md FIT.md "$out/"; cd "$out"
export CUDA_DISABLE_PTX_JIT=1 CUDA_VISIBLE_DEVICES="$gpu"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,memory.free,driver_version --format=csv > device.csv
n=10807417
for width in 65536 65521; do
    /usr/bin/time -v -o "$width.time" "$dev" cuda "$compact" "$train" "$cs" "$uort" "$us" "$width.features" "$width" > "$width.stdout" 2> "$width.stderr"
    test ! -s "$width.stdout"
test "$(stat -c %s "$width.features")" -eq $((112 + 140*n))
    need=$((3912334481 + 64*1024*1024 + 11*6*8 + 380*width))
    awk -v k="$(((n+width-1)/width))" -v need="$need" '
        /^rows=10807417 batches=[0-9]+ uploads=1 upload_bytes=3912334481 upload_ms=[0-9]+ total_ms=[0-9]+ device_free_bytes=[0-9]+ required_device_bytes=[0-9]+$/ {split($7,f,"="); split($8,r,"="); ok=($2 == "batches=" k && r[2]+0==need && f[2]+0>=need)} END{exit !(NR==1 && ok)}' "$width.stderr"
done
flock -u 8; exec 8>&-
cmp 65536.features 65521.features
# Enumerate probe ordinals using only origin IDs and record boundaries.
expected=$(od -An -v -j64 -tu2 "$uort" | awk '{for(i=1;i<=NF;i++){v=$i+0;if(v==0){frames++;first=1;continue} if(first || v==1 || n%65536==0 || n%65536==65535 || n%4093==197) seen++;first=0;n++}}END{if(n!=10807417 || frames!=2670) exit 1;print seen}')
/usr/bin/time -v -o native.time "$native" "$compact" "$train" "$cs" "$uort" "$us" 65536.features 65536 > native.json
sum=$(sha256sum 65536.features | cut -d ' ' -f1)
jq -e --arg sha "$sum" --argjson expect "$expected" '.sha256==$sha and .native_rows==$expect and .words_per_native_row==23 and .rows==10807417 and .records==2670 and .width==65536 and .bytes==(112+140*10807417)' native.json >/dev/null
sha256sum -- GO.json SHEET.md FIT.md device.csv *.features *.time *.stdout *.stderr native.json > receipt.partial
sha256sum --check --strict receipt.partial && mv receipt.partial receipt.sha256
