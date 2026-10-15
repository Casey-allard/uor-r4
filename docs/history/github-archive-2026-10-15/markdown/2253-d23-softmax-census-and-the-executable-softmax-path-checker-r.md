# #2253 — D23 softmax census and the executable softmax-path checker (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T12:48:12Z
- **Updated:** 2026-10-13T02:49:35Z
- **Closed:** 2026-10-12T13:11:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2253
- **Merged:** 2026-10-12T13:11:51Z
- **Merge commit:** 3f6d97bc427d2add27b6805176c3e6196e20b5bc
- **Base / Head:** main / UOR-Foundation:kimi/softmax-census-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 (instrument PR; prepares the D23-compliant runs) · headline: unchanged

**What this is.** The owner direction's next source task — trace the complete executed output/objective/gradient path before any compliance claim — delivered as (1) the census record and (2) the executable checker the D23 record says does not exist yet. Claimed on #2028 comment 6124257305. Source/tooling only: no model compute, fully D23-compliant.

**Files**
- `scripts/softmax_path_check.py` — scans an executed path's source for banned ops (softmax / log-softmax / LSE normalization / softmax-CE / exp-normalize), skipping tests and comments; prints CLEAN or file:line hits per profile (`stack-objective`, `stack-serve`, `d11-serve`, `native-learner`, `training-aids`); records scalar exponentials (sigmoid/tanh/exp tables) separately so the classification is reviewable; `--self-test` included.
- `docs/labs/softmax-census-2026-10-12/README.md` — the map: every active executed path computes a normalized exponential today (verdicts at this head: 14 / 14 / 7 / 21 / 63 banned-op sites across the five profiles), with the softmax-free inventory for the replacement (rank reads + Q31 D11 path + export gate, greedy decoding, the copy-span oracle, hard native generation, rank/margin objectives) and the gap analysis: no wired non-softmax objective exists in Line A.
- `docs/integration/current-state.md` — the entry.

**Checks at exact head:** `check_claim_wording.py` PASS; `softmax_path_check.py --self-test` PASS; the checker's verdicts cross-read against an independent line-by-line sweep of the same tree. Tooling + docs only: no compile-relevant change.

**Review:** exact-head self-review (owner rule), scope the three files.

**Decision test:** every lab needs this verdict before claiming D23 compliance and before re-registering any run — it changes what the replacement learning law must cover and what each lab does next.

References #2028.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

