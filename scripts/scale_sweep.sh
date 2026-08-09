#!/usr/bin/env bash
# scale_sweep.sh --- the #514 saturation sweep.
#
# Sub-sample ONE observation corpus to several record counts and record where
# held-out top-1 and EXCT-miss flatten. That knee is the teacher's needed scale;
# fitting log(N_knee) against log(S) across teachers calibrates the `beta` in
# `r4 transformerless recommend-scale` (docs/scaling_law.md). You observe once at
# the top scale --- this reuses that one corpus, no re-observation.
#
# Usage:
#   scripts/scale_sweep.sh <src-meta> <src-recs> <vocab-size> [record-sizes...]
# Example (a 360M wiki observe at 2M records):
#   scripts/scale_sweep.sh obs/state.bin obs/merged.bin 49152 \
#       50000 200000 800000 2000000
#
# R4 overrides the r4 binary path (default ./target/release/r4); WORK overrides
# the scratch dir (default /tmp/scale-sweep).
set -euo pipefail

R4=${R4:-./target/release/r4}
WORK=${WORK:-/tmp/scale-sweep}

if [ "$#" -lt 3 ]; then
    echo "usage: scripts/scale_sweep.sh <src-meta> <src-recs> <vocab-size> [sizes...]" >&2
    exit 2
fi
SRC_META="$1"; SRC_RECS="$2"; VOCAB="$3"; shift 3
SIZES=("$@")
if [ "${#SIZES[@]}" -eq 0 ]; then
    SIZES=(50000 200000 800000 2000000)
fi

mkdir -p "$WORK"
printf '%-12s %-10s %-12s %-12s\n' records held_out top1_rule12 exct_miss_%

# NOTE: `compile-recorded --out "$D"` re-emits `corpus.meta`/`corpus.records`
# into "$D", so the sub-sampled corpus is written under DISTINCT names
# (`sub.meta`/`sub.records`) that compile cannot clobber; cover and score read
# those, never compile's emitted `corpus.*`.
for N in "${SIZES[@]}"; do
    D="$WORK/n-$N"
    mkdir -p "$D"
    python3 scripts/mc1_subsample_corpus.py \
        --src-meta "$SRC_META" --src-recs "$SRC_RECS" \
        --out-meta "$D/sub.meta" --out-recs "$D/sub.records" \
        --records "$N" >/dev/null
    "$R4" transformerless compile-recorded \
        --corpus-meta "$D/sub.meta" --corpus-recs "$D/sub.records" \
        --vocab-size "$VOCAB" --out "$D" >/dev/null 2>&1
    "$R4" transformerless cover \
        --corpus-meta "$D/sub.meta" --corpus-recs "$D/sub.records" \
        --artifacts "$D/tless_artifacts.bin" --out "$D/graph-cover" >/dev/null 2>&1
    "$R4" transformerless score \
        --corpus-meta "$D/sub.meta" --corpus-recs "$D/sub.records" \
        --artifacts "$D/tless_artifacts.bin" --cover "$D/graph-cover/cover.r4g1" \
        --quality-profile relative_tla --out "$D/graph" >/dev/null 2>&1
    python3 - "$D/graph/score_report.json" "$N" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
n = sys.argv[2]
gc = d["gate_c"]
dist = d.get("distribution", {})
ho = gc.get("held_out_population", 0)
t1 = gc.get("rule12_precedence", {}).get("top1_agreement", 0.0)
miss = dist.get("exct_miss_rate", float("nan"))
print(f"{n:<12} {ho:<10} {t1 * 100:<12.2f} {miss * 100:<12.2f}")
PY
done

echo
echo "The knee is the smallest N past which top1 and exct_miss stop moving."
echo "Feed the (N_knee, teacher-S) points into the recommend-scale beta calibration (docs/scaling_law.md)."
