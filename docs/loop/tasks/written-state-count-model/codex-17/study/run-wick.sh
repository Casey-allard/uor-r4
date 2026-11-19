#!/usr/bin/env bash
# No protected command is run until the explicit pre-run receipt and stage
# spec have both been independently pinned by the top session.
set -euo pipefail
umask 077
[[ $# == 6 ]] || { echo "usage: run-wick.sh ABS_SPEC SPEC_SHA ABS_GO GO_SHA primary|repeat ABS_NEW_DIR" >&2; exit 2; }
spec="$1"; spec_sha="$2"; go="$3"; go_sha="$4"; capture_phase="$5"; out="$6"
runner_dir=$(cd "$(dirname "$0")" && pwd -P)
sum() { sha256sum "$1" | cut -d' ' -f1; }
pin() {
    [[ "$1" == /* && "$2" =~ ^[0-9a-f]{64}$ ]]
    printf '%s  %s\n' "$2" "$1" | sha256sum --check --status
}
[[ "$out" == /* && ( "$capture_phase" == primary || "$capture_phase" == repeat ) ]]
pin "$spec" "$spec_sha"
pin "$go" "$go_sha"
jq -e --arg spec "$spec_sha" '
  .schema=="uor-r4.value-rank-pre-run/1" and .decision=="GO" and .spec_sha256==$spec
  and (.non_author_fit_sha256|test("^[0-9a-f]{64}$"))
  and (.finite_controls_sha256|test("^[0-9a-f]{64}$"))
' "$go" >/dev/null
jq -e '
  ["action_writer","base","binder","chat","common","device","fit_grades","fit_stage",
    "grader","held_grades","held_stage","native","paired","reader_a","reducer","span","supplement","tokenizer","validity","writer"]
     as $required |
  .schema=="uor-r4.value-rank-study-wick/1" and (.seed==1 or .seed==2)
  and (.wick_root|startswith("/"))
  and (.captures|keys)==["fit","held"]
  and all(.captures[];keys==["primary","repeat"])
  and all(.captures[][];
       (keys==["raw","root"]) and (.root|startswith("/"))
       and (.raw|keys)==["path","sha256"] and (.raw.sha256|test("^[0-9a-f]{64}$")))
  and ((.files|keys)==$required or (.files|keys)==($required+["learned_chat","pack_artifact"]|sort))
  and all(.files[]; keys==["path","sha256"] and (.path|startswith("/") and (test("[\\r\\n]")|not))
    and (.sha256|test("^[0-9a-f]{64}$")))
' "$spec" >/dev/null
resolve() {
    local filename want
    filename=$(jq -er --arg name "$1" '.files[$name].path' "$spec")
    want=$(jq -er --arg name "$1" '.files[$name].sha256' "$spec")
    pin "$filename" "$want"
    printf '%s\n' "$filename"
}
binder=$(resolve binder); common=$(resolve common)
native=$(resolve native); device=$(resolve device); writer=$(resolve writer)
action=$(resolve action_writer); chat=$(resolve chat); grader=$(resolve grader)
learned_chat=""; pack_artifact=""
if jq -e '.files.learned_chat!=null' "$spec" >/dev/null; then
    learned_chat=$(resolve learned_chat)
    pack_artifact=$(resolve pack_artifact)
fi
base=$(resolve base); paired=$(resolve paired); tokenizer=$(resolve tokenizer); span=$(resolve span)
fit_stage=$(resolve fit_stage); held_stage=$(resolve held_stage)
fit_grades=$(resolve fit_grades); supplement=$(resolve supplement)
held_grades=$(resolve held_grades); reader=$(resolve reader_a)
validity=$(resolve validity); reducer=$(resolve reducer)
wick=$(jq -er .wick_root "$spec")
seed=$(jq -er .seed "$spec")
lock="$wick/20261118-diagnostics/wick-codex-gpu.lock"
jq -en --argjson seed "$seed" --arg wick "$wick" --arg base "$(sum "$base")" --arg paired "$(sum "$paired")" \
  --arg writer "$(sum "$writer")" --arg span "$(sum "$span")" \
  --slurpfile fit "$fit_stage" --slurpfile held "$held_stage" '
  all([$fit[0],$held[0]][]; .seed==$seed and .wick_root==$wick and .inputs.base.sha256==$base
      and .inputs.paired.sha256==$paired and .inputs.writer.sha256==$writer and .inputs.span.sha256==$span)
  and $fit[0].inputs.blind.sha256!=$held[0].inputs.blind.sha256
' >/dev/null
mkdir "$out"
cd "$out"
cp "$spec" study-spec.json
cp "$go" authorization.json
phase=setup
finalize() {
    local status=$?
    trap - EXIT
    jq -n --arg phase "$phase" --argjson code "$status" '{phase:$phase,exit_code:$code}' > status.json
    # Semantic hashes are independent of elapsed times and absolute paths.
    {
        find . -type f \( -name '*.packet' -o -name '*.bank' -o -name '*.trace' -o -name '*.json' -o -name cold.sha256 \) \
          ! -name archive-audit.json ! -name study-cold.sha256 ! -name study-raw.sha256 -print |
          LC_ALL=C sort | xargs -r sha256sum
    } > study-cold.sha256
    find . -type f ! -name study-raw.sha256 -print | LC_ALL=C sort | xargs -r sha256sum > study-raw.sha256
    exit "$status"
}
trap finalize EXIT
gpu() {
    local output="$1"
    shift
    (
        flock 8
        local free host
        free=$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)
        host=$(awk '$1=="MemAvailable:"{print $2}' /proc/meminfo)
        [[ "$free" -ge 4096 && "$host" -ge 8388608 ]]
        /usr/bin/time -v -o "$output.time" "$@" >"$output.log" 2>&1
    ) 8>"$lock"
}
nvidia-smi --query-gpu=name,driver_version,memory.free --format=csv,noheader,nounits > device.txt
fit_blind=$(jq -er '.inputs.blind.path' "$fit_stage")
held_blind=$(jq -er '.inputs.blind.path' "$held_stage")
check_shell() {
    local path="$1" expect="$2"
    [[ "$(sum "$path")" == "$expect" ]]
}
pack() {
    local candidates="$1" packet="$2" base_sha tok_sha site
    base_sha=$(jq -er .base_d11 "$candidates")
    tok_sha=$(jq -er .tokenizer_sha256 "$candidates")
    site=$("$native" site-key "$candidates")
    "$native" pack-json "$candidates" "$base_sha" "$tok_sha" "$site" "$packet"
}
record() {
    local mode="$1" status="$2" reason="$3"
    jq -n --arg mode "$mode" --arg status "$status" --arg reason "$reason" \
        '{mode:$mode,status:$status,reason:$reason}' > "$out/modes/$mode/status.json"
}

phase=capture-reuse
"$runner_dir/reuse-capture.sh" "$spec" "$capture_phase" "$out"
fit_cap="$out/fit/requested-diagnostic/candidates.json"
held_cap="$out/held/requested-diagnostic/candidates.json"
phase=independent-admissibility
# These three keys are the independently sealed labels used by the input
# validity manifest. Reject a self-consistent study spec carrying other keys.
pin "$fit_grades" 4d84c611a7f612e15e5830a0289373ecdac559649767704487bff38db79f275b
pin "$supplement" 7d8ddb929d58b75e20b523f1509ff4a8c740dbd792e3eb8a6c69e162153961e1
pin "$held_grades" 0ebbc7a52393134c2769b1d843d1be49d0733a5f57baa1fea090ea68d86f773b
pin "$fit_blind" "$(jq -er .blind_sha256 "$fit_cap")"
pin "$held_blind" "$(jq -er .blind_sha256 "$held_cap")"
jq -e --arg fit_sha "$(sum "$fit_cap")" --arg held_sha "$(sum "$held_cap")" '
  .schema=="uor-r4.value-rank-input-validity/1" and .fit.capture_sha256==$fit_sha
  and .held.capture_sha256==$held_sha and .fit.rows==94 and .held.rows==42
  and .fit.supported==88 and .held.supported==36 and .matched.rows==24
  and .eligible_for_deciding_run==true
' "$validity" > independent-admissibility.json
phase=fit-label-join
"$binder" fit-labels "$fit_cap" "$fit_grades" "$supplement" "$out/fit-labels.json"
pack "$fit_cap" fit.packet
mkdir modes arms
phase=bank-fits
for mode in ordered neutral position; do
    m="$out/modes/$mode"
    mkdir "$m"
    if gpu "$m/fit" "$native" fit "$device" "$out/fit.packet" "$out/fit-labels.json" "$mode" "$m/model.bank"; then
        [[ "$(stat -c '%s' "$m/model.bank")" == 8416 ]]
    else
        # Only the device's declared whole-bank fit refusals qualify as a
        # configuration negative. Lock, provenance, crash and I/O failures stop.
        refusal=$(sed -n 's/.*fit_status=\([2345]\).*/\1/p' "$m/model.bank.work/device.log")
        case "$refusal" in
            2) record "$mode" configuration_negative bank_overflow ;;
            3) record "$mode" configuration_negative opposed_identical_geometry ;;
            4) record "$mode" configuration_negative unsupported_fit_gold ;;
            5) record "$mode" configuration_negative missing_fit_class ;;
            *) exit 1 ;;
        esac
        [[ ! -e "$m/model.bank" ]]
    fi
