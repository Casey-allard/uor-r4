#!/usr/bin/env bash
# Device qualification on planted text before opening the fresh held prompts.
set -euo pipefail
umask 077
stage=${1:?sealed absolute stage}
out=${2:?fresh finite output}
[[ "$stage" == /* && "$out" == /* ]]
root=$(dirname "$stage")
prior="$root/20261119-requested-occurrence"
d19="$prior/sources/docs/loop/tasks/written-state-count-model/codex-19"
mkdir "$out"
phase=upstream
trap 'rc=$?; echo "$phase $rc" > "$out/status"; exit $rc' EXIT
export CUDA_DISABLE_PTX_JIT=1 RAYON_NUM_THREADS=1
unset CUDA_FORCE_PTX_JIT
bash "$d19/dispatch/finite.sh" "$prior" "$out/upstream" > "$out/upstream.log" 2>&1
cp "$out/upstream/device.txt" "$out/device.txt"
(cd "$stage" && sha256sum --status --check inputs.sha256)
phase=clause_and_veto
(
    flock -w 25 8
    "$stage/bin/requested-role-device" --finite-clause-to-entity "$stage/bin/requested-role-sm89" "$out/planted.json"
) 8>>"$root/20261118-diagnostics/wick-codex-gpu.lock"
"$stage/bin/all-starts-native" connector-finite "$stage/bin/earlier-veto-sm89" "$out/connector" > "$out/connector.log" 2>&1
(cd "$out" && find connector -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > connector.sha256)
phase=ordinary
tokenizer=$(jq -er '.files.tokenizer.path' "$stage/specs/seed1.json")
text='Mira kept blue green. Nora kept red. Please give what Mira collected before closing, rather than Nora beside the entrance.'
for name in rebuilt-assertion-agreement  clause-to-entity; do
    artifact="$stage/artifacts/seed1/$name.d11"
    "$stage/bin/uor-chat" --discrete "$artifact" --tokenizer "$tokenizer" --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" > "$out/$name.json"
    jq -c --arg text "$text" '[{text:$text,prefix:.prompt_ids}]' "$out/$name.json" > "$out/$name-blind.json"
    (
        flock -w 25 8
        "$stage/bin/requested-role-device" "$artifact" "$out/$name-blind.json" "$stage/bin/requested-role-sm89" "$out/$name-role.json"
    ) 8>>"$root/20261118-diagnostics/wick-codex-gpu.lock"
    jq -en --slurpfile c "$out/$name.json" --slurpfile d "$out/$name-role.json" '($c[0].paired.role_words|length)==8 and $c[0].paired.role_words==$d[0].rows[0].role_words and ($c[0].generated_ids|length)>0' >/dev/null
done
# The same pinned artifact lives at different paths in the two study roots.
jq -en --slurpfile rebuilt "$out/rebuilt-assertion-agreement.json" --slurpfile original "$out/upstream/treatment.json" '($rebuilt[0]|del(.artifact))==($original[0]|del(.artifact))' >/dev/null
# A D19 reader must reject the new artifact instead of silently serving the earlier policy.
if "$prior/bin/uor-chat" --discrete "$stage/artifacts/seed1/clause-to-entity.d11" --tokenizer "$tokenizer" --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" > "$out/old-reader.log" 2>&1; then
    exit 1
fi
grep -Eiq 'missing field|unknown (variant|field)|unsupported.*schema|unrecognized.*schema|requires schema' "$out/old-reader.log"
sha256sum "$out"/{device.txt,connector.sha256,planted.json,upstream/finite.sha256,old-reader.log,rebuilt-assertion-agreement.json,clause-to-entity.json,*-role.json} > "$out/finite.sha256"
phase=done
