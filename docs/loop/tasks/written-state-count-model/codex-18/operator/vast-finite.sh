#!/usr/bin/env bash
set -euo pipefail
umask 077
[[ $# == 2 && "$1" == /* && "$2" == /* ]] || {
    echo "usage: vast-finite.sh ABS_UOR_CODEX_ROOT ABS_NEW_OUTPUT" >&2
    exit 2
}
wick=$1
out=$2
fixture_list="$(cd "$(dirname "$0")" && pwd)/VAST-FINITE-INPUTS.sha256"
cd "$wick"
sha256sum --status --check "$fixture_list"
mkdir "$out"
stage="$wick/20261119-factorial"
old="$wick/20261118-ordered-role-transfer"
value="$wick/20261118-value-connectors"
span="$wick/20261118-span-connectors/results/span-01/seed1-state.json"
lock="$wick/20261118-diagnostics/wick-codex-gpu.lock"
mkdir -p "$(dirname "$lock")"

{
    uname -m
    getconf GNU_LIBC_VERSION
    nvidia-smi --query-gpu=uuid,name,compute_cap,driver_version,memory.free --format=csv,noheader
} > "$out/device.txt"
[[ "$(uname -m)" == x86_64 ]]
[[ "$(printf '%s\n' 2.39 "$(getconf GNU_LIBC_VERSION | cut -d' ' -f2)" | sort -V | head -1)" == 2.39 ]]
[[ "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" == 8.9 ]]
[[ "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096 ]]
[[ "$(awk '$1=="MemAvailable:" {print $2}' /proc/meminfo)" -ge 8388608 ]]
unset CUDA_FORCE_PTX_JIT
export CUDA_DISABLE_PTX_JIT=1

(
    flock -w 20 9
    timeout 120s "$stage/bin/all-starts-native" factorial-finite "$stage/bin/native-sm89" "$out/selector"
) 9>>"$lock" > "$out/selector.log" 2>&1
cmp "$out/selector.log" "$stage/finite/selector.log"
(
    cd "$out"
    sed 's|  finite/selector/|  selector/|' "$stage/finite/selector-artifacts.sha256" |
        sha256sum --status --check -
)

fixture="$old/finite-exposed"
writer="$old/bins/writer-c"
timeout 120s "$writer" chain "$wick" 1 "$fixture/blind.json" "$out/chain" \
    "$fixture/pub-b.json" "$fixture/oracle-b.json" > "$out/chain.log" 2>&1
cmp "$out/chain/nominees/chosen.json" "$fixture/chain-c/nominees/chosen.json"
cmp "$out/chain/receipt.json" "$fixture/chain-c/receipt.json"
(
    flock -w 20 9
    timeout 120s "$writer" replay "$wick/20261118-role-boundary-count/bin/action-writer" \
        "$value/inputs/seed1.d11" "$fixture/blind.json" "$out/writer" \
        "$fixture/pub-b.json" "$fixture/oracle-b.json"
) 9>>"$lock" > "$out/writer.log" 2>&1
diff -u <(jq -S '.rows' "$out/chain/nominees/chosen.json") <(jq -S '.rows' "$out/writer/gpu.json")

carrier="$old/fresh-role-trace/run-02/seed1-fixtures/favor_correct.d11"
tokenizer="$old/assets/tokenizer-seed1.json"
bank="$stage/operator-01/public-finite-01/known-right.bank"
text='Mira kept blue green. Nora kept red. Please give what Mira collected before closing, rather than Nora beside the entrance.'
mkdir "$out/client"
for mode in ordered ranked veto both; do
    arm="$out/client/$mode"
    timeout 60s "$stage/bin/pack-all-starts" "$carrier" "$span" "$bank" \
        "$stage/finite/public/$mode.profile.json" "$arm.d11"
    expected=$(awk -v path="finite/public/$mode.d11" '$2==path {print $1}' "$stage/finite/public/SHA256SUMS")
    printf '%s  %s\n' "$expected" "$arm.d11" | sha256sum --status --check -
    timeout 60s "$stage/bin/uor-chat" --discrete "$arm.d11" --tokenizer "$tokenizer" --protocol 2 \
        --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" |
        jq -S -c '{prompt_ids, selected, generated_ids, copy_step_words, stop}' > "$arm.witness.json"
    cmp "$arm.witness.json" "$stage/finite/public/$mode.witness.json"
done
(
    cd "$out"
    sha256sum device.txt selector.log chain/{receipt.json,nominees/chosen.json} writer/gpu.json \
        client/*.{d11,witness.json} > outputs.sha256
    sha256sum --status --check outputs.sha256
)
