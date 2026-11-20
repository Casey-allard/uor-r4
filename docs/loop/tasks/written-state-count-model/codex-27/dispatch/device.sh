#!/bin/bash
set -euo pipefail
cd /workspace/d27
case "${1:?finite or held}" in
finite)
    mkdir finite
    sha256sum --quiet --check finite-inputs.sha256
    nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > finite/gpu.csv
    ./d27-score canary.bin > finite/actual.bin
    cmp canary.expected finite/actual.bin
    touch finite/complete
    ;;
held)
    test -f finite/complete
    sha256sum --quiet --check held-inputs.sha256
    mkdir held
    for run in primary cold; do
        ./d27-score input.bin > "held/$run.bin"
    done
    cmp held/primary.bin held/cold.bin
    sha256sum held/{primary,cold}.bin > held/outputs.sha256
    touch held/complete
    ;;
*) exit 2 ;;
esac
