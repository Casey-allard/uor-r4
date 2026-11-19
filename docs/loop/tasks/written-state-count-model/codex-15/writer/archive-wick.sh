#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-ordered-role-transfer"
out=${1:?fresh archive directory}
mkdir "$out"
out=$(realpath "$out")
cmp oracle-primary/cold.sha256 oracle-repeat/cold.sha256
for seed in 1 2; do
    cmp "diagnose-primary/seed$seed/cold.sha256" "diagnose-repeat/seed$seed/cold.sha256"
    for stage in primary repeat; do
        (cd "diagnose-$stage/seed$seed" && sha256sum -c cold.sha256)
    done
done
# Project synthetic panel only. The base models and separate historical corpora
# are identified by checksum. Each ordinary reply includes original prompt IDs.
tar -cf - panel grading assets/SHA256SUMS driver/{oracle-wick.sh,oracle-gate.sh,oracle-pins.sha256,adjudicate-wick.sh,onset.jq,adjudication-pins.sha256} bins/writer-c \
    oracle-{primary,repeat} oracle-{primary,repeat}.{log,time} oracle-gate.log \
    diagnose-{primary,repeat} diagnose-{primary,repeat}-seed{1,2}.{log,time} \
    finite-exposed device/build-d/{sources.sha256,build.sha256,ordered-role-device,ordered-role-sm89,device-sass.txt,nvcc.txt} \
    native-finite/{source-checksums.txt,run-01.sha256,legacy-compare.sha256} native-finite/memory-check/run-0{1,2}/SHA256SUMS \
    | zstd -T2 -8 -o "$out/wick-ordered-role-receipts.tar.zst"
cd "$out"
sha256sum wick-ordered-role-receipts.tar.zst > SHA256SUMS
zstd --test wick-ordered-role-receipts.tar.zst
repo=caseyallard/uor-r4-store
path=results/codex/ordered-role-transfer-20261118/wick-ordered-role-receipts.tar.zst
hf=$HOME/.local/bin/hf
"$hf" upload "$repo" wick-ordered-role-receipts.tar.zst "$path" --repo-type dataset --commit-message 'Preserve requested-span oracle, failed gate and fixed-source Wick adjudication' --format json > upload.json 2> upload.log
revision=$(jq -er '.url|capture("/commit/(?<oid>[0-9a-f]{40})$").oid' upload.json)
"$hf" download "$repo" "$path" --repo-type dataset --revision "$revision" --local-dir readback --force-download --quiet > readback.txt
cmp wick-ordered-role-receipts.tar.zst "readback/$path"
jq -n --arg repo "$repo" --arg revision "$revision" --arg path "$path" --arg sha "$(cut -d' ' -f1 SHA256SUMS)" --argjson bytes "$(stat -c %s wick-ordered-role-receipts.tar.zst)" '{repo:$repo,repo_type:"dataset",revision:$revision,path:$path,bytes:$bytes,sha256:$sha,pinned_download_byte_equal:true}' > store.json
