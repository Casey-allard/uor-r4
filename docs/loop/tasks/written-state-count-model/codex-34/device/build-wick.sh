#!/usr/bin/env bash
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
source_dir=$(cd -- "$(dirname -- "$0")" && pwd -P)
cuda_dir=/usr/local/cuda/bin
mkdir -- "${1:?new AOT stage}"
stage=$(cd -- "$1" && pwd -P)
cp -- "$source_dir"/{device.cu,build-wick.sh,run-wick.sh} "$stage/"
cd -- "$stage"
"$cuda_dir/nvcc" --version > nvcc.txt
"$cuda_dir/nvcc" -std=c++17 -O3 -gencode arch=compute_89,code=sm_89 \
    -o detached-endpoint device.cu
"$cuda_dir/cuobjdump" --list-elf detached-endpoint > elf.txt
"$cuda_dir/cuobjdump" --list-ptx detached-endpoint > ptx.txt 2> ptx.log
"$cuda_dir/cuobjdump" --dump-sass detached-endpoint > sass.txt
grep -q 'sm_89' elf.txt
test ! -s ptx.txt
grep -Fq "No PTX file found" ptx.log
test "$(grep -c '^[[:space:]]*Function :' sass.txt)" -eq 1
grep -Fq 'Function : _Z7predict' sass.txt
awk '/^[[:space:]]*\/\*[0-9a-f]+\*\// {
    for (i = 2; i <= NF; i++) {
        if ($i ~ /^[A-Z][A-Z0-9_.]*;?$/) {
            sub(/;$/, "", $i);
            split($i, opcode, ".");
            print opcode[1];
            break;
        }
    }
}' sass.txt > opcodes.txt
test -s opcodes.txt
test "$(wc -l < opcodes.txt)" -eq "$(grep -Ec '^[[:space:]]*/\*[0-9a-f]+\*/' sass.txt)"
LC_ALL=C sort -u opcodes.txt > opcodes.unique.txt
if grep -E '^(F|DF|DADD|DMUL|DSET|HADD|HFMA|HMUL|HSET|MUFU|I2F|I2FP|[A-Z]*MMA|[A-Z]*MMA[A-Z]+|[A-Z]*MMA[0-9]+|[A-Z]*DP4A)' opcodes.unique.txt; then
    echo "Unexpected arithmetic opcode. Inspect the pinned SASS and stop." >&2
    exit 3
fi
sha256sum -- build-wick.sh run-wick.sh device.cu detached-endpoint nvcc.txt \
    elf.txt ptx.txt ptx.log sass.txt opcodes.txt opcodes.unique.txt > stage.sha256
sha256sum stage.sha256
