#!/usr/bin/env bash
set -euo pipefail
mode=${1:?project or score}
stage=$(realpath "${2:?stage directory}")
mkdir "${3:?new observation directory}"
out=$(realpath "$3")
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -w 12 8
test -z "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)"
cd "$stage"
sha256sum --quiet -c inputs.sha256
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > "$out/device.txt"
case "$mode" in
  project)
    for split in train held exposed; do
      ./d25-context "$split.ctx" "$out/$split.post" "$out/$split.pre"
      ./d24-project positional tables.bin "$split.local-input.txt" "$out/$split.local"
    done;;
  score)
    for run in primary cold; do
      mkdir "$out/$run"
      for view in post relation recurrence; do
        dest="$out/$run/$view"
        mkdir "$dest"
        ./d25-source fit "$view/train.rows" "$dest/bank"
        for split in held exposed donor; do
          ./d25-source predict "$view/$split.rows" "$dest/bank" "$dest/$split.trace" "$dest/$split.results"
        done
      done
    done
    diff -rq "$out/primary" "$out/cold";;
  *) echo "unsupported phase" >&2; exit 2;;
esac
(cd "$out"; find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) > "$out.sha256"
