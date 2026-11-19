#!/usr/bin/env bash
set -euo pipefail
cd /workspace/d26
mkdir -p "$HOME/uor-codex/20261118-diagnostics"
mkdir raw
sha256sum --quiet -c stage.sha256
score-stage/d25-source finite raw/planted-bank > raw/finite.log
score-stage/d25-source fit finite/train.rows raw/donor-bank
for mode in native donors; do
    src=finite/held.rows
    if [[ "$mode" == donors ]]; then src=finite/held-donors.rows; fi
    score-stage/d25-source predict "$src" raw/donor-bank "raw/$mode.trace" "raw/$mode.results"
    cmp "finite/$mode.expected.tsv" "raw/$mode.results"
done
./wick.sh project project-stage raw/projection
for s in train held exposed; do
    for ext in pre post local; do cmp "reference/$s.$ext" "raw/projection/$s.$ext"; done
done
./wick.sh score score-stage raw/score
(cd raw; find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) > raw.sha256
touch complete
