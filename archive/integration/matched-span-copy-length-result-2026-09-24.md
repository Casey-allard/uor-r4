# Matched span curriculum and the length-specific copy/stop gap — September 24, 2026 UTC

This record completes the milestone named by
[language-transport-result-2026-09-24.md](language-transport-result-2026-09-24.md): implement the declared
**separate span RNG** so `max_span ∈ {3, 8}` arms are exactly data-matched and re-run that grid, and isolate the
**multi-token copy/stop gap**. It is an **exposed, component-scoped result** on bounded authored panels. The
terminal objective is unchanged: fully transformerless geometric language modelling with exact addressed evidence
and learned geometric operations replacing floating-point matrix multiplication at serving.

Base `48c2908f`. Worktree `…/uor-r4-worktrees/matched-span-curriculum-20260924`, branch
`codex/matched-span-curriculum-20260924`. Evidence root
`/Users/casey.allard/uor-r4-investigations/matched-span-20260924` (19 sealed attempts; ignored payloads, not in a Git
clone). Parent artifact `causal-candidate.tlx` `1ac70065…955f`. All arms start from that parent, `ARM=moments`,
seed `20260925`, `UOR_LANGUAGE_STEPS=512`, two threads, one cargo process at a time.

## What changed (runner and cursor format only)

- `learner/tl_checkpoint.rs`: `TrainingCursor` gains `#[serde(default = "span_rng_default")] span_rng_state: u64`
  (default `20260926`). The default must be nonzero: the module xorshift is in-place with a fixed point at zero.
  Legacy checkpoint cursors lack the key and still deserialize; new checkpoints round-trip.
- `bin/support/language_continuation.rs`: the span block now draws **only** from `cursor.span_rng_state` and
  consumes exactly `1 + 2·SPAN_POOL` draws per span example (`SPAN_POOL = 32`) regardless of arm or of the drawn
  `len`, so the two arms share an identical span pool. A new `UOR_LANGUAGE_SPAN_DRAW_MAX` (default `max_span`)
  separates the **declared cap** from the **draw range**. A new `UOR_LANGUAGE_COPY_SET ∈ {base,two,three,four}`
  selects the copy/stop-length instruction curriculum (matched 32-value additions: `A2 = 50..=76, 78..=82`
  2-token; `A3 = 140..=171` 3-token; `A4 = 1000..=1023, 1025..=1032` 4-token). A new read-only
  `UOR_LANGUAGE_MODE=diagnose` reports served integer margins and state saturation. The frozen held-out
  `EVAL_PANEL` (27 values, classes L2/L2_run/L3_near/L3_far/L3_run/L4/L4_run/A3_pos) is hashed into the receipt.
- `bin/support/causal_continuation.rs`: the two `TrainingCursor` literals only.
- **Unchanged:** `transferable_lexical.rs` (`59801a61…4956e`) and `tl_execution.rs` (`71e7888b…06ed`) are
  byte-identical to `origin/main`. No served-model arithmetic, D0-b kernel or compiled path change. The runner
  metadata now binds `runner_source`, `checkpoint_source`, `causal_runner_source`, `parent_checkpoint_source`,
  `eval_panel_sha256`, `span_pool`, `span_seed`, `copy_set` and `copy_ranges`.

## Corrections to the prior record (each independently re-checked)

1. **The parent fails all eight evaluation values**, including 2-token `37`/`42`; the "two BPE tokens reproduce"
   statement belongs to the *trained* arms, not the parent. The correct H2 baseline is a fresh matched control arm.
2. **The failure is Stop-after-copy, not copy.** Trained 3–4-token rows copy correctly and then emit `Generate`
   instead of `Stop` (`integrate.correct 16/64` = exactly the 2-token rows; selection is 32/32 everywhere).
3. **The requested "in-range 3-token numeric payload" cannot exist** for this tokenizer. The pinned SmolLM2 BPE
   splits digits individually (`Digits(individual_digits=true)`), so token length equals digit count: `0..31` are
   1–2 tokens, `32..35`/`37`/`42` are 2, `105/208/317/512` are 3, `1024/2048` are 4. Every digit token already
   occurs in `0..31`, so "out-of-range value" has no token-coverage channel. The milestone therefore reframes the
   factorisation as **token length × combination novelty**, and the matched control is a 2-token addition of equal
   pool size.

