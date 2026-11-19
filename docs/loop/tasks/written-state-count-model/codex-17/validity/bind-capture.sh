#!/usr/bin/env bash
# Input custody only. Do not open learned banks, predicted sources or grades.
set -euo pipefail
umask 077
[[ $# == 7 ]] || { echo "usage: bind-capture.sh STAGE SEED primary|repeat DRAFT DRAFT_SHA SEALED_RAW_INDEX NEW_MANIFEST" >&2; exit 2; }
stage=$(cd "$1" && pwd -P)
seed=$2; phase=$3; draft=$4; index=$6; out=$7
[[ "$seed" =~ ^[12]$ && "$phase" =~ ^(primary|repeat)$ && "$5" =~ ^[0-9a-f]{64}$ && "$out" == /* && ! -e "$out" ]]
here=$(cd "$(dirname "$0")" && pwd -P)
hash() { sha256sum "$1" | cut -d' ' -f1; }
[[ "$(hash "$draft")" == "$5" ]]
(cd "$stage" && sha256sum --status -c stages.sha256 && sha256sum --status -c launch.sha256)
declare -a fields=()
bind() {
    local k=$1 p=$2 sha=$3
    [[ "$p" == /* && "$sha" =~ ^[0-9a-f]{64}$ && "$(hash "$p")" == "$sha" ]]
    fields+=(--argjson "$k" "$(jq -n --arg path "$p" --arg sha256 "$sha" '{path:$path,sha256:$sha256}')")
}
# Key hashes are independent of the host running this join, recorded by the
# blind author and semantic packet before the model ran.
bind fg "$stage/grades/fit/grades.json" 4d84c611a7f612e15e5830a0289373ecdac559649767704487bff38db79f275b
bind hg "$stage/grades/held/grades.json" 0ebbc7a52393134c2769b1d843d1be49d0733a5f57baa1fea090ea68d86f773b
bind extra "$stage/grades/supplement.json" 7d8ddb929d58b75e20b523f1509ff4a8c740dbd792e3eb8a6c69e162153961e1
bind binder "$(jq -er '.files.binder.path' "$draft")" "$(jq -er '.files.binder.sha256' "$draft")"
bind checker "$here/check-inputs.sh" "$(hash "$here/check-inputs.sh")"
bind query "$here/input.jq" "$(hash "$here/input.jq")"
for split in fit held; do
    code=${split:0:1}
    spec="$stage/specs/$split-seed$seed.json"
    root="$stage/capture/$phase-$split-seed$seed"
    [[ "$(hash "$spec")" == "$(jq -er --arg k "${split}_stage" '.files[$k].sha256' "$draft")" ]]
    cmp "$root/stage-spec.json" "$spec"
    pin=$(awk -v k="$phase-$split-seed$seed.all.sha256" '$2==k{print $1}' "$index")
    [[ "$pin" =~ ^[0-9a-f]{64}$ && "$(hash "$root.all.sha256")" == "$pin" ]]
    file=./requested-diagnostic/candidates.json
    bind "${code}c" "$root/${file#./}" "$(awk -v k="$file" '$2==k{print $1}' "$root.all.sha256")"
    bind "${code}b" "$(jq -er .inputs.blind.path "$spec")" "$(jq -er .inputs.blind.sha256 "$spec")"
done
set -C
jq -n "${fields[@]}" '{schema:"uor-r4.rank-validity-inputs/1",files:$ARGS.named}' > "$out"
