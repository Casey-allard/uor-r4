# Reader utility transfer — prospective design (frozen before implementation and scoring)

> **Principal correction after PR #1328:** [review](reader-utility-review-2026-09-21.md) and [saved-data audit](../evidence/reader-utility-principal-review-2026-09-21.json). The small text CE gain verifies, but fit events use zero gap thresholds and serving uses negative thresholds. The intended gap-conditioned learner was not tested; the representation-limit diagnosis is withdrawn. Actual-action regret, emitted/absence decision criteria, admission denominators, source/binding and resolved-cost claims require the review's qualifications. Only four reader parents were bound, not five. Original prospective design/result below is preserved; the [corrective constructive task](deepseek-reader-policy-contract-step-2026-09-21.md) now owns execution.

September 21, 2026, recorded before any code change, fit or held-out result. Execution prompt:
[deepseek-reader-utility-transfer-step-2026-09-21.md](deepseek-reader-utility-transfer-step-2026-09-21.md).
Principal review: [relational-learning-review-2026-09-21.md](relational-learning-review-2026-09-21.md). Reuse map:
[geometric-attention-mechanism-synthesis-2026-09-20.md](geometric-attention-mechanism-synthesis-2026-09-20.md).
Base revision `31972e34`. Frozen parents: `relational-learning-4/artifacts/{relational,categorical,relational_ctx,categorical_ctx,exact}.rlr2`.

## Retained, and not re-litigated

The improved relation learners are **kept and frozen**. H4 beats exact recurrence by **−0.524968
bits/candidate-bearing position [−0.654447,−0.396747]**; the matched categorical+contextual arm is better
than H4+contextual by **−0.124149 [−0.227536,−0.022590]**; the marginal contextual controller is
**−0.012093 [−0.023107,+0.003536]** with identical decisions at all 119 present final queries, and the
parent's `retained_component_positive=false` / `unique_geometric_benefit=false` stand. PR #1323's original
narrow controller positive is preserved as its own scoped measurement. H4+contextual still harms the eight
reader-held-out documents by **+0.274243 bits/token [0.174735,0.406041]**, and those documents are now
inspected development data.

Also accepted from the review: the old "fresh" population is a **byte-identical replay** (regression only);
`−1` is the central involution and `−g` need not be central (the implemented test checks the right
property); the parent's read-disabled comparison was mis-matched; the embedded artifact digests are
abbreviated and `ExactGroupTable::build()` is not a group hash.

## The one constructive hypothesis

**Learn whether, and how strongly, an already selected source should influence the local predictor, with a
direct hard finite-action objective and sufficient causal observations.**

Frozen: E + S-query-only local emission, the 128-token ring, the current admission rule, ≤24 candidates, and
the improved source maps/rankers. Only *influence* is learned. Training starts from the **ungated top
source** so NoRead cannot hide it. Serving never uses a target.

### Actions

`{NoRead} ∪ {0.0625, 1, 8}` nats (`AMP_SHIFTS = [6,10,13]`, `f_bits = 10`). NoRead cost is exactly zero.

### Causal observations and buckets (32 = 4 × 4 × 2)

| Field | Values |
| --- | --- |
| `gap_bin` | selected payload's local logit gap `z_local[payload] − max z_local`, in `f_bits` integer units, quantized by **three integer thresholds stored in the artifact** |
| `ctx_class` | bit0 = the top candidate's payload equals the current input token (`feats[3]`); bit1 = its ordered two-neighbour agreement (`feats[1]|feats[2]`) |
| `margin_bit` | 1 iff the ungated top source score is strictly greater than the runner-up's (0 when the pool has one candidate) |

`bucket = gap_bin*8 + ctx_class*2 + margin_bit`. Every observation is available **before** the action, uses
no target, no coverage, no correctness, no family/role/domain label, no query-type metadata and no future
token. The integer gap is an **observation, not a probability estimate**.

### Fitting (direct empirical finite policy)

