#!/usr/bin/env bash
# Execute the top session's conditional decision on Wick after capture and input validation.
set -euo pipefail
umask 077
[[ $# == 5 ]] || { echo "usage: after-verified-capture.sh STAGE FIT_FILE FIT_SHA256 VALIDITY_DIR NEW_OUTPUT" >&2; exit 2; }
stage=$(cd "$1" && pwd -P)
fit=$2; fit_sha=$3; valid=$4; out=$5
here=$(cd "$(dirname "$0")" && pwd -P)
sum() { sha256sum "$1" | cut -d' ' -f1; }
pin() { [[ "$2" =~ ^[0-9a-f]{64}$ && "$(sum "$1")" == "$2" ]]; }
[[ "$out" == /* && ! -e "$out" && "$valid" == /* && "$fit" == /* ]]
pin "$fit" "$fit_sha"
build="$stage/src/d17-study.fnabsw"
binder="$build/build/target/release/uor-rank-study-bind"
reducer="$stage/src/input-validity-4be36d43f/docs/loop/tasks/written-state-count-model/codex-17/validity/reduce.jq"
normal="$stage/src/normal-finite.w8WP0P"
pin "$normal.manifest.sha256" e1f51c5ff27199493b168e2a1085ca4b3d56dcf3dc42a9bf5c302709c9ea01c9
(cd "$normal" && sha256sum --status -c "$normal.manifest.sha256")
pin "$binder" e4d571cbfa008d9369de8812549ea6d38e7a8f1049b212db0c988b82aee989bc
pin "$reducer" e440d0b6ce76c8e83cb3eeee37139f9204cef9c4b6b13151ce147310711b2525
(cd "$valid" && sha256sum --status -c receipt.sha256)
(cd "$stage" && sha256sum --status -c stages.sha256 && sha256sum --status -c launch.sha256)
(cd "$stage/capture" && sha256sum --status -c "$valid/raw-index.sha256")
mkdir "$out"
cd "$out"
cp "$fit" non-author-fit.md
phase=eligibility
trap 'code=$?; printf "%s %s\n" "$phase" "$code" > dispatch.status' EXIT
for seed in 1 2; do
    receipt="$valid/primary-seed$seed.validity.json"
    cmp "$receipt" "$valid/repeat-seed$seed.validity.json"
    jq -e '.eligible_for_deciding_run==true and .fit.rows==94 and .fit.supported==88 and
           .held.rows==42 and .held.supported==36 and .held.original_literal_reachable==36 and
           .matched.rows==24 and .matched.confirmed_structural_ceiling==8' "$receipt" >/dev/null
    spec="$out/spec-$seed.json"
    "$here/stage-spec-inputs.sh" "$stage" "$seed" "$binder" "$spec" "$receipt" "$(sum "$receipt")" "$reducer" "$(sum "$reducer")"
    draft="$build/specs/draft-seed$seed.json"
    case "$seed" in
        1) pin "$draft" 135f297e948973a201fc02909e0eb70421179c3e38337bdbcabb811fd7407e57;;
        2) pin "$draft" 273991aab09aa31a1426baace23d5dda80a5bd7c89bbd846c9984d23e53e3d67;;
    esac
    # Finalization fills only the four raw-manifest digests and the validity and reducer pins.
    jq -e --slurpfile reviewed "$draft" '
      (del(.files.validity,.files.reducer) | .captures[][] .raw.sha256=null)
      == ($reviewed[0]|del(.draft_reason,.files.validity,.files.reducer))
    ' "$spec" >/dev/null
    jq -n --arg spec "$(sum "$spec")" --arg fit "$fit_sha" --arg finite "$(sum "$normal.manifest.sha256")" \
      '{schema:"uor-r4.value-rank-pre-run/1",decision:"GO",spec_sha256:$spec,
        non_author_fit_sha256:$fit,finite_controls_sha256:$finite}' > "go-$seed.json"
done
phase=study
for phase_name in primary repeat; do
    for seed in 1 2; do
        root="$out/$phase_name-seed$seed"
        phase="$phase_name-seed$seed"
        while :; do
            pids=$(nvidia-smi --query-compute-apps=pid --format=csv,noheader,nounits)
            [[ -z "$pids" ]] && break
            sleep 15
        done
        { date --utc; nvidia-smi --query-gpu=name,memory.free,driver_version --format=csv; } > "$root.device"
        [[ -z "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader,nounits)" ]]
        spec="$out/spec-$seed.json"; go="$out/go-$seed.json"
        /usr/bin/time -v -o "$root.time" "$here/run-wick.sh" \
           "$spec" "$(sum "$spec")" "$go" "$(sum "$go")" "$phase_name" "$root" > "$root.log" 2>&1
        jq -e '.phase=="done" and .exit_code==0' "$root/status.json" >/dev/null
    done
done
phase=cold
for seed in 1 2; do
    cmp "primary-seed$seed/study-cold.sha256" "repeat-seed$seed/study-cold.sha256"
    cmp "primary-seed$seed/summary.json" "repeat-seed$seed/summary.json"
done
phase=done
