#!/usr/bin/env bash
# Reproduce the prior read artifacts and change only the packed policy profile.
set -euo pipefail
umask 077
[[ $# == 1 ]] || { echo "usage: pack-wick.sh ABS_STAGE" >&2; exit 2; }
stage=$(realpath "$1")
wick=$(dirname "$stage")
study="$wick/20261119-value-ranking/live-rank-d4688456ef77"
sum() { sha256sum "$1" | cut -d' ' -f1; }
pin() { printf '%s  %s\n' "$2" "$1" | sha256sum --status --check; }
cd "$stage"
sha256sum --status --check build-receipt/executables.sha256
test "$(cat "$study/dispatch.status")" = "done 0"
mkdir artifacts
for seed in 1 2; do
    original="$study/spec-$seed.json"
    paired=$(jq -er '.files.paired.path' "$original")
    span=$(jq -er '.files.span.path' "$original")
    pin "$paired" "$(jq -er '.files.paired.sha256' "$original")"
    pin "$span" "$(jq -er '.files.span.sha256' "$original")"
    dir="artifacts/seed$seed"
    mkdir "$dir"
    for mode in ordered position; do
        fit="$study/primary-seed$seed/modes/$mode"
        repeat="$study/repeat-seed$seed/modes/$mode"
        for name in model.bank profile.json model.bank.work/fit.packet model.bank.work/fit.labels; do
            cmp "$fit/$name" "$repeat/$name"
        done
        test "$(stat -c %s "$fit/model.bank")" = 8416
        jq -e --arg mode "$mode" --arg train "$(sum "$fit/model.bank.work/fit.packet")" \
            --arg labels "$(sum "$fit/model.bank.work/fit.labels")" \
            '.mode==$mode and .train_sha256==$train and .labels_sha256==$labels' "$fit/profile.json" >/dev/null
    done
    for mode in ordered ranked veto both position; do
        source_mode=ordered
        [[ "$mode" != position ]] || source_mode=position
        fit="$study/primary-seed$seed/modes/$source_mode"
        profile="$dir/$mode.json"
        artifact="$dir/$mode.d11"
        jq --arg mode "$mode" '.mode=$mode' "$fit/profile.json" > "$profile"
        "$stage/bin/pack-all-starts" "$paired" "$span" "$fit/model.bank" "$profile" "$artifact"
        if [[ "$mode" == "$source_mode" ]]; then cmp "$fit/normal.d11" "$artifact"; fi
    done
    test "$(sha256sum "$dir"/*.d11 | cut -d' ' -f1 | sort -u | wc -l)" = 5
    test "$(stat -c %s "$dir"/*.d11 | sort -u | wc -l)" = 1
done
sha256sum artifacts/seed{1,2}/* > artifacts/identities.sha256
