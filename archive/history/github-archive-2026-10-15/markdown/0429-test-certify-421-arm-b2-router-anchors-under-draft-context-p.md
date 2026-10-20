# #429 — test(certify): #421 ARM B2 — router anchors under draft context: POSITIVE (non-oracle survives)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T04:56:53Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-08-05T05:50:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/429
- **Merged:** 2026-08-05T05:50:33Z
- **Merge commit:** c61219d84cddc9c2338971c8cb3e7d1f4bae7aa9
- **Base / Head:** main / UOR-Foundation:i421-b2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #421 #273. Removes M-R1's main limitation: ARM B2 queries the router with a DRAFTED context (base-scorer greedy fill — what real generation holds), not true corpus tokens. Only 251/9,667 draft contexts equal the oracle ones, so the arm is a genuinely different test. **B2 free top-1 4.9% vs unigram 2.1% (+2.74pp, rule ≥2) and vs shuffled 3.1% (+1.79pp, rule >0) → POSITIVE.** Oracle B was 7.6% — draft drift costs ~2.7pp but the signal survives the pre-declared rule. The reconnection result now stands WITHOUT the oracle-context caveat; non-oracle evidence for #273's routed-state claim.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

