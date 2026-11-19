#!/usr/bin/env bash
set -euo pipefail
test -s results/selector-repeat/oracle.sha256
cmp results/selector/{oracle.sha256,../selector-repeat/oracle.sha256}
for p in transfer challenge primary fresh unseen; do
  case "$p" in
    transfer) tape=../20261118-continuation-eos/panels/transfer;;
    challenge) tape=../20261118-entity-abstention/panels/challenge;;
    primary) tape=../20261118-span-connectors/panels/held;;
    fresh) tape=../20261118-value-connectors/panels/fresh;;
    unseen) tape=panels/unseen;;
  esac
  for n in 1 2; do
    for arm in baseline role boundary both query fact anchors value wrong no-read; do
      key=seed$n-$p
      jq --slurpfile baseline "results/selector/$key-baseline.json" --slurpfile grad "$tape-grade.json"         -f selector/grade.jq "results/selector/$key-$arm.json" |
        jq -c --arg panel "$p" --arg arm "$arm" --argjson seed "$n" '.+{$panel,$arm,$seed}'
    done
  done
done