## Diagnostic — the gap is a missing terminal Stop (not state saturation)

Read-only teacher-forced diagnostic on the prior trained `moments-8` artifact (`549dfaaa…2e48`),
`diagnose-moments8-2`: at the first post-copy decision the Stop row is the top legal row for L=2
(`stop_margin +130 / +126`) and **below the vocabulary maximum for every L≥3 and L=4 row** (`−20` to `−336`). After
every copy transition, the number of `h` coordinates pinned at `±h_clamp` (`h_clamp = 256`) is **exactly 0**. So the
copy-count information is not erased by saturation; the missing object is supervision of the **Stop decision after a
completed multi-token copy run**.

## H1 — the matched span grid (COPY_SET=two)

| arm (`max_span`, `span_draw_max`) | model sha256 | span 1/2/3/5/8 | fresh span 13/21/34 | `repository_bits` | gates |
|---|---|---|---|---|---|
| parent `causal-candidate.tlx` | `1ac70065` | 16/0/0/0/0 = 16/80 | 0/48 | 6.343260 | — |
| `span3` (3, 3) | `aab0b63f` | 16/15/14/6/7 = **58/80** | 4/4/4 = 12/48 | 6.550110 | 32/4/16/3 |
| `span8m3` (8, 3) | **`aab0b63f`** | identical | identical | 6.550110 | identical |
| `span8` (8, 8) | `4c7f8bed` | 16/14/14/10/13 = **67/80** | 13/13/14 = **40/48** | **6.464870** | 32/4/16/3 |

- **Instrument witness:** `span8m3` and `span3` produced **byte-identical `final.tlx`**. The two runs differ only in
  `UOR_LANGUAGE_MAX_SPAN` (the declared cap, which enters metadata but not the batch); the effective span length is
  drawn by the new `UOR_LANGUAGE_SPAN_DRAW_MAX`, fixed at 3 in both. Identical model bytes, with identical
  `fit_targets`/`dev_targets`/`tune_targets` and parent/tokenizer identities, prove the non-span stream, the span
  pool and the realised span examples are all matched and that `max_span` does not leak into the batch. This is the
  matching witness the prior record's grid could not provide. (Scope: it witnesses inertness of the *declared cap*
  while the draw range is the binding constraint; the draw range is the treatment and does differ between `span3`
  and `span8`.)
- **Treatment:** with matched data, `span8` beats `span3` on the long cells (5: 10 vs 6; 8: 13 vs 7), on fresh-span
  extrapolation (40/48 vs 12/48) and on `repository_bits` (6.464870 vs 6.550110, i.e. better, not merely
  non-inferior); short lengths 1/3 match and length 2 loses one row. All four historical gates are retained.

## H2 — the length-specific copy/stop curriculum (matched 64-value pools, `max_span=3`)

Frozen `EVAL_PANEL`, prompt **variant 0** (the trained prompt), exact `Copy^L + Stop` rows:

| class | `span3` = base∪A2 (2-token) | `add3` = base∪A3 (3-token) | `add4` = base∪A4 (4-token) |
|---|---|---|---|
| L2 | 8/8 | 5/8 | 7/8 |
| L2_run | 4/4 | 3/4 | 4/4 |
| L3_near | 1/10 | **8/10** | 1/10 |
| L3_far | 0/8 | **4/8** | 0/8 |
| L3_run | 0/6 | **5/6** | 0/6 |
| L4 | 0/8 | 1/8 | **4/8** |
| L4_run | 0/4 | 2/4 | 2/4 |
| A3_pos | 1/6 | **6/6** | 1/6 |
| **total** | **14/54** | **34/54** | **19/54** |

