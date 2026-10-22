# ORDER: m1-a8-erosion-diagnosis, claude-40 (P_succ targets and TRAIN predecessors in the census)

Lab: claude. Item: [m1-a8-erosion-diagnosis](../../../../roadmap/items/m1-a8-erosion-diagnosis.md) (#2829). Base: `b00d9edad`. Item PR (partial: `References #2829`; the item stays `ready`).

## Goal

Unit 3 extends the census merged in #2994 with the design's Q2, Q4 (i, ii) and Q5. Per arm and checkpoint, to Q1, it counts:
- **Q2:** the P_succ target tokens and P_succ rows whose target's cell changed. For each target, it also gives the bigram-weighted root-metric distance between its cell and its TRAIN predecessors' cells. That relation is what A8 placement built on.
- **Q4 (i):** cell-changing writes on P_succ target tokens, per arm.
- **Q4 (ii):** cell-changing writes on those targets' TRAIN predecessors, per arm.
- **Q5:** gate writes per interval, as a timeline only.

**P_succ is the in-context copy rule.** In `is_copy_successor` (`attention_evidence.rs:7-20`), a row is P_succ when the token after the query's nearest earlier occurrence in the same window equals the target. That module is private, so the census copies the rule.

**The predecessors** are `BigramCounts::count` over eligible TRAIN positions (`address_cells.rs:43-77`).

## Cancel probe (run first)

At `b00d9edad`, does the census already count P_succ targets or predecessors? The base README defers Q2, Q4 (i, ii) and Q5 to this unit. If it already counts them, the task is void. Class: instrument.

## Files in scope

- `docs/labs/m1-a8-erosion-diagnosis/census/**`
- `docs/labs/m1-a8-erosion-diagnosis/README.md`
- This task folder

## Done when

1. **Inputs are checked first.** The four corpus files are read from `local/` and checked against the code pins:
   - TRAIN `3dee858f…` / `b5e7352d…`;
   - DEV `0919e90f953e988e…` / `a838bb1a67e96572…`.
2. **Control 6:** the copied P_succ rule, over the DEV panel with the A8 config's context and windowing, reproduces exactly **3,847 of 14,287** rows.
3. **Control 7:** the bigram counts reproduce the A8 checkpoint-0 placement facts the builder relied on. At minimum, the eligible TRAIN target count is 2,425,925.
4. **A planted fault is caught:** with the copy rule flipped to the oldest occurrence, a test fails.
5. **The README reports the results.** It tabulates Q2, Q4 (i, ii) and Q5 per arm and states what they do and do not support. It still says a placement census is not admission.
6. **The checks pass:** crate tests, clippy `-D warnings`, `report --check` and `pr-check --local`.

## Stop and ask if

- Control 6 or 7 fails after one careful check of the windowing. That is an instrument finding: record it and stop.
