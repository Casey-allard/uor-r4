# ORDER: one-model-end-to-end, claude-385 (V2(b) part 1: repair the vector reads V2(a) exposed)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `cb91a1baa879feb9a16e4906aaa64a036411d845` (claude-381, V2(a), #3978).

This ORDER replaces the V2(b) ORDER of this folder's first commit, before any work landed. Casey, 15 Nov 2026, relayed by the starting session: "can we please resolve the problems this is exposing? please figure out how to resole the issues". V2(b) is now two PRs: this repair unit (instrument, no bar), then the selection test (claude-386) on the repaired reads, with its sheet's non-author FIT first.

## Goal

Repair every fault claude-381's REPORT names, each with a test that fails without its fix. Flags off stays bit-identical to the base. If a repair turns out to be the mechanism and not the instrument, the REPORT says so; nothing is tuned away.

## Cancel probe

The faults already repaired on `main`. At base: none (claude-381 "What did not work" 1–4 and "Still open").

## Repairs (`vector_reads.rs`, which only this unit writes)

1. **Starling never opens.** Distances scale with the state's norm, far above the fixed offset 2.5. The first training forward sets the offset to the lower median of each row's nearest earlier distance. Test: a fixture with a known positive at distance `d` opens and chooses it.
2. **The credit costs the trunk.** Stop-gradient on the stream under the address, the default; `--vector-read-stream-grad 1` restores it, `--vector-read-credit` sets the weight. Test: with the stop, the trunk's gradient from the credit is exactly zero.
3. **`--recall` refuses a vector read.** Build that path: optimizer, credit, counts, save.
4. **No cut control.** An eval-only cut reading: the read unset for a second held-out pass, and the read's choice at each scored row.
5. **Lorentz precision, and the one product between positions.** `e = (|q - k|^2 - (l_q - l_k)^2) / 2`, `l_q - l_k = (|q|^2 - |k|^2) / (l_q + l_k)`. Test against the f64 arccosh order on a large-norm fixture.
6. **Prime-route memory.** A detached 120 × 120 cell-pair energy table indexed by the two cells, `(batch, time, time)` per block.
7. **The straight-through factor's gradient is untested.** Test: the payload's gradient equals minus the chosen distance's, against central differences of an f64 reference distance.
8. **Ties.** Every read takes the latest position on equal scores, Flock too. Test equal scores for each kind.
9. **Shared-card timing.** Off and each read timed side by side at the same moment on wick; ratios reported.

## Native-space five items (SKILL)

None is met: a float reference repair, an inventory-class unit with no bar. The five items belong to V2(c).

## Files in scope

`vector_reads.rs`, `in_space_long.rs` (`run_recall`, the recall reading and their tests only; claude-380 edits `run_long`), `bin/train-in-space.rs` (flags only), this folder, the notebook row in `float-training.md`.

## Done when

1. Laptop: each repair's host test, with a negative gate.
2. wick: every `vector_read` cuda test passes at the head; each repair's device test fails with its fix set aside.
3. wick: the `p` recipe with flags off, 20 steps: 3 of 3 eval lines and 7 of 7 checkpoint files byte-identical to the base; `--recall` B2 with no read, 64 steps: `recall.jsonl` and the model files identical to the base.
4. wick: each read smokes on `p` (256 steps) and on `--recall` B2 (64 steps), no NaN, step time beside off at the same moment.
5. REPORT with negatives first, one notebook row; the PR through review, queue and `close --verify`, before any selection run.