For each fit position, with `top = ungated_top_source`, `payload = cands[top].payload`, and `p =
P_local(payload)` computed offline only:

`Δ(a) = log(1 + p·(exp(a)−1)) − a·I[payload == target]`, and `Δ(NoRead) = 0`.

Per bucket, average the **individual** weighted costs (never the nonlinear formula at the mean `p`) and choose
`argmin_a mean_w Δ(a)` with:

- **shrinkage / minimum support:** a bucket needs `MIN_SUPPORT = 20.0` total declared weight; below that it
  takes the **global** action (the argmin over the whole declared fit population), which is also the explicit
  unseen-bucket fallback;
- **safe-abstention rule:** a read is chosen only if `mean(best read) < mean(NoRead) − 1e-12`; ties and
  near-ties abstain. This is declared now, not tuned after results.

### Fit mixture (declared weights)

One mixture, identical for both source arms: the parent's **construction fit split** (`SEED_FIT`,
`values_fit`) plus **natural-text candidate-bearing positions** from the reader **fit** documents, with the
text stratum scaled so that construction and text carry **equal total weight**
(`w_text = n_construction / n_text`). Weighting uses fit strata only; serving never sees them.

## Populations (declared before scoring)

- **Reader text, document-disjoint by path and content hash**, Dev pool sorted by `sha256`: documents
  `[0,8)` = **fit**, `[8,16)` = **tune**, `[16,24)` = **final**. Up to `WINDOWS_PER_DOC = 4` 64-token
  windows per document, chosen by a declared in-run preparation probe that reports encode/observe/score
  rates and stops at `min(4, ...)` windows so the fit text position count clears a declared **minimum
  information target of 600 candidate-bearing positions**; if it cannot, the shortfall is reported rather
  than hidden. The frozen local prior's exposure to this corpus stays a separate disclosure.
- **Construction**: `SEED_FIT` (fit), `SEED_TUNE` (threshold/margin selection), the parent's `SEED_FRESH`
  replay (**regression**), and a **new frozen draw `SEED_FINAL = 0x5C0F_F1A1`** over the held-out payload
  bank for final assessment. All seeds are frozen here.
- **Full prediction stream**: every position `i ≥ 2` with `i+1 < len`, **including empty-pool positions**,
  which take the local prediction. Every pool/policy comparison uses the same stream and the same
  denominators; candidate-bearing strata are reported separately, never instead.

## Diagnostics and the conditional admission question

On the same full stream, target-using and never a serving feature: full causal-ring eligibility, admitted
support, `Lring` (best finite action over **all** ring occurrences), `Lpool` (over the admitted pool), and
the nonnegative **admission regret `Lpool − Lring ≥ 0`**. **Trigger, declared now:** if the mean admission
regret over candidate-bearing positions reaches **0.25 nats**, admission becomes a named candidate for the
successor; the utility task is completed first regardless, because influence and admission are different
quantities. No admission change is made in this run.

## Evaluation and the repaired attribution

- All-position hard CE with the same denominators everywhere; separately: candidate-bearing vs empty-pool,
  construction strata (final_present, final_absent, query_key, intermediate role/key/value), real-text
  copy-available vs copy-unavailable, and candidate-count strata.
- Per-document loss sums and counts are **serialized** so document intervals can be independently
  reproduced; labels say documents and **bits/token**.
- The repaired identity `ranking + gate + dose = actual − Lpool` is verified for both new arms, with
  per-position verification that the served source equals the ungated top source.
- Duplicate correct payloads all count as correct; exact intended-source identity is a separate provenance
  metric.
- Read/NoRead/strength distributions, selected payload and emitted token are reported per arm.

## Prospective success categories (thresholds frozen now)

Let `Δ_text` be the new policy's held-out-document harm versus local, document-cluster paired interval;
`Δ_text_parent = +0.274243` bits/token; `MARGIN = 0.05` bits/token; present-query and absent-query effects
carry the same 0.05 margin in bits/query.

