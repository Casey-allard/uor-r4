#!/usr/bin/env bash
# scale_sweep.sh --- the #514 saturation sweep, corrected by #729.
#
# Derive several fixed-partition corpora from ONE finalized recorded corpus and
# run compile -> cover -> score in four sibling roots per requested size. The
# Rust sampler preserves the source corpus's train/held story partition and
# reports the number of rows it could select without splitting a story run.
#
# Usage:
#   scripts/scale_sweep.sh <src-meta> <src-recs> <vocab-size> [record-sizes...]
# Example:
#   scripts/scale_sweep.sh obs/state.bin obs/merged.bin 49152 \
#       50000 200000 800000 1995026
#
# With no record sizes, the sweep uses each of 50k, 200k, and 800k that is
# strictly below the finalized source size, followed by the exact source size.
# R4 overrides the r4 binary path (default ./target/release/r4); WORK overrides
# the scratch dir (default /tmp/scale-sweep).
set -euo pipefail

R4=${R4:-./target/release/r4}
WORK=${WORK:-/tmp/scale-sweep}

if [ "$#" -lt 3 ]; then
    echo "usage: scripts/scale_sweep.sh <src-meta> <src-recs> <vocab-size> [sizes...]" >&2
    exit 2
fi
SRC_META="$1"
SRC_RECS="$2"
VOCAB="$3"
shift 3

# Resolve and validate EVERY requested target before creating WORK or a case.
# This is deliberately one preflight: a late invalid target must not leave an
# apparently usable prefix of a sweep behind.
TARGET_WORDS="$(python3 - "$SRC_META" "$@" <<'PY'
import struct
import sys

meta_path = sys.argv[1]
try:
    with open(meta_path, "rb") as source:
        meta = source.read()
except OSError as error:
    raise SystemExit(f"cannot read source corpus metadata {meta_path}: {error}")
if len(meta) != 25 or meta[24] != 1:
    raise SystemExit(
        f"source corpus metadata is not one finalized 25-byte record: {meta_path}"
    )
source_records = struct.unpack_from("<Q", meta)[0]
if source_records == 0:
    raise SystemExit("source corpus contains zero records")

raw_targets = sys.argv[2:]
if raw_targets:
    targets = []
    seen = set()
    for raw in raw_targets:
        if not raw.isascii() or not raw.isdecimal():
            raise SystemExit(f"record target must be a positive decimal integer: {raw!r}")
        target = int(raw)
        if target == 0:
            raise SystemExit("record target must be greater than zero")
        if target > source_records:
            raise SystemExit(
                f"record target {target} exceeds finalized source size {source_records}"
            )
        if target not in seen:
            targets.append(target)
            seen.add(target)
else:
    targets = [n for n in (50_000, 200_000, 800_000) if n < source_records]
    targets.append(source_records)

print(" ".join(str(target) for target in targets))
PY
)"
read -r -a SIZES <<< "$TARGET_WORDS"

mkdir -p "$WORK"
printf '%-12s %-12s %-12s %-12s %-12s %-12s\n' \
    requested actual train held top1_rule12 exct_miss_%

for N in "${SIZES[@]}"; do
    CASE="$WORK/n-$N"
    INPUT="$CASE/input"
    COMPILED="$CASE/compiled"
    COVER="$CASE/cover"
    SCORE="$CASE/score"
    mkdir -p "$CASE"

    # Always use the certified Rust transaction, including for the exact-full
    # case. That keeps execution provenance and dense/attention sidecars on
    # the same publication path instead of special-casing a file copy.
    "$R4" transformerless subsample-recorded-corpus \
        --src-meta "$SRC_META" --src-recs "$SRC_RECS" \
        --out-meta "$INPUT/corpus.meta" --out-recs "$INPUT/corpus.records" \
        --records "$N" >/dev/null

    "$R4" transformerless compile-recorded \
        --corpus-meta "$INPUT/corpus.meta" --corpus-recs "$INPUT/corpus.records" \
        --vocab-size "$VOCAB" --out "$COMPILED" >/dev/null

    ATTENTION_OPERATOR="$(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1])),separators=(",",":")))' "$COMPILED/attention_operator.json")"
    OPERATOR_ARGS=(--attention-operator "$ATTENTION_OPERATOR")
    if [ -f "$COMPILED/dense_operator.json" ]; then
        DENSE_OPERATOR="$(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1])),separators=(",",":")))' "$COMPILED/dense_operator.json")"
        OPERATOR_ARGS+=(--dense-operator "$DENSE_OPERATOR")
    fi

    "$R4" transformerless cover \
        --corpus-meta "$COMPILED/corpus.meta" --corpus-recs "$COMPILED/corpus.records" \
        --artifacts "$COMPILED/tless_artifacts.bin" \
        "${OPERATOR_ARGS[@]}" \
        --out "$COVER" >/dev/null

    "$R4" transformerless score \
        --corpus-meta "$COMPILED/corpus.meta" --corpus-recs "$COMPILED/corpus.records" \
        --artifacts "$COMPILED/tless_artifacts.bin" --cover "$COVER/cover.r4g1" \
        --quality-profile relative_tla --out "$SCORE" >/dev/null

    python3 - \
        "$N" "$INPUT/corpus.meta" "$COVER/cover_report.json" \
        "$SCORE/score_report.json" <<'PY'
import json
import math
import struct
import sys

requested = int(sys.argv[1])
with open(sys.argv[2], "rb") as source:
    meta = source.read()
if len(meta) != 25 or meta[24] != 1:
    raise SystemExit("subsample output is not one finalized 25-byte corpus metadata record")
actual = struct.unpack_from("<Q", meta)[0]

with open(sys.argv[3], encoding="utf-8") as source:
    cover = json.load(source)
with open(sys.argv[4], encoding="utf-8") as source:
    score = json.load(source)

def count(value, label):
    if type(value) is not int or value < 0:
        raise SystemExit(f"{label} is not a nonnegative integer")
    return value

inputs = cover.get("inputs", {})
train = count(inputs.get("train_observations"), "cover train_observations")
held = count(inputs.get("held_out_observations"), "cover held_out_observations")
if train + held != actual:
    raise SystemExit(
        f"cover partition mismatch: train {train} + held {held} != actual {actual}"
    )

gate = score.get("gate_c", {})
distribution = score.get("distribution", {})
score_held = count(gate.get("held_out_population"), "score held_out_population")
distribution_held = count(
    distribution.get("held_out_positions"), "score distribution held_out_positions"
)
if score_held != held or distribution_held != held:
    raise SystemExit(
        "score population mismatch: "
        f"cover held {held}, Gate C held {score_held}, distribution held {distribution_held}"
    )

top1 = gate.get("rule12_precedence", {}).get("top1_agreement")
miss = distribution.get("exct_miss_rate")
for value, label in ((top1, "top1_agreement"), (miss, "exct_miss_rate")):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise SystemExit(f"score {label} is not numeric")
    if not math.isfinite(value) or not 0.0 <= value <= 1.0:
        raise SystemExit(f"score {label} is outside [0, 1]")

print(
    f"{requested:<12} {actual:<12} {train:<12} {held:<12} "
    f"{top1 * 100:<12.2f} {miss * 100:<12.2f}"
)
PY
done

echo
echo "Compare requested and actual before interpreting a curve; complete-story selection may undershoot."
echo "The #729 fixed-partition harness produces evidence for a future scaling-law decision; it does not reuse the retired #531 calibration claim."
