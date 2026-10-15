# #1580 — Preserve identity-to-payload alignment in geometric attention

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T16:38:01Z
- **Updated:** 2026-10-09T13:17:18Z
- **Closed:** 2026-10-01T17:38:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1580
- **Merged:** 2026-10-01T17:38:24Z
- **Merge commit:** a74f36860645b173224a5a96c33c9484d5b148e3
- **Base / Head:** main / UOR-Foundation:codex/geometric-attention-binding-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Geometric reads on the existing Rust stack learned value-position roles but failed query-specific key/value association under the tested training conditions. An opt-in causal predecessor input now keeps identity with the current payload: query/key use the previous normalized state, while value, NoRead, age and candidate support remain current. No labels or token parser enter prediction, and default behavior is unchanged.

Sixteen matched 320-step fits separate identity carry from training-only occurrence supervision. Both Lorentz carry-only seeds and both Dot carry-only seeds answer192/192 original development rows correctly; all four also answer192/192 on a fresh2/8/16-fact draw, including twice the training maximum. Lorentz carry-plus-binding does the same while concentrating both heads on the correct source. Read-output ablation reduces carry-only accuracy to0–18/192. Source supervision alone does not repair the no-carry task. The report preserves Dot failures and the carry log-odds diagnostic domain failure; this does not establish natural language, geometric superiority, integer serving or energy savings.

Adds an optional exact-occurrence mass/loss to the same fused read, removed before prediction; versioned carry save/load metadata binds operation/config/weights. Carry is refused by served mode, integer export/grid reference and the checkpoint envelope until their corresponding port exists. Saved-model probes and both-head fresh challenge retain sealed per-attempt artifacts on the internal drive. Current state and plan identify variable-gap typed identity capture/hold/commit/reset as the next mechanism, with occurrence credit optional.

Validation executed locally: four occurrence-binding tests; five carry/gradient/reload/export/checkpoint tests; example compilation and all sixteen fits, independent reload checks and three saved-model fresh challenges; formatting, diff and claim-wording checks. Independent principal engineer and mathematician reviewed actual source and results. Queue compatibility checks are separate from executed validation. Full results/source identities and limitations are in `docs/integration/geometric-attention-binding-2026-10-01.md` and the compact evidence JSON.

References #1512. Wider geometric-attention language/serving acceptance remains open.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5936979585 — 2026-10-01T17:37:30Z — Casey-allard

Independent principal mathematician approval binds final head `1a6ef8de26fdccb74a7edc6529f4d237fc44a217`.

The reviewer confirmed live HEAD and the clean worktree. The sole delta from reviewed `f3ffbc2f` is four documentation/evidence files; all four SHA-256 hashes exactly match the content already reviewed. No code changed and no new blocker was found. This is independent source/evidence approval, separate from queue compatibility statuses.

Executed validation is preserved in the result document and artifact receipts: four occurrence-binding tests, five carry/boundary tests, sixteen matched fits with save/reload checks, and saved-model fresh development challenges. These authored association results do not establish natural language, geometric superiority, integer serving, or energy advantage.

All five required PR compatibility contexts pass. Proceeding through the ordinary protected queue under the owner's standing authorization, matching this exact head. The advisory delivery publisher is not a required context and is not being repaired or bypassed.



---
_Comment/review ids in this file: 5936979585_

