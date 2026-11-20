#!/usr/bin/env bash
set -euo pipefail
umask 077

if [[ $# != 7 ]]; then
    echo 'usage: replay-strict.sh BUILD TRUNK COUNTS FIT-MANIFEST WEIGHTS FIT-RESULTS-SHA256 NEW-OUT' >&2
    exit 2
fi
build=$(realpath "$1")
trunk=$(realpath "$2")
counts=$(realpath "$3")
manifest=$(realpath "$4")
weights=$(realpath "$5")
fit_results=$(realpath "$6")
out=$(realpath -m "$7")
here=docs/loop/tasks/written-state-count-model/codex-31
host="$build/target/release/d31-client-device"
mixer="$build/client-mix"
blind="$build/blind.json"
sha() { sha256sum "$1" | cut -d ' ' -f1; }

sha256sum --strict -c GO.sha256
grep -qx 'TEST FITNESS: FIT' FIT.md
(cd "$build" && sha256sum --strict -c stage.sha256 build.sha256 finite.sha256 >/dev/null)
if [[ -f "$build/source-all.sha256" ]]; then
    (cd "$build/source" && sha256sum --strict -c "$build/source-all.sha256" >/dev/null)
fi
gpu=$(nvidia-smi --query-gpu=uuid --format=csv,noheader)
weights_sha=$(sha "$weights")
compact_sha=$(sha "$counts")
manifest_sha=$(sha "$manifest")
blind_sha=$(sha "$blind")
fit_results_sha=$(sha "$fit_results")
trunk_sha=$(sha "$trunk")
check_hash_entry() {
    awk -v expected="$1" -v path="$2" '$1 == expected && ($2 == path || $2 == "*" path) {count++} END {exit count != 1}' "$fit_results"
}
check_hash_entry "$weights_sha" "$weights"
check_hash_entry "$manifest_sha" "$manifest"

jq -e --arg gpu "$gpu" --arg count "$compact_sha" --arg weights "$weights_sha" \
    --arg manifest "$manifest_sha" --arg fit_results "$fit_results_sha" --arg trunk "$trunk_sha" \
    --arg blind "$blind_sha" --arg host "$(sha "$host")" --arg mixer "$(sha "$mixer")" \
    --arg stage "$(sha "$build/stage.sha256")" --arg build "$(sha "$build/build.sha256")" \
    --arg machine "$(sha "$build/machine.sha256")" --arg finite "$(sha "$build/finite.sha256")" \
    --arg host_source "$(sha "$build/source/$here/client/device/main.rs")" \
    --arg replay_source "$(sha "$build/source/$here/client/device/replay.rs")" \
    --arg mixer_source "$(sha "$build/source/$here/client/device/device.cu")" \
    --arg fit_device_source "$(sha "$build/source/$here/fit/device.cu")" \
    --arg fit "$(sha FIT.md)" --arg dispatch "$(sha "$0")" '
    .schema == "uor-r4.d31.client-go/1" and .decision == "GO" and .lab == "codex"
    and (.pod | test("^vast-[0-9]+$")) and .lease == "codex/count31-repaired-chain"
    and .gpu == $gpu and .off_pod_c1_verified == true and .off_pod_fit_verified == true
    and .compact_sha256 == $count and .weights_sha256 == $weights
    and .fit_manifest_sha256 == $manifest and .fit_results_sha256 == $fit_results
    and .trunk_sha256 == $trunk and $trunk == "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d"
    and .blind_sha256 == $blind and $blind == "a4fb823cdd5031cc7ffdb1c402f7d7df2c149002b1335a102a8a012371f081dc"
    and .panel_sha256 == "e50d7812f4a4dcc3016793e07cf2efb55fa7524d880bf1253324ec255b4b8b11"
    and .host_sha256 == $host and .mixer_sha256 == $mixer
    and .stage_sha256 == $stage and .build_sha256 == $build
    and .machine_sha256 == $machine and .finite_sha256 == $finite
    and .host_source_sha256 == $host_source and .replay_source_sha256 == $replay_source
    and .mixer_source_sha256 == $mixer_source and .fit_device_source_sha256 == $fit_device_source
    and .fit_sha256 == $fit and .dispatch_sha256 == $dispatch
' GO-CLIENT.json >/dev/null
jq -e --arg count "$compact_sha" --arg trunk "$trunk_sha" '
    .schema == "uor-r4.d31.a1-fit/1" and .rows == 15947584 and .passes == 96
    and .historical_q12_pass_sums_equal == 97 and .historical_q32_pick_words_equal == 306
    and .prepared.compact_sha256 == $count and .prepared.trunk_sha256 == $trunk
    and .prepared.r_sha256 == "5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a"
' "$manifest" >/dev/null
jq -e '
    keys == ["max_tokens","panel_sha256","protocol","rows"] and .protocol == 2 and .max_tokens == 12
    and (.rows|length) == 8 and all(.rows[]; keys == ["id","prefix"] and (.prefix|type) == "array")
' "$blind" >/dev/null

test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 12.0
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 8388608
test "$(df -Pk "$(dirname "$out")" | awk 'NR == 2 {print $4}')" -ge 2097152
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 8192
mkdir "$out"
cp GO-CLIENT.json GO.sha256 FIT.md "$out/"
cp "$blind" "$out/blind.json"
(
    cd "$out"
    for panel in panel-primary panel-cold; do
        /usr/bin/time -v -o "$panel.resources.txt" "$host" replay "$mixer" \
            "$trunk" "$counts" "$manifest" "$weights" "$weights_sha" blind.json "$panel" > "$panel.log"
    done
    cmp panel-primary/steps.jsonl panel-cold/steps.jsonl
    cmp panel-primary/receipt.json panel-cold/receipt.json
    sha256sum GO-CLIENT.json GO.sha256 FIT.md blind.json panel-{primary,cold}/{steps.jsonl,receipt.json} \
        panel-{primary,cold}/*/* panel-{primary,cold}.{resources.txt,log} > results.sha256
    sha256sum --strict -c results.sha256 >/dev/null
)
