#!/usr/bin/env bash
# Stage contains only native coordinates/tapes and TRAIN grades. No held key is transferred.
set -euo pipefail
umask 077
stage=$(realpath "${1:?sealed stage}")
out=${2:?new result directory}
receipt=$(realpath "${3:?external pre-run receipt}")
cd "$stage"
sha256sum -c inputs.sha256 --status
read -r decision stage_hash fit_hash device_uuid < "$receipt"
[[ $decision == GO && $stage_hash == "$(sha256sum inputs.sha256 | cut -d' ' -f1)" ]]
[[ $fit_hash == "$(sha256sum ../FIT.md | cut -d' ' -f1)" ]]
[[ $device_uuid == "$(nvidia-smi --query-gpu=uuid --format=csv,noheader | head -1)" ]]
mkdir "$out"
for seed in seed1 seed2; do
  for mode in identity positional; do
    for pass in primary cold; do
      run="$out/$seed/$mode/$pass"
      mkdir -p "$run"
      for split in train held; do
        ./d24-project "$mode" "$seed/tables.bin" "$seed/$split.candidates.txt" "$run/$split.features"
        cmp "$run/$split.features" "$seed/$split.$mode.txt"
      done
      ./d24-role fit "$run/train.features" "$seed/train.labels.txt" "$run/role.bank"
      [[ $(wc -c < "$run/role.bank") == 800 ]]
      # Bind the executable, arm, seed tables, training bytes and learned state before inference.
      sha256sum d24-role d24-project "$seed/tables.bin" "$seed/train.labels.txt" \
        "$seed/train.$mode.txt" "$seed/held.$mode.txt" "$run/role.bank" > "$run/binding.sha256"
      sha256sum -c "$run/binding.sha256" --status
      ./d24-role predict "$run/held.features" "$run/role.bank" "$run/predictions.tsv"
    done
    for item in train.features held.features role.bank predictions.tsv; do
      cmp "$out/$seed/$mode/primary/$item" "$out/$seed/$mode/cold/$item"
    done
  done
done
cd "$out"
find . -type f \( -name '*.tsv' -o -name '*.bank' -o -name '*.features' -o -name 'binding.sha256' \) -print0 |
 LC_ALL=C sort -z | xargs -0 sha256sum > results.sha256
sha256sum -c results.sha256 --status
