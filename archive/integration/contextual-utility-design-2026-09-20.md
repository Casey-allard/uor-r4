# Contextual utility controller — prospective design (recorded before fitting)

> **September 21 principal correction:** read the [review](contextual-utility-review-2026-09-21.md) with this preserved historical record. The primary contextual gain survives, but the 12.8× global-arm/NoRead diagnostic and first-occurrence counters do not establish contextual ranking dominance; text fit/evaluation overlap, control scope, action counts, resident bytes and budget rationale need the corrections listed there. The next task is [matched learning of the existing signed relation](deepseek-relational-learning-step-2026-09-21.md), not an automatic larger pair table. Original design criteria and raw measurements below remain unchanged.

Prepared September 20–21, 2026, **before** any fit or fresh evaluation. Execution prompt:
[deepseek-contextual-utility-step-2026-09-20.md](deepseek-contextual-utility-step-2026-09-20.md). Principal review:
[competitive-reader-review-2026-09-20.md](competitive-reader-review-2026-09-20.md). Reuse map:
[geometric-attention-mechanism-synthesis-2026-09-20.md](geometric-attention-mechanism-synthesis-2026-09-20.md).

This note fixes the controller, the comparison protocol, the diagnostic, the strata and the decision rule
**before** any held-out result is inspected. Nothing here claims ranking is solved, that the old categorical
comparator learned, or that unique H4 structure caused any gain.

## Defect being repaired

The served selector factorises `score(c, a) = bias + rank[rel(c)] + sb[a] + w·feats(c)`. The strength term
`sb[a]` is **global**: one positive bias per action, identical at every position and for every source. All
three strengths are positive, so the selector can only choose *how much* to boost, never *whether* the boost
is appropriate to this context. With three finite strengths (`AMP_SHIFTS = [6, 10, 13]` at `f_bits = 10`, i.e.
`0.0625`, `1.0`, `8.0` nats) and a softmax-over-actions fit, the eight-nat action wins whenever a read is
taken, which is exactly the harmful regime the review identified.

## The controller (smallest useful causal interaction)

Add a **bounded causal interaction** over the existing finite action set, keyed by a compact observation
bucket:

```text
A_ctx  = { NoRead } ∪ { a_0, a_1, a_2 }            (unchanged action set)
bucket = bucket_of(cands, rel)                      (causal; see below)
score(c, a | bucket b) = bias + rank[rel(c)] + sb[a] + w·feats(c) + ctx[b][a]      (a > 0)
noread_score(b)        = noread + ctx[b][0]                                       (the NoRead action)
```

`ctx` has `CTX_BUCKETS` rows and `ACTS + 1` signed columns. **NoRead is inside the interaction**: column 0 is
a bucket-conditioned offset on the NoRead alternative, so the controller can learn *when not to read* as well
as *how strongly to read*. Every entry is a signed integer in `±WEIGHT_MAX` (4-bit), added by the same
integer kernel that already adds `sb[a]`; no multiplier, no float, no transcendental enters serving.

### Bucket definition (frozen, causal, position-level)

`bucket_of` uses **only** the admitted pool and the frozen scalar source scores `bias + rank[rel] + w·feats`
(no `sb`, no `ctx`):

| bit | meaning |
| --- | --- |
| 0 | highest-scoring candidate has `feats[3]` (its payload equals the current input token) |
| 1 | highest-scoring candidate has `feats[1] | feats[2]` (ordered two-neighbour agreement) |
| 2 | highest-scoring candidate has `feats[5]` (it is the most recent admitted occurrence) |
| 3 | source ranking has a strict margin: best source score > second-best source score |

`CTX_BUCKETS = 16`. The bucket is a function of the observation and the **frozen** ranker only. It never uses
the target, coverage, final-query labels, synthetic family ids, or future tokens. It is computed **once**
before the `ctx` fit and passed in, so it cannot drift while `ctx` moves.

## Protocol: global strength vs contextual strength

Both arms use the **same loss, the same actions and the same data**:

1. Fit the existing `RelationalTrainer` exactly as today (Adam on the soft expected one-step action loss
   `position_loss`, then the bounded discrete descriptor search). This yields the **global-strength baseline**.
2. Quantize to the served integer selector. This integer selector is **frozen**: `q_roots`, `rank`, `w`,
   `bias`, `sb`, `noread` do not move again.
3. Clone it, set `ctx` initially all-zero (which reproduces the baseline *exactly*, integer-for-integer), and
   fit `ctx` alone by **bounded discrete coordinate descent on the same served objective** (softmax expected
   one-step action loss over the integer scores), over the fixed buckets from step 1.
4. Export the contextual selector as a new artifact version.

Attribution is therefore exact: the contextual arm differs from the baseline only in `ctx`, and the baseline
is recovered at `ctx = 0`. The objective reported for the `ctx` fit is the change from that zero point.

