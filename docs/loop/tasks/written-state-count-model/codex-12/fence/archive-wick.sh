#!/usr/bin/env bash
set -euo pipefail
base=$HOME/uor-codex/20261118-paired-role-fence
cd "$base"
for stage in primary repeat; do
    for domain in historical prior-nominee fresh-oracle paired-role; do
        (cd "fence/results/$stage/$domain"; sha256sum --status -c SHA256SUMS)
    done
    (cd "fence/cross-grades/$stage"; sha256sum --status -c SHA256SUMS)
done
(cd fence/grades; sha256sum --status -c SHA256SUMS)
(cd fence/results/final-verification; sha256sum --status -c SHA256SUMS)
cmp fence/cross-grades/{primary,repeat}/SHA256SUMS
for domain in historical prior-nominee fresh-oracle paired-role; do
    test -f "fence/runlogs/$domain-repeat-diff.txt"
    test ! -s "fence/runlogs/$domain-repeat-diff.txt"
done
mkdir fence/archive
code=fence/tree/docs/loop/tasks/written-state-count-model/codex-12
integer=fence/tree/crates/uor-r4-integer/src
cu=fence/tree/docs/labs/m1-d11-artifact-scorer/instrument/src
tar -cf - fence/results fence/bin fence/d11_tables.h fence/runlogs fence/finite fence/finite.txt \
    fence/finite-final.txt fence/grades fence/cross-grades fence/identity fence/README.md \
    fence/run-wick{,-next,-final}.sh fence/grade{,-cross}-wick.sh fence/verify-final-wick.sh \
    fence/{grade,realized}.jq "$code"/FENCE-{SHEET,FIT}.md "$code"/fence/{device.cu,driver.rs,Cargo.toml,Cargo.lock} \
    "$integer"/{assertion_value.rs,value_selector.rs,continuation_selector.rs,native_geometry.rs} \
    "$cu"/{value.cu,continuation.cu} role/panels \
    writer/results/{held,fresh}/none | pigz -5 > fence/archive/fence-wick.tar.gz
cd fence/archive
sha256sum fence-wick.tar.gz > SHA256SUMS
repo=caseyallard/uor-r4-store
path=results/codex/paired-role-fence-count-20261118/fence-wick.tar.gz
"$HOME/.local/bin/hf" upload "$repo" fence-wick.tar.gz "$path" --repo-type dataset \
    --commit-message 'Preserve assertion fence and paired role writer factorial' --format json > upload.json
revision=$(jq -er '.url | capture("/commit/(?<oid>[0-9a-f]{40})$").oid' upload.json)
"$HOME/.local/bin/hf" download "$repo" "$path" --repo-type dataset --revision "$revision" \
    --local-dir readback --force-download --quiet > readback.txt
cmp fence-wick.tar.gz "readback/$path"
sha256sum "readback/$path" > readback.sha256
printf '%s\n' "$revision" > revision.txt
