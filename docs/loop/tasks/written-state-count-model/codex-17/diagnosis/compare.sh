#!/usr/bin/env bash
# Read the protected raw results only after the v2 cold pair was released.
set -euo pipefail
umask 077
here=$(cd "$(dirname "$0")" && pwd -P)
raw=$(cd "$1" && pwd -P)
evidence=$(cd "$2" && pwd -P)
mkdir "$3"
out=$(cd "$3" && pwd -P)
for seed in 1 2; do
    primary="$raw/primary-seed$seed"
    repeat="$raw/repeat-seed$seed"
    (cd "$primary" && shasum -a 256 --status -c cold.sha256)
    (cd "$repeat" && shasum -a 256 --status -c cold.sha256)
    cmp "$primary/cold.sha256" "$repeat/cold.sha256"
    test "$(shasum -a 256 "$primary/cold.sha256" | cut -d' ' -f1)" = \
         "$(jq -r .verified_cold_manifest_sha256 "$evidence/results-seed$seed.json")"
    cd "$primary"
    shasum -a 256 "$evidence/results-seed$seed.json" cold.sha256 > "$out/seed$seed.inputs.sha256"
    groups=()
    for group in chain/level-*-chunk-*; do
        nominees="$group/nominees"
        group_inputs=("$group/primary/seed$seed-panel-nominees.json"
                      "$nominees/nomination.json" "$nominees/sites.json" "$nominees/selection.json"
                      "$nominees/chosen.json")
        for path in "${group_inputs[@]}"; do cmp "$path" "$repeat/$path"; done
        ranks=$(jq -r .max "$nominees/sites.json")
        for ((rank=0; rank<ranks; rank++)); do
            ranked=("$nominees/rank-$rank-chain.json" "$nominees/rank-$rank-cont.json"
                    "$nominees/rank-$rank-writer.json")
            for path in "${ranked[@]}"; do cmp "$path" "$repeat/$path"; done
            group_inputs+=("${ranked[@]}")
        done
        shasum -a 256 "${group_inputs[@]}" >> "$out/seed$seed.inputs.sha256"
        packed="$out/seed$seed-${group##*/}.json"
        jq -ces --arg group "$group" '
          {group:$group,endpoint:.[0],nomination:.[1],sites:.[2],selection:.[3],chosen:.[4],
           ranks:[. as $src|range(5;length;3) as $i|
              {rank:(($i-5)/3),chain:$src[$i],cont:$src[$i+1],writer:$src[$i+2]}]}' \
          "${group_inputs[@]}" > "$packed"
        groups+=("$packed")
    done
    jq -es -f "$here/rows.jq" "$evidence/results-seed$seed.json" requested.json "${groups[@]}" \
       > "$out/rows-seed$seed.json"
done
jq -es '
   (.[0] | map(del(.seed))) == (.[1] | map(del(.seed)))' \
   "$out/rows-seed1.json" "$out/rows-seed2.json"
