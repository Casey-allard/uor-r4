#!/bin/bash
set -euo pipefail
cd /workspace/uor-r4/jobs/codex22-finite
umask 077
mkdir out
trap 'echo $? > out/exit' EXIT
sha256sum --strict --check inputs.sha256 > out/input-check
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv,noheader > out/device
for pass in primary cold; do
  for seed in 1 2; do
    ./projection-sm89 < "seed$seed.bin" > "out/$pass-seed$seed.bin"
  done
done
for s in 1 2; do cmp "out/primary-seed$s.bin" "out/cold-seed$s.bin"; done
(cd out && sha256sum *.bin > outputs.sha256)
