#!/usr/bin/env bash
# A separately cold client and CUDA rollout, from original unmarked user text.
set -euo pipefail
cd "$HOME/uor-codex/20261118-ordered-role-transfer"
stage=${1:?primary or repeat}
seed=${2:?seed 1 or 2}
[[ "$stage" == primary || "$stage" == repeat ]]
[[ "$seed" == 1 || "$seed" == 2 ]]
out=$PWD/$stage/seed$seed
role=$PWD/roles-$stage
mkdir -p "$PWD/$stage"
mkdir "$out"
w=$PWD/bins/writer-c
client=$PWD/native-finite/source/target/release/uor-chat
blind=$PWD/panel/held-blind.json
labels=$PWD/grading/held-grades.json
bounds=$PWD/panel/held-bounds.json
tok=$PWD/assets/tokenizer-seed$seed.json
base=$PWD/assets/seed$seed-paired.d11
# Each deciding artifact must first pass both independent downstream oracles.
for verify_seed in 1 2; do
    cmp "oracle-primary/seed$verify_seed/oracle-grade.json" "oracle-repeat/seed$verify_seed/oracle-grade.json"
    jq -e '(.rows|length)==256 and ([.rows[]|select(.exact==true)]|length)>=248' "oracle-primary/seed$verify_seed/oracle-grade.json" >/dev/null
done
sha256sum -c driver/served-pins.sha256
(cd panel && sha256sum -c panel-inputs.sha256)
(cd assets && sha256sum -c SHA256SUMS)
declare -a selections=() clients=()
for arm in frozen identity positional; do
    m=$base
    if [[ "$arm" != frozen ]]; then
        gpu=$role/seed$seed-$arm
        m=$out/$arm.d11
        "$w" pack "$base" "$tok" "$gpu/train/bank.role" "$gpu/train/train.receipt" "$m"
    fi
    for op in public bounded; do
        region=-
        [[ "$op" == public ]] || region=$bounds
        path=$out/$op-$arm.json
        "$w" select "$m" "$blind" "$op" "$region" - "$path"
        selections+=("$path")
        if [[ "$arm" != frozen ]]; then
            jq -en --slurpfile device "$gpu/$op.json" --slurpfile served "$path" --slurpfile blind "$blind" --slurpfile packed "$out/$arm.receipt.json" -f driver/certify.jq >"$out/parity-$op-$arm.json"
        fi
    done
    # Every row is a fresh normal uor-chat invocation, using the same seed tokenizer.
    "$w" public "$client" "$m" "$tok" "$blind" public "$out/say-$arm" >"$out/say-$arm.log" 2>&1 & clients+=("$!")
done
cmp <(wc -c <"$out/identity.d11") <(wc -c <"$out/positional.d11")
"$w" select "$base" "$blind" lexical "$bounds" - "$out/lexical.json"
"$w" select "$base" "$blind" no-read - - "$out/no-read.json"
selections+=("$out/lexical.json" "$out/no-read.json")
"$w" public "$client" "$base" "$tok" "$blind" no-read "$out/say-no-read" >"$out/say-no-read.log" 2>&1 & clients+=("$!")
"$w" chain "$HOME/uor-codex" "$seed" "$blind" "$out/device" "${selections[@]}"
fail=0
for job in "${clients[@]}"; do wait "$job" || fail=1; done
[[ $fail == 0 ]]
for file in "${selections[@]}"; do
    arm=$(basename "$file" .json)
    say=-
    case "$arm" in public-*) say=$out/say-${arm#public-};; no-read) say=$out/say-no-read;; esac
    "$w" grade "$file" "$out/device" "$labels" "$say" "$out/$arm-grade.json"
done
(cd "$out" && sha256sum *-grade.json parity-*.json *.receipt.json device/nominees/chosen.json > cold.sha256)
