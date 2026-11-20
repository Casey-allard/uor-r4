#!/usr/bin/env bash
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
stage=$(realpath "${1:?stage}")
go_hash=${2:?GO sha256}
cd "$stage"
check() { printf '%s  %s\n' "$2" "$1" | sha256sum --strict -c -; }
check GO.json "$go_hash"
field() { jq -er --arg key "$1" '.[$key]' GO.json; }
test "$(field decision)" = GO
test "$(field host)" = wick
test "$(field phase)" = 'D36 finite resident compact batch'
test "$(jq -cr '.widths' GO.json)" = '[1,3,10,26]'
test "$(jq -cr '.passes' GO.json)" = '["primary","cold"]'
test "$(field targets)" = 26
check dispatch.sh "$(field dispatch_sha256)"
check SHEET.md "$(field sheet_sha256)"
check FIT.md "$(field fit_sha256)"
check FIT-COLD.md "$(field cold_fit_sha256)"
check source.sha256 "$(field source_sha256)"
check build-and-input.sha256 "$(field build_and_input_sha256)"
check target/release/d36-count-batch "$(field executable_sha256)"
check aot/d36-count-device.2.sm_89.cubin "$(field cubin_sha256)"
check finite/seams.uort "$(field uort_sha256)"
check finite/native.features "$(field oracle_sha256)"
check ../pruner/final/c128-8m.counts "$(field compact_sha256)"
test "$(stat -c %s finite/native.features)" = 3752
sha256sum --check --strict build-and-input.sha256
(cd "$(field source_worktree)" && sha256sum --check --strict "$stage/source.sha256")
dest=$(field output)
test ! -e "$dest"
gpu=$(field device_uuid)
test "$(nvidia-smi --query-gpu=uuid --format=csv,noheader)" = "$gpu"
test "$(nvidia-smi -i "$gpu" --query-gpu=compute_cap --format=csv,noheader)" = 8.9
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -n 8 || { echo 'shared Wick GPU lock busy' >&2; exit 75; }
foreign=$(nvidia-smi -i "$gpu" --query-compute-apps=pid --format=csv,noheader)
if [[ -n "$foreign" ]]; then echo "Wick GPU has active compute: $foreign" >&2; exit 75; fi
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
mkdir "$dest"
cp GO.json "$dest/GO.json"
nvidia-smi -i "$gpu" --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > "$dest/device.csv"
train=$(field train_sha256)
compact=$(field compact_sha256)
uort=$(field uort_sha256)
for width in 1 3 10 26; do
    launches=$(((26 + width - 1) / width))
    for pass in primary cold; do
        out="$dest/batch-$width-$pass"
        /usr/bin/time -v -o "$out.time" target/release/d36-count-batch cuda \
            ../pruner/final/c128-8m.counts "$train" "$compact" finite/seams.uort "$uort" \
            "$out.features" "$width" >"$out.stdout" 2>"$out.stderr"
        test ! -s "$out.stdout"
        grep -Eq "^rows=26 batches=$launches uploads=1 upload_bytes=4517125 upload_ms=[0-9]+ total_ms=[0-9]+ device_free_bytes=[0-9]+ required_device_bytes=[0-9]+$" "$out.stderr"
        cmp finite/native.features "$out.features"
        check "$out.features" "$(field oracle_sha256)"
        test ! -e "$out.features.partial"
    done
    cmp "$dest/batch-$width-primary.features" "$dest/batch-$width-cold.features"
done
(cd "$dest" && sha256sum GO.json device.csv batch-* > receipts.sha256 && sha256sum --check --strict receipts.sha256)
