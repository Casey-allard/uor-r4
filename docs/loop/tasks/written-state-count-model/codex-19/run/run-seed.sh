#!/usr/bin/env bash
# A cold seed replay. Run only through the sealed dispatcher.
set -euo pipefail
umask 077
[[ $# == 2 && "$1" == /* && "$2" == /* ]]
source "$(dirname "$0")/common.sh"
spec=$1
out=$2
file() {
    local path hash
    path=$(jq -er --arg key "$1" '.files[$key].path' "$spec")
    hash=$(jq -er --arg key "$1" '.files[$key].sha256' "$spec")
    pin "$path" "$hash"
    printf '%s\n' "$path"
}
jq -e '.schema=="uor-r4.requested-occurrence/1" and (.seed==1 or .seed==2)' "$spec" >/dev/null
wick=$(jq -er .wick_root "$spec"); seed=$(jq -er .seed "$spec")
lock="$wick/20261118-diagnostics/wick-codex-gpu.lock"
writer=$(file writer); action=$(file action_writer); capture=$(file capture)
base=$(file base); paired=$(file paired); span=$(file span); tokenizer=$(file tokenizer)
blind=$(file blind); resolved=$(file resolved); binder=$(file binder); labels=$(file labels)
readers=$(file readings); grader=$(file fresh-validity-check); project=$(file project-role)
chat=$(file uor-chat); role=$(file requested-role-device); role_device=$(file requested-role-sm89)
native=$(file native); value_device=$(file device); bank=$(file bank)
ordered=$(file artifact_ordered); treatment=$(file artifact_treatment)
reducer=$(file reducer); controls=$(file controls); prior_chain=$(file prior_chain)
(cd "$wick/20261118-ordered-role-transfer" && sha256sum --status --check "$prior_chain")

mkdir "$out"
cd "$out"
mkdir capture arms modes grades clients
cp "$spec" spec.json
phase=setup
finalize() {
    local status=$?
    trap - EXIT
    printf '%s %s\n' "$phase" "$status" > status
    find . -type f ! -name raw.sha256 -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > raw.sha256
    exit "$status"
}
trap finalize EXIT
gpu() {
    local name="$1"
    shift
    (
        flock 8
        local free host
        free=$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)
        host=$(awk '$1=="MemAvailable:"{print $2}' /proc/meminfo)
        [[ "$free" -ge 4096 && "$host" -ge 8388608 ]]
        /usr/bin/time -v -o "$name.time" "$@" >"$name.log" 2>&1
    ) 8>>"$lock"
}
nvidia-smi --query-gpu=uuid,name,driver_version,memory.free --format=csv,noheader > device.txt

phase=blind-role
"$writer" select "$paired" "$blind" public - - capture/ordered.json
"$writer" select "$paired" "$blind" no-read - - capture/no-read.json
gpu capture/role-ordered "$role" "$ordered" "$blind" "$role_device" capture/role-ordered.json
gpu capture/role-treatment "$role" "$treatment" "$blind" "$role_device" capture/role-treatment.json
jq -en --arg artifact "$(sum "$ordered")" --slurpfile r capture/role-ordered.json --slurpfile p capture/ordered.json '
    $r[0].schema=="uor-r4.requested-role-device/1" and $r[0].policy=="legacy"
    and $r[0].artifact==$artifact and $r[0].blind==$p[0].blind
    and ($r[0].rows|map({index,prefix_sha256,role_words}))
        ==($p[0].rows|map({index,prefix_sha256,role_words:.original_role_words}))
' > capture/role-baseline.json
"$project" "$paired" "$(sum "$treatment")" "$blind" capture/ordered.json capture/role-treatment.json capture/treatment.json
jq -en --slurpfile r capture/role-treatment.json --slurpfile p capture/role-ordered.json '
    ($r[0].rows|map(.role_words[:2]))!=($p[0].rows|map(.role_words[:2]))
    and ($r[0].rows|map(.role_words[2:4]))==($p[0].rows|map(.role_words[2:4]))
' > capture/role-treatment-exercised.json

phase=endpoint-controls
"$capture" requested "$paired" "$blind" capture/ordered.json "$resolved" "$(sum "$resolved")" capture/supplied.json
jq -c --slurpfile g "$labels" '
    .rows |= map(.index as $i | $g[0][$i] as $grade
      | if .prefix_sha256!=$grade.prefix_sha256 then error("wrong endpoint tape shifted")
        elif $grade.stratum=="supported" then .requested_span=$grade.excluded
        else .requested_span=null end)
' "$resolved" > capture/wrong-role-endpoints.json
"$capture" requested "$paired" "$blind" capture/ordered.json capture/wrong-role-endpoints.json \
    "$(sum capture/wrong-role-endpoints.json)" capture/wrong-role.json
# The historical fence and agreement chain takes the Wick advisory lock internally.
/usr/bin/time -v -o capture/chain.time "$writer" chain "$wick" "$seed" "$blind" capture/chain \
    capture/{ordered,treatment,supplied,wrong-role}.json > capture/chain.log 2>&1
for arm in ordered treatment supplied wrong-role; do
    extras=()
    if [[ "$arm" == supplied || "$arm" == wrong-role ]]; then
        endpoints="$resolved"
        [[ "$arm" != wrong-role ]] || endpoints=capture/wrong-role-endpoints.json
        extras=(capture/ordered.json "$endpoints" "$(sum "$endpoints")")
    fi
    "$capture" capture "$base" "$paired" "$span" "$blind" "capture/$arm.json" capture/chain "capture/$arm" "${extras[@]}"
done
jq -e --arg mode force-gold --arg selection_sha "$(sum capture/supplied.json)" --slurpfile capture capture/supplied/candidates.json \
    --slurpfile labels "$labels" -f "$controls" capture/supplied.json > arms/forced-gold.json
jq -e --arg mode force-wrong --arg selection_sha "$(sum capture/wrong-role.json)" --slurpfile capture capture/wrong-role/candidates.json \
    --slurpfile labels "$labels" -f "$controls" capture/wrong-role.json > arms/forced-wrong.json
jq -e --arg mode wrong-key --arg selection_sha "" --slurpfile capture /dev/null --slurpfile labels /dev/null -f "$controls" "$labels" > grades/wrong-key-input.json
gpu control-writer "$writer" replay "$action" "$base" "$blind" control-writer arms/{forced-gold,forced-wrong}.json
mkdir control-writer/nominees
ln control-writer/gpu.json control-writer/nominees/chosen.json

grade() {
    local name=$1 selection=$2 replay=$3 public=$4 key=${5:-$labels} original=${6:--}
    "$writer" grade "$selection" "$replay" "$key" "$public" "grades/$name-writer.json"
    "$grader" "$tokenizer" "$key" "$readers" "$original" "grades/$name-writer.json" "grades/$name.json" 70 64
}
grade forced-gold arms/forced-gold.json control-writer -
grade forced-wrong arms/forced-wrong.json control-writer -
grade wrong-key arms/forced-gold.json control-writer - grades/wrong-key-input.json "$labels"
jq -en --slurpfile gold grades/forced-gold.json --slurpfile wrong grades/wrong-key.json \
    --slurpfile rival grades/forced-wrong.json --slurpfile rival_key grades/wrong-key-input.json '
    $gold[0].counts.rows==64 and $gold[0].counts.independently_exact==64
    and $rival[0].counts.rows==64 and $rival[0].counts.independently_exact==0
    and all($rival[0].rows[]; .index as $i
        | .source!=null and .terminal_eos and .generated==$rival_key[0][$i].expected)
    and $wrong[0].counts.independently_exact==64 and $wrong[0].counts.active_label_exact==0
    and $gold[0].gpu_sha256==$wrong[0].gpu_sha256
    and $gold[0].selection_sha256==$wrong[0].selection_sha256
    and $gold[0].generated_id_tape_sha256==$wrong[0].generated_id_tape_sha256
' > grades/controls-verified.json

phase=frozen-value-device
for arm in ordered treatment supplied wrong-role; do
    cap="capture/$arm/candidates.json"
    m="modes/$arm"
    mkdir "$m"
    "$native" pack-json "$cap" "$(jq -er .base_d11 "$cap")" \
        "$(jq -er .tokenizer_sha256 "$cap")" "$("$native" site-key "$cap")" "$m/input.packet"
    gpu "$m/score" "$native" score-as "$value_device" "$m/input.packet" "$bank" ordered "$m/score.trace"
    "$native" decode-trace-as "$m/input.packet" "$bank" ordered "$m/score.trace" "$m/sources.json"
    jq -en --arg bank "$(sum "$bank")" --arg cap "$(sum "$cap")" \
        --arg packet "$(sum "$m/input.packet")" --arg trace "$(sum "$m/score.trace")" \
        --slurpfile view "$m/sources.json" --slurpfile receipt "$m/score.trace.work/bindings.json" '
        $view[0].mode=="ordered"
        and all([$view[0],$receipt[0]][]; .bank_sha256==$bank and .input_packet_sha256==$packet
            and .blind_capture_sha256==$cap and .trace_sha256==$trace)
    ' > "$m/verified.json"
    "$binder" rewrite ordered "$cap" "$blind" "capture/$arm.json" "$m/sources.json" \
        "arms/$arm-blind.json" "arms/$arm-source-audit.json"
    artifact="$ordered"
    [[ "$arm" != treatment ]] || artifact="$treatment"
    if [[ "$arm" == supplied || "$arm" == wrong-role ]]; then
        jq -ec --slurpfile g "$labels" '
            .rows |= map(select(.index as $i | $g[0][$i].stratum=="supported"))
        ' "arms/$arm-blind.json" > "arms/$arm-supported.json"
    else
        ln "arms/$arm-blind.json" "arms/$arm-supported.json"
    fi
    jq -c --arg artifact "$(sum "$artifact")" --arg origin "$(sum "arms/$arm-blind.json")" \
        '.artifact=$artifact | .original_blind_selection_sha256=$origin' "arms/$arm-supported.json" > "arms/$arm.json"
done
jq -c --arg artifact "$(sum "$treatment")" '.artifact=$artifact' capture/no-read.json > arms/no-read.json
replay=(arms/{ordered,treatment,supplied,wrong-role,no-read}.json)
sha256sum "${replay[@]}" > blind-selections.sha256

phase=writer-replay
gpu writer "$writer" replay "$action" "$base" "$blind" writer "${replay[@]}"
mkdir writer/nominees
ln writer/gpu.json writer/nominees/chosen.json
sha256sum --status --check blind-selections.sha256

phase=normal-clients
for arm in ordered treatment no-read; do
    artifact="$treatment"; read_mode=public
    [[ "$arm" != ordered ]] || artifact="$ordered"
    [[ "$arm" != no-read ]] || read_mode=no-read
    /usr/bin/time -v -o "clients/$arm.time" "$writer" public "$chat" "$artifact" "$tokenizer" \
        "$blind" "$read_mode" "clients/$arm" > "clients/$arm.log" 2>&1
done

phase=client-parity-and-grade
for arm in ordered treatment no-read; do
    grade "$arm" "arms/$arm.json" writer "clients/$arm"
done
for arm in supplied wrong-role; do
    grade "$arm" "arms/$arm.json" writer -
done

phase=paired-summary
jq -en --argjson seed "$seed" --slurpfile labels "$labels" \
    --slurpfile ordered grades/ordered-writer.json --slurpfile treatment grades/treatment-writer.json \
    --slurpfile supplied grades/supplied-writer.json --slurpfile wrong_role grades/wrong-role-writer.json \
    --slurpfile no_read grades/no-read-writer.json \
    --slurpfile ordered_candidates capture/role-ordered.json \
    --slurpfile treatment_candidates capture/role-treatment.json \
    -f "$reducer" > summary.json
{
    sha256sum blind-selections.sha256 summary.json capture/{ordered,treatment,supplied,wrong-role,no-read,role-ordered,role-treatment,role-baseline,role-treatment-exercised,wrong-role-endpoints}.json \
        capture/{ordered,treatment,supplied,wrong-role}/candidates.json
    find arms modes grades clients writer control-writer -type f \( -name '*.json' -o -name '*.trace' -o -name '*.packet' -o -name '*.raw' \) \
        -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
} > cold.sha256
sha256sum --status --check cold.sha256
phase=done