- **Length-specific supervision.** `add3` fixes 3-token rows (L3_near 1→8/10, L3_far 0→4/8, L3_run 0→5/6, and the
  three trained positives 1→6/6) but does **not** transfer to 4-token rows (L4 0→1/8); `add4` fixes 4-token rows
  (L4 0→4/8) but not 3-token rows. The three arms have identical pool size (64 fit values, 128 instruction examples),
  identical span cap, identical steps and identical seeds, so the difference is the added values' token length. This
  refutes both "mere extra steps/volume" and a compositional length-generalisation account: the terminal Stop after
  a copy run of length `L` is learned only when length `L` is supervised.
- **Anchors:** `integrate` (unchanged value list) moves **16/64 → 28/64** (`span3` → `add3`); the per-value split is
  `37` 4/8, `42` 4/8, `105` 8/8, `317` 4/8, `512` 4/8, `1024` 4/8, `208` 0/8, `2048` 0/8. Selection remains 32/32.
- **Cost, and the pre-declared criterion is not met.** Against the matched control, `repository_bits` regresses
  **+0.2148** (`add3`) and **+0.2683** (`add4`) — beyond the pre-declared `+0.05` tolerance — and 2-token rows lose
  a little (L2 8/8→5/8, L2_run 4/4→3/4). All arms retain the four historical gates (temporal 32/32, class 4/4,
  feedback 16/16, held-out 3/3). So the extended curriculum is a **retained scoped component with a measured
  prose cost**, not a promoted successor artifact.
- Prompt variant 1 (the prior record's evaluation prompt, never trained) tracks variant 0 closely
  (`span3` 13/54, `add3` 33/54), so the finding is not an artifact of the prompt variant.

## Parity and cost on the changed artifact

The `integrate` runner asserts native/compiled equality on all 64 rollouts (a failing assertion aborts the run; this
is a runner assertion, not a counter recorded in that receipt). `cost` on `add3` re-measures and records **156**
identical blind/disabled grounded rollouts, median **1261.87 → 110.43 µs/token (11.43×)**, plan payload
**366,654 B**. The compiled path and the D0-b serving kernel are unchanged.

## Limits

- Bounded authored copy/stop-length and forced-feedback span panels on one 466,711-byte artifact; **not** general
  copying, open-vocabulary generation, dialogue, executed Rust, geometric superiority, or energy.
- The `A3_pos` values are in-fit positive controls by design; every other held-out class is disjoint from every fit
  set (asserted in the source test and recorded in each `preflight.json`).
- The span panel's lengths 5 and 8 are distinguished only through the fingerprint and rotations (the typed block
  clamps at 4), so the H1 long-cell effect is a fingerprint-level effect.
- No fresh sealed acceptance draw was taken; the value classes were frozen before the runs but this remains an
  exposed development result.
- The `+0.05` `repository_bits` tolerance was fixed in the pre-execution run design (the experiment-designer's
  acceptance criteria, adopted before the grid executed); it is **not** recorded in a sealed artifact predating the
  runs, and the raw regressions (`+0.2148` / `+0.2683`) are reported regardless of the criterion.
- Single training seed (`20260925`); no second seed was run, so the arm-to-arm deltas carry no seed-variance
  estimate. Arms were selected on the declared validation set and the four historical gates only; every reported
  held-out number is from the final 512-step artifact whose `final_sha256` is bound in its receipt.
- Only the three code files above are part of the code change; this record's own documentation edits are delivery
  metadata, not part of the tested code diff.
- An independent evidence audit of the sealed roots on 2026-09-24 recorded:
  the matching witness (C1), the H1 comparison (C2), the mechanism and length-specificity (C3), the disjointness
  assertion (C5) and the receipt binding (C7) are confirmed; the `cost-add3` counters are confirmed exactly; and
  the two caveats above (unsealed `+0.05` criterion; `integrate` parity is a runner assertion rather than a receipt
  counter) were raised. No numeric claim here is asserted on the auditor's authority.

## One recommended next milestone

Reduce the prose cost of the length-specific Stop fix — one bounded arm on the same grounded learner that keeps the
`add3` copy/stop endpoint while restoring `repository_bits` within the `+0.05` tolerance (for example rebalancing
instruction/span weight or adding a post-copy Stop calibration), evidence-scoped and matched to `add3`. Then resume
the forward line: broader-source dialogue/code and complete-session work.
