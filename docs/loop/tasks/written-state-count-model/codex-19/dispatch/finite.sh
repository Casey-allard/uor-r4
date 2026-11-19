#!/usr/bin/env bash
# Run only planted and archived finite cases on the device reserved for the study.
set -euo pipefail
umask 077
stage=${1:?sealed absolute stage}
out=${2:?new absolute output}
[[ "$stage" == /* && "$out" == /* ]]
root=$(dirname "$stage")
prior="$root/20261119-factorial"
tasks="$stage/sources/docs/loop/tasks/written-state-count-model"
old="$root/20261118-ordered-role-transfer"
mkdir "$out"
phase=compatibility
trap 'rc=$?; echo "$phase $rc" > "$out/status"; exit $rc' EXIT
read -r gpu < <(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)
[[ "$gpu" == 8.9 ]]
[[ -z "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)" ]]
export CUDA_DISABLE_PTX_JIT=1
unset CUDA_FORCE_PTX_JIT
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv,noheader > "$out/device.txt"
(cd "$stage" && sha256sum --status --check build-receipt/executables.sha256)
(
    flock -w 25 8
    "$stage/bin/requested-role-device" --finite "$stage/bin/requested-role-sm89" "$out/role.json"
) 8>>"$root/20261118-diagnostics/wick-codex-gpu.lock" > "$out/role.log" 2>&1
phase=legacy
bash "$tasks/codex-18/operator/vast-finite.sh" "$root" "$out/original" > "$out/legacy.log" 2>&1
phase=current-client
text='Mira kept blue green. Nora kept red. Please give what Mira collected before closing, rather than Nora beside the entrance.'
"$stage/bin/uor-chat" --discrete "$out/original/client/ordered.d11" --tokenizer "$old/assets/tokenizer-seed1.json" --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" |
    jq -S -c '{prompt_ids, selected, generated_ids, copy_step_words, stop}' > "$out/current.witness.json"
cmp "$out/current.witness.json" "$prior/finite/public/ordered.witness.json"
phase=treatment-client
tokenizer="$old/assets/tokenizer-seed1.json"
if "$prior/bin/uor-chat" --discrete "$stage/artifacts/seed1/assertion-agreement.d11" --tokenizer "$tokenizer" --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" > "$out/legacy-on-treatment.log" 2>&1; then
    echo 'legacy reader silently accepted the new treatment schema' >&2
    exit 1
fi
test -s "$out/legacy-on-treatment.log"
"$stage/bin/uor-chat" --discrete "$stage/artifacts/seed1/assertion-agreement.d11" --tokenizer "$tokenizer" --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" > "$out/treatment.json"
jq -c --arg text "$text" '[{text:$text,prefix:.prompt_ids}]' "$out/treatment.json" > "$out/treatment-blind.json"
(
    flock -w 25 8
    "$stage/bin/requested-role-device" "$stage/artifacts/seed1/assertion-agreement.d11" "$out/treatment-blind.json" "$stage/bin/requested-role-sm89" "$out/treatment-role.json"
) 8>>"$root/20261118-diagnostics/wick-codex-gpu.lock"
jq -en --slurpfile client "$out/treatment.json" --slurpfile device "$out/treatment-role.json" '
    $device[0].policy=="assertion_agreement" and ($client[0].paired.role_words|length)==8 and
    $client[0].paired.role_words==$device[0].rows[0].role_words and
    ($client[0].generated_ids|length)>0' > "$out/arm-parity.json"
(cd "$out/original" && sha256sum --status --check outputs.sha256)
sha256sum "$out"/{device.txt,role.json,current.witness.json,legacy-on-treatment.log,treatment.json,treatment-blind.json,treatment-role.json,arm-parity.json,original/outputs.sha256} > "$out/finite.sha256"
phase=done
