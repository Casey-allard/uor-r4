# #2244 — Kimi lab joins: joining review, direction recommendations, session-goal prompt (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T06:44:57Z
- **Updated:** 2026-10-12T06:47:48Z
- **Closed:** 2026-10-12T06:46:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2244
- **Merged:** 2026-10-12T06:46:10Z
- **Merge commit:** 77e257939d688a4cd05b53c505fd026fd8219345
- **Base / Head:** main / UOR-Foundation:kimi/direction-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 0/3 · headline: unchanged (M1 v5 memory 20/40, reply 37/232)

**What this is.** The Kimi lab's joining delivery (owner request today; registration #2028 comment 6118939798): a full repository + live-GitHub + external-literature read, with direction recommendations and a proposed `/goal` prompt. Docs only; no training, no pods, no model measurement.

**Files**
- `docs/labs/kimi-direction-2026-10-12/README.md` — the memo: project state, six-month shape, mechanism inventory, internal + external adversarial findings, recommendations F1–F7 (owner decisions requested: geometry-audit answers, bench re-baseline, forced-copy oracle, integer-softmax discriminator, synthetic-KEEP transfer gates, standards).
- `docs/labs/session-goal.md` — proposed Kimi `/goal` prompt (3,986 chars, under the 4,000 limit); takes effect when the owner pastes it.
- `docs/integration/current-state.md` — newest-first entry.

**Checks at exact head `6821402c7`** (base `c4d682cd0`): `python3 scripts/check_claim_wording.py` PASS. Docs-only: no compile-relevant change. Local verification recorded in the memo: uor-r4-integer lib 284/284; the joint_campaign red-on-main diagnosed (env-bound `source_commit`; 23/23 pass with `UOR_BUILD_SOURCE_COMMIT` set) and posted on #2028 comment 6119168767.

**Review:** exact-head self-review (owner rule), scope: the three files above; evidence: this body + the record.

**Decision test:** if the owner rejects the F1–F7 recommendations or the line assignment, the next cycle changes (the oracle and bench work proceed only as adopted); so this PR changes what Kimi does next.

References #2028. References #2029. References #2032.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

