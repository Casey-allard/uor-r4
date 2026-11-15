# ORDER: one-model-end-to-end, claude-386 (V2(b) part 2: which vector read chooses the asked fact)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: claude-385's approved head `34c00f47878857e1beac2f8f5faa6bea7b6688b4` (#3985, V2(b) part 1, waiting on the org CI queue), which already holds `origin/main` at `2cac733bd`; merged onto `main` once #3985 lands. The runs use the binary built at that head, the same float-crate tree #3985 merges.

Casey, 15 Nov 2026, relayed by the starting session: V2(b) is two PRs, the repair unit (claude-385, merged) then "the selection test on the repaired reads, with a non-author FIT first." "If a repair turns out to be the mechanism and not the instrument … report that as the result. Do not tune it away."

## Goal

Run the pre-registered selection test of [the sheet](../../../../harness/prep/one-model-end-to-end-vector-read-selection-2026-11-14.md) as updated here: does any of the four state-keyed reads (Starling Lorentz, Starling L2, prime-route, Flock) learn to choose the asked fact on the two-class recall task when the query is not the key? Report the outcome the sheet's map names, with every arm's numbers.

## Cancel probe

A selection result for these reads on `main`. At base: none (claude-381 built the reads, claude-385 repaired them; neither ran the bar).

## Steps

1. The sheet updated for claude-385's repairs, and a fresh non-author FIT (FIT WITH CHANGES, applied; recorded on the sheet's last line). `uor-harness prep-check` passes.
2. wick, at the head: build, the `vector_read` cuda tests and the binary's tests.
3. wick: 28 runs of 4,096 steps (14 arm-tasks, seeds 1 and 2), two at a time; a third seed where the sheet's outcome 7 calls for it. Outcome 0 checked at step 0 and step 512 of every read arm.
4. Results fetched to the laptop, `SHA256SUMS`, uploaded to the HF store under `results/claude/one-model-end-to-end-vector-read-selection/`.
5. REPORT with negatives first, the outcome named, one notebook row.

## Native-space five items (SKILL)

None is met: a float reference test (D85), an inventory-class unit. Its result picks the read for V2(c), where the integer form and the five items belong.

## Files in scope

The sheet, this folder, the notebook row in `docs/research/notebook/float-training.md`. No code change; a fault outcome 0 names is repaired in its own unit.

## Done when

1. The sheet's FIT line and prep-check pass.
2. wick: every run finished, or stopped by a named stop rule; the outcome 0 checks recorded.
3. REPORT names the sheet's outcome with the bar's numbers per arm, bucket and seed; the PR through review, queue and `close --verify`.
