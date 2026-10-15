# #375 — Gate C never exercises NGRAM rows (empty recent-token window) — window mode + serving-gate decision needed for the trigram A/B

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T15:10:21Z
- **Updated:** 2026-08-03T19:48:16Z
- **Closed:** 2026-08-03T19:48:16Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/375
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Found while running the #364 trigram A/B (tracker: #374). `evaluate_gate_c_row` calls `score_candidates_coded(&sig, Some(&code), &[])` — an **empty recent-token window** — so NGRAM context rows (#362) can never fire on the Gate C surface. Measured consequence (fixture, TLA7, schema-14 reports): a default bundle and a `--context-order 0` bundle produce Gate C rows identical to the last digit; `exact_context_ngram = 0` of 85,429 despite 64,774 compiled rows; the rows cost +2,887,480 bytes (+16.8%) with zero effect on every metric the #358–#370 series used.

Two linked decisions before the trigram-replacement A/B can be measured:

1. **Gate C window mode.** Add an opt-in mode passing real recent-token windows to the row evaluation, so lexical rows participate. This is a measurement-semantics change: the exact-context slice would then blend NGRAM hits into every row — the #371 `exact_context_ngram` split exists precisely so era comparisons survive this, but the mode must stay opt-in with the default reproducing today's rows byte-exactly.
2. **Serving-surface alternative.** The C serving row measures rows naturally (the engine passes real context), but `validate_quality_report` unconditionally refuses bundles where graph top-1 < TLA baseline — which the current-era fixture bundle is (36.47% vs 39.09%), so `R4_CERTIFY_C_ONLY` refuses it (same class as the #299/#309 refusals). Either the A/B runs on a distribution where the graph clears the baseline, or the gate needs a measurement-bundle policy — the gate is process-owner territory, flagging for @auser / Alex.

Refs #364, #362, #374, #299.


## Comments (2)

### comment 5168661403 — 2026-08-03T15:52:01Z — Casey-allard

## Decision (maintainer-approved): opt-in Gate C window mode; no gate carve-out

Proceeding with the harness-side fix: an opt-in flag that feeds real story-bounded recent-token windows into the Gate C row evaluation. Default stays off and reproduces today's rows byte-exactly; with the flag on, the #371 `exact_context_ngram`/`exact_context_probe` split decomposes the exact-context slice so lexical-row contribution vs graph contribution is directly attributable on the same fixture and positions — that is the #362/#364 trigram-replacement A/B.

Explicitly rejected: weakening `validate_quality_report` to admit measurement bundles. The gate protects the serving path; the window mode answers the question without touching it. A serving-surface confirmatory run stays queued for a distribution where the graph clears the baseline.

Standing observation for the record: the current-era fixture failing that gate at all — deployed rule12 36.47% vs plain store baseline 39.09% — is itself evidence for the cover-induction line being the binding work.

Implementation PR follows on this issue.

### comment 5170992677 — 2026-08-03T19:48:13Z — auser

## Resolved

Implemented and merged in [PR #378](https://github.com/UOR-Foundation/uor-r4/pull/378), merge commit `73ab76b`:

- Gate C has an opt-in `--gate-c-context-window on` mode.
- The mode passes a story-bounded recent-token window capped at 32 tokens into every coded Gate C scorer, allowing compiled bigram/trigram NGRAM rows to participate.
- The default remains off, preserving the historical empty-window measurement semantics.
- `exact_context_ngram` / `exact_context_probe` attribution and the schema-15 report flag make window-on results distinguishable.
- The serving quality gate remains unchanged, per the maintainer decision here.

The follow-up sweep in [PR #385](https://github.com/UOR-Foundation/uor-r4/pull/385) confirmed the window mode is measurement-free at zero repetition penalty and that the observed windowed cost is attributable to the existing repetition penalty. The serving-surface confirmatory run remains appropriately separate and requires a distribution where the graph clears the baseline.


---
_Comment/review ids in this file: 5168661403, 5170992677_

