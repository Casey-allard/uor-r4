# ORDER — written-state-count-model (claude-422)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-422, P0.5 (R\*), part 5 of 5. Parts:
- claude-418: the counts (merged);
- claude-419: the host scorer (merged);
- claude-420: the streams (merged);
- claude-421: the filter, the GPU scorer and the bits instrument (merged, 9465b92d6);
- claude-422 (this unit): the sheet, FIT, N tuned on DEV-tune, and the sealed DEV-test run.

## What R\* is

The reference of bar Q (claude-409 DECISION §2): BPB(model) ≤ 0.80 × BPB(R\*) on sealed D87
DEV-test bytes. R\* is interpolated modified Kneser–Ney with three discounts per order and full
N1/N2/N3+ counts, unpruned, orders 1..N, with N chosen on DEV-tune. It is built from exact counts of
TRAIN. claude-421 added one rule, which the sheet names as part of R\*: an order whose
count-of-counts holds a zero, or whose Chen–Goodman estimate leaves [0, i], takes KenLM's
`--discount_fallback` discounts (1/2, 1, 3/2) for the whole order.

**Naming.** The instrument's `N` is the longest context, in tokens. An `N` of k scores (k+1)-grams.
The decision card's "8–12 expected" counts n-gram length, so it is `N` = 7..11 here. The prime-route
key holds at most 10 context tokens (`NGRAM_MAX_ORDER`), so this unit can reach `N` = 10 (11-grams).

## What it does

1. **A prep sheet and its FIT** (`docs/harness/prep/r-star-sealed-2026-11-17.md`), before any run
   on real data. A fresh-context non-author reviewer gives the FIT.
2. **The oracle at the chosen N.** claude-421's ignored real-slice test compares the GPU path with
   the independent `KnOracle` at N = 2 on 1%. It gains two settings, `UOR_KN_N` and
   `UOR_KN_PER_MILLE`; their defaults keep claude-421's check. No other code changes was the plan.
   *Amended at the REPORT:* three memory-only repairs (`53cb85ee2`, `a15bb17b1`, `347f0122e`, each
   `cmp`-identical on DEV-tune) and the one-unit floor (`c63510aa2`, governed by the second sheet,
   `r-star-sealed-2-2026-11-17.md`) were added after the sweep and the first sealed crash.
3. **The sweep on DEV-tune** (wick). `kn-reference` at N = 1..=10, each run with `--check-host`,
   against all of TRAIN and all of DEV-tune, with DEV-tune's novel mask.
4. **The choice.** N\* is the N with the fewest Q12 bits over all DEV-tune targets; a tie goes to the
   smaller N.
5. **The oracle at N\*** on a slice of TRAIN and DEV-tune, and a second DEV-tune run at N\* that must
   be `cmp`-identical to the first.
6. **The sealed run.** One run of `kn-reference --sealed-run` at N\* on DEV-test, with `--check-host`
   and DEV-test's novel mask. Its JSON is R\*'s sealed record.

## Gates

- G1, laptop: stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` is clean;
  `kn_` host tests pass.
- G2, the sheet has TEST FITNESS: FIT from a non-author before step 3.
- G3, wick: every sweep run exits 0 with host = GPU at every target, and its target and byte totals
  equal claude-420's DEV-tune counts (all 10,807,417 targets / 21,928,475 bytes; novel 7,566,174 /
  15,995,822).
- G4, wick: the oracle at N\* passes. A negative is run as well: the oracle's range test in
  `oracle_discounts` is set aside and the check must fail, or, if no order of the slice reaches that
  branch, any other sabotage of the oracle's mass rule must fail it. Restored after.
- G5, wick: the second DEV-tune run at N\* is `cmp`-identical to the first.
- G6, wick: the sealed run exits 0 with host = GPU at every target, `sealed_run` true, and DEV-test's
  totals (all 21,349,576 targets / 43,326,982 bytes; novel 14,795,227 / 31,328,319).

## The seal

Nothing scores DEV-test before G2 to G5 pass. The sealed number is the first JSON the sealed run
writes. A later run, for any reason, is reported beside it and never replaces it. The one exception
is a first JSON that fails G6: a repaired run under a new sheet may replace it only through a
decision card that amends this paragraph, and the first JSON stays on record.

## Out of scope

The PPMd anchor (DECISION §2: reported, does not gate) is its own unit. No model is scored here.
