#!/usr/bin/env bash
set -euo pipefail
old=$HOME/uor-codex/20261118-role-boundary-count
panel=$HOME/uor-codex/20261118-content-support/panels
mkdir -p grades
for n in 1 2;do
 token=$HOME/uor-codex/20261118-value-connectors/inputs/tokenizer.json
 if [[ $n == 2 ]];then token=$HOME/uor-codex/20261118-value-connectors/inputs/tokenizer-seed-2.json;fi
 for p in held mixed unseen transfer;do
  case $p in held)g=$panel/held-grade.json;;mixed)g=results/seed$n-mixed/oracle-grade.json;;unseen)g=$old/panels/unseen-grade.json;;transfer)g=$HOME/uor-codex/20261118-continuation-eos/panels/transfer-grade.json;;esac
  for arm in old nominee support both query;do
   jq -ne --slurpfile tok "$token" --slurpfile g "$g" --slurpfile a "results/seed$n-$p-$arm.json" -f grade.jq > "grades/seed$n-$p-$arm.json"
  done
 done
done
(cd grades;sha256sum *.json > SHA256SUMS)
