#!/usr/bin/env bash
set -euo pipefail
umask 077
[[ $# == 8 ]] || {
    echo "usage: dispatch.sh ABS_STAGE ABS_GO GO_SHA ABS_FIT ABS_FINITE ABS_REVIEW STAGE_SHA ABS_NEW_OUT" >&2
    exit 2
}
stage=$(realpath "$1")
source "$(dirname "$0")/../../codex-19/run/common.sh"
go=$2; go_sha=$3; fit=$4; finite=$5; review=$6; stage_sha=$7; out=$8
[[ "$out" == /* && ! -e "$out" ]]
for file in "$fit" "$finite" "$review"; do [[ "$file" == /* && -s "$file" ]]; done
go_schema=uor-r4.user-text-only-go/1
if [[ "$(basename "$(dirname "$(dirname "$0")")")" == codex-21 ]]; then
    go_schema=uor-r4.clause-entity-veto-go/1
fi
pin "$go" "$go_sha"
pin "$stage/inputs.sha256" "$stage_sha"
jq -e --arg schema "$go_schema" --arg inputs "$stage_sha" --arg fit "$(sum "$fit")" --arg finite "$(sum "$finite")" --arg review "$(sum "$review")" '
    .schema==$schema and .decision=="GO"
    and .inputs_sha256==$inputs and .non_author_fit_sha256==$fit
    and .finite_controls_sha256==$finite and .pre_run_review_sha256==$review
' "$go" >/dev/null
cd "$stage"
sha256sum --status --check "$finite"
sha256sum --status --check inputs.sha256
(cd "$stage/sources" && test -s Cargo.lock)
sha256sum --status --check build-receipt/source.sha256
runner=$(jq -er '.files.runner.path' specs/seed1.json)
[[ "$runner" == "$(jq -er '.files.runner.path' specs/seed2.json)" ]]
mkdir "$out"
cp "$go" "$out/authorization.json"
cp "$fit" "$out/non-author-fit.md"
cp "$finite" "$out/finite-controls.sha256"
cp "$review" "$out/pre-run-review.md"
cp inputs.sha256 "$out/inputs.sha256"
phase=setup
trap 'code=$?; printf "%s %s\n" "$phase" "$code" > "$out/status"; exit "$code"' EXIT
for repetition in primary repeat; do
    for seed in 1 2; do
        phase="waiting-exclusive-device-seed$seed-$repetition"
        while :; do
            occupancy=$(nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader)
            printf '%s\n%s\n' "$(date -u +%FT%TZ)" "$occupancy" >> "$out/occupancy.log"
            [[ -z "$occupancy" ]] && break
            sleep 20
        done
        phase="seed$seed-$repetition"
        "$runner" "$stage/specs/seed$seed.json" "$out/$phase" > "$out/$phase.log" 2>&1
    done
done
phase=cold-parity
for seed in 1 2; do
    diff -u "$out/seed$seed-primary/cold.sha256" "$out/seed$seed-repeat/cold.sha256" > "$out/seed$seed-cold.diff"
done
sha256sum --status --check inputs.sha256
phase=done
