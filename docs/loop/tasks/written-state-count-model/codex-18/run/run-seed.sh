#!/usr/bin/env bash
# Called only by the SHA- and FIT-gated dispatch; it reconstructs a seed from immutable inputs.
set -euo pipefail
umask 077
[[ $# == 2 && "$1" == /* && "$2" == /* ]]
spec=$1; out=$2
sum() { sha256sum "$1" | cut -d' ' -f1; }
file() {
    local path hash
    path=$(jq -er --arg key "$1" '.files[$key].path' "$spec")
    hash=$(jq -er --arg key "$1" '.files[$key].sha256' "$spec")
    [[ "$path" == /* && "$hash" =~ ^[0-9a-f]{64}$ ]]
    printf '%s  %s\n' "$hash" "$path" | sha256sum --status --check
    printf '%s\n' "$path"
}
jq -e '.schema=="uor-r4.value-start-factorial/1" and (.seed==1 or .seed==2)' "$spec" >/dev/null
wick=$(jq -er .wick_root "$spec"); seed=$(jq -er .seed "$spec")
lock="$wick/20261118-diagnostics/wick-codex-gpu.lock"
writer=$(file writer); action=$(file action_writer); cap=$(file capture); base=$(file base)
paired=$(file paired); span=$(file span); chat=$(file chat); tokenizer=$(file tokenizer)
blind=$(file blind); resolved=$(file resolved); binder=$(file binder); grader=$(file grader)
labels=$(file grades); readers=$(file readings); native=$(file native); device=$(file device)
reducer=$(file reducer); prior_chain=$(file prior_chain)
(cd "$wick/20261118-ordered-role-transfer" && sha256sum --status --check "$prior_chain")
mkdir "$out"
cd "$out"
mkdir capture arms modes grades clients
cp "$spec" spec.json
phase=capture
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
"$writer" select "$paired" "$blind" public - - capture/public.json
"$writer" select "$paired" "$blind" no-read - - capture/no-read.json
"$cap" requested "$paired" "$blind" capture/public.json "$resolved" "$(sum "$resolved")" capture/requested.json
# The unchanged capture chain acquires the same device lock internally.
# Paths are relative to the seed root so receipts remain stable across cold runs.
/usr/bin/time -v -o capture/chain.time "$writer" chain "$wick" "$seed" "$blind" capture/chain \
    capture/public.json capture/no-read.json capture/requested.json > capture/chain.log 2>&1
"$cap" capture "$base" "$paired" "$span" "$blind" capture/public.json capture/chain capture/public
"$cap" capture "$base" "$paired" "$span" "$blind" capture/requested.json capture/chain capture/requested \
    capture/public.json "$resolved" "$(sum "$resolved")"
public_cap=capture/public/candidates.json
requested_cap=capture/requested/candidates.json

phase=controls
"$binder" verify-input-rows "$blind" "$requested_cap" "$labels" -
gate=$(file fit_input_gate)
[[ "$(file fit_input_gate_jq)" == "$(dirname "$gate")/input.jq" ]]
bash "$gate" "$(file fit_capture)" "$requested_cap" "$(file fit_blind)" "$blind" "$(file fit_grades)" \
    "$(file supplement)" "$labels" capture/captured-ceiling.json
jq -e '.eligible_for_deciding_run and .fit.source_in_rank_domain==88
       and .held.source_in_rank_domain==36 and .held.original_literal_reachable==36
       and .matched.rows==24 and .matched.confirmed_structural_ceiling==8' \
    capture/captured-ceiling.json >/dev/null
"$binder" membership "$labels" capture/matched-membership.json
ordered=$(file bank_ordered); position=$(file bank_position)
for mode in ordered position; do
    bank="$ordered"
    [[ "$mode" != position ]] || bank="$position"
    "$binder" availability "$mode" "$bank" "$requested_cap" capture/matched-membership.json \
        "capture/$mode-availability.json"
done
jq -es 'all(.[]; .no_scoring and .matched_ready==24 and .allocated_bytes==8416)
        and (map(.used_records) | unique | length)==1' capture/{ordered,position}-availability.json \
    > capture/frozen-support.json
gpu_scope_args=(--query-compute-apps=pid,process_name,used_memory --format=csv,noheader)
nvidia-smi "${gpu_scope_args[@]}" > capture/observed-after-structural-gate.txt
"$binder" gold-reach "$requested_cap" "$labels" grades/gold-reach.json
forced=(forced-gold forced-rival-1 forced-rival-2)
forced_paths=()
for arm in "${forced[@]}"; do
    "$binder" force "${arm#forced-}" "$requested_cap" "$blind" capture/requested.json "$labels" \
        "arms/$arm.json" "arms/$arm-plan.json"
    forced_paths+=("arms/$arm.json")
done
gpu control-writer "$writer" replay "$action" "$base" "$blind" control-writer "${forced_paths[@]}"
mkdir control-writer/nominees
ln control-writer/gpu.json control-writer/nominees/chosen.json
grade() {
    local name=$1 selection=$2 replay=$3 public=$4
    "$writer" grade "$selection" "$replay" "$labels" "$public" "grades/$name-writer.json"
    "$grader" "$tokenizer" "$labels" "$readers" - "grades/$name-writer.json" "grades/$name.json"
}
for arm in "${forced[@]}"; do
    grade "$arm" "arms/$arm.json" control-writer -
    "$binder" verify-forced "arms/$arm-plan.json" "grades/$arm-writer.json" "grades/$arm-verified.json"
done
"$binder" wrong-key "$labels" grades/wrong-key-input.json
"$writer" grade arms/forced-gold.json control-writer grades/wrong-key-input.json - grades/wrong-key-writer.json
"$grader" "$tokenizer" grades/wrong-key-input.json "$readers" "$labels" grades/wrong-key-writer.json grades/wrong-key.json
jq -en --slurpfile gold grades/forced-gold.json --slurpfile wrong grades/wrong-key.json '
    $gold[0].counts.independently_exact==36 and $wrong[0].counts.independently_exact==36
    and $wrong[0].counts.active_label_exact==0 and $gold[0].gpu_sha256==$wrong[0].gpu_sha256
    and $gold[0].selection_sha256==$wrong[0].selection_sha256
    and $gold[0].generated_id_tape_sha256==$wrong[0].generated_id_tape_sha256
' > grades/wrong-key-verified.json

phase=score
pack() {
    local cap=$1 packet=$2
    "$native" pack-json "$cap" "$(jq -er .base_d11 "$cap")" \
        "$(jq -er .tokenizer_sha256 "$cap")" "$("$native" site-key "$cap")" "$packet"
}
pack "$public_cap" modes/input.packet
pack "$requested_cap" modes/requested.packet
policies=(ordered ranked veto both position)
selections=()
score() {
    local mode=$1 bank=$2 packet=$3 cap=$4 original=$5 m=$6 arm=$7
    gpu "$m/score" "$native" score-as "$device" "$packet" "$bank" "$mode" "$m/score.trace"
    "$native" decode-trace-as "$packet" "$bank" "$mode" "$m/score.trace" "$m/sources.json"
    jq -en --arg bank "$(sum "$bank")" --arg cap "$(sum "$cap")" --arg mode "$mode" \
        --arg packet "$(sum "$packet")" --arg trace "$(sum "$m/score.trace")" \
        --slurpfile view "$m/sources.json" --slurpfile receipt "$m/score.trace.work/bindings.json" '
        $view[0].mode==$mode
        and all([$view[0],$receipt[0]][]; .bank_sha256==$bank and .input_packet_sha256==$packet
            and .blind_capture_sha256==$cap and .trace_sha256==$trace)
    ' > "$m/verified.json"
    "$binder" rewrite "$mode" "$cap" "$blind" "$original" "$m/sources.json" \
        "arms/$arm-blind.json" "arms/$arm-source-audit.json"
}
for mode in "${policies[@]}"; do
    bank="$ordered"
    [[ "$mode" != position ]] || bank="$position"
    artifact=$(file "artifact_$mode")
    m="modes/$mode"
    mkdir "$m" "$m/requested"
    ln -s "$artifact" "$m/policy.d11"
    score "$mode" "$bank" modes/input.packet "$public_cap" capture/public.json "$m" "$mode"
    score "$mode" "$bank" modes/requested.packet "$requested_cap" capture/requested.json "$m/requested" "requested-$mode"
    jq -c --arg artifact "$(sum "$artifact")" --arg origin "$(sum "arms/$mode-blind.json")" \
        '.artifact=$artifact | .original_blind_selection_sha256=$origin' "arms/$mode-blind.json" > "arms/$mode.json"
    "$binder" supported-view "$labels" "arms/requested-$mode-blind.json" "arms/requested-$mode.json"
    selections+=("arms/$mode.json" "arms/requested-$mode-blind.json" "arms/requested-$mode.json")
done
jq -c --arg artifact "$(sum "$(file artifact_both)")" '.artifact=$artifact' capture/no-read.json > arms/no-read.json
selections+=(arms/no-read.json)
sha256sum "${selections[@]}" > blind-selections.sha256

phase=writer-replay
gpu writer "$writer" replay "$action" "$base" "$blind" writer "${selections[@]}"
mkdir writer/nominees
ln writer/gpu.json writer/nominees/chosen.json
sha256sum --status --check blind-selections.sha256

phase=normal-clients
workers=()
for arm in "${policies[@]}" no-read; do
    mode=$arm; read_mode=public
    if [[ "$arm" == no-read ]]; then mode=both; read_mode=no-read; fi
    /usr/bin/time -v -o "clients/$arm.time" "$writer" public "$chat" "modes/$mode/policy.d11" "$tokenizer" \
        "$blind" "$read_mode" "clients/$arm" > "clients/$arm.log" 2>&1 &
    workers+=("$!")
done
status=0
for worker in "${workers[@]}"; do wait "$worker" || status=1; done
test "$status" = 0

phase=client-parity-and-grade
for arm in "${policies[@]}" no-read; do
    grade "$arm" "arms/$arm.json" writer "clients/$arm"
done
for mode in "${policies[@]}"; do
    grade "requested-$mode" "arms/requested-$mode.json" writer -
done
phase=paired-summary
summary_arm() {
    local arm=$1
    name=$arm
    native_view=/dev/null; audit=/dev/null
    if [[ -f "modes/$arm/sources.json" ]]; then
        native_view="modes/$arm/sources.json"; audit="arms/$arm-source-audit.json"
        name="learned-normal-$arm"
    elif [[ "$arm" == no-read ]]; then
        name=frozen-normal-no-read
    elif [[ "$arm" == requested-* ]]; then
        native_view="modes/${arm#requested-}/requested/sources.json"; audit="arms/$arm-source-audit.json"
    fi
    jq -c --arg name "$name" --slurpfile native "$native_view" --slurpfile audit "$audit" '{
        arm:$name,report:.,native_sources:$native[0],source_audit:$audit[0]
    }' "grades/$arm.json"
}
for arm in "${policies[@]}" no-read "${forced[@]}" wrong-key; do summary_arm "$arm"; done |
    jq -es --slurpfile labels "$labels" -f "$reducer" |
    jq -e 'def short: sub("^learned-normal-";"") | sub("^frozen-normal-";"");
        .arms[].arm |= short | .paired_exact[] |= (.left |= short | .right |= short)' > summary.json
for mode in "${policies[@]}"; do summary_arm "requested-$mode"; done |
    jq -es --slurpfile labels "$labels" -f "$reducer" > requested-summary.json
{
    sha256sum blind-selections.sha256 capture/{public,no-read,requested,captured-ceiling,frozen-support,ordered-availability,position-availability}.json \
        capture/{public,requested}/candidates.json
    find arms modes grades clients writer control-writer -type f \( -name '*.json' -o -name '*.trace' -o -name '*.packet' -o -name '*.raw' \) \
        -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
    sha256sum summary.json requested-summary.json
} > cold.sha256
sha256sum --status --check cold.sha256
phase=done
