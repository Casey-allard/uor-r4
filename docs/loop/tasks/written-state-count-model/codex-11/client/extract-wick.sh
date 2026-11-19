#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/client-witness"
for a in full pruned restored; do
  t="../count/run/$a-fit.gpu"
  q="../count/run/query.u16"
  if [[ $a == restored ]]; then
    t=../precision/primary/loaded.gpu
    q=../precision/primary/query.u16
    expected=$(jq -er .score.device_tape_sha256 primary.json)
  else
    expected=$(jq -er --arg name "$a-fit" '.scores[]|select(.arm==$name)|.device_tape_sha256' result.json)
  fi
  echo "$expected  $t" | sha256sum --status -c -
  echo "$(jq -er '.query.sha256' result.json)  $q" | sha256sum --status -c -
  mapfile -t ids < <(jq -sr '[.[]|.previous,.generated_ids[]]|unique[]' "$a-raw.jsonl")
  ./witness "$t" "$q" "${ids[@]}" > "$a.tsv"
done
