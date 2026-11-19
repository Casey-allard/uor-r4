#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/panels"
old="$HOME/uor-codex/20261118-role-boundary-count"
arm=${1:?arm}
p=${2:?panel}
case "$p" in
held) pref=held ;;
mixed) pref=$old/panels/action-held ;;
unseen) pref=$old/panels/unseen ;;
transfer) pref=$HOME/uor-codex/20261118-continuation-eos/panels/transfer ;;
*) exit 2 ;;
esac
jq -en --slurpfile b "$pref-blind.json" --slurpfile g "$pref-grade.json" '$b[0]==[$g[0][]|{prefix}]' >/dev/null
mkdir -p "results/grades/$p/$arm"
for n in 1 2; do
  here=results/writer/$p
  case "$p" in
  held) r=$here/role/seed$n-primary.json; o=$here/old/seed$n-primary.json; none=$here/none/seed$n-primary.json ;;
  mixed) r=$old/results/action-writer-primary/seed$n-held-role.json; o=$old/results/action-writer-primary/seed$n-held-action.json
    none=$old/results/action-writer-primary/seed$n-held-no-read.json ;;
  *) r=$old/results/writer-primary/seed$n-$p-role.json; o=$old/results/action-writer-primary/seed$n-$p-action.json
    none=$old/results/writer-primary/seed$n-$p-no-read.json ;;
  esac
  file=$here/$arm/seed$n-primary.json
  case "$arm" in role) file=$r ;; old) file=$o ;; none) file=$none ;; esac
  for fmt in summary rows; do
    jq -ce --arg panel "$p" --arg arm "$arm" --arg mode "$fmt" --slurpfile role "$r" --slurpfile old "$o" \
     --slurpfile none "$none" --slurpfile grades "$pref-grade.json" -f grade.jq "$file" \
     > "results/grades/$p/$arm/seed$n-$fmt.json"
  done
done
