#!/bin/bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
d26="$here/../../codex-26/panel"
extractor=${1:?d25-contextual-source-panel executable}
panel=${2:?d30-panel executable}
legacy=${3:?D21 assertion-agreement carrier}
tokenizer=${4:?protocol-2 tokenizer}
root=${5:?new run directory}
mkdir "$root"
mkdir "$root/fixture"
cp "$d26/train.tsv" "$root/fixture/train.tsv"
cp "$here/held.tsv" "$root/fixture/held.tsv"
cp "$here/held.donors.tsv" "$root/fixture/held.donors.tsv"
awk -F '\t' '$1 >= 501' "$d26/key.tsv" > "$root/fixture/key.tsv"
cat "$here/held.key.tsv" >> "$root/fixture/key.tsv"
"$extractor" prepare-d26 "$legacy" 02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3 \
    "$tokenizer" "$root/fixture" "$root/native"
"$panel" "$tokenizer" "$root/native" "$root/panel"
mv "$root/native" "$root/panel/native"
