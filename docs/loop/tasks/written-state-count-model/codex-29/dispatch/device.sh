#!/bin/bash
set -euo pipefail
cd /workspace/d29
phase=${1:?finite, train or held}
sha256sum --quiet --check "$phase.sha256"
mkdir "$phase"
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > "$phase/device.csv"
case "$phase" in
finite)
    c=canary
    ./d29-rival fit "$c/train.packet" "$c/ordered.raw" finite/state
    cmp "$c/expected-calibration.raw" finite/state
    ./d29-rival score "$c/score.d28i" "$c/ordered.raw" finite/state finite/score
    cmp "$c/expected-score.d28o" finite/score
    refuse() {
        expected=$1; shift
        rc=0; "$@" || rc=$?
        [[ $rc == "$expected" && ! -e finite/refused ]]
    }
    refuse 4 ./d29-rival fit "$c/bad-train.packet" "$c/ordered.raw" finite/refused
    refuse 3 ./d29-rival score "$c/score.d28i" "$c/ordered.raw" "$c/bad-calibration.raw" finite/refused
    refuse 4 ./d29-rival score "$c/bad-score.d28i" "$c/ordered.raw" finite/state finite/refused
    ;;
train)
    test -f finite/complete
    for run in primary cold; do ./d29-rival fit train.packet fitted.raw "train/$run.raw"; done
    cmp train/primary.raw train/cold.raw
    ;;
held)
    test -f train/complete
    for run in primary cold; do
        ./d28-score input.d28i fitted.raw "held/baseline-$run"
        ./d29-rival score input.d28i fitted.raw train/primary.raw "held/calibrated-$run"
        ./d29-rival score input.d28i fitted.raw shuffled.raw "held/shuffled-$run"
    done
    for arm in baseline calibrated shuffled; do cmp "held/$arm-primary" "held/$arm-cold"; done
    ;;
*) exit 2 ;;
esac
touch "$phase/complete"
