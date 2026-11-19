#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/panels"
test -s SUPPLEMENT-FIT.md
arm=${1:?agree,query-agree,compare}
case "$arm" in
  agree) parent=both ;;
  query-agree) parent=query ;;
  compare)
    for arm in agree query-agree; do
      for n in 1 2; do
        key=seed$n-held-$arm.json
        cmp "../nominee/agreement/results/$key" "../nominee/agreement/results-repeat/$key"
        test -s "results/grades/held/$arm/seed$n-paired.json"
      done
    done
    date -u -Is > SUPPLEMENT-SEALED
    exit ;;
  *) exit 2 ;;
esac
bash writer-wick.sh "$arm" held
bash grade-wick.sh "$arm" held
for n in 1 2; do
  base=results/grades/held
  jq --arg form summary --slurpfile parent "$base/$parent/seed$n-rows.json" -f agreement-grade.jq \
    "$base/$arm/seed$n-rows.json" > "$base/$arm/seed$n-paired.json"
done
