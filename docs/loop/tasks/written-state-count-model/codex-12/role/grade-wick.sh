#!/usr/bin/env bash
set -euo pipefail
[[ $# == 2 && "$1" =~ ^[A-Za-z0-9-]+$ && "$2" =~ ^[A-Za-z0-9-]+$ ]]
in=$1
out=$2
(cd "$in"; sha256sum -c decisions.sha256)
base=$HOME/uor-codex
value=$base/20261118-value-connectors
old=$base/20261118-role-boundary-count
prior=$base/20261118-content-support
continuation=$base/20261118-continuation-eos
mkdir "$out"
for n in 1 2; do
 tokenizer=$value/inputs/tokenizer.json
 [[ $n == 1 ]] || tokenizer=$value/inputs/tokenizer-seed-2.json
 for p in held fresh historical mixed unseen transfer; do
  words=/dev/null
  case "$p" in
   held|fresh) grade=panels/$p-grade.json ;;
   historical) grade=$prior/panels/held-grade.json; words=$prior/nominee/results/seed$n-held/words.json ;;
   mixed) grade=$old/panels/action-held-grade.json; words=$prior/nominee/results/seed$n-mixed/words.json ;;
   unseen) grade=$old/panels/unseen-grade.json; words=$prior/nominee/results/seed$n-unseen/words.json ;;
   transfer) grade=$continuation/panels/transfer-grade.json; words=$prior/nominee/results/seed$n-transfer/words.json ;;
  esac
  input=$in/seed$n-$p.json
  expected=$(jq -r .bare_sha256 "$input")
  if [[ "$words" != /dev/null ]]; then
   artifact=$(jq -r .artifact_sha256 "$input")
   jq -e --arg bare "$expected" --arg model "$artifact" '.bare==$bare and .model==$model' "$words" >/dev/null
  fi
  blind=${grade%-grade.json}-blind.json
  jq -ne --slurpfile g "$grade" --slurpfile b "$blind" '($g[0]|map(.prefix))==($b[0]|map(.prefix))' >/dev/null
  [[ "$expected" == "$(sha256sum "${grade%-grade.json}-blind.json" | cut -d' ' -f1)" ]]
  jq -nec --slurpfile tok "$tokenizer" --slurpfile g "$grade" --slurpfile a "$input" --slurpfile old "$words" -f grade.jq > "$out/seed$n-$p.json"
 done
done
(cd "$out"; sha256sum seed*.json > grades.sha256)
