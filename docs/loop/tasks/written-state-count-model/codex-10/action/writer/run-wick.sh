#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-role-boundary-count"
stage=${1:?expected seal, primary or repeat}
out=results/action-writer-$stage
mkdir "$out"
sha() { sha256sum "$1"|cut -d' ' -f1; }
if [[ $stage == seal ]]; then
  mkdir results/action-writer-picks
else
  manifest=results/action-writer-seal/manifest.json
  read -r digest _ < results/action-writer-seal/manifest.sha256
  (cd results/action-writer-seal; sha256sum --check manifest.sha256)
  date -u -Is > "$out/start.txt"
  exec 8>../20261118-diagnostics/wick-codex-gpu.lock
  nvidia-smi --query-gpu=name,uuid,driver_version,memory.free --format=csv > "$out/device.txt"
fi
for p in held transfer challenge primary fresh unseen unanswerable; do
  case "$p" in
    transfer) pref=../20261118-continuation-eos/panels/transfer ;;
    primary) pref=../20261118-span-connectors/panels/held ;;
    challenge) pref=../20261118-entity-abstention/panels/challenge ;;
    fresh) pref=../20261118-value-connectors/panels/fresh ;;
    held) pref=panels/action-held ;;
    *) pref=panels/$p ;;
  esac
  for n in 1 2; do
    key=seed$n-$p
    model=../20261118-value-connectors/inputs/seed$n.d11
    bare=$pref-blind.json
    gate=results/action/$key.json
    role=results/selector/$key-role.json
    arms=(action)
    if [[ "$p" == held ]]; then role=results/action-role/$key.json; arms=(role no-read action forced-read); fi
    for arm in "${arms[@]}"; do
      hand=results/action-writer-picks/$key-$arm.json
      if [[ "$stage" == seal ]]; then
        extra=(--argjson grade '[]' --argjson pred '[]')
        if [[ "$arm" == action ]]; then extra=(--argjson grade '[]' --slurpfile pred "$gate"); fi
        if [[ "$arm" == forced-read ]]; then extra=(--slurpfile grade "$pref-grade.json" --argjson pred '[]'); fi
        jq -c --arg arm "$arm" "${extra[@]}" -f action-writer-handoff.jq "$role" > "$hand"
        jq -n --arg model "$(sha "$model")" --arg bare "$(sha "$bare")" --arg picks "$(sha "$hand")" \
          '{model:$model,bare:$bare,picks:$picks}' > "$out/$key-$arm.json"
      else
        flock 8
        test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
        bin/action-writer entity-writer-action "$manifest" "$digest" "$model" "$bare" "$hand" "$out/$key-$arm.json"
        flock -u 8
        if [[ "$stage" == repeat ]]; then cmp "results/action-writer-primary/$key-$arm.json" "$out/$key-$arm.json"; fi
      fi
    done
  done
done
if [[ $stage == seal ]]; then
  jq -s 'if length==20 and (unique|length)==20 then . else error("wrong manifest size") end' "$out"/seed*.json > "$out/manifest.json"
  (cd "$out"; sha256sum manifest.json > manifest.sha256)
else
  date -u -Is > "$out/end.txt"
  (cd "$out"; sha256sum seed*.json > SHA256SUMS)
fi
