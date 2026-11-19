#!/usr/bin/env bash
# Authenticate a completed pair before exposing either archived capture to the
# deciding launcher. Recompute the raw inventory without opening paths named
# by the raw inventory. No recapture and no parsing of model outcomes.
set -euo pipefail
umask 077
[[ $# == 3 ]] || { echo "usage: reuse-capture.sh ABS_SPEC primary|repeat ABS_STUDY_DIR" >&2; exit 2; }
spec=$1
selection=$2
out=$3
[[ "$spec" == /* && "$out" == /* && ( "$selection" == primary || "$selection" == repeat ) ]]
sum() { sha256sum "$1" | cut -d' ' -f1; }
check() {
    local split="$1" phase="$2" stage_spec="$3" root manifest expected
    root=$(jq -er --arg s "$split" --arg p "$phase" '.captures[$s][$p].root' "$spec") || return 1
    manifest=$(jq -er --arg s "$split" --arg p "$phase" '.captures[$s][$p].raw.path' "$spec") || return 1
    expected=$(jq -er --arg s "$split" --arg p "$phase" '.captures[$s][$p].raw.sha256' "$spec") || return 1
    [[ "$root" == /* && "$manifest" == "$root.all.sha256" && "$expected" =~ ^[0-9a-f]{64}$ ]] || return 1
    [[ -d "$root" && ! -L "$root" && "$(sum "$manifest")" == "$expected" ]] || return 1
    [[ -z "$(find "$root" -type l -print -quit)" ]] || return 1
    cmp "$stage_spec" "$root/stage-spec.json" || return 1
    (cd "$root" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) | cmp - "$manifest" || return 1
    # The independent raw inventory above binds the cold manifest. Check it
    # against bytes only after we know the entire tree matched its custody.
    (
        cd "$root" || exit 1
        # Require only in-root relative path entries in this subordinate index.
        awk 'NF!=2 || $2 ~ /^\// || $2 ~ /(^|\/)\.\.(\/|$)/ { exit 1 }' cold.sha256 || exit 1
        sha256sum --status --check cold.sha256
    ) || return 1
    printf '%s\n' "$root"
}
for split in fit held; do
    stage_spec=$(jq -er --arg key "${split}_stage" '.files[$key].path' "$spec")
    expected=$(jq -er --arg key "${split}_stage" '.files[$key].sha256' "$spec")
    [[ "$stage_spec" == /* && "$(sum "$stage_spec")" == "$expected" ]]
    first=$(check "$split" primary "$stage_spec")
    second=$(check "$split" repeat "$stage_spec")
    [[ "$first" != "$second" ]]
    cmp "$first/cold.sha256" "$second/cold.sha256"
    chosen="$first"
    if [[ "$selection" == repeat ]]; then chosen="$second"; fi
    ln -s "$chosen" "$out/$split"
    jq -n --arg split "$split" --arg primary "$first" --arg repeat "$second" \
        --arg common_spec_sha256 "$expected" \
        --arg primary_raw_sha256 "$(sum "$first.all.sha256")" \
        --arg repeat_raw_sha256 "$(sum "$second.all.sha256")" \
        --arg cold_sha256 "$(sum "$first/cold.sha256")" \
        '{split:$split,primary:$primary,repeat:$repeat,common_spec_sha256:$common_spec_sha256,
          primary_raw_sha256:$primary_raw_sha256,repeat_raw_sha256:$repeat_raw_sha256,cold_sha256:$cold_sha256,
          all_raw_files_verified:true,cold_pair_identical:true}' > "$out/reused-$split.json"
done
printf '%s\n' "$selection" > "$out/reused-phase.txt"
