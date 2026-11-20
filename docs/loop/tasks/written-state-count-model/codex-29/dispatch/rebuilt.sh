#!/bin/bash
set -euo pipefail
cd /workspace/d29-rebuild
sha256sum --quiet -c build.sha256
test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = "8.9"
mkdir result
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > result/gpu.csv
for run in primary cold; do
    ./new fit in/canary/train.packet in/canary/ordered.raw "result/finite-fit-$run"
    cmp in/canary/expected-calibration.raw "result/finite-fit-$run"
    ./new score in/canary/score.d28i in/canary/ordered.raw "result/finite-fit-$run" "result/finite-score-$run"
    cmp in/canary/expected-score.d28o "result/finite-score-$run"
done
refuse() {
    expected=$1; shift
    rc=0; "$@" || rc=$?
    test "$rc" -eq "$expected" && test ! -e result/rejected
}
refuse 4 ./new fit in/canary/bad-train.packet in/canary/ordered.raw result/rejected
refuse 3 ./new score in/canary/score.d28i in/canary/ordered.raw in/canary/bad-calibration.raw result/rejected
refuse 4 ./new score in/canary/bad-score.d28i in/canary/ordered.raw in/canary/expected-calibration.raw result/rejected
for run in primary cold; do
    ./new fit in/train.packet in/fitted.raw "result/trained-$run"
    cmp in/expected-training.raw "result/trained-$run"
    ./old in/input.d28i in/fitted.raw "result/baseline-$run"
    ./new score in/input.d28i in/fitted.raw "result/trained-$run" "result/calibrated-$run"
    ./new score in/input.d28i in/fitted.raw in/shuffled.raw "result/shuffled-$run"
    for arm in baseline calibrated shuffled; do cmp "in/expected-$arm" "result/$arm-$run"; done
done
sha256sum result/* > result.sha256
touch result/complete
