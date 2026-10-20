#!/bin/sh
# Per-package gates. Run at a checkout root. TD = target dir; PKGS = packages to check; NVCC_STUB = stub nvcc.
set -u
C=~/.cargo/bin/cargo
export CARGO_TARGET_DIR="$TD"
for p in $PKGS; do
  n=$($C check -q -p "$p" --all-targets 2>&1 | grep -cE '^error(\[|:)')
  echo "check $p: errors=$n"
done
echo "check uor-r4-discrete+cuda(stub): errors=$(NVCC="$NVCC_STUB" UOR_NVCC_ARCH=compute_120 $C check -q -p uor-r4-discrete --features cuda --all-targets 2>&1 | grep -cE '^error(\[|:)')"
echo "test xtask: $($C test -q -p xtask 2>&1 | grep -E 'test result' | tail -1)"
echo "test claim-wording: $($C test -q -p uor-r4-claim-wording 2>&1 | grep -E 'test result' | sed -n 2p)"
echo "test harness: $($C test -q -p uor-r4-harness 2>&1 | grep -E 'test result' | tail -1)"
echo "test discrete lib: $($C test -q -p uor-r4-discrete --lib 2>&1 | grep -E 'test result' | tail -1)"
echo "claim-wording gate: $($C run -q --locked -p uor-r4-claim-wording -- . 2>&1 | tail -1)"
echo GATES2-DONE
