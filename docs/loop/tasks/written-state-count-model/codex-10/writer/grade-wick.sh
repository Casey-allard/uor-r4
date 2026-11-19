#!/usr/bin/env bash
# Run from the Wick experiment directory after both actual-feedback replays.
set -euo pipefail
sha() { sha256sum "$1" | cut -d' ' -f1; }
out=results/writer-grades
mkdir "$out"
cmp results/writer-primary/SHA256SUMS results/writer-repeat/SHA256SUMS
for panel in transfer challenge primary fresh unseen unanswerable; do
  case "$panel" in
    transfer) prefix=../20261118-continuation-eos/panels/transfer ;;
    challenge) prefix=../20261118-entity-abstention/panels/challenge ;;
    primary) prefix=../20261118-span-connectors/panels/held ;;
    fresh) prefix=../20261118-value-connectors/panels/fresh ;;
    *) prefix=panels/$panel ;;
  esac
  arms=(baseline role boundary both query fact anchors value wrong no-read)
  if [[ "$panel" == unanswerable ]]; then arms=(baseline role boundary both no-read); fi
  for n in 1 2; do
    for arm in "${arms[@]}"; do
      key=seed$n-$panel-$arm
      base=results/writer-primary/seed$n-$panel-baseline.json
      raw=results/writer-primary/$key.json
      args=(--arg arm "$arm" --arg panel "$panel" --arg model_sha "$(sha ../20261118-value-connectors/inputs/seed$n.d11)"
        --arg pick_sha "$(sha results/selector/$key.json)" --arg bare_sha "$(sha "$prefix-blind.json")"
        --slurpfile grades "$prefix-grade.json" --slurpfile bare "$prefix-blind.json" --slurpfile base "$base")
      jq "${args[@]}" --arg format summary -f writer-grade.jq "$raw" > "$out/$key.json"
      jq -r "${args[@]}" --arg format tsv -f writer-grade.jq "$raw" > "$out/$key.tsv"
    done
  done
done
(cd "$out"; sha256sum seed* > SHA256SUMS)
