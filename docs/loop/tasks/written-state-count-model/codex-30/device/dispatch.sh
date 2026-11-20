#!/usr/bin/env bash
set -euo pipefail

cd /workspace/d30
mode=${1:?build-finite, project, or score}
case "$mode" in
build-finite)
  sha256sum --quiet -c build-input.sha256
  mkdir -p bin finite-out
  nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > finite-out/device.csv
  nvcc --version > finite-out/nvcc.txt
  sm=$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader | tr -d '.')
  [[ "$sm" =~ ^[0-9]+$ ]]
  arch=(-gencode "arch=compute_${sm},code=sm_${sm}")
  sed '/^extern /,$d' discrete.cu > discrete_read.cuh
  nvcc -std=c++17 -O2 "${arch[@]}" -I . context.cu -o bin/d25-context
  nvcc -std=c++17 -O2 "${arch[@]}" codex-24/device/project.cu -o bin/d24-project
  nvcc -std=c++17 -O2 "${arch[@]}" codex-25/device/source.cu -o bin/d25-source
  nvcc -std=c++17 -O2 "${arch[@]}" codex-30/capacity/capacity.cu -o bin/d30-capacity
  for exe in bin/*; do
    cuobjdump -lelf "$exe"
    cuobjdump -lptx "$exe"
    ldd "$exe"
  done > finite-out/binary-inspect.txt
  if grep -E 'libnvrtc|libcublas|\.ptx([[:space:]]|$)' finite-out/binary-inspect.txt; then
    exit 1
  fi
  sha256sum bin/* > finite-out/binaries.sha256
  bin/d25-source finite finite-out/standalone.bank
  bin/d25-source fit finite-in/train.rows finite-out/planted.bank
  for mode in native donors zero; do
    case "$mode" in
      native) input=held;;
      donors) input=held-donors;;
      zero) input=held-zero;;
    esac
    bin/d25-source predict "finite-in/$input.rows" finite-out/planted.bank \
      "finite-out/$mode.trace" "finite-out/$mode.results"
    cmp "finite-in/$mode.expected.tsv" "finite-out/$mode.results"
  done
  bin/d30-capacity finite roots.tsv finite-out/capacity.map
  cp build-input.sha256 finite-out/
  ;;
project)
  sha256sum --quiet -c project-input.sha256
  sha256sum --quiet -c finite-out/binaries.sha256
  for out in projection projection-cold; do
    mkdir "$out"
    bin/d25-context project-in/held.packet "$out/held.post" "$out/held.pre"
    bin/d25-context project-in/trajectory.packet "$out/trajectory.post" "$out/trajectory.pre"
    bin/d24-project positional project-in/tables.bin project-in/held.local-input.txt "$out/held.local"
    cmp project-in/held.local-expected.txt "$out/held.local"
  done
  diff -rq projection projection-cold
  ;;
score)
  sha256sum --quiet -c score-input.sha256
  sha256sum --quiet -c finite-out/binaries.sha256
  for out in score score-cold; do
    mkdir "$out"
    for arm in nominal swap zero; do
      bin/d25-source predict "score-in/$arm.rows" recurrence.bank "$out/$arm.trace" "$out/$arm.results"
    done
    bin/d30-capacity run roots.tsv score-in/nominal.rows recurrence.bank "$out/capacity.map"
  done
  diff -rq score score-cold
  ;;
*)
  exit 2
  ;;
esac
