#!/usr/bin/env bash
set -euo pipefail
umask 077

if [[ $# != 4 ]]; then
    echo "usage: finite.sh CONTAINER_TOOL CHAT D31_CUSTODY NEW_OUT" >&2
    exit 2
fi
tool=$1 chat=$2 prior=$3 out=$4
fixture=$prior/normal-5ea09ea47/fixtures
archived=$prior/normal-5ea09ea47/finite
test "$(sha256sum "$fixture/planted.p1" | cut -d ' ' -f1)" = c2bbb1141fe973db6804eda6d3f10135de4adf2791e94820dfd5024f7803d62c
test "$(sha256sum "$fixture/seed1.d11" | cut -d ' ' -f1)" = 04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d
mkdir "$out"

"$tool" pack "$fixture/seed1.d11" "$fixture/planted.p1" "$out/c128-planted.d11"
"$tool" check-finite "$out/c128-planted.d11" "$fixture/seed1.d11" "$fixture/planted.p1" > "$out/finite-controls.json"
"$tool" readback "$out/c128-planted.d11" > "$out/readback-c128.json"
jq -e --slurpfile original "$prior/repaired-v4/count-v4/runs/c128/receipt.json" \
    '.counts | (.raw == $original[0].counts.raw) and (.adjusted == $original[0].counts.adjusted)' \
    "$out/readback-c128.json" > "$out/cuda-census-equal.txt"

compare_pair() {
    local base=$1 mode=$2
    shift 2
    local common=(--p1-trace --temperature 0 --max-tokens 1 --read-mode "$mode")
    "$chat" --discrete "$out/c128-planted.d11" "${common[@]}" "$@" > "$base.embedded.json"
    "$chat" --discrete "$fixture/seed1.d11" --p1-state "$fixture/planted.p1" "${common[@]}" "$@" > "$base.sidecar.json"
    jq -e --slurpfile sidecar "$base.sidecar.json" --slurpfile readback "$out/readback-c128.json" '
        .artifact_sha256 == $readback[0].artifact_sha256 and
        .p1_state_sha256 == $readback[0].p1_state_sha256 and
        .p1_trunk_sha256 == $readback[0].p1_trunk_sha256 and
        ((del(.p1_state_sha256, .p1_trunk_sha256) |
            .artifact = $sidecar[0].artifact |
            .artifact_sha256 = $sidecar[0].artifact_sha256 |
            .model_schema = $sidecar[0].model_schema) == $sidecar[0])
    ' "$base.embedded.json" > "$base.parity.txt"
}

for mode in enabled no_read; do
    for prompt in 0 0,17,22,44,17,22 0,17,22,45,17,22; do
        base="$out/$prompt.$mode"
        compare_pair "$base" "$mode" --prompt-ids "$prompt"
        jq -e --slurpfile original "$archived/$prompt.$mode.json" '
            .generated_ids == $original[0].generated_ids and
            .p1_steps == $original[0].p1_steps and
            .p1_components == $original[0].p1_components
        ' "$base.embedded.json" > "$base.archived-parity.txt"
    done
    compare_pair "$out/text.$mode" "$mode" --tokenizer "$fixture/tokenizer-seed1.json" --say "hello"
done

(
    cd "$out"
    sha256sum -- * > ../finite-pinned.sha256
)
