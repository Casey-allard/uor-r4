#!/usr/bin/env bash
# Prospective operator only; this packet contains no execution release.
set -euo pipefail
base=/workspace/codex-cf1-reuse-timing-run2
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
awk -F', *' 'NF >= 2 { n++; if ($2 != "595.71.05") bad=1 } END { exit !(n == 1 && !bad) }' gpu.csv || { echo 'BLOCKED: timing admits only accepted finite3 driver595.71.05; no retry'; exit 1; }
# Hardware identity is pinned to the accepted finite environment. No silent
# UUID/memory/library-mapping generalization after the prospective review.
cmp qualified-gpu.csv gpu.csv || { echo 'BLOCKED: actual GPU identity differs from accepted finite3; no binary invocation'; exit 1; }
ldd ./k16-timing > ldd.txt
sed -E 's/[[:space:]]+\(0x[0-9a-fA-F]+\)//g' ldd.txt > ldd.normalized.txt
cmp expected-ldd.normalized.txt ldd.normalized.txt || { echo 'BLOCKED: loader dependency mappings differ; no binary invocation'; exit 1; }
# Finite3 retained mapping names, not library byte hashes. Retain actual hashes
# prospectively; equality of dependency paths is not a library-byte claim.
awk '/=> \/|^[[:space:]]*\// {if ($2 == "=>") print $3; else print $1}' ldd.txt | sort -u > library-paths.txt
while IFS= read -r lib; do sha256sum "$lib"; done < library-paths.txt > library-SHA256SUMS
sha256sum -c INPUT-SHA256SUMS
./k16-timing reuse-identity > actual-identity.json
cmp expected-identity.json actual-identity.json
./k16-timing reuse-check --pin timing-pin.json > checked-plan.json
./k16-timing reuse-plan --pin timing-pin.json > execution-plan.json
# Exactly one entry-point invocation; the immutable binary owns the fixed32 dose.
printf '1\n' > model-command-invocation-count.txt
./k16-timing reuse-timing --pin timing-pin.json
