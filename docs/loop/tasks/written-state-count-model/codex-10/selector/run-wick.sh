#!/usr/bin/env bash
# Run from the private experiment directory on Wick, after finite-test release.
set -euo pipefail
export LD_LIBRARY_PATH=/usr/local/cuda/lib64
base=../20261118-value-connectors
span=../20261118-span-connectors
content=../20261118-content-selector
prior=../20261118-entity-abstention
dir=results
exec 9>../20261118-diagnostics/wick-codex-gpu.lock
device(){
  flock 9
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
  "$@"
  flock -u 9
}
select_panel(){
  case "$1" in
    transfer) tape=../20261118-continuation-eos/panels/transfer;;
    challenge) tape=$prior/panels/challenge;;
    primary) tape=$span/panels/held;;
    fresh) tape=$base/panels/fresh;;
    unseen|unanswerable) tape=panels/$1;;
    *) exit 2;;
  esac
}
mkdir -p "$dir"/{fit,fit-repeat,selector,selector-repeat,overrides,overrides-repeat}
date -u -Is
for n in 1 2; do
  for stage in fit fit-repeat; do
    device bin/cross-input fit bin/cross-device "$base/inputs/seed$n.d11" "$span/results/span-01/seed$n-state.json"       "$span/panels/train-grade.json" "$dir/$stage/seed$n.json"
  done
  cmp "$dir/fit/seed$n.json" "$dir/fit-repeat/seed$n.json"
done
for mode in blind oracle; do
  if test "$mode" = oracle; then
    test -s "$dir/selector/blind.sha256"
    cmp "$dir/selector/blind.sha256" "$dir/selector-repeat/blind.sha256"
  fi
  for suffix in '' -repeat; do
    stage=$dir/selector$suffix
    for p in transfer challenge primary fresh unseen unanswerable; do
      if test "$p" = unanswerable && test "$mode" = oracle; then continue; fi
      select_panel "$p"
      for n in 1 2; do
        key=seed$n-$p
        hand=$dir/overrides$suffix/$key
        mkdir -p "$hand"
        model=$base/inputs/seed$n.d11
        ss=$span/results/span-01/seed$n-state.json
        old=$prior/results/chain-01/$key-repair.json
        if [[ $p = unseen || $p = unanswerable ]]; then old=$dir/independent-baseline/$key-repair.json; fi
        if test "$mode" = blind; then
          extra=(); arms=(baseline role boundary both no-read)
        else
          extra=("$tape-grade.json"); arms=(query fact anchors value wrong)
        fi
        device bin/cross-input "$mode" bin/cross-device "$model" "$ss" "$dir/fit/seed$n.json"           "$tape-blind.json" "$old" "$hand" "${extra[@]}"
        for arm in "${arms[@]}"; do
          dst=$stage/$key-$arm.json
          device bin/chain cross "$arm" "$model" "$content/results/selector-run-01/seed$n/trained.json"             "$ss" "$base/results/value-01/seed$n-state.json" "$tape-blind.json" "$hand/$arm.json" "$dst"
          if test "$arm" = baseline; then
            jq -en --slurpfile b "$old" --slurpfile r "$dst"               '$b[0].rows|map(del(.site_scores)) == $r[0].rows' >/dev/null
          fi
          if test -n "$suffix"; then cmp "$dst" "$dir/selector/$key-$arm.json"; fi
        done
        printf '%s %s %s %s\n' "$(date -u -Is)" "$mode" "$stage" "$key"
      done
    done
    (cd "$stage"; sha256sum *.json > "$mode.sha256")
  done
done
cmp "$dir/selector/oracle.sha256" "$dir/selector-repeat/oracle.sha256"
date -u -Is
