#!/usr/bin/env bash
# Seal a new D19 stage using exact upstream carriers and freshly tokenized blind text.
set -euo pipefail
umask 077
[[ $# == 1 && "$1" == /* ]]
stage=$(realpath "$1")
wick=$(dirname "$stage")
prior="$wick/20261119-factorial"
runner=$(cd "$(dirname "$0")" && pwd -P)
source_root=$(dirname "$runner")
sum() { sha256sum "$1" | cut -d' ' -f1; }
pin() { printf '%s  %s\n' "$2" "$1" | sha256sum --status --check; }
entry() {
    local digest
    digest=$(sum "$2") || return
    jq -nc --arg key "$1" --arg path "$2" --arg sha "$digest" '{key:$key,value:{path:$path,sha256:$sha}}'
}
cd "$stage"
pin "$prior/inputs.sha256" 0fdd7b2967f960625b512e27517f94b9a0e4cd5cdac8e286053d215a5f847c70
(cd "$prior" && sha256sum --status --check inputs.sha256)
(cd panel && sha256sum --status --check INPUTS.sha256)
sha256sum --status --check build-receipt/source.sha256
sha256sum --status --check build-receipt/executables.sha256
mkdir artifacts specs prepared
for seed in 1 2; do
    old="$prior/specs/seed$seed.json"
    old_file() {
        local path sha
        path=$(jq -er --arg key "$1" '.files[$key].path' "$old") || return
        sha=$(jq -er --arg key "$1" '.files[$key].sha256' "$old") || return
        pin "$path" "$sha" || return
        printf '%s\n' "$path"
    }
    held="$stage/panel/seed$seed/held"
    tokenizer=$(old_file tokenizer)
    paired=$(old_file paired)
    span=$(old_file span)
    bank=$(old_file bank_ordered)
    profile=$(old_file profile_ordered)
    baseline=$(old_file artifact_ordered)
    mkdir "artifacts/seed$seed"
    rebuilt="$stage/artifacts/seed$seed/rebuilt-ordered.d11"
    treatment="$stage/artifacts/seed$seed/assertion-agreement.d11"
    "$stage/bin/pack-all-starts" "$paired" "$span" "$bank" "$profile" "$rebuilt"
    cmp "$baseline" "$rebuilt"
    "$stage/bin/pack-all-starts" "$paired" "$span" "$bank" "$profile" "$treatment" \
        --requested-policy assertion_agreement "$tokenizer"
    [[ "$(sum "$rebuilt")" != "$(sum "$treatment")" ]]
    capture=$(old_file capture)
    "$capture" prepare "$tokenizer" "$(sum "$tokenizer")" "$held/public-text.json" \
        "$(sum "$held/public-text.json")" "$stage/panel/held-names.json" \
        "$(sum "$stage/panel/held-names.json")" "$stage/prepared/seed$seed"
    blind="$stage/prepared/seed$seed/blind.json"
    resolved="$stage/prepared/seed$seed/resolved-requested.json"
    cmp "$blind" "$held/blind.json"
    cmp "$resolved" "$stage/panel/seed$seed/prepared/resolved-requested.json"
    labels="$held/grades.json"
    jq -en --slurpfile g "$labels" --slurpfile r "$resolved" --slurpfile b "$blind" '
      ($g[0]|length)==70 and ($r[0].rows|length)==70 and ($b[0]|length)==70
      and ($g[0]|map(select(.stratum=="supported"))|length)==64
      and all(range(70); . as $i | $g[0][$i] as $grade | $r[0].rows[$i] as $role
         | $role.index==$i and $role.prefix_sha256==$grade.prefix_sha256
         and $role.requested_span==$grade.requested
         and (if $grade.stratum=="supported"
              then $grade.requested!=null and $grade.excluded!=null and $grade.requested!=$grade.excluded
              else $grade.expected==null end))
    ' >/dev/null
    (
        set -e
        for key in base paired span writer action_writer tokenizer binder native device prior_chain; do
            path=$(old_file "$key")
            entry "$key" "$path"
        done
        for name in uor-chat requested-role-device requested-role-sm89 project-role fresh-validity-check; do
            entry "$name" "$stage/bin/$name"
        done
        entry stage_inputs "$runner/stage-inputs.sh"
        entry dispatcher "$runner/dispatch.sh"
        entry runner "$runner/run-seed.sh"
        entry common "$runner/common.sh"
        entry reducer "$source_root/analysis/reduce.jq"
        entry controls "$source_root/analysis/controls.jq"
        entry capture "$capture"
        entry bank "$bank"
        entry profile "$profile"
        entry old_spec "$old"
        entry artifact_ordered "$baseline"
        entry artifact_rebuilt "$rebuilt"
        entry artifact_treatment "$treatment"
        entry blind "$blind"
        entry resolved "$resolved"
        entry labels "$labels"
        entry readings "$stage/panel/reader-grader.json"
    ) > "specs/seed$seed.entries"
    jq -es --argjson seed "$seed" --arg wick "$wick" '
        select((map(.key)|length)==(map(.key)|unique|length)
            and all(.[]; .value.sha256 | test("^[0-9a-f]{64}$")))
        | {schema:"uor-r4.requested-occurrence/1",seed:$seed,wick_root:$wick,files:from_entries}
    ' "specs/seed$seed.entries" > "specs/seed$seed.json"
    test -s "specs/seed$seed.json"
done
{
    sha256sum specs/seed{1,2}.json build-receipt/{source,executables}.sha256 panel/INPUTS.sha256
    (cd panel && awk '{print $2}' INPUTS.sha256 | sed 's|^|panel/|') | xargs sha256sum
    jq -r '.files[] | "\(.sha256)  \(.path)"' specs/seed{1,2}.json | LC_ALL=C sort -u
} > inputs.sha256
sha256sum --status --check inputs.sha256
