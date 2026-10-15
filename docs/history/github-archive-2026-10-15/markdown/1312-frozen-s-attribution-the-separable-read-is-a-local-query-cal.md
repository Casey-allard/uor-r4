# #1312 — Frozen S attribution: the separable read is a local query calibration, not history (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T18:35:36Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-09-20T18:41:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1312
- **Merged:** 2026-09-20T18:41:07Z
- **Merge commit:** 0bca39b541e0ee56cc58bfff65a625a7854d5433
- **Base / Head:** main / UOR-Foundation:codex/separable-attribution
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes [deepseek-separable-attribution-step-2026-09-20.md](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/deepseek-separable-attribution-step-2026-09-20.md) from refreshed `main` at PR #1311 (`8bbdb63d`).

**One complete evaluation-only experiment.** No optimizer update, reset/write fit, history-capacity expansion, new corpus, projection campaign or decoder change: `bin/query-read-attribution.rs` constructs no trainer.

## Repairs (the ones the #1311 review identified)

- **Tokenizer binding.** Production `CPX3` export now **requires** the real raw 32-byte tokenizer identity; the all-zero placeholder is refused by `QueryHard::from_parts` and `QueryTrainer::new`. The hash-pinned legacy zero-digest files load **only** through the restricted `QueryHard::import_legacy`, and the run verifies the production loader rejects them. Corrected descendants are the identical size with **exactly 32 changed bytes, all inside 73..105**, plus full row/integer-logit parity. Q `341f0721…`→`5adba9712ec46888…`, S `9f8cda09…`→`ed9affd638ee9367…`, L `b0e50887…`→`167e2ea045ed3ac1…`.
- **Inference seam.** `inference_rows`/`rows_from_older` separate inference row selection from training-chain construction: S folds the older prefix **62** times instead of 124, **L 3 instead of 65** (instrumented compose counts), with row equality against the training path at every position for all three arms.
- **Checked boundary.** `validate_tokens`/`inference_rows_checked`/`generate_checked` reject out-of-vocabulary tokens and empty prompts instead of silently clamping.
- **Provenance.** The run binds its own executable SHA256 (`65463ebd…`), both declared source-file hashes (`query_read.rs` `b7c8fd48…`, `query-read-attribution.rs` `a65c624a…`), the base revision and the dirty state.

**18** `query_read` module tests pass (13 retained + 5 new); 8 head-projection tests pass; `cargo fmt --check`, the touched-package offline check and `scripts/check_claim_wording.py` are clean.

## Reproduction first

All **16** retained old-panel vectors (E; own/donor/identity-query/reversed/disabled for Q, S, L) recompute to a **0.0 maximum absolute delta** against `query-read-2/vectors/*.f64`; E micro CE 7.170815717556; the 288-window / 17,342-target panel and its donor map match the retained manifest record-for-record; S11 reproduces its retained six-prompt generation exactly.

## Finding — no history lead

On `Z_S = E + u(q) + u(b)`, with the integer identity `Z11+Z00=Z10+Z01` verified with **0 failures** on both panels:

| condition | old micro (all) | new micro (all) |
| --- | ---: | ---: |
| E | 7.170816 | 7.048239 |
| S11 = E+u(q)+u(b) | 7.032228 | 6.919775 |
| **Qonly = E+u(b)** | **7.022581** | **6.912660** |
| Honly = E+u(q) | 7.171048 | 7.045576 |
| M01 fit-average (offline) | 6.984779* | 6.894588* |

\* older-present positions. Matched older-present, paired 2,000-draw bootstrap, seed `0x12345678`:

| comparison | new | old |
| --- | --- | --- |
| **primary M01−S11** | **−0.018695 [−0.022344, −0.015019]** | **−0.023622 [−0.026746, −0.020352]** |
| **Qonly−S11** | **−0.007348 [−0.012050, −0.002327]** | **−0.009978 [−0.014586, −0.005420]** |
| Honly−S11 | +0.129926 [0.114523, 0.146087] | +0.143589 [0.131690, 0.155425] |
| E−S11 | +0.132676 [0.114919, 0.152045] | +0.143349 [0.130147, 0.155997] |

Every interval is entirely below or above zero with 0 zero-support resamples. Conclusion: **S's measured improvement is a current-token query/emission calibration**. The older row alone is indistinguishable from E, dropping it *improves* CE on both panels, and a content-free length-conditioned fit average beats the individual older state by ~0.02 bits on both panels. E/S01/S00/Qonly are *exactly* invariant to the older-prefix donor and reversal maps. The exact-tail donor is silent because it holds the length-conditioned offset fixed — the only part of the history term with a measurable effect.

The sealed `result.json` carries a conservative `"inconclusive"` string from a two-branch rule keyed on S01 (which retains a harmful learned constant); the review records the corrected classification — this prompt's second branch: **preserve the local result and target state maintenance directly; do not advertise the 120-state group product as useful memory.** The sealed artifact is left unchanged.

## Frozen populations and controls

Old panel 288 windows / 17,342 targets / 16,766 older-present. New-position panel frozen from document/window identities **before scoring**: 277 windows / 17,451 targets / 35 of 36 dev documents (one offers no eligible chunk after the exclusions; shortfalls disclosed per document), donor seed `0x5e9a11b7`, 1,230 eligible of 16,692 strata. New positions in the same open-development documents — not independent documents and not held-out qualification.

## Generation and cost

Six retained prompts, 64 greedy tokens, unchanged decoder: every condition repeats on 6 of 6 prompts at the local-pair level; only the all-32 prompt reaches a repeated *complete bounded ring* (1 of 6). S11 reproduces its retained output exactly. Repetition reported, not read as language quality.

One declared protocol (retained prompt, 64 greedy tokens, discarded warm-up then five timed repeats, `black_box`, complete numerical path): E 0.530 ms, S11 seam 0.770 ms, S11 old row path 0.800 ms, S01 0.780 ms, Qonly 0.650 ms (medians). Whole-run peak RSS **722,681,856 B**; wall **102.47 s**. Parent 454,788 B; each corrected artifact 53,555 B. **Physical energy UNAVAILABLE.**

## Resources

Standing-authorized **+2400000 ms** extension to **180500000 ms** recorded in the live ledger **before** consumption. This step charges **2,400,000 ms** once. New cumulative **178,538,565 / 180,500,000 ms**, remaining 1,961,435 ms.

Retained: `.uor-models/realtext-prior-2026-09-20/s-attribution-3` — sealed, verified, 30 files, 4,390,694 bytes. `s-attribution-1` and `s-attribution-2` preserved sealed with identical numbers. 128 MiB storage margin intact; no deletion, corpus download or paid/external compute.

## Next handoff

Give the state a mechanism that can make older content **addressable** rather than merely present — a bounded selective write/reset over exact 2I state (a four-bit `Continue`/`ResetTo` monoid candidate, **not** implemented or fitted here) or exact occurrence/version access — and qualify it against fresh behavioural criteria. Do not re-fit S, sweep width/precision/corpus, or treat the offline `M01` comparator as a serving candidate. **CPQK production continuation must be repaired before any new fit.** `#973`/`#820`/`#963`/`#964` remain open.

**References #973, #820, #963, #964 — does not close any of them.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

