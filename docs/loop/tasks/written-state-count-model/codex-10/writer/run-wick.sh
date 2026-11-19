#!/usr/bin/env bash
# Run from the owned Wick experiment directory with the pinned selector outputs.
set -euo pipefail
stage=${1:?expected seal, primary or repeat}
out=results/writer-$stage
mkdir "$out"
selector=results/selector
panels=(transfer challenge primary fresh unseen unanswerable)
panel_prefix() {
  case "$1" in
    transfer) echo ../20261118-continuation-eos/panels/transfer ;;
    challenge) echo ../20261118-entity-abstention/panels/challenge ;;
    primary) echo ../20261118-span-connectors/panels/held ;;
    fresh) echo ../20261118-value-connectors/panels/fresh ;;
    unseen|unanswerable) echo "panels/$1" ;;
    *) return 1 ;;
  esac
}
sha() { sha256sum "$1" | cut -d' ' -f1; }
case "$stage" in
  seal) ;;
  primary|repeat)
    (cd results/writer-seal; sha256sum --check manifest.sha256)
    manifest=results/writer-seal/manifest.json
    read -r digest _ < results/writer-seal/manifest.sha256
    exec 8>../20261118-diagnostics/wick-codex-gpu.lock
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    nvidia-smi --query-gpu=name,uuid,driver_version,memory.free --format=csv > "$out/device.txt"
    date -u -Is > "$out/start.txt" ;;
  *) exit 2 ;;
esac
for panel in "${panels[@]}"; do
  prefix=$(panel_prefix "$panel")
  arms=(baseline role boundary both query fact anchors value wrong no-read)
  if [[ "$panel" == unanswerable ]]; then arms=(baseline role boundary both no-read); fi
  bare=$prefix-blind.json
  for n in 1 2; do
    model=../20261118-value-connectors/inputs/seed$n.d11
    for arm in "${arms[@]}"; do
      key=seed$n-$panel-$arm
      picks=$selector/$key.json
      [[ "$(jq -r .arm "$picks")" == "$arm" ]]
      if [[ "$stage" == seal ]]; then
        jq -e --arg model "$(sha "$model")" --arg bare "$(sha "$bare")" \
          '.artifact_sha256==$model and .bare_sha256==$bare' "$picks" >/dev/null
        if [[ "$arm" == baseline ]]; then
          if [[ "$panel" == unseen || "$panel" == unanswerable ]]; then
            old="results/independent-baseline/seed$n-$panel-repair.json"
          else
            old="../20261118-entity-abstention/results/chain-01/seed$n-$panel-repair.json"
          fi
          jq -e --slurpfile old "$old" '[.rows[]|[.prefix_sha256,.continuation]]==[$old[0].rows[]|[.prefix_sha256,.continuation]]' "$picks" >/dev/null
        fi
        jq -n --arg model "$(sha "$model")" --arg bare "$(sha "$bare")" --arg picks "$(sha "$picks")" \
          '{model:$model,bare:$bare,picks:$picks}' > "$out/$key.json"
      else
        test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
        bin/crossed-writer entity-writer-crossed "$manifest" "$digest" "$model" "$bare" "$picks" "$out/$key.json"
        if [[ "$stage" == repeat ]]; then cmp "results/writer-primary/$key.json" "$out/$key.json"; fi
        if [[ "$stage" == primary && "$arm" == baseline && "$panel" != unseen && "$panel" != unanswerable ]]; then
          old=../20261118-entity-abstention/results/writer-primary/seed$n-$panel-repair.json
          jq -e --slurpfile old "$old" '.rows==$old[0].rows and .transitions==$old[0].transitions' "$out/$key.json" >/dev/null
        fi
      fi
    done
  done
done
if [[ "$stage" == seal ]]; then
  jq -s 'if length==110 and (unique|length)==110 then . else error("duplicate handoff or missing arms") end' "$out"/seed*.json > "$out/manifest.json"
  (cd "$out"; sha256sum manifest.json > manifest.sha256)
else
  (cd "$out"; sha256sum seed*.json > SHA256SUMS)
  date -u -Is > "$out/end.txt"
fi
