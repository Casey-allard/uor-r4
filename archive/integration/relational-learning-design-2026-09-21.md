# Matched relational learning — prospective design (recorded before implementation and scoring)

> **Principal correction, September 21:** read the [review](relational-learning-review-2026-09-21.md) and [independent evidence](../evidence/relational-learning-principal-review-2026-09-21.json). The strong learned-source improvement survives. The construction is a replay; 646/1813 does not establish dominant admission loss, and the full stream has 608 omitted empty-pool positions. Thirty manifest files are verified. Refitting helped but is not isolated as the sole cause; earlier utility gains retain their scope. Control, artifact, continuation and text-interval limits are corrected in the review. The [active successor](deepseek-reader-utility-transfer-step-2026-09-21.md) is useful language read influence, with admission conditional on measured opportunity. Original design/outcome below is preserved.

Prepared September 21, 2026, before any code change, fit or held-out result. Execution prompt:
[deepseek-relational-learning-step-2026-09-21.md](deepseek-relational-learning-step-2026-09-21.md). Principal review:
[contextual-utility-review-2026-09-21.md](contextual-utility-review-2026-09-21.md). Reuse map:
[geometric-attention-mechanism-synthesis-2026-09-20.md](geometric-attention-mechanism-synthesis-2026-09-20.md).
Base revision `472767dc`. Parent result `29f680fb` / `contextual-utility-2`.

## Retained from the parent (not re-litigated)

The contextual utility controller is retained. Its implemented decision for the strength question stays
`positive=true` at the parent scope: contextual minus frozen global strength **−0.257644 bits/candidate
position [−0.327088, −0.182968]**. The parent's ranking-dominance falsifier and its softened
interpretation remain visible; the retired 12.8× diagnostic measured the **global** arm and conflated
NoRead with ranking, so it is not used as a capacity diagnosis here.

## The one constructive change

**Bounded alternating fit**, with phase selection on the actual exported decision quality:

1. scalar fit (`STEPS` Adam steps on the soft expected-action objective),
2. discrete descriptor refinement (`ROUNDS` coordinate sweeps),
3. **scalar refit** (another `STEPS` Adam steps — the step the parent schedule lacked),
4. discrete descriptor refinement again.

Each phase is snapshotted by `quantize()`, and the arm is selected by the **exported hard decision loss**
on a declared development construction (`SEED_TUNE`, the fit payload bank, disjoint from fit and fresh
draws), not by the soft surrogate. The soft objective and the exported objective are recorded separately
so optimization, quantization and integration failures stay distinguishable. Both learned arms
(`Relational`) and (`Categorical`) receive the identical schedule, dose and selection rule. Two sweeps
remain a coordinate result, not a proven global optimum.

## Matched arms (bounded, no Cartesian product)

| Arm | Role |
| --- | --- |
| `local` | NoRead everywhere; reader-free reference |
| `exact` | repaired exact-recurrence comparator |
| `categorical` | learned nongeometric code map, global strength |
| `categorical_ctx` | **same code map + the same contextual utility procedure** |
| `relational` | signed H4 relative element, global strength |
| `relational_ctx` | **signed H4 + the same contextual utility procedure** |
| `relational_ctx_lesion` | relation-channel lesion of the contextual arm (reliance check, **not** query blindness) |

