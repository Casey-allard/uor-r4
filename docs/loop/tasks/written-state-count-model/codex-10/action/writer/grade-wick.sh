#!/usr/bin/env bash
set -eu
cd "$HOME/uor-codex/20261118-role-boundary-count"
out=results/action-writer-grades
mkdir "$out"
cmp results/action-writer-primary/SHA256SUMS results/action-writer-repeat/SHA256SUMS
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
    raw=results/action-writer-primary/$key-action.json
    extra=(--argjson forced '[]')
    ctr=results/writer-primary
    if [[ "$p" == held ]]; then
      extra=(--slurpfile forced results/action-writer-primary/$key-forced-read.json)
      ctr=results/action-writer-primary
    fi
    args=(--slurpfile grades "$pref-grade.json" --slurpfile role "$ctr/$key-role.json" --slurpfile none "$ctr/$key-no-read.json"
      --slurpfile pred "results/action/$key.json" --arg panel "$p" "${extra[@]}")
    jq "${args[@]}" --arg form summary -f action-writer-grade.jq "$raw" > "$out/$key.json"
    jq -c "${args[@]}" --arg form rows -f action-writer-grade.jq "$raw" > "$out/$key-rows.json"
  done
done
(cd "$out"; sha256sum seed*.json > SHA256SUMS)
