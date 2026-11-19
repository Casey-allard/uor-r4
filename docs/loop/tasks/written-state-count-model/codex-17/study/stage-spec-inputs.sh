#!/usr/bin/env bash
# Metadata-only study spec assembly. This reads archived hashes and tool/source
# bytes for pins. It never deserializes any prompt, grade, or model outcome.
set -euo pipefail
umask 077
[[ $# == 4 || $# == 8 ]] || { echo "usage: stage-spec-inputs.sh STAGE SEED BINDER DRAFT_OR_SPEC [VALIDITY VALIDITY_SHA REDUCER REDUCER_SHA]" >&2; exit 2; }
stage=$(cd "$1" && pwd -P)
seed="$2"; binder="$3"; out="$4"
wick=$(dirname "$stage")
[[ "$seed" == 1 || "$seed" == 2 ]]
[[ "$out" == /* && ! -e "$out" && "$binder" == /* ]]
sum() { sha256sum "$1" | cut -d' ' -f1; }
take() { jq -en --arg path "$1" --arg sha "$(sum "$1")" '{path:$path,sha256:$sha}'; }
capture() {
    local root="$stage/capture/$1-$2-seed$seed" raw sha
    raw="$root.all.sha256"
    sha=null
    if [[ $# -ge 3 ]]; then
        [[ -d "$root" && -s "$raw" ]] || return 1
        sha=$(sum "$raw") || return 1
    fi
    jq -en --arg root "$root" --arg path "$raw" --arg sha "$sha" \
      '{root:$root,raw:{path:$path,sha256:(if $sha=="null" then null else $sha end)}}'
}
fit="$stage/specs/fit-seed$seed.json"
held="$stage/specs/held-seed$seed.json"
(cd "$stage" && sha256sum --status --check launch.sha256)
prev="$wick/20261118-ordered-role-transfer"
fresh="$wick/20261119-fresh-boundary-validity/onset-public-integration/source/target/release"
args=()
for key in base paired span writer; do
    args+=(--argjson "$key" "$(jq -e --arg key "$key" '.inputs[$key]' "$fit")")
    cmp <(jq -ec --arg key "$key" '.inputs[$key]' "$fit") <(jq -ec --arg key "$key" '.inputs[$key]' "$held")
done
position=(
    action_writer "$wick/20261118-role-boundary-count/bin/action-writer"
    binder "$binder"
    chat "$prev/native-finite/source/target/release/uor-chat"
    common "$stage/src/docs/loop/tasks/written-state-count-model/codex-17/capture/run-wick.sh"
    device "$stage/bin/native-sm89"
    fit_grades "$stage/grades/fit/grades.json"
    fit_stage "$fit"
    grader "$stage/bin/fresh-validity-check"
    held_grades "$stage/grades/held/grades.json"
    held_stage "$held"
    learned_chat "$fresh/uor-chat"
    native "$stage/bin/all-starts-native"
    pack_artifact "$fresh/pack-all-starts"
    reader_a "$stage/grades/v5-reader-a-held.json"
    supplement "$stage/grades/supplement.json"
    tokenizer "$prev/assets/tokenizer-seed$seed.json"
)
for ((i=0;i<${#position[@]};i+=2)); do
    args+=(--argjson "${position[i]}" "$(take "${position[i+1]}")")
done
if [[ $# == 8 ]]; then
    for key in "$5" "$7"; do [[ "$key" == /* ]]; done
    [[ "$6" =~ ^[a-f0-9]{64}$ && "$8" =~ ^[a-f0-9]{64}$ && "$(sum "$5")" == "$6" && "$(sum "$7")" == "$8" ]]
    args+=(--argjson validity "$(take "$5")" --argjson reducer "$(take "$7")")
    draft=false
    pin_capture=(yes)
else
    args+=(--argjson validity null --argjson reducer null)
    draft=true
    pin_capture=()
fi
for split in fit held; do
    for phase in primary repeat; do
        args+=(--argjson "${split}_$phase" "$(capture "$phase" "$split" "${pin_capture[@]}")")
    done
done
jq -en --argjson seed "$seed" --arg wick "$wick" --argjson draft "$draft" "${args[@]}" '
  {schema:"uor-r4.value-rank-study-wick/1",seed:$seed,wick_root:$wick,
   files:($ARGS.named|del(.seed,.wick,.draft,.fit_primary,.fit_repeat,.held_primary,.held_repeat)),
   captures:{fit:{primary:$fit_primary,repeat:$fit_repeat},held:{primary:$held_primary,repeat:$held_repeat}}}
  + (if $draft then {draft_reason:"awaiting independent validity and both authorized raw runs"} else {} end)
' > "$out"