done

phase=held-frozen-reuse
for lane in public requested-diagnostic; do
    pack "$out/held/$lane/candidates.json" "$out/$lane.packet"
done
phase=eligibility-without-distance
"$binder" membership "$held_grades" "$out/matched-membership.json"
for mode in ordered neutral position; do
    m="$out/modes/$mode"
    [[ -e "$m/model.bank" ]] || continue
    "$binder" availability "$mode" "$m/model.bank" "$out/held/requested-diagnostic/candidates.json" \
        "$out/matched-membership.json" "$m/availability.json"
    if jq -e '.no_scoring==true and .matched_ready==24' "$m/availability.json" >/dev/null; then
        record "$mode" ready supported_matched_shape_domain
    else
        record "$mode" configuration_negative insufficient_matched_shape_support
    fi
done
shopt -s nullglob
shape_receipts=(modes/*/availability.json)
if ((${#shape_receipts[@]})); then
    jq -es 'all(.[];.allocated_bytes==8416) and ([.[].used_records]|unique|length)==1' "${shape_receipts[@]}" > same-bytes.json
fi

phase=forced-control-plan-and-cursor
mkdir grade-output
forced=()
for kind in gold rival-1 rival-2; do
    name="forced-$kind"
    "$binder" force "$kind" "$out/held/requested-diagnostic/candidates.json" "$held_blind" \
      "$out/held/requested-diagnostic.json" "$held_grades" "$out/arms/$name.json" "$out/arms/$name-plan.json"
    forced+=("$out/arms/$name.json")
done
gpu "$out/control-writer" "$writer" replay "$action" "$base" "$held_blind" "$out/control-writer" "${forced[@]}"
mkdir "$out/control-writer/nominees"
ln "$out/control-writer/gpu.json" "$out/control-writer/nominees/chosen.json"
for kind in gold rival-1 rival-2; do
    name="forced-$kind"
    "$writer" grade "$out/arms/$name.json" "$out/control-writer" "$held_grades" - "$out/grade-output/$name-writer.json"
    "$grader" "$tokenizer" "$held_grades" "$reader" - "$out/grade-output/$name-writer.json" "$out/grade-output/$name.json"
    "$binder" verify-forced "$out/arms/$name-plan.json" "$out/grade-output/$name-writer.json" \
        "$out/grade-output/$name-verified.json"
done
"$binder" wrong-key "$held_grades" "$out/grade-output/wrong-key-input.json"
"$writer" grade "$out/arms/forced-gold.json" "$out/control-writer" "$out/grade-output/wrong-key-input.json" - \
    "$out/grade-output/wrong-key-writer.json"
"$grader" "$tokenizer" "$out/grade-output/wrong-key-input.json" "$reader" "$held_grades" \
    "$out/grade-output/wrong-key-writer.json" "$out/grade-output/wrong-key.json"
jq -en --slurpfile gold "$out/grade-output/forced-gold.json" --slurpfile changed "$out/grade-output/wrong-key.json" '
  $gold[0].counts.independently_exact==36 and $changed[0].counts.independently_exact==36
  and $changed[0].counts.active_label_exact==0 and $gold[0].gpu_sha256==$changed[0].gpu_sha256
  and $gold[0].generated_id_tape_sha256==$changed[0].generated_id_tape_sha256' > grade-output/wrong-key-verified.json

phase=blind-native-scores
writer_selections=("$out/held/public.json" "$out/held/no-read.json" "$out/held/requested-diagnostic.json")
positive_arms=(requested-legacy)
learned_arms=()
normal_arms=()
"$binder" supported-view "$held_grades" "$out/held/requested-diagnostic.json" "$out/arms/requested-legacy.json"
writer_selections+=("$out/arms/requested-legacy.json")
score_selection() {
    local lane="$1" mode="$2" m="$3" target="$4"
    local cap="$out/held/$lane/candidates.json" packet="$out/$lane.packet" bank="$m/model.bank"
    gpu "$m/$lane-score" "$native" score "$device" "$packet" "$bank" "$m/$lane.trace"
    "$native" decode-trace "$packet" "$bank" "$m/$lane.trace" "$m/$lane-sources.json"
    jq -en --arg bank "$(sum "$bank")" --arg cap "$(sum "$cap")" \
      --arg packet "$(sum "$packet")" --arg trace "$(sum "$m/$lane.trace")" \
      --slurpfile view "$m/$lane-sources.json" --slurpfile receipt "$m/$lane.trace.work/bindings.json" '
        all([$view[0],$receipt[0]][]; .bank_sha256==$bank and .input_packet_sha256==$packet
             and .blind_capture_sha256==$cap and .trace_sha256==$trace)
      ' >/dev/null
    "$binder" rewrite "$mode" "$cap" "$held_blind" "$out/held/$lane.json" "$m/$lane-sources.json" \
       "$out/arms/$target-all.json" "$out/arms/$target-source-audit.json"
    "$binder" supported-view "$held_grades" "$out/arms/$target-all.json" "$out/arms/$target.json"
    writer_selections+=("$out/arms/$target-all.json" "$out/arms/$target.json")
    positive_arms+=("$target")
    learned_arms+=("$target")
}
for mode in ordered neutral position; do
    m="$out/modes/$mode"
    if jq -e '.status=="ready"' "$m/status.json" >/dev/null; then
        score_selection public "$mode" "$m" "public-$mode"
        score_selection requested-diagnostic "$mode" "$m" "requested-$mode"
        if [[ -n "$pack_artifact" ]]; then
            normal="learned-normal-$mode"
            jq -n --arg mode "$mode" --arg train "$(sum "$out/fit.packet")" \
                --arg labels "$(sum "$m/model.bank.work/fit.labels")" \
                '{mode:$mode,train_sha256:$train,labels_sha256:$labels}' > "$m/profile.json"
            "$pack_artifact" "$paired" "$span" "$m/model.bank" "$m/profile.json" "$m/normal.d11"
            jq -c --arg artifact "$(sum "$m/normal.d11")" --arg origin "$(sum "$out/arms/public-$mode-all.json")" \
                '.artifact=$artifact|.original_blind_selection_sha256=$origin' \
                "$out/arms/public-$mode-all.json" > "$out/arms/$normal.json"
            writer_selections+=("$out/arms/$normal.json")
            normal_arms+=("$normal")
        fi
    fi
done
for lane in public requested-diagnostic; do
    target="$lane-word-start"
    "$binder" rewrite word-start "$out/held/$lane/candidates.json" "$held_blind" "$out/held/$lane.json" - \
        "$out/arms/$target-all.json" "$out/arms/$target-source-audit.json"
    "$binder" supported-view "$held_grades" "$out/arms/$target-all.json" "$out/arms/$target.json"
    writer_selections+=("$out/arms/$target-all.json" "$out/arms/$target.json")
    positive_arms+=("$target")
done
# Freeze deciding selections before the blind writer runs.
sha256sum "${writer_selections[@]}" > blind-selections.sha256

phase=frozen-writer-and-client
gpu "$out/writer" "$writer" replay "$action" "$base" "$held_blind" "$out/writer" "${writer_selections[@]}"
mkdir "$out/writer/nominees"
ln "$out/writer/gpu.json" "$out/writer/nominees/chosen.json"
for lane in public no-read; do
    /usr/bin/time -v -o "$out/client-$lane.time" "$writer" public "$chat" "$paired" "$tokenizer" "$held_blind" "$lane"        "$out/client-$lane" >"$out/client-$lane.log" 2>&1
done
for name in "${normal_arms[@]}"; do
    mode="${name#learned-normal-}"
    # The public response records the literal artifact argument. The current
    # directory is the phase root, so this also preserves raw cold row hashes.
    /usr/bin/time -v -o "$out/client-$name.time" "$writer" public "$learned_chat" "modes/$mode/normal.d11" \
       "$tokenizer" "$held_blind" public "$out/client-$name" >"$out/client-$name.log" 2>&1
done
if ((${#normal_arms[@]})); then
    for name in "${normal_arms[@]}"; do
        mode="${name#learned-normal-}"
        jq -n --arg mode "$mode" --argjson model "$(stat -c %s "modes/$mode/normal.d11")" \
            --argjson bank "$(stat -c %s "modes/$mode/model.bank")" \
            --argjson client "$(stat -c %s "$learned_chat")" --argjson tokenizer "$(stat -c %s "$tokenizer")" \
            '{mode:$mode,d11_bytes:$model,embedded_bank_bytes:$bank,client_bytes:$client,
              tokenizer_bytes:$tokenizer,delivered_file_bytes:($model+$client+$tokenizer)}'
    done | jq -s '{available:.,all_three_available:(length==3),
                   equal_delivered_file_bytes:(if length>=2 then (map(.delivered_file_bytes)|unique|length)==1 else null end)}' \
       > delivered-bytes.json
fi
sha256sum --check blind-selections.sha256

phase=post-seal-grade
"$binder" gold-reach "$out/held/requested-diagnostic/candidates.json" "$held_grades" "$out/grade-output/gold-reach.json"
grade_arm() {
    local name="$1" selection="$2" prose="$3"
    "$writer" grade "$selection" "$out/writer" "$held_grades" "$prose" "$out/grade-output/$name-writer.json"
    "$grader" "$tokenizer" "$held_grades" "$reader" - "$out/grade-output/$name-writer.json" "$out/grade-output/$name.json"
}
for lane in public no-read; do
    grade_arm "frozen-normal-$lane" "$out/held/$lane.json" "$out/client-$lane"
done
for name in "${normal_arms[@]}"; do
    # The historical grader refuses unless normal --say role words, original
    # emitted IDs and all eight cursor words agree with the archival replay.
    grade_arm "$name" "$out/arms/$name.json" "$out/client-$name"
done
for name in "${positive_arms[@]}"; do
    grade_arm "$name" "$out/arms/$name.json" -
done
for name in "${learned_arms[@]}"; do
    # The changed key only enters the grader, then prove both selection bytes
    # and the emitting GPU words still denote the frozen original trajectory.
    "$binder" supported-view "$out/grade-output/wrong-key-input.json" "$out/arms/$name-all.json" \
        "$out/arms/$name-wrong-key-view.json"
    cmp "$out/arms/$name.json" "$out/arms/$name-wrong-key-view.json"
    "$writer" grade "$out/arms/$name.json" "$out/writer" "$out/grade-output/wrong-key-input.json" - \
        "$out/grade-output/$name-wrong-key-writer.json"
    "$grader" "$tokenizer" "$out/grade-output/wrong-key-input.json" "$reader" "$held_grades" \
        "$out/grade-output/$name-wrong-key-writer.json" "$out/grade-output/$name-wrong-key.json"
    jq -en --slurpfile original "$out/grade-output/$name.json" --slurpfile altered "$out/grade-output/$name-wrong-key.json" '
      $original[0].selection_sha256==$altered[0].selection_sha256
      and $original[0].gpu_sha256==$altered[0].gpu_sha256
      and $original[0].generated_id_tape_sha256==$altered[0].generated_id_tape_sha256
      and $original[0].counts.independently_exact==$altered[0].counts.independently_exact
    ' > "$out/grade-output/$name-wrong-key-verified.json"
done
reports=(frozen-normal-public frozen-normal-no-read forced-gold forced-rival-1 forced-rival-2 "${positive_arms[@]}" "${normal_arms[@]}")
for name in "${reports[@]}"; do
    args=()
    if [[ -e "$out/arms/$name-source-audit.json" ]]; then
        args+=(--slurpfile audit "$out/arms/$name-source-audit.json")
    else args+=(--slurpfile audit /dev/null); fi
    case "$name" in
        public-ordered|public-neutral|public-position|learned-normal-*)
            mode="${name##*-}"; native_view="$out/modes/$mode/public-sources.json" ;;
        requested-ordered|requested-neutral|requested-position)
            mode="${name##*-}"; native_view="$out/modes/$mode/requested-diagnostic-sources.json" ;;
        *) native_view=/dev/null ;;
    esac
    jq -c --arg name "$name" "${args[@]}" --slurpfile sources "$native_view" '
        {arm:$name,report:.} + (if ($audit|length)>0 then {source_audit:$audit[0]} else {} end)
        + (if ($sources|length)>0 then {native_sources:$sources[0]} else {} end)' "$out/grade-output/$name.json"
done | jq -es --slurpfile labels "$held_grades" -f "$reducer" > "$out/summary.json"
phase=done
