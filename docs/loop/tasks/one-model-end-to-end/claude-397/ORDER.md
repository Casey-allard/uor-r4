# ORDER: one-model-end-to-end, claude-397 (can C2′'s target be found from the stream at all?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `d9d532522`. It reads claude-395's saved results (PR #4017) and nothing else from that branch.

## Why this unit

claude-395 trained the native Q31 read's gates on real states. Its outcome is 6, "not the targets": the other-fact control C2′ broke bar 4 at both ext states. Its REPORT left two readings open:
- the rule drifts toward rows the stream makes salient;
- the gates cannot reach the other fact on these states.

The task's generator suggests a third reading, which this unit tests: **the control's target may not be recoverable from the stream at all.** In `recall_query_window`:
- an episode's two relations are `relations[2·slot]` and `relations[2·slot + 1]` of a random shuffle;
- nothing in the tokens marks which facts pair up;
- the other relation never appears in a tail slot.

At slot `i`'s scored row, a reader that knows the generator can only narrow the other fact to the facts that are:
- not the asked fact;
- not asked in an earlier slot (that query is visible);
- at a distance from slot `i`'s query inside the asked fact's bucket range, clipped as the generator clips it.

On rows where more than one fact qualifies, **no read, with any gates, can know which fact the target is.** Then C2′'s miss says nothing about either the rule or the states.

This is a host count over the task generator and claude-395's saved picks. No model computes, so it needs no GPU and no sheet. The readings and their meanings are registered here, before anything is counted.

## Definitions

For a held-out query at slot `i`, with query position `q`, asked fact `a`, true other fact `o`, and the asked fact's bucket `[low, high]`:
- `F` is the window's 8 facts, each identified by its value position.
- `C_i` is the set of facts `f ∈ F` that satisfy all three:
  - `f ≠ a`;
  - `low ≤ q − p_f ≤ min(high, q − 2)`;
  - `f` is not the asked fact of any slot `i′ < i`.
- `K_i = |C_i|`. The generator places `o` in `C_i`, so `K_i ≥ 1`; the instrument asserts this.
- `U = mean(1 / K_i)` is the rate of a reader that knows this rule and guesses uniformly over `C_i`. **It equals the Bayes rate only if the posterior over `C_i` is uniform, which this unit does not prove.** It is a reference, not a proof.

A pick `s + 1` lands in one of these classes:
- **asked:** the fact `a`;
- **other:** the fact `o`;
- **candidate:** a fact in `C_i` other than `o`;
- **fact:** a fact in `F` outside `C_i ∪ {a}`;
- **row:** any other row.

## Readings

Per ext state (ext-s1, ext-s2) and far bucket (b1, b2), on the 256 held-out windows (512 queries a bucket):
1. **R1.** The histogram of `K`, the share of rows with `K = 1`, and `U`. Also `U` and the `K = 1` share on the first 2,048 TRAIN windows, the ones claude-395 trained on.
2. **R2.** C2′'s held-out `chose_other` on `K = 1` rows and on `K ≥ 2` rows.
3. **R3.** The classes of the picks of C2′, C2, A at the chosen shift, and C1 (the float pick).

## Outcome map, checked in this order

0. **Instrument.** The regenerated ask, other and bucket of every held-out row must equal claude-395's `held-*.q31.ask.u32`, `.other.u32` and `.bucket.u8` at both states, and `o ∈ C_i` must hold on every query. Any fault stops the unit, to be reported and repaired.
1. **The control asks more than the stream holds (instrument, §4.1).** At a (state, bucket), `U < C0 chose_other + 0.10` (the bar's own threshold; claude-395's C0 counts). Then bar 4b's `chose_other` half is above an unbiased reader's reach there.
2. **The rule follows the target as far as it is identifiable.** With at least 30 `K = 1` rows in the bucket, C2′'s `chose_other` on `K = 1` rows is at least 0.25 and at least twice its rate on `K ≥ 2` rows.
3. **The rule does not follow even identifiable targets (rule or gate limit).** With at least 30 `K = 1` rows, C2′'s `chose_other` on `K = 1` rows is at most 0.05 above A's on the same rows.
4. **The `chose_target` rise has an explanation.** Let `A` be the number of rows where C2′ picks the asked fact. Let `P` be the per-fact rate on the consistent set: the sum over rows of `[pick ∈ C_i] / K_i`, kept exact as a fraction.
   - If `A ≤ 2 P`, the rise fits "learned to pick facts in the right range, cannot tell which": a control fault.
   - If `A > 2 P`, it is drift toward the salient asked fact, which is the sheet's concern.
5. **Inconclusive.** Fewer than 30 `K = 1` rows, or none of 1 to 4 decides.

Outcomes 2 and 3 are exclusive. Outcomes 1 and 4 are read beside them.

**What each outcome unlocks.**
- 1 with 2 or 4 (a control fault): a decision card that replaces bar 4b's C2′ with a target the stream determines, for example the nearest earlier fact other than the asked one. Its GPU arm needs a sheet addendum and a non-author FIT (claude-398). The bar itself changes only through that card (§5.1).
- 3, or 4 read as drift: the update rule's next unit, a gate-family reachability probe.

## What is built

- A library function in `in_space_long.rs`: the candidate set `C_i` and the pick class for one window's queries, with tests on hand-built windows.
- A bin, `recall-other-ceiling`, that rebuilds the held-out (or TRAIN) windows from a seed and header. It checks itself against the run's target files, and writes R1 to R3 as JSON from integer counts, with no float. It runs on the laptop: no model and no device.

## Native-space five items (SKILL)

This is an analysis unit. It changes no state, operator or artifact: it reads saved picks.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs` (one new function and its tests)
- `crates/uor-r4-float-train/src/bin/recall-other-ceiling.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-397/**`, `docs/research/notebook/float-training.md`

## Done when

1. The function's tests pass on the laptop, including a hand-built window where `K = 1` and one where `K = 3`. One of them fails with the earlier-slot exclusion set aside. (The generator's `q − 2` clip never removes a fact, since every fact sits at `p ≥ 2`, so there is nothing for a test to set aside there. Corrected at build time, before any count.)
2. Outcome 0 passes at both ext states.
3. R1 to R3 are in the REPORT, mapped to outcomes in the order above.
4. `loop pr-check --local` and a fresh-context review at the exact head pass.

## Stop and ask if

- the regenerated targets differ from claude-395's files (outcome 0) and the cause is not in this unit's code.
