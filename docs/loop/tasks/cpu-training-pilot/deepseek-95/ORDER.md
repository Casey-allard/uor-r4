# ORDER: repeated-key memory-weight comparison, deepseek-95

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `5e5c2590e567dd6d080b990e5977caccc5147a78`.

## Goal

Run the first CPU training comparison whose updates are selected and graded
through the complete served full path rather than writer-only rank margin.
Derive opening/reply memory records from even-ordinal public documents, attach
them to both public raw FBS artifacts, train movement weights only on those
documents, and evaluate odd-ordinal examples that share the same eight-token
opening keys.

The matched arms are frozen zero weights, eight trained Q24 steps, a
deterministic random-move control with the same number of accepted moves, and
memory-off. This remains CPU development research under D84 and cannot replace
GPU qualification.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-memory-heldout-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-95/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. The corpus reader recovers contiguous masked assistant responses as
   teacher-forced prompt/reply examples with deterministic document ordinals
   and context-safe truncation.
2. The memory path can derive deterministic opening/reply tables from examples,
   attach them to a UORLUT artifact without changing existing bytes, read them
   back, and validate weights.
3. A driver selects repeated opening keys present in both even training and odd
   evaluation documents, trains only on even examples, and evaluates only odd
   examples. Training and evaluation keys may overlap; examples and documents do
   not.
4. Frozen, trained, random-move and memory-off arms start from the same artifact
   and sample set. The random arm makes the same number of Q24 coordinate moves
   as the trained arm and records every coordinate.
5. Every published arm passes the complete CPU/serving audit across every
   evaluation example. Artifact hashes, sample counts, selected key groups,
   accepted moves, objective readings, top-1 and wall/RSS are retained.
6. A FIT pre-registration exists before the nonzero-step run and fixes the two
   artifact seeds, selected-key rule, eight-step budget, controls, positive bar
   and stop conditions.
7. Focused/full CPU release tests, strict Clippy, package formatting, report,
   roadmap and loop checks pass.
8. The REPORT preserves a negative if trained held-out ranking or distance does
   not beat both frozen and random controls on both seeds.

## Stop if

- Any artifact or corpus identity differs from the prep sheet.
- The memory tables or key selection differ between seeds.
- Any arm loses exact CPU/serving parity.
- The first seed projects the two-seed run beyond 30 CPU minutes.
- Any implementation needs float serving, softmax/squashing, dense matmul or an
  edit to Claude's active branch.
