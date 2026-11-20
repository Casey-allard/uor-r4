#!/usr/bin/env bash
set -euo pipefail
umask 077

case "${1:?build or run}" in
build)
  sha256sum --strict -c stage.sha256
  test "$(df -Pk . | awk 'NR == 2 {print $4}')" -ge 20971520
  mkdir source
  tar -xf source.tar.gz -C source
  rustc --version > rustc.txt
  nvcc --version > nvcc.txt
  nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap,memory.free --format=csv > device.csv
  test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 12.0
  UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR="$PWD/target" cargo build --locked --release \
    --manifest-path source/docs/labs/kn-reference/instrument/Cargo.toml \
    --features cuda --bin full-compact-count
  ldd target/release/full-compact-count > linked.txt
  if grep -E 'libnvrtc|libcublas' linked.txt; then exit 1; fi
  sha256sum target/release/full-compact-count > build.sha256
  for image in ngram-count kn-score kn-lookup; do
    path=$(find target/release/build -path "*/out/$image.cubin" -print -quit)
    test -n "$path"
    cuobjdump -lelf "$path" > "$image-elf.txt"
    cuobjdump -lptx "$path" > "$image-ptx.txt"
    cuobjdump -sass "$path" > "$image-sass.txt"
    grep -E 'sm_120' "$image-elf.txt"
    if grep -E '\.ptx([[:space:]]|$)' "$image-ptx.txt"; then exit 1; fi
    if grep -E '/\*[0-9a-f]+\*/[[:space:]]+(@!?P[0-9]+[[:space:]]+)?(I2F|F2I|MUFU|FADD|FMUL|FFMA|DADD|DMUL|DFMA|HMMA|IMMA|DMMA|BMMA|QGMMA|MMA)([.[:space:]]|$)' "$image-sass.txt"; then exit 1; fi
    if grep -E '/\*[0-9a-f]+\*/[[:space:]]+(@!?P[0-9]+[[:space:]]+)?(HADD2|HMUL2|HFMA2)([.[:space:]]|$)' "$image-sass.txt" |
      grep -vE 'HFMA2[[:space:]]+R[0-9]+,[[:space:]]+-RZ,[[:space:]]+RZ,'; then exit 1; fi
    sha256sum "$path" "$image-elf.txt" "$image-ptx.txt" "$image-sass.txt" >> build.sha256
  done
  ;;
run)
  sha256sum --strict -c stage.sha256 build.sha256 GO.sha256
  awk '$2 == "GO-COUNT.json" || $2 == "*GO-COUNT.json" { count++ } END { exit count != 1 }' GO.sha256
  grep -qx 'TEST FITNESS: FIT' FIT.md
  gpu=$(nvidia-smi --query-gpu=uuid --format=csv,noheader)
  jq -er --arg gpu "$gpu" '
    if .schema != "uor-r4.d31.full-c1-go/1" or .decision != "GO" or .lab != "codex"
      or .scope != "Complete registered C1 count and lookup" or .gpu != $gpu
      or (.pod | test("^vast-[0-9]+$") | not) or (.lease | test("^codex/[a-z0-9-]+$") | not)
    then error("count GO does not authorize the requested scope and device")
    else [
      [.stage_sha256, "stage.sha256"], [.source_sha256, "source.tar.gz"],
      [.source_manifest_sha256, "source-all.sha256"],
      [.sheet_sha256, "source/docs/loop/tasks/written-state-count-model/codex-31/count/SHEET.md"],
      [.fit_sha256, "FIT.md"], [.build_sha256, "build.sha256"],
      [.machine_sha256, "machine.sha256"], [.finite_sha256, "finite.sha256"],
      [.exe_sha256, "target/release/full-compact-count"], [.c1_sha256, "c1.tokens.u16"],
      [.c128_sha256, "finite/c128/compact.counts"]
    ][] | if (.[0] | test("^[0-9a-f]{64}$")) then join("  ")
          else error("count GO is missing a required digest") end
    end' GO-COUNT.json | sha256sum --strict -c -
  sha256sum --strict -c machine.sha256 finite.sha256
  test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 8388608
  test "$(df -Pk . | awk 'NR == 2 {print $4}')" -ge 6291456
  test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 8192
  mkdir runs
  /usr/bin/time -v -o runs/c128.resources.txt target/release/full-compact-count count c128 \
    c128.uort d30-receipt.json seed1.d11 runs/c128 >runs/c128.log 2>&1
  /usr/bin/time -v -o runs/c1.resources.txt target/release/full-compact-count count c1 \
    c1.tokens.u16 pick-p10.json seed1.d11 runs/c1 >runs/c1.log 2>&1
  sha256sum runs/{c128,c1}/{lookup.jsonl,compact.counts,receipt.json} \
    runs/{c128,c1}.{resources.txt,log} > results.sha256
  ;;
*) exit 2 ;;
esac
