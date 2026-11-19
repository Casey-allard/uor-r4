#!/usr/bin/env bash
set -euo pipefail
[[ $# == 8 ]] || { echo "usage: check-inputs.sh FIT_CAPTURE HELD_CAPTURE FIT_BLIND HELD_BLIND BASE_FIT_KEY FIT_SUPPLEMENT_KEY HELD_KEY NEW_RECEIPT" >&2; exit 2; }
here=$(cd "$(dirname "$0")" && pwd -P)
hash() { shasum -a 256 "$1" | cut -d' ' -f1; }
for p in "$1" "$2" "$3" "$4" "$5" "$6" "$7"; do [[ -f "$p" ]]; done
[[ "$(hash "$3")" == "$(jq -er .blind_sha256 "$1")" ]]
[[ "$(hash "$4")" == "$(jq -er .blind_sha256 "$2")" ]]
set -C
jq -n --slurpfile fc "$1" --slurpfile hc "$2" --slurpfile fb "$3" --slurpfile hb "$4" \
  --slurpfile fg "$5" --slurpfile extra "$6" --slurpfile hg "$7" \
  --arg fit_sha "$(hash "$1")" --arg held_sha "$(hash "$2")" -f "$here/input.jq" > "$8"
jq -e .eligible_for_deciding_run "$8"
