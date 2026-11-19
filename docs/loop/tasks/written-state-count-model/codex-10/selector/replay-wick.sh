#!/usr/bin/env bash
set -euo pipefail
mkdir -p results/selector-audit
base=../20261118-value-connectors
span=../20261118-span-connectors
prior=../20261118-entity-abstention
# Only proceed once learned actions for the unopened panel and both seeds are sealed.
test -s results/selector/blind.sha256
test -s results/selector-repeat/blind.sha256
cmp results/selector/{blind.sha256,../selector-repeat/blind.sha256}
for n in 1 2; do
  args=("$base/inputs/seed$n.d11" "results/fit/seed$n.json" "$span/results/span-01/seed$n-state.json")
  bin/cross-verify "${args[@]}" "$span/panels/train-grade.json" TRAIN > "results/selector-audit/seed$n-train.json"
  for p in transfer challenge primary fresh unseen; do
    case "$p" in
      transfer) tape=../20261118-continuation-eos/panels/transfer;;
      challenge) tape=$prior/panels/challenge;;
      primary) tape=$span/panels/held;;
      fresh) tape=$base/panels/fresh;;
      unseen) tape=panels/unseen;;
    esac
    bin/cross-verify "${args[@]}" "$tape-grade.json" "results/overrides/seed$n-$p" > "results/selector-audit/seed$n-$p.json"
  done
done