The contextual controller is **re-fitted to each source arm by the same procedure** (`CTX_ROUNDS`
bounded sweeps from zero over that arm's own frozen buckets). The review's point 5 is explicit: freezing
`ctx` bytes while the scorer changes does not isolate source selection, so a shared frozen table is not
used. Every arm is then exercised through **independently reloaded artifacts**, not the fitted structs.

## Expressivity witness (test-only; no artifact parameter comes from it)

The fixed fixture has 14 symmetric partner pairs. The signed 120-element relation can express them:
pick fourteen distinct antipodal classes `{g_i, −g_i}` excluding `{+1,−1}` and set `Q(a_i)=g_i`,
`Q(b_i)=−g_i`, every other token identity. Then both partner directions give relative element `−1`
(the central involution), because `−g` is central and `g^-1(−g) = −1`. A witness selector with
`rank[−1]=+7`, other ranks `−7`, exact-key weight `+7`, all other weights/bias zero, all strength biases
zero, `ctx=0` and NoRead threshold `7` scores a same-key partner `14`, a wrong-key partner at most `7`
and a same-key non-partner `0`; with strict `>` over NoRead it abstains on the latter two. The focused
test checks group identity/sign, the exact-key feature, the strict NoRead tie rule and separation of the
authored present/absent final cases against the **actual tables and fixture**. It is a source-selection
and NoRead instrument only: tied strength biases choose the weakest boost, so it says nothing about
emitted answers. **No gold partner assignment ever initialises the experimental learner.**

## Attribution repairs integrated into the run

For every exported arm, per position, save: the **ungated top source** (argmax of the ctx-free source
score, i.e. the source ordering *before* NoRead/strength), its exact `(seq, abs)` reference, the served
source reference, payload, bucket, selected action and emitted token.

**Any correct payload counts as a correct prediction** (duplicate-aware: compare payload tokens, never a
first-matching occurrence index). Exact-occurrence provenance is reported separately from payload
correctness.

**Exact decomposition** in nats relative to local, using the served ungated top source `c*`:
`L+(c)=min_a delta(c,a)`, `L*(c)=min(0,L+)`, `Lpool=min_c L*(c)`, ranking regret `= L*(c*)−Lpool`,
gate regret `= (L+(c*) if read else 0) − L*(c*)`, dose regret `= actual − L+(c*)` for a read else `0`.
These are nonnegative and sum to `actual − Lpool`. A test asserts the identity and its edge cases (tie,
NoRead, no-candidate). Because the served source is the argmax of the ctx-free source score while the
action is a common per-bucket offset, the ordering is factorized and the identity applies; the run also
verifies served source `== c*` on every scored position rather than assuming it. `Lpool` cannot see a
source excluded from admission — reported as such, never as an admission oracle.

## Honest populations

- **Construction**: repaired fixture; fit (`SEED_FIT`) / tune (`SEED_TUNE`) / final fresh (`SEED_FRESH`)
  with disjoint seeds and the held-out payload bank for the final draw. All previously inspected
  construction outcomes are development/regression only.
- **Reader text**: **document-separated**. The Dev pool is sorted by content hash; the first `TEXT_WINDOWS`
  documents form the reader's text-fit set and the **next** `TEXT_WINDOWS` disjoint documents form the
  held-out reader text evaluation. Path and `sha256` disjointness is asserted before fitting and recorded.
  One 64-token window per document, declared fit-token and candidate-position weights. The frozen local
  prior's older corpus exposure is disclosed separately and is not claimed to be removed.
- No genuinely fresh external corpus is downloaded; this is a clearly scoped** reader-held-out local
  evaluation**, not an externally unseen claim.

## Causal controls (repaired)

- One shared target-free `read_step`/`predict_next` path; ring strictly before the current input.
- The **reloaded contextual arms** are what evaluation, generation, interventions and timing call.
- **Selected-source intervention**: change only the payload of the occurrence the *exported policy
  actually selected*, and only when that occurrence lies **outside the fixed recent query context**
  (`abs + 3 < query_abs`). The query tokens and the admission conditions are preserved. Report every
  attempted pair, including lost support and NoRead, plus an **unrelated-source control** (mutate a
  different admitted occurrence) and read-disabled behaviour. The first target-bearing occurrence is
  never silently substituted.
- Future-token invariance including the predecessor of the changed token; stale reference after reset;
  zero-controller parity; learned-code export; actual-used fitter phase-boundary continuation.

## Cost

Measure an **active-candidate** probe with representative candidate counts and observed read/NoRead
actions (a fresh construction sequence, where admission is non-empty), together with a genuine
no-reader-work local baseline on the same token stream and on a real text window. Count predictions,
records scanned, candidates admitted, reads and table traffic. Report **resident** entries/metadata/
capacity separately from **serialized** bytes (16 `[i32;4]` rows are 256 resident entry bytes plus Vec
metadata; the serialized payload is 64 bytes). Physical energy stays `UNAVAILABLE`.

## Artifact binding

The run writes a `binding.json` alongside `result.json` containing: full input digests (construction fit
+ text-fit positions with candidate payloads and targets as *training labels*, the held-out payload bank,
the document split identities), tokenizer source/derived digests, parent E and S digests, the exact group
table identity, all seeds, the optimization/contextual-phase configuration, export semantics and the
format versions. A source hash alone is not the binding.

## Prospective decision rule (frozen now, before final scoring)

Primary endpoints, all on the **final fresh** construction draw unless stated:

1. **Present final queries (n≈119)** — actual hard loss in **bits per present final query** and the count
   of **correct emitted next tokens**. Primary comparison: `relational_ctx` versus `relational`, paired by
   sequence, cluster interval.
2. **Absent final queries (n≈21)** — harm in bits per absent query; reads count. An increase is reported
   as harm, not suppressed.
3. **Document-held-out reader text** — reader minus local in **bits/token** on the disjoint held-out
   documents. Old inspected documents may appear only as a separate regression row.
4. **Matched categorical behaviour** — `categorical_ctx` versus `relational_ctx` on the same endpoints.
   Unique geometric benefit requires this comparison, not the lesion alone.
5. **All-position total** — reported with every stratum, never instead of them.
6. **Decomposition** — ranking / gate / dose regret in nats per position and per stratum.

Intervals: sequence-cluster (construction) and document-cluster (text) paired bootstrap, ratio of
resampled loss sums to resampled position counts, 2,000 draws.

**Interpretation rule.** A component-learning result is retained but not promoted as chat. A learned
source-selection improvement with unresolved prose is a retained near miss. Unique geometric benefit is
claimed only if `relational_ctx` resolves better than `categorical_ctx` under the matched comparison. An
all-NoRead collapse cannot count as attention progress. If the compact learner cannot learn the witness
fixture, the exact optimization/quantization/observation failure is delivered with one justified next
action — no automatic escalation to S7 or a giant pair table.

## Resources

Live ledger at recording: **186736749 / 194900000 ms**, **8163251 ms (~136 min)** remaining. Free space
**43.06 GB** against a 36.77 GB reserve; 128 MiB model-storage stop margin retained.

Projection for this complete block: source changes to `learner/relational.rs` and
`bin/competitive-reader.rs`; ~4–6 incremental compile/focused-test cycles (≈60–180 s each on the retained
target); one complete harness run with 7 arms, alternating fit, both contextual fits, document-separated
text, the decomposition, the repaired intervention and the active-candidate cost probe (estimated
**240–420 s**, larger than the parent's 113.8 s because the fit is doubled, two more arms are evaluated and
the intervention/cost blocks are richer); then documentation, delivery and knowledge-store work. Complete
allowance **≤ 60 min wall**, ≤ 2 compiler workers / 1 model worker, ≤ 8 GiB peak RSS, ≤ 512 MiB new
reports/data, ≤ 6 GiB incremental reusable build.

This projection fits the live balance, so **no new ledger extension is taken**; the parent review corrected
the parent extension's arithmetic rationale and all prior charges are kept. If a measured phase overruns,
stop at a completed phase boundary, checkpoint and record the actual debit rather than raising the limit.

## Addendum B — outcome and attempts ledger (recorded after execution)

The design above was frozen first. Three superseded attempts are preserved and never resealed; the
measurement was **bit-identical** in all four runs, so the defects are reporting/control defects only:

| Attempt | Root | Defect |
| --- | --- | --- |
| 1 | `relational-learning-1` | the one-shared-path control compared the contextual arm's `predict_next` against the **global** arm's `choose` (a control-side call I retargeted incompletely), so `instrument_ok` was false |
| 2 | `relational-learning-2` | the cost `timings` vector was omitted from `result.json` |
| 3 | `relational-learning-3` | the no-reader-work local baseline was on a text window rather than the matched construction stream |
| 4 | `relational-learning-4` | **delivered**: instrument checks pass, cost complete and matched, 0 unlisted files |

**Outcome.** The alternation is decisive and the two preregistered comparisons both fail at their margins:

- `relational_ctx - relational` = −0.012093 [−0.023107, +0.003536], spanning zero, with **zero** differing
decisions across the 119 present final queries. `retained_component_positive = false` under the frozen
0.05 margin. The parent's −0.257644 strength gain was largely the stale-scalar artefact.
- `categorical_ctx - relational_ctx` = −0.124149 [−0.227536, −0.022590]: the matched nongeometric arm is
significantly **better**, so `unique_geometric_benefit = false`.
- But `relational - exact` = −0.524968 [−0.654447, −0.396747], where the pre-refit parent measured
−0.010151 [−0.104470, +0.083675] on the same construction. The earlier "the relation is not better than
exact recurrence" reading was an **optimisation artefact**, not a representation limit; the retired
capacity diagnosis is not revived and no capacity increase is licensed.

The falsifier in the parent review (ranking dominance) is not reinstated either: the exact decomposition puts
ranking regret at 0.600 nats/position against gate 0.209 and dose 0.121, while only 646 of 1,813 positions
admit the correct payload, so **coverage is the largest measured loss**.

New claimed, sealed, verified report root
`.uor-models/realtext-prior-2026-09-20/relational-learning-1`. Parent and older roots are preserved
untouched, including their known historical unlisted helpers. Update current state, canonical roadmap,
README/front doors where claims change, the result/receipt/resource records and live owning issues
(none closed). Compact revision-pinned knowledge import with verified retrieval.
