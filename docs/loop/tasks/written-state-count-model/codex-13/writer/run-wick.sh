#!/usr/bin/env bash
set -euo pipefail
if [[ ${1:-} == gpu ]]; then
    shift
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    exec "$@"
fi
stage=${1:?primary or repeat}
case "$stage" in primary|repeat) ;; *) exit 2 ;; esac
split=${2:?held, fresh, or nuisance}
case "$split" in held|fresh|nuisance) ;; *) exit 2 ;; esac
seed=${3:?seed 1 or 2}
case "$seed" in 1|2) ;; *) exit 2 ;; esac
cd "$(dirname "$0")"
root=$HOME/uor-codex
study=$root/20261118-query-region-pick
prior=$root/20261118-paired-role-fence
tag=seed$seed-$split
role=$study/region/$stage
for arm in original identity region adverse role-oracle; do
    test -s "$role/$tag-$arm.json"
done
(cd "$role"; sha256sum --status -c decisions.sha256)
(cd "$study/region"; sha256sum --status -c panels/collection.sha256)

sha256sum --status -c PREDECESSORS.sha256
sha256sum --status -c "$prior/fence/identity/INPUTS.sha256"
(cd "$prior/fence/results/$stage/paired-role"; sha256sum --status -c SHA256SUMS)
(cd "$prior/fence/cross-grades/$stage"; sha256sum --status -c SHA256SUMS)
(cd "$prior/role/$stage"; sha256sum --status -c decisions.sha256)
if [[ $split != nuisance ]]; then
    for kind in blind grade; do
        cmp "$study/region/panels/$split-$kind.json" "$prior/role/panels/$split-$kind.json"
    done
fi
mkdir -p "$stage"
out=$PWD/$stage/$tag
mkdir -p logs
/usr/bin/time -v -o "logs/$stage-$tag.time" bin/query-region-writer "$stage" "$split" "$seed" "$root" "$study" "$PWD/run-wick.sh" \
    > "logs/$stage-$tag.log" 2>&1
bash grade-wick.sh "$out" "$study/region/panels/$split-grade.json" "$split"
(cd "$out"; find . -type f -name '*.json' -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