| Category | Condition |
| --- | --- |
| **harm containment** | `Δ_text ≤ +0.05` bits/token |
| **useful transfer** | `Δ_text < 0` with the document interval entirely below zero |
| **relational behaviour preserved** | final-present-query hard CE not worse than the frozen parent by more than 0.05 bits/query **and** correct emitted next tokens not below the parent by more than 2 |
| **absence improved** | absent-query reads and absent-query loss both ≤ the frozen parent's |
| **matched categorical visible** | the identical procedure on the categorical source arm is reported on the same endpoints; a geometry-specific claim needs it, and a useful geometric component may be retained without winning it |

An all-NoRead policy is a **reference, not progress**: it is reported as such and cannot satisfy "useful
transfer". If no meaningful language read survives, that limitation is reported instead of claiming success
because text returned to baseline.

## Causal controls and artifact boundary (repaired here)

- Evaluation, generation and timing all call the **independently reloaded** arms through the one target-free
  `read_step`/`predict_next` path, with the ring strictly before the current input.
- **Selected-source intervention compares four matched conditions**: enabled-original, enabled-changed,
  disabled-original, disabled-changed, on the same prefix. **Disabled-before and disabled-after must be
  invariant** (their local inputs are unchanged); enabled/disabled on the same prefix measures the reader's
  contribution. The replacement payload is absent from the whole sequence, the query is fixed, and changes
  to neighbouring records/admission are counted. **Source-reference identity** is recorded alongside the
  payload response, and support loss/NoRead are counted rather than dropped.
- **Group identity**: a real digest over the exact group table bytes (product, inverse, identity), replacing
  the builder-name string.
- **Manifest**: a versioned `manifest.json` carrying parent artifact digests, fit/tune/final document and
  window identities, the full observation digest, the group digest, feature/bin definitions, action
  semantics, parameters and configuration, plus a `manifest_digest` that the runner **recomputes and
  verifies** on load.
- **Schema check before the expensive pass**: the expected result field set is validated at the start of the
  run, not after the scoring.
- Focused tests: direct hard-action fitting (including an unseen/empty bucket and the NoRead tie rule),
  integer thresholds and overflow safety, artifact v4 round-trip and rejection, changed observation indexing
  through `read_step`, and the existing causal/group tests are retained.

## Cost

Paired, **interleaved** same-stream local/reader timings with enough active candidates to resolve a
difference, repeated and reported as medians; bounded selector-only accounting; resident vs serialized bytes
included. Physical energy `UNAVAILABLE`. No token-pair cache in this run; it remains optional.

## Resources

Live ledger at recording **189266749 / 194900000 ms**, **5633251 ms (~94 min)** remaining. Free space
**39.55 GiB** against the 36,766,079,385-byte reserve; 128 MiB model-storage stop margin retained.

Projection: `learner/relational.rs` (policy field, bucket, direct fitter, artifact v4, tests) and
`bin/competitive-reader.rs` (utility-transfer mode, full-stream observation, diagnostics, four-condition
intervention, manifest, costs); ~4–6 incremental compile/test cycles (≈60–180 s each on the retained
target); one complete harness run over the frozen parents with the declared mixture, the full stream on four
construction populations and three document sets (estimated **240–420 s**); documentation, delivery and
knowledge work. Complete allowance **≤ 60 min wall**, ≤ 2 compiler workers / 1 model worker, ≤ 8 GiB peak
RSS, ≤ 512 MiB new reports, ≤ 6 GiB incremental reusable build. **No new ledger extension is taken**; if a
measured phase overruns, stop at a completed phase boundary, checkpoint and record the actual debit.

## Deliverables

New claimed, sealed, verified report root `.uor-models/realtext-prior-2026-09-20/reader-utility-1`. All
parents and superseded attempts are preserved untouched, including the 30 manifest-listed files of
`relational-learning-4`. Update current state, canonical roadmap, README/front doors where claims change,
result/receipt/resource records and owning issues (none closed); compact revision-pinned knowledge import
with verified retrieval.
