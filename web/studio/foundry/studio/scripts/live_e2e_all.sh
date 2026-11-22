#!/bin/bash
# Every Prism app fixture through tools/live_e2e.mjs, against a served Forge or a built shell directory.
# A timing budget can be missed by a busy machine, so each app gets three attempts and fails only when all
# three miss; a functional failure (an edit not saved, refused or undone, a reload) misses every attempt.
#   scripts/live_e2e_all.sh <base url | built shell dir> <out dir>
set -uo pipefail
cd "$(dirname "$0")/.."
TARGET="$1"; OUT="$2"; failed=0; mkdir -p "$OUT"
for spec in tools/fixtures/live/rsvp.json tools/fixtures/specs/parking-fee.json tools/fixtures/specs/library-fee.json tools/fixtures/live/verdant.lex.tex; do
  name=$(basename "$spec"); ok=""
  for attempt in 1 2 3; do
    if node tools/live_e2e.mjs "$TARGET" "$spec" "$OUT/$name-$attempt" > "$OUT/$name-$attempt.log" 2>&1; then ok=1; break; fi
    echo "$name, attempt $attempt:"; tail -n 40 "$OUT/$name-$attempt.log"
  done
  if [ -n "$ok" ]; then echo "ok   $name (attempt $attempt): $(grep -E '^first node' "$OUT/$name-$attempt.log" | cut -c30-)"
  else echo "FAIL $name missed its budgets three times"; failed=1; fi
done
exit $failed
