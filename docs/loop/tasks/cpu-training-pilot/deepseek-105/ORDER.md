# ORDER: cpu-training-pilot, deepseek-105 (8-row point neighborhood)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `79843cca8`.

## Goal

Screen one materially different unseen-key point representation after
deepseek-104's nearest-row negative. Keep the same production training-derived
point rows, but name the opening by a majority vote among the eight nearest
rows instead of trusting the single nearest row. Run the fixed 60/60 unseen-key
split on both public model seeds. Implement serving semantics only if this
screening configuration clears its registered bar.

## Design

- Extend `cpu-point-transfer-driver` with an explicit `--neighbor-vote 8`
  candidate.
- Reuse the same artifact, corpus, training/evaluation groups, 48 trained point
  rows and shuffled-target control as deepseek-104.
- Extract each held-out opening query from the memory-miss served trunk.
- Assert that the nearest-row result from this extraction equals the existing
  point-artifact served result.
- Rank all rows by exact L1 distance, take the first eight, count their targets,
  and choose the commonest target, ties to the smaller token id.
- Compare point-off, nearest, eight-row vote and shuffled-target vote.

This is an offline geometry screening unit. It does not add a new artifact
field, alter the integer point-key schema or claim serving support for the
eighth-nearest rule.

## Tests first

1. A finite table where the nearest row is wrong but the eight-row majority is
   right.
2. Exact-distance ordering, lower-target tie break and `k > rows` handling.
3. The extracted opening query reproduces the existing served nearest result.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-point-neighborhood-2026-11-03.md`
before the two public runs. The bar requires, on both seeds:

- zero query, token or kept-cell changes outside the point fallback;
- nearest extraction matches the existing served point artifact;
- eight-row opening top-1 strictly exceeds point-off;
- eight-row opening top-1 strictly exceeds both nearest and shuffled-target
  nearest on the same 60 openings;
- report the paired three-sigma difference separately.

## Done when

1. Both seeds complete without tuning the fixed point rows, key split, `k = 8`,
   random seed or bar.
2. The candidate screen outcome is recorded and, if positive, the next serving
   implementation is explicitly queued.
3. Package tests, strict Clippy, formatting, evidence checksums, FIT, report,
   exact-head review, protected merge and fresh-main verification pass.

## Cancel probe (run first)

- deepseek-104 is merged as `1968430e3` and records the nearest-row negative.
- `PointKeyImage` exposes ordered Q12 rows and targets without changing the
  serving schema.
- The fixed public inputs and 60/60 split remain available locally.
- Claude's open #3629 changes float scorer defaults, not this CPU crate.

## Stop and report

Stop on split drift, nearest extraction mismatch, non-exact distance ordering or
any proposal to present the offline vote as already served. Do not tune after
observing the two seeds. A positive screen requires a separate serving-schema
unit with its own tests and audit.
