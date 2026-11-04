# ORDER: cpu-training-pilot, deepseek-130 (unseen-key target prototypes)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `6f653f2b4`.

## Goal

Test a target-specific local point representation on the fixed unseen-key
opening panel. For each supervised first-reply token, retain an observed medoid
and one farthest residual point, rather than either all raw points or one
medoid. Evaluate the resulting two-prototype table against point-off and a
matched shuffled-target prototype table on both public model seeds.

This is a D84 CPU development-research run under #3506. It can close only the
fixed two-prototype construction. It cannot establish GPU qualification,
complete-reply quality or useful conversation.

## Cancel probe (run first)

1. `origin/main` contains the accepted deepseek-129 merge and the CPU driver
   still derives the fixed 8-key / 60-training / 60-evaluation split.
2. The preceding raw-point, eight-row vote and one-medoid comparisons remain
   recorded as negatives or subthreshold evidence; none already tests a
   balanced multi-prototype table.
3. No open ORDER claims the point-transfer driver or this experiment. Claude's
   open #3657 touches only the long-context sheet and its `one-model-end-to-end`
   ORDER.
4. The public artifact, corpus and expected hashes remain available and match
   the preceding FIT sheets.

**Result at `6f653f2b4`:** not cancelled. Deepseek-104 was mixed and failed its
strict control bar; deepseek-123's one-medoid table failed after losing the
raw neighborhood. The fixed two-prototype construction is materially different
and remains untested. No open order overlaps this CPU driver.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-point-transfer-driver.rs`
- `docs/harness/prep/cpu-training-pilot-point-prototypes-2026-11-04.md`
- `docs/harness/claims.json`
- `docs/labs/cpu-training-pilot-point-prototypes-2026-11-04/resource-ledger.json`
- `docs/loop/tasks/cpu-training-pilot/deepseek-130/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No serving, integer-session, CUDA, Claude-owned or model-artifact source change
is in scope.

## Done when

1. The driver derives a deterministic observed-medoid-plus-farthest-residual
   table with exactly two or fewer rows per supervised target, and refuses a
   malformed, one-target, unchanged or vacuous control.
2. Focused tests cover selection, L1 residual choice and ties, target balance,
   shuffled-target preservation, served fallback, distance equality and exact
   CPU/serving parity.
3. A prospective sheet with pass, fail, stop, two-seed power, resource and
   storage boundaries receives TEST FITNESS before any public model run.
4. Seeds 1 and 2 run on the fixed split; a seed-1 repeat checks every
   non-timing report field. Every arm retains exact query, token and kept-cell
   parity.
5. The REPORT records the treatment and matched controls, preserves both
   negative and positive outcomes, and links the evidence and binary pins.

## Stop and ask if

- The public inputs, split, source worktree or binary pins drift.
- The prototype construction equals the raw table or cannot produce at least
  two targets.
- The shuffled-target control preserves points and target counts but does not
  change the served image.
- Any CPU/serving parity mismatch, exact-memory hit on held-out keys,
  nondeterministic repeat or evaluation-time tuning occurs.
- A change would require integer-serving or Claude-owned source.
