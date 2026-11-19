#!/usr/bin/env bash
set -euo pipefail
top=$HOME/uor-codex/20261118-paired-role-fence
cd "$top/fence"
stage=${1:?primary or repeat}
case "$stage" in primary|repeat) ;; *) exit 2;; esac
out_root=cross-grades/$stage
mkdir -p cross-grades
mkdir "$out_root"
for n in 1 2; do
    for split in held fresh; do
        tag=seed$n-$split
        base=$top/role/panels/$split
        fallback=$top/writer/results/$split/none/seed$n/$stage.json
        jq -en --slurpfile b "$base-blind.json" --slurpfile g "$base-grade.json" '$b[0]==[$g[0][]|{prefix}]' >/dev/null
        for role in prior-nominee paired-role; do
            for boundary in old fenced; do
                dir=results/$stage/$role/$tag/$boundary
                out=$out_root/$tag-$role-$boundary-writer.json
                jq -sc --slurpfile select "$dir/selection.json" --slurpfile fallback "$fallback" \
                    '{fallback:$fallback[0],ranks:.,select:$select}' "$dir"/rank-*-writer.json |
                    jq -ce -f realized.jq > "$out"
            done
            for mode in rows summary; do
                jq -ce --arg mode "$mode" --slurpfile old "$out_root/$tag-$role-old-writer.json" \
                    --slurpfile grade "$base-grade.json" -f grade.jq \
                    "$out_root/$tag-$role-fenced-writer.json" > "$out_root/$tag-$role-$mode.json"
            done
        done
        for boundary in old fenced; do
            for mode in rows summary; do
                jq -ce --arg mode "$mode" --slurpfile old "$out_root/$tag-prior-nominee-$boundary-writer.json" \
                    --slurpfile grade "$base-grade.json" -f grade.jq \
                    "$out_root/$tag-paired-role-$boundary-writer.json" > "$out_root/$tag-$boundary-role-contrast-$mode.json"
            done
        done
    done
done
(cd "$out_root"; sha256sum *.json > SHA256SUMS)
