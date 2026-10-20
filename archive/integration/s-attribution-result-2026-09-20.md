# Result — frozen S attribution: what the separable reader actually contributes

> **Post-#1312 principal correction and successor:** [review](occurrence-reader-review-2026-09-20.md), [next occurrence-reader prompt](deepseek-occurrence-reader-step-2026-09-20.md). The numerical attribution supports moving on. It does not establish universal absence of older content or that length dependence causes M01's gain. M01 includes terminal positions without targets; only34 extras affect evaluated lengths. Timings use cached/precomputed logits, not direct serving. Decision-rule/provenance/API scope corrections are documented. This document's original result/instructions remain below as history; no full rerun is selected. Continuation is required for the actual next fitter, not automatically for unused CPQK.

Date: 2026-09-20. Base source `8bbdb63d1c686d078292562bb879c0bf4dadde34` (merge of PR #1311), executing
[the frozen attribution prompt](deepseek-separable-attribution-step-2026-09-20.md). Isolated worktree
`codex/separable-attribution` at `.worktrees/geometric-query-read`. Retained root
`.uor-models/realtext-prior-2026-09-20/s-attribution-3` (30 files, 4,390,694 bytes, sealed and
verified, 0 unlisted members, `result.json` `994a1a15…`). Two earlier roots are preserved sealed and
superseded: `s-attribution-1` (corrected cost protocol and phase accounting) and `s-attribution-2`
(after `cargo fmt`, whose rebuilt binary had a different digest and so could not be bound to the
retained run). All three report identical losses and controls.

**Decision: S11's improvement is a current-token query/emission calibration, not older-content
memory.** Two independent comparators point the same way on both panels, each with a paired interval
entirely below zero: replacing the individual older state by the fit-average history at the same
length (M01) **improves** CE, and dropping the older reader row entirely (Qonly) **improves** CE.
The older row on its own is indistinguishable from the frozen parent. No candidate is promoted and
no new learning was performed.

## What was executed — evaluation only

No optimizer update, no reset/write fit, no history-capacity expansion, no new corpus, no projection
campaign, no decoder change. The harness constructs no trainer; its schema field is
`analysis_kind: "evaluation_only"`.

**Provenance.** The run binds its own executable SHA256
(`65463ebdb87980b05df1622f5e7ff197840278a32bcf2440a0876c01e935c4a1`), the two declared source-file
hashes (`query_read.rs` `b7c8fd48b75afa17…`, `query-read-attribution.rs` `a65c624aa1c65d8e…`), the
base revision and the working-tree dirty state. The superseded run's executable identity remains
**UNAVAILABLE** and is not fabricated.

**Repairs delivered with this step (the ones the review identified).**

1. **Tokenizer binding.** All three legacy `CPX3` artifacts had an all-zero tokenizer field at bytes
   73–104. `QueryTrainer::hard_core` supplied `[0; 32]` and the runner bound a hash of the *hexadecimal
   digest string* instead. Production export now **requires** the real raw 32-byte identity
   (`ZERO_DIGEST` is rejected by `from_parts` and by `QueryTrainer::new`), and the raw digest
   `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f` is propagated into the artifact.
   The hash-pinned legacy files load **only** through the restricted `QueryHard::import_legacy`, and
   the production loader is verified to reject the placeholder.
2. **Corrected descendants.** Each legacy artifact was re-exported with the real digest. Every
   descendant has the identical size (53,555 bytes) and **exactly 32 changed bytes, all inside
   73–104** (verified in code, not asserted in prose).
3. **Inference seam.** `QueryHard::inference_rows` selects the two reader rows and folds the older
   prefix at most once, building no training chain; `rows_from_older` consumes a supplied validated
   state without rebuilding a recipient chain; `generate` now uses the seam. The training path is
   unchanged and `read_path`/`read_path_from_older` remain for it.
4. **Public replay boundary.** `validate_tokens`, `inference_rows_checked` and `generate_checked`
   reject out-of-vocabulary tokens and empty prompts instead of silently clamping to another token.
5. **Evaluation-only entry point.** `bin/query-read-attribution.rs`, which cannot fall into fitting.

New hashes: Q `341f0721…` → `5adba9712ec46888…`; S `9f8cda09…` → `ed9affd638ee9367…`;
L `b0e50887…` → `167e2ea045ed3ac1…`.

| Focused test | Result |
| --- | --- |
| `query_read` module | **18 passed** (13 retained + 5 new) |
| Nonzero digest round trip + mismatch rejection + `from_parts` placeholder refusal | pass |
| Legacy zero-digest import restricted; production refused; repair changes exactly 32 bytes | pass |
| Inference seam compose counts: S = 62 (training path 124), Q = 63, **L = 3** (training path 65) | pass |
| `inference_rows` equals the training path's rows at every position, all three arms | pass |
| Checked boundary rejects out-of-vocabulary tokens and empty prompts | pass |

## Reproduction of the old evidence

All **16** retained old-panel vectors (E, and own/donor/identity-query/reversed/disabled for Q, S and
L) were recomputed from the corrected artifacts and compared to
`query-read-2/vectors/*.f64`: **maximum absolute delta 0.0** (exact) against a 1e−8 bit tolerance.
Frozen E micro CE reproduces at **7.170815717556** bits/target. The reconstructed 36-document /
288-window / 17,342-target panel and the old donor map match the retained `panel.json`
record-for-record and index-for-index. **S11 reproduces its retained old generation exactly** for all
six prompts.

## Frozen analysis populations

- **Old panel:** 288 windows, 17,342 targets, 16,766 older-present (matches the review's count).
- **New-position panel**, frozen from document/window identities before any score: 277 windows,
  17,451 targets, **35 of the 36** dev documents (one document offers no eligible chunk after the
  old selection and duplicate-hash exclusions; disclosed in `selection-manifest.json` per document
  with its shortfall). Selection rule and per-window rank digests are saved. Donor map rebuilt with
  the separately bound seed `0x5e9a11b7` (1,230 eligible of 16,692 strata). One dev document in the
  old panel also contributes no eligible new chunk.
- **Scope:** new positions in the *same* open-development documents. This is a replication check, not
  independent documents and not final held-out qualification.

## The decomposition

`Z_S = E + u(q) + u(b)`, with `u(s) = 128 · W R[s]`. Identity-anchored:
`C = 2u(e)`, `H(q) = u(q) − u(e)`, `K(b) = u(b) − u(e)`. Per-vocabulary-row integer identity
`Z11 + Z00 = Z10 + Z01` verified in wide integers: **0 failures** on both panels.

Micro CE bits/target:

| Condition | old, all | old, older-present | new, all | new, older-present |
| --- | ---: | ---: | ---: | ---: |
| E | 7.170816 | 7.151750 | 7.048239 | 7.045959 |
| S11 = E + u(q) + u(b) | 7.032228 | 7.008400 | 6.919775 | 6.913283 |
| S01 = E + u(e) + u(b) | 7.117573 | 7.096678 | 7.012454 | 7.009001 |
| S10 = E + u(q) + u(e) | 7.222888 | 7.205611 | — | — |
| S00 = E + 2u(e) | 7.501118 | 7.493400 | 7.387922 | 7.396780 |
| Qonly = E + u(b) | **7.022581** | **6.998422** | **6.912660** | **6.905935** |
| Honly = E + u(q) | 7.171048 | 7.151990 | 7.045576 | 7.043209 |
| M01, fit-average history (offline) | — | **6.984779** | — | **6.894588** |

Comparisons `CE_a − CE_b` on matched older-present positions (positive means `b` is better), 2,000
document-bootstrap draws, seed `0x12345678`, ratio of resampled loss sums/counts; zero-support
resamples counted explicitly (0 dropped in every interval):

| Comparison | new panel | old panel |
| --- | --- | --- |
| **primary M01 − S11** | **−0.018695 [−0.022344, −0.015019]** | **−0.023622 [−0.026746, −0.020352]** |
| Qonly − S11 | **−0.007348 [−0.012050, −0.002327]** | **−0.009978 [−0.014586, −0.005420]** |
| S01 − S11 | +0.095717 [0.083671, 0.108199] | +0.088277 [0.074894, 0.101805] |
| S10 − S11 | +0.192411 [0.173018, 0.212720] | +0.197210 [0.180399, 0.214052] |
| Honly − S11 | +0.129926 [0.114523, 0.146087] | +0.143589 [0.131690, 0.155425] |
| E − S11 | +0.132676 [0.114919, 0.152045] | +0.143349 [0.130147, 0.155997] |
| E − Qonly | +0.140024 [0.124527, 0.156375] | +0.153327 [0.141358, 0.165063] |

## Reading

- **The current-token query row is the whole measured gain.** `Honly = E + u(q)` is statistically
  indistinguishable from E (7.171048 vs 7.170816 old; 7.045576 vs 7.048239 new). Dropping `u(b)`
  costs 0.130–0.144 bits. B selects only eight reader rows for a 4,096-token vocabulary, so this is a
  learned local emission/class calibration, exactly the hypothesis the review said to measure.
- **The individual older state adds nothing and is mildly harmful.** Dropping `u(q)` entirely
  (Qonly) *improves* CE by 0.0073 (new) / 0.0100 (old) bits with the paired interval entirely below
  zero on both panels. The point gains are below the newly declared 0.01-bit component margin, so
  this is a signed, replicated effect rather than a decisive removal — but it is not evidence of
  benefit.
- **A content-free, length-conditioned history replacement is better still.** M01 keeps the query
  branch and the fit-average history by older-prefix length and removes each observation's specific
  older state; it beats S11 by 0.0187 (new) / 0.0236 (old) bits, interval entirely below zero on both
  panels. So the useful part of the history term is a **length-conditioned offset**, and the
  individual content is worse than that average.
- **The identity-anchored constant is harmful.** `2u(e)` costs 0.485 bits; `u(e)` is the main reason
  S01 and S10 look bad. This is why S01 is a poor "local-only" comparator: it retains a learned
  constant, as the review warned. Qonly, which drops the row rather than substituting the identity,
  is the clean local condition.
- **Why the conditional donor is silent.** The exact-tail donor map preserves the local pair and the
  older length, so it never perturbs the length-conditioned offset — the only part of the history
  term that carries a measurable effect. Its null result (−0.002298 old, reproducing the retained
  value exactly; +0.005278 new, interval spanning zero) is therefore *consistent* with the
  decomposition rather than in tension with it.
- **Older-order reversal** is −0.005064 [−0.009529, −0.000330] old and −0.001385 [−0.005892, +0.003060]
  new: a small, inconsistent directional effect, not a usable ordering signal.
- **Correction to the automated label.** The sealed `result.json` carries a conservative
  `"inconclusive at this support"` string produced by a two-branch rule keyed on S01. On this
  evidence the correct classification is the prompt's second branch: **history contributes nothing
  as individual content while the local query is useful** — preserve the local result and target the
  state-maintenance problem directly; do not advertise the 120-state group product as useful memory.
  The sealed artifact is left unchanged; this paragraph is the interpretation-layer correction.

**No history lead.** The predeclared lead required primary and conditional-donor effects ≥0.01 bits
with positive lower bounds on the new panel, and no old-panel comparison below −0.01 with its whole
interval negative. The primary is significantly *negative* on both panels and the donor interval
spans zero, so the lead condition fails decisively.

## Matched interventions and instrument checks

| Check | old | new |
| --- | --- | --- |
| Strata / eligible / changed reads | 16,583 / 1,177 / 564 | 16,692 / 1,230 / 648 |
| Singleton strata | 15,589 | 15,667 |
| No-history positions excluded | 576 | 554 |
| Eligible documents | 36 | 35 |
| E, S01, S00, Qonly invariant to donor **and** reversal | **exactly** | **exactly** |
| `q = e` / `b = e` / `q = b` positions | 303 / 4,501 / 290 | 277 / 4,365 / 249 |

Every content-independent condition is byte-identical under both interventions, so the constants are
counted correctly; the identity and coincidence counts are reported separately.

## Generation

Six retained prompts, 64 greedy tokens, unchanged tokenizer, lowest-ID ties and decoder, from E and
from S11/S01/S10/S00/Qonly/Honly as frozen conditions. S11 reproduces its retained old token IDs
exactly. **Every condition repeats**: 6 of 6 prompts show a repeated local pair, and only the all-32
prompt (`[32; 16]`, prompt index 3) reaches a repeated *complete bounded ring* at offset 48 —
one of six for each condition. A repeated local pair cannot certify a history-dependent condition;
the ring is its sufficient state. Observed repetition is reported, not treated as language quality.

## Cost

Identical protocol for every label: the retained prompt, 64 greedy tokens, one discarded warm-up
repeat then five timed repeats, `black_box` on input and output, results consumed, complete
numerical path including the frozen parent score.

| Label | min (s) | median (s) | spread (s) |
| --- | ---: | ---: | ---: |
| E | 0.000530 | 0.000530 | 0.00001 |
| S11, inference seam | 0.000770 | 0.000770 | 0.00000 |
| S11, old training-path rows | 0.000790 | 0.000800 | 0.00004 |
| S01, inference seam | 0.000770 | 0.000780 | 0.00001 |
| Qonly, inference seam | 0.000650 | 0.000650 | 0.00003 |

The seam removes half the older-fold compose calls for S and (65 → 3) for L, and is measurably
faster here (0.77 vs 0.80 ms median; about 4%), but the residual row work dominates so the saving is
small at this scope. Loaded bytes: parent **454,788**; each corrected artifact **53,555**;
frozen-parent score cache 16,390 entries at the end of the timed section. Whole-process peak RSS
**722,681,856 B** for the complete run (`/usr/bin/time -l`), wall **102.47 s** — that is whole-run
RSS, not steady-state inference allocation. **Physical energy remains UNAVAILABLE.**

## Boundaries

This is a fitted-model dependence diagnostic on a reused open-development document set. It is not a
causal sufficiency theorem, not an equivalence proof (Qonly's interval is not contained in ±0.01),
not model promotion, not a language result, and not a statement that the 2I group is useless — it
says this *arrangement* at this *dose* carries no usable individual older content. M01 is an offline
comparator only and is never a serving incumbent. The four-bit `Continue`/`ResetTo` monoid, exact
occurrence/version access, shared composition, useful conversation and executed Rust on one artifact
with complete M1 cost all remain unrun.
