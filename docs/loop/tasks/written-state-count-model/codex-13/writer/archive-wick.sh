#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
verify=${1:?verified output directory}
archive=${2:?fresh archive directory}
test -s "$verify/SHA256SUMS"
(cd "$verify"; sha256sum --status -c SHA256SUMS)
for primary in primary/*/SHA256SUMS; do
    cmp "$primary" "repeat/${primary#primary/}"
done
mkdir "$archive"
sha256sum Cargo.toml Cargo.lock driver.rs *.sh grade.jq PREDECESSORS.sha256 bin/query-region-writer > "$archive/instrument.sha256"
tar -cf - Cargo.toml Cargo.lock driver.rs *.sh grade.jq PREDECESSORS.sha256 bin/query-region-writer \
    primary repeat finite logs "$verify" "$archive/instrument.sha256" | zstd -T2 -8 > "$archive/writer-wick.tar.zst"
cd "$archive"
sha256sum writer-wick.tar.zst > SHA256SUMS
zstd --test writer-wick.tar.zst
repo=caseyallard/uor-r4-store
path=results/codex/query-region-pick-20261118/writer-wick.tar.zst
hf=$HOME/.local/bin/hf
"$hf" upload "$repo" writer-wick.tar.zst "$path" --repo-type dataset \
    --commit-message 'Preserve frozen query-region CUDA writer replays and controls' --format json > upload.json 2> upload.log
revision=$(jq -er '.url | capture("/commit/(?<oid>[0-9a-f]{40})$").oid' upload.json)
"$hf" download "$repo" "$path" --repo-type dataset --revision "$revision" \
    --local-dir readback --force-download --quiet > readback.txt
cmp writer-wick.tar.zst "readback/$path"
sha256sum "readback/$path" > readback.sha256
printf '%s\n' "$revision" > revision.txt
