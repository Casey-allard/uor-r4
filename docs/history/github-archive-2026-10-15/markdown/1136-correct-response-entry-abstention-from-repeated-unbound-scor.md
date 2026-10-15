# #1136 — Correct response-entry abstention from repeated unbound scores

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-05T20:11:58Z
- **Updated:** 2026-10-09T13:17:43Z
- **Closed:** 2026-09-05T20:37:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1136
- **Merged:** 2026-09-05T20:37:37Z
- **Merge commit:** 597a86a30b87558c4783590e02e8a45933188dee
- **Base / Head:** main / UOR-Foundation:codex/native-source-entry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Repeated unbound response-entry features can make the model emit ` Unknown.\nKyoto.\n` even though the requested value is retained. Add a Rust offline intervention that zeros only the thirteen token scores in prefix feature 32/value 0, preserves candidate postings and all other parameters, and refreshes/validates artifact identities. The existing native serving path then emits ` Kyoto.\n`; no candidate refit, new serving operation, persistent state or context expansion is introduced.

The corrected artifact improves the wording diagnostic from 12/16 to 16/16 and the previously opened transfer set from 8/32 to 16/32 (supported answers 0/24 → 8/24, unsupported 8/8 preserved). It restores all 62 preservation responses and 8/8 binding outputs. Sixteen first-use prompts after fixed design selection reach 13/16 versus 11/16 for the shared-binding parent and 3/16 for the prior selected artifact. All three failures use “Which city is … in?” The broader 16/16 milestone remains unmet; this is retained development progress.

Provenance limitation: Pune had prior exposure. A supplemental vocabulary-novelty assertion failed, and the shell continued into evaluation. Model selection and Rust source preparation preceded evaluation, with no later fitting/tuning; the supplemental provenance receipt was written afterward. The record explicitly avoids fully sealed or vocabulary-disjoint qualification. The prior source and all results are preserved.

Validation: release example build, one focused fitted-artifact test covering only-intended-score changes/admission/reload/identity/errors/idempotence, formatting, architecture policy and claim wording pass. Actual native CLI Generation matches the evaluator exactly. All 62 outputs and causal states match with copy dispatch disabled; full/disabled complete generation takes 47.210/63.598 ms in one pass, retaining the existing dispatch optimization. All 32 generated Rust sources match previously compiled/executed source hashes. Broader release checks remain NOT_RUN; no geometric advantage is claimed.

Resource use:22.785 s model work,289.008 s engineering. The 6.8 s rounded fixture-test execution is conservatively included in both categories. Cumulative model work 1,081.641/1,800 s;718.359 s remains. Storage cap increased 256 MiB under standing owner approval;128 MiB margin preserved, no deletions or paid external compute. Local full results: `.uor-models/native-typed-value-2026-09-05/source-entry-checkpoint.json`.

Continues #973. Does not close the broader native-model objective.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