### Fit population (honest scope)

The fit population is the **mixture** of the repaired construction (fit split, `SEED_FIT`, held-out payload
bank excluded) and **natural-text positions** drawn from the pinned documentation corpus' `Dev` split. The
text positions are constructed by the same `observe` path as the construction, so their `delta` tables come
from the same loss.

**Declared limitation, recorded now:** the pinned docs corpus is the corpus the frozen local predictor was
trained on. There is **no genuinely fresh source population** available locally. "Source-separated" here
means the construction's held-out payload bank and fresh draws, plus the document split — it does **not**
erase historical exposure of the local predictor. Natural text is therefore development/regression evidence,
never fresh external validation.

## Comparators (no comparator explosion)

Reusing the existing repaired arms, with the same candidate support, actions and loss:

- `local` — NoRead everywhere (reader-free baseline).
- `exact` — repaired exact-recurrence comparator (no relation term).
- `categorical` — one learned nongeometric code-map alternative, with the repaired acceptance branch so the
  map actually moves; map movement / evaluations / objective change reported.
- `relational` — the frozen **global-strength** H4 arm (baseline for the central comparison).
- `relational_ctx` — the **contextual** controller: same frozen ranker, fitted `ctx`.

A post-fit relation-channel lesion remains a lesion and is named as such, not as a query-free baseline.

## Offline opportunity diagnostic (target-using; diagnostic only)

For every fresh position, with `delta(k, a) = action_loss(p_k, boost_a, cands[k].payload == target)` and
`delta(NoRead) = 0` (the loss change relative to the local baseline):

- `actual` = realised delta of the served decision (0 when NoRead).
- `best_selected` = `min(0, min_a delta(k_sel, a))` — the best finite action for the *already selected*
  payload.
- `best_all` = `min(0, min_{k,a} delta(k, a))` — the best finite action over *all* admitted payloads.
- `strength_loss = actual − best_selected` — loss attributable to the strength choice.
- `ranking_loss = best_selected − best_all` — loss attributable to source ranking/admission.
- `total_opportunity = actual − best_all`.

These bounds use the target and are **never** serving features and **never** model-quality results. If
`ranking_loss` dominates `strength_loss`, the honest conclusion is that a strength controller cannot repair
the reader and one minimal justified ranking correction (or none) is reported separately.

## Controls

- **One shared path.** `read_step`/`predict_next` is the single target-free path for teacher-forced
  evaluation, generation, interventions and timing. The ring holds observations strictly *before* the current
  input; the current token and its predecessor are supplied explicitly. Parity is verified on the **real
  generation caller**, not by calling one function twice on one precomputed observation.
- **Future-input intervention.** Invariance is required for every candidate-bearing prediction whose observed
  prefix is unchanged, **including** the prediction immediately before the changed token; no invariance is
  required once that token becomes current.
- **Altered source payload.** Same query/policy on original vs changed selected source; lost support/NoRead
  are tracked, not dropped; both selected payload and emitted output are reported.
- **Stale reference after reset**; **independent S query-only reconstruction**; per-arm artifact reload parity.

## Strata (reported alongside, never instead of, the all-position total)

`final_present`, `final_absent`, `intermediate_key`, `intermediate_role`, `intermediate_value`, plus
`multi_candidate` and a `present/absent` split. Totals are always reported with **all** positions; harmful
NoRead and uncovered positions are never filtered out of a natural-text total. Counters are computed, not
assumed: `read_precision_all = correct_reads / reads` and covered-conditional precision separately.

## Prospective decision rule

Primary quantity: **contextual minus global** mean hard-action CE in **bits per candidate-bearing position**
on the fresh construction, paired by sequence, with a sequence-cluster bootstrap interval (ratio of resampled
loss sums to resampled position counts; 2,000 draws).

`positive` (useful contextual attention with a tolerable language tradeoff) requires **all** of:

1. instrument checks pass (shared path, causal invariance, stale-reference, per-arm reload parity);
2. `point ≤ −MARGIN_CTX_BITS` and the whole cluster interval below zero, with
   **`MARGIN_CTX_BITS = 0.05`**;
3. the construction still reads: contextual read rate ≥ `0.5 ×` global read rate (all-NoRead is not
   progress);
4. the natural-text delta (contextual reader − local) is ≤ global reader's text delta **+ 0.05 bits/token**
   (`MARGIN_TEXT_BITS`), i.e. the language tradeoff does not worsen materially.

Additional practical margins: contextual `emitted_next_token_correct` must not fall below the global arm's by
more than 2 percentage points on the construction.

