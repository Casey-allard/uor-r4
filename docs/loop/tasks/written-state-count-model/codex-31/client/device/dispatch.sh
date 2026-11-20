#!/usr/bin/env bash
set -euo pipefail
umask 077

here=docs/loop/tasks/written-state-count-model/codex-31/client/device
case "${1:?build, finite, or replay}" in
build)
  sha256sum -c stage.sha256
  mkdir source
  tar -xf source.tar.gz -C source
  rustc --version > rustc.txt
  nvcc --version > nvcc.txt
  nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > device.csv
  test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 12.0
  UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR="$PWD/target" cargo build --locked --release \
    --features cuda --manifest-path "source/$here/Cargo.toml"
  nvcc -O3 -std=c++17 -gencode arch=compute_120,code=sm_120 "source/$here/device.cu" -o client-mix
  ldd target/release/d31-client-device > host.linked.txt
  ldd client-mix > mix.linked.txt
  if grep -E 'libnvrtc|libcublas' {host,mix}.linked.txt; then exit 1; fi
  sha256sum target/release/d31-client-device client-mix > build.sha256
  for image in client-mix ordinary kn-score kn-lookup; do
    if [[ $image == client-mix ]]; then path=client-mix; else
      path=$(find target/release/build -path "*/out/$image.cubin" -print -quit)
    fi
    test -n "$path"
    cuobjdump -lelf "$path" > "$image-elf.txt"
    cuobjdump -lptx "$path" > "$image-ptx.txt"
    cuobjdump -sass "$path" > "$image-sass.txt"
    grep -E 'sm_120' "$image-elf.txt"
    if grep -E '\.ptx([[:space:]]|$)' "$image-ptx.txt"; then exit 1; fi
    if grep -E '/\*[0-9a-f]+\*/[[:space:]]+(@!?P[0-9]+[[:space:]]+)?(I2F|F2I|MUFU|FADD|FMUL|FFMA|DADD|DMUL|DFMA|HMMA|IMMA|DMMA|BMMA|QGMMA|MMA)([.[:space:]]|$)' "$image-sass.txt"; then exit 1; fi
    if grep -E '/\*[0-9a-f]+\*/[[:space:]]+(@!?P[0-9]+[[:space:]]+)?(HADD2|HMUL2|HFMA2)([.[:space:]]|$)' "$image-sass.txt" |
      grep -vE 'HFMA2[[:space:]]+R[0-9]+,[[:space:]]+-RZ,[[:space:]]+RZ,[[:space:]]+-?0(\.0)?[[:space:]]*,[[:space:]]+-?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?([[:space:]]|,)'; then exit 1; fi
    sha256sum "$path" "$image-elf.txt" "$image-ptx.txt" "$image-sass.txt" >> build.sha256
  done
  ;;
finite)
  sha256sum -c stage.sha256
  sha256sum -c build.sha256
  target/release/d31-client-device finite "$PWD/client-mix" "${2:?trunk}" finite > finite.log
  head -c 15 finite/1-0-00/candidate.rows > finite/invalid.rows
  if ./client-mix finite/invalid.rows finite/finite.weights finite/invalid.masses; then exit 1; fi
  test ! -e finite/invalid.masses
  sha256sum finite/finite.jsonl finite.log > finite.sha256
  ;;
replay)
  sha256sum -c stage.sha256
  sha256sum -c build.sha256
  sha256sum -c finite.sha256
  sha256sum -c GO.sha256
  grep -q '^TEST FITNESS: FIT' FIT.md
  test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 8388608
  test "$(df -Pk . | awk 'NR == 2 {print $4}')" -ge 2097152
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 8192
  for panel in panel-primary panel-cold; do
    /usr/bin/time -v -o "$panel.resources.txt" target/release/d31-client-device replay "$PWD/client-mix" \
      "${2:?trunk}" "${3:?count artifact}" "${4:?fit manifest}" "${5:?fit weights}" "${6:?weights sha}" \
      blind.json "$panel" > "$panel.log"
  done
  cmp panel-primary/steps.jsonl panel-cold/steps.jsonl
  cmp panel-primary/receipt.json panel-cold/receipt.json
  sha256sum panel-{primary,cold}/{steps.jsonl,receipt.json} panel-{primary,cold}/*/* \
    panel-{primary,cold}.{resources.txt,log} > results.sha256
  ;;
*) exit 2 ;;
esac
