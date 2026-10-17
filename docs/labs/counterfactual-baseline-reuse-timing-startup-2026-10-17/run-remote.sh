#!/usr/bin/env bash
# Prospective operator only; this packet contains no execution release.
set -euo pipefail
base=/workspace/codex-cf1-reuse-timing-run1
cd "$base"
finish() {
  rc=$?; trap - EXIT
  date -u +%FT%TZ > ended.txt
  printf '%s\n' "$rc" > command.exit.tmp
  mv command.exit.tmp command.exit
  exit "$rc"
}
trap finish EXIT
exec > command.log 2>&1
date -u +%FT%TZ > started.txt
export LD_LIBRARY_PATH=/usr/local/cuda/lib64:/usr/local/nvidia/lib:/usr/local/nvidia/lib64:${LD_LIBRARY_PATH:-}
df -Pk "$base" | tee df-preflight.log
free_kib=$(df -Pk "$base" | awk 'NR==2 {print $4}')
[ "$free_kib" -ge 31457280 ] || { echo 'BLOCKED: remote disk floor'; exit 1; }
nvidia-smi > nvidia-smi.txt
nvidia-smi --query-gpu=name,driver_version,uuid,memory.total --format=csv,noheader > gpu.csv
grep -q 'RTX 5090' gpu.csv || { echo 'BLOCKED: GPU identity'; exit 1; }
awk -F', *' 'NF >= 2 { n++; if ($2 != "580.95.05") bad=1 } END { exit !(n == 1 && !bad) }' gpu.csv || { echo 'BLOCKED: timing admits only finite-qualified driver580.95.05; no retry'; exit 1; }
ldd ./k16-timing > ldd.txt
sha256sum -c INPUT-SHA256SUMS
./k16-timing reuse-identity > actual-identity.json
cmp expected-identity.json actual-identity.json
./k16-timing reuse-check --pin timing-pin.json > checked-plan.json
./k16-timing reuse-plan --pin timing-pin.json > execution-plan.json
# Exactly one entry-point invocation; the immutable binary owns the fixed32 dose.
printf '1\n' > model-command-invocation-count.txt
./k16-timing reuse-timing --pin timing-pin.json
