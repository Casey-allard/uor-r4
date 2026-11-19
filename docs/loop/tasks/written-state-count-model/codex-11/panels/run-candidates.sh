#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/panels"
stage=${1:?blind,oracle,compare}
case "$stage" in
 blind) arms=(nominee support both) ;;
 oracle) arms=(query) ;;
 compare)
   for p in held mixed unseen transfer; do
     for arm in nominee support both query; do
       for n in 1 2; do
         key=seed$n-$p-$arm.json
         cmp "../nominee/results/$key" "../nominee/results-repeat/$key"
         jq -en --slurpfile p "../nominee/results/$key" --slurpfile w "results/writer/$p/$arm/seed$n-picks.json" '
           $p[0].artifact_sha256==$w[0].artifact_sha256 and $p[0].bare_sha256==$w[0].bare_sha256 and
           ([$p[0].rows[]|[.prefix_sha256,.source]]==[$w[0].rows[]|[.prefix_sha256,.continuation.source]])' > /dev/null
         test -s "results/grades/$p/$arm/seed$n-summary.json"
       done
     done
   done
   date -u -Is > RESULTS-SEALED
   exit ;;
 *) exit 2 ;;
esac
for p in held mixed unseen transfer; do
  for arm in "${arms[@]}"; do
    if [[ -s "results/writer/$p/$arm/SHA256SUMS" ]]; then
      (cd "results/writer/$p/$arm"; sha256sum -c SHA256SUMS >/dev/null)
    else
      bash writer-wick.sh "$arm" "$p"
    fi
    bash grade-wick.sh "$arm" "$p"
  done
done