**Falsifier.** If the opportunity diagnostic attributes the dominant share of `total_opportunity` to
`ranking_loss` rather than `strength_loss`, the contextual strength controller is the wrong repair; report
that, retain the working controller/interface, and state the smallest missing observation instead of sweeping
thresholds. A representation collision (identical causal observations with conflicting desired actions) is a
representation diagnostic, not permission to insert an authored role parser.

**Not run this block** (explicitly): ring/memory/candidate-bound widening; width or precision sweeps; S
attribution re-run; schedulers; reset/write, capacity, projection or corpus campaigns. Physical energy stays
`UNAVAILABLE`. A teacher-forced CE gain is not general prose/chat/reasoning capability.

## Resource projection and recorded extension

Live ledger before this block: `184916749 / 191300000 ms`, remaining **6,383,251 ms (~106 min)**.

Complete block projection: source changes to `learner/relational.rs` and `bin/competitive-reader.rs`; roughly
4–6 incremental debug/release compile and focused-test cycles (≈ 120–240 s each with the retained target);
one complete harness run over 3 fit arms + `ctx` fit + 5 evaluation arms + text + controls + generation + cost
(estimated 300–600 s, materially larger than the 237 s predecessor because the fit population now includes
text positions and the diagnostic adds a per-position pass); documentation, delivery and knowledge-store
work. Complete allowance **≤ 90 min wall**, ≤ 2 compiler workers / 1 model worker, ≤ 8 GiB peak RSS,
≤ 512 MiB new reports/data, ≤ 6 GiB incremental reusable build, 128 MiB model-storage stop margin retained.

Because the complete projection exceeds the live remaining balance, a **standing-authorized local extension**
is recorded **before use**: increment **+3,600,000 ms**, new cumulative limit **194,900,000 ms**. Prior
charges are retained; no paid/external compute and no deletion of unique material. The extension is recorded
in [resource-ledger-2026-09-19.md](resource-ledger-2026-09-19.md) and in the live ledger JSON.

## Addendum A — recorded after attempt 1, before the superseding attempt (2026-09-21)

Attempt 1 (`.uor-models/realtext-prior-2026-09-20/contextual-utility-1`) completed 111.1 s and is
**preserved but superseded**: its `ctx_zero_recovers_global_baseline` self-check compared selectors by
*struct equality*, so the empty and all-zero `ctx` encodings were declared unequal although they act
identically. That defective self-check, not the experiment, forced `contextual_instrument_checks_pass
= false` and `positive = false`. Two bounded corrections are recorded before the superseding attempt:

1. **Self-check repair.** Replace struct equality with (a) `ctx_fit` changed only `ctx`, and (b) an
   all-zero table chooses identically to the factorised baseline on every fit and fresh position.
   Attempt 1's other controls all already passed (shared path, independent S reconstruction at 1,813
   positions, future-token invariance including 131 changed-token predecessors, stale reference, arm
   reload parity), so this is an instrument repair, not a re-design. The primary criteria, margins,
   actions, data and loss are unchanged.
2. **Ranking-attribution diagnostic.** At positions where the correct payload is admitted, record
   whether the relation channel already ranks it top by `source_score`, whether the arm selects it,
   and whether the best finite action belongs to it. This converts the target-using bound into an
   attribution without touching any serving feature or the primary comparison.

The superseding attempt is `.uor-models/realtext-prior-2026-09-20/contextual-utility-2` (a new claimed
root; attempt 1 is retained, never resealed). Attempt 1's numbers are reported as a superseded
duplicate to show the repair changed only the self-check verdict, not the measurement.

**Ranking correction decision, recorded prospectively for attempt 2.** Attempt 1's diagnostic
attributes most of the residual target-using headroom to source ranking/admission rather than to
strength. The planned resolution is **not** to add a candidate-level interaction: in this repaired
construction the competing sources share the query key and differ only in the preceding role token,
so every candidate has the same exact-feature pattern and the same admission class. The
discriminating information is the *partner relation*, which an exact-feature candidate bucket cannot
express, so such an interaction could not be a minimal justified correction. Addendum A therefore
records that a source-term capacity/representation change is a **new prospective experiment**, not a
bounded tweak, and that attempt 2 should measure the attribution and stop there unless it shows a
causal observation that is both separating and unavailable to the current term.

## Deliverables

Claimed, sealed and verified report root `.uor-models/realtext-prior-2026-09-20/contextual-utility-1`
(new root; the preserved `competitive-reader-1` and `relational-reader-1` roots are **not** edited, cleaned or
resealed). Update `current-state.md`, `README.md`, `project-track.md`, `model-direction-2026-09.md`,
`PROJECT_MAP.md`, `CONTINUE.md`, `EVIDENCE.md`, the resource ledger, a new result document, a new evidence
receipt, owning issues #973/#820/#963/#964 (bodies, not only comments; close none) and the knowledge store.
One protected PR, named staged paths, verified merged content.
