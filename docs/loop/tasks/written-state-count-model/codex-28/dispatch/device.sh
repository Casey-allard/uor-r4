#!/bin/bash
set -euo pipefail
cd /workspace/d28
case "${1:?expected finite, train or held}" in
finite)
    mkdir finite
    sha256sum --quiet --check finite.sha256
    nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > finite/gpu.csv
    ./score canary.d28i ordered.raw finite/score
    cmp canary.oracle.d28o finite/score
    ./fit canary.fit finite/fit
    cmp canary.oracle.raw finite/fit
    for adverse in inverse-label-collision eight-piece-negative; do
        rc=0
        ./fit "$adverse.fit" "finite/$adverse.raw" || rc=$?
        [[ $rc == 4 && ! -e "finite/$adverse.raw" ]]
    done
    ./augment three.raw ordered.raw finite/augmented
    cmp ordered.raw finite/augmented
    touch finite/complete
    ;;
train)
    test -f finite/complete
    sha256sum --quiet --check train.sha256
    mkdir train
    for run in primary cold; do
        ./fit train.packet "train/$run.raw"
        ./augment frozen.raw "train/$run.raw" "train/$run.aug"
    done
    cmp train/primary.raw train/cold.raw
    cmp train/primary.aug train/cold.aug
    touch train/complete
    ;;
held)
    test -f train/complete
    sha256sum --quiet --check held.sha256
    mkdir held
    for arm in frozen fitted augmented; do
        case "$arm" in
            frozen) bank=frozen.raw ;;
            fitted) bank=train/primary.raw ;;
            augmented) bank=train/primary.aug ;;
        esac
        for reach in legacy whole; do
            for run in primary cold; do
                ./score "$reach.d28i" "$bank" "held/$arm-$reach-$run.bin"
            done
            cmp "held/$arm-$reach-primary.bin" "held/$arm-$reach-cold.bin"
        done
    done
    sha256sum held/*.bin > held/checksums
    touch held/complete
    ;;
*) exit 2 ;;
esac
