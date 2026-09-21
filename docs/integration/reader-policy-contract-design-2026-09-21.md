# Reader policy contract — prospective design (frozen before the corrected fit and its final evaluation)

> **Principal correction after PR #1330:** [review](policy-objective-review-2026-09-21.md) and [independent audit](../evidence/policy-objective-principal-review-2026-09-21.json). The fit/serve repair and all four negative criteria verify. The old artifact's text gain remains valid; the changed successor does not refute it. Attempt 4 repeats attempt 3's final data/outputs. Objective-only and sufficient-support diagnoses are unestablished; alternative-action emission/stratum statistics needed for joint feasibility were not saved. Loader rejection and neighbor-intervention claims require correction; different-seed admission is not rounding, and construction's paired timing median is positive. The original dated design/report below is preserved as history, not active authority. The [new constructive prompt](deepseek-policy-feasibility-step-2026-09-21.md) owns execution.

September 21, 2026, recorded before the corrected fit and before any held-out result. Execution prompt:
[deepseek-reader-policy-contract-step-2026-09-21.md](deepseek-reader-policy-contract-step-2026-09-21.md).
Principal review: [reader-utility-review-2026-09-21.md](reader-utility-review-2026-09-21.md). Reuse map:
[geometric-attention-mechanism-synthesis-2026-09-20.md](geometric-attention-mechanism-synthesis-2026-09-20.md).
Base revision `0492926d`. Frozen parents: `relational-learning-4/artifacts/*.rlr2` (SHA256-pinned).

## What is being repaired, and what is retained

PR #1328 reported `useful_transfer=true` for a misindexed policy. The review's decisive finding is accepted:
the runner computed tune-selected gap thresholds `[-6784,-4992,-2944]` and then built **both** fit-event
streams from the unchanged legacy parent, whose thresholds are `[0,0,0]`. Because `gap <= 0` and the bucket
counts strict `gap > threshold`, every fit event landed in gap band zero — buckets 0–7 only — while serving
addressed all 32. The exported selector's `data_digest` was zero and the seed tables were never fitted at
their served addresses.

Retained: the measured teacher-forced text improvement **as its original scoped measurement with the defect
attached**, the stronger parent source learners, and PR #1323's narrow controller positive. **Withdrawn:** the
claim that this evidence demonstrates a contextual-representation limitation. The rounded i32 arithmetic, the
`choose`-based regret helper and the admission denominator are all accepted as defects.

## The one implementation repair

A single immutable **`PolicyConfig`** is created from the tune-derived gap thresholds **before any
`PolicyEvent` exists**. Its `digest()` covers the bucket formula, gap units, the strict widened-i64
comparison, the context bits, the margin rule, the action encoding, the bucket/gap/action counts, the shifts
and the thresholds. That configuration produces the fit-time selector (`PolicyConfig::apply`), the events, the
exported selector and the serving selector, so the two partitions are the same array by construction.
`RelationalSelector::verify_policy_contract(&cfg)` is a **verified loader contract** the consumer must call
before prediction; the artifact carries the digest of the **fit inputs** in its `data_digest`, and the
delivery manifest carries each artifact's byte hash. The binding is acyclic: config digest ← declared
semantics + thresholds; artifact ← config digest + fit-input digest; manifest ← artifact bytes.

Frozen for this first corrected fit — no refits, no admission widening, no bin or threshold sweeps:
improved H4 and categorical source maps/rankers, E+S local predictor, 128-token ring, ≤24 candidates, the
four actions `{NoRead, 0.0625, 1, 8}` nats, the existing support rule and features, and the equal-total-weight
construction/text mixture.

## Metric repairs in the same run

- `eval_stream` passes the **actual served action** from the shared path into a policy-aware decomposition
  (`regret_decomposition_acted`); the scored `choose`-based helper returned `None` for a policy selector and
  therefore measured NoRead for an executed read. The identity `ranking + gate + dose = actual − Lpool` is
  asserted per position, and `actual` is compared against the loss change implied by the **actually emitted
  logits** (mismatches counted and reported).
- Per-stratum `reads`, `NoRead`, per-action counts, correct payload reads and **correct emitted tokens**,
  alongside the local emitted-correct count.
