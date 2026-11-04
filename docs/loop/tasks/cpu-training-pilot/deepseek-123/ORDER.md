# ORDER: cpu-training-pilot, deepseek-123 (target point medoids)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `5b3b8f361`.

## Goal

Run the next distinct training-through-serving comparison after rank-margin,
memory-weight and raw point rows. Derive one geometric medoid per supervised
opening target from training-only point rows, store it in the existing point-key
format, and evaluate it against point-off and a duplicate-safe shuffled medoid
on both public seeds and the fixed unseen-key split.

This is D84 CPU development research. It is not GPU qualification, complete-reply
evidence or a language-capability claim.

## Design

- Preserve all existing raw-row outputs byte for byte.
- Group the complete training point rows by supervised target.
- For each target, select one observed member minimizing the exact sum of L1
  distances to every member of that target group. Break equal total distances by
  lexicographic point bytes and then target id. This is an observed medoid, so no
  new coordinate is invented and no evaluation row participates in training.
- Sort medoids deterministically and attach them to the same memory-bearing
  artifact. The medoid table uses the existing point-key schema at `vote_k = 1`.
- Build a matched control with the same medoid points and a deterministic
  permutation of their targets. If the first permutation leaves the image
  unchanged, rotate once; refuse an unchanged or all-equal control.
- Evaluate the fixed 60 held-out examples with:
  1. point-off;
  2. raw-row nearest;
  3. raw-row vote at `k = 8` when the existing run requests it;
  4. target medoid;
  5. shuffled-target medoid.
- Run the complete served-forward audit for every arm and report opening and
  complete teacher-forced readings, row counts and artifact hashes.

## Pre-registration

Write
`docs/harness/prep/cpu-training-pilot-point-medoid-2026-11-04.md` before the two
public runs. The primary bar is:

- exact CPU/serving parity and all instrument checks pass on both seeds;
- medoid opening top-1 exceeds both point-off and shuffled-medoid on each seed;
- each paired medoid-minus-control gain is at least
  `max(2, ceil(2 * sqrt(b + c)))` over the 60 held-out rows;
- medoid complete teacher-forced top-1 is not lower than either control.

Raw-row nearest and vote are reported descriptively. Their outcomes do not
change the medoid bar after the run. A positive is a bounded unseen-key
geometry result; a negative closes only this medoid construction.

Cancel probe: public inputs are local; the driver has raw rows but no medoid;
Claude's open #3649 touches only `crates/uor-r4-float-train/**`.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-point-medoid-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-123/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. Finite tests cover exact medoid selection, distance ties, target ties,
   deterministic construction, non-vacuous shuffled targets, changed served
   decisions and malformed-dimension refusal.
2. The pre-registration is FIT before either public run.
3. Both public seeds run all arms over the fixed 60-example unseen-key split;
   every arm reaches exact CPU/integer parity over query coordinates, tokens and
   kept cells.
4. The report records the registered paired result, artifact hashes, timings,
   resource use, deterministic reconstruction and whether the construction
   closes or earns a follow-up.
5. Full CPU release tests, strict no-deps Clippy, formatting, report, roadmap,
   catalog, prep and loop gates pass, followed by fresh exact-head review,
   protected merge and fresh-main verification.

## Stop

Stop on selection overlap, an evaluation exact-memory hit, a non-vacuous medoid
set smaller than two targets, an unchanged shuffled control, nondeterministic
ordering, a CPU/serving mismatch, input drift, or any need to edit Claude-owned
float-training files.
