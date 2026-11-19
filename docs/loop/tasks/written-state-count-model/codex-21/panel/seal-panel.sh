#!/usr/bin/env bash
set -euo pipefail
umask 077
[[ $# == 7 ]] || {
    echo "usage: seal-panel.sh STAGE PANEL_BIN CAPTURE TOKENIZER1 TOKENIZER2 READER_A READER_B" >&2
    exit 2
}
stage=$1
panel=$2
capture=$3
tokenizers=("$4" "$5")
reader_a=$6
reader_b=$7
source=$(cd "$(dirname "$0")" && pwd -P)
sha() { shasum -a 256 "$1" | cut -d' ' -f1; }

raw="$stage/seed1/held/public-text.json"
cmp "$raw" "$stage/seed2/held/public-text.json"
jq -ne --slurpfile raw "$raw" --slurpfile reader_a "$reader_a" --slurpfile reader_b "$reader_b" \
    --slurpfile seed1 "$stage/seed1/held/grades.json" --slurpfile seed2 "$stage/seed2/held/grades.json" \
    -f "$source/adjudicate.jq" > "$stage/adjudication.json"
jq -e '.all_agree' "$stage/adjudication.json" >/dev/null
cp "$reader_a" "$stage/read-a.json"
cp "$reader_b" "$stage/read-b.json"
jq 'map({index,person})' "$reader_a" > "$stage/person-only.json"
jq -f "$source/../../codex-18/panel/grader-reading.jq" "$reader_b" > "$stage/reader-grader.json"
names="$stage/held-names.json"
"$panel" seal "$raw" "$stage/person-only.json" "$names"
for seed in 1 2; do
    tokenizer=${tokenizers[$((seed - 1))]}
    held="$stage/seed$seed/held"
    prepared="$stage/seed$seed/prepared"
    "$capture" prepare "$tokenizer" "$(sha "$tokenizer")" "$raw" "$(sha "$raw")" "$names" "$(sha "$names")" "$prepared"
    cmp "$held/blind.json" "$prepared/blind.json"
    jq -en --slurpfile r "$prepared/resolved-requested.json" --slurpfile intended "$held/resolved.json" '
        ($r[0].rows | map({index, prefix_sha256, requested: .requested_span})) == $intended[0]
    ' >/dev/null
done
(
    cd "$stage"
    export LC_ALL=C
    find . -type f ! -name INPUTS.sha256 | sort | while IFS= read -r file; do
        shasum -a 256 "${file#./}"
    done > INPUTS.sha256
    shasum -a 256 --check INPUTS.sha256
)
