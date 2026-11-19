#!/usr/bin/env bash
set -euo pipefail
stage=$(realpath "${1:?staged input directory}")
mkdir "${2:?fresh output directory}"
out=$(realpath "$2")
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -w 10 8
test -z "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)"
cd "$stage"
sha256sum --quiet -c inputs.sha256
nvidia-smi --query-gpu=uuid,name,memory.used,driver_version,compute_cap --format=csv > "$out/device.txt"
for split in train held exposed; do
    ./d25-context "$split.ctx" "$out/$split.context.tsv"
    ./d24-project positional tables.bin "$split.local-input.txt" "$out/$split.local"
    # Compare only features here. No held labels or expected onsets are in the stage.
    awk 'NR>1 && $4 {for(i=6;i<=37;i++)printf "%s%s",$i,(i==37?"\n":" ")}' "$split.rows" > "$out/$split.local.expected"
    awk 'NR>1 && $4 && !seen[$1]++ {printf "%s",$1;for(i=38;i<=69;i++)printf " %s",$i;print ""}' "$split.rows" > "$out/$split.context.expected"
    awk '{$1=$1; print}' "$out/$split.context.tsv" > "$out/$split.context.spaces"
    cmp "$out/$split.local" "$out/$split.local.expected"
    cmp "$out/$split.context.spaces" "$out/$split.context.expected"
done
for repeat in primary cold; do
    dir="$out/$repeat"
    mkdir "$dir"
    ./d25-source fit train.rows "$dir/bank.bin"
    for split in held exposed held-swapped-context; do
        ./d25-source predict "$split.rows" "$dir/bank.bin" "$dir/$split.trace" "$dir/$split.results"
    done
done
diff -rq "$out/primary" "$out/cold"
(cd "$out"; find . -type f ! -name observations.sha256 -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > observations.sha256)