- Admission regret for **every** position including an empty admitted pool (`Lpool = 0`), reported with
  **both** denominators: candidate-only and complete-stream. The parent's 0.25 trigger was candidate-only.
- Per-document text sums and per-sequence construction comparisons are serialized; document-cluster
  intervals come from those sums.
- The four-condition intervention additionally records whether the **original occurrence is still admitted**,
  whether the candidate pool size changed and whether neighbouring records' payloads/features changed.
- A serialized-structure preflight validates the object actually written against the declared paths.

## Populations (declared before scoring)

- **New** construction seed `SEED_FINAL2 = 0x5C0F_F2B2` over the held-out payload bank is the **final**
  assessment. `SEED_FINAL`, inspected by PR #1328, and `SEED_FRESH` are retained as
  `construction_previous_seed` / `construction_regression` **development** panels.
- Reader text, document-disjoint by path and content hash (Dev sorted by `sha256`): fit `[0,8)`,
  tune `[8,16)`, **previous** `[16,24)` (development), **final `[24,32)`** (new). Four 64-token windows per
  document, chosen by the declared preparation probe against a 600-candidate-position minimum target.
- Every comparison uses the same full prediction stream (empty pools take the local prediction) and the same
  denominators.

## Arms (bounded)

`local`, `exact_parent`, `relational_parent`, `relational_ctx_parent`, `categorical_parent`, `h4_policy`,
`categorical_policy`, and a fixed **`h4_one_nat_fixed`** comparator (opcode 2 everywhere on the same ungated
source and contract). The constant comparator tests whether a fitted table adds anything beyond a generic
small boost; it is not a new model path. All arms are the **independently reloaded** artifacts.

## Prospectively declared outcomes

- **Relational preservation** uses final-present **emitted correctness** and loss versus the retained parent
  under the existing 0.05 bits/position margin.
- **Absence improvement** requires absent-query **read counts** and loss each no worse than the parent,
  reported separately; payload correctness is deliberately excluded because an unknowable absent answer is
  not expected to be guessed. If a count check is unavailable it is reported as unavailable, not PASS.
- **Harm containment**, **useful text probability transfer**, **improved generated behavior** and general
  language competence are separate outcomes and are reported separately.
- The parent's 0.25 admission trigger and the corrected candidate-only / complete-stream means are reported;
  admission is unchanged in this run.

## The one bounded extension (decided from development evidence after the corrected fit)

Following the prompt's own ordering: (1) if the corrected policy preserves useful relational emission and
text benefit, retain it and formulate the next structural-context step; (2) if a weighted-CE choice trades
away emission, inspect per-bucket/per-action costs and emitted-correct counts by fit/tune stratum and use
**at most one** prospectively declared constrained utility objective on the same features; (3) if a causal
distinction is still absent despite correct indexing and support, name the missing information concretely
(equal gaps hiding different normalization mass, or the one-bit margin discarding relation identity) and
consider exactly one causal observation; (4) if target support is the bottleneck, distinguish retention,
admission and generation. No bucket/width/strength/corpus/threshold sweep.

## Resources

Live ledger at recording **191016749 / 194900000 ms**, **3883251 ms (~65 min)** remaining. Free space
**40.57 GB** against a **36.77 GB** reserve; 128 MiB model-stop margin retained.

Projection: library and runner changes plus the new contract test; ~4–6 incremental compile/test cycles
(≈60–180 s each on the retained target); one complete harness run over eight arms and seven full streams
(estimated **240–360 s**); documentation, delivery and knowledge work (~1,200 s). Complete allowance
**≤ 60 min wall**, ≤ 2 compiler workers / 1 model worker, ≤ 8 GiB peak RSS, ≤ 512 MiB new reports,
≤ 6 GiB incremental reusable build. **No new ledger extension is taken**; if a phase overruns, stop at a
completed boundary and record the actual debit.

## Deliverables

New claimed, sealed, verified root `.uor-models/realtext-prior-2026-09-20/reader-utility-3`. `reader-utility-1`
stays unsealed and `reader-utility-2` stays sealed, byte-identical. Update current state, roadmap/README
claims, result/receipt/ledger and owning issues (none closed); compact revision-pinned knowledge import with
verified retrieval.
