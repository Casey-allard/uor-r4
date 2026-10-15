# #447 — feat(certify): #446 M1 — two-sided context arms in Gate C (with foreign-key falsifier)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T03:14:16Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-06T05:01:47Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/447
- **Merged:** 2026-08-06T05:01:47Z
- **Merge commit:** 9fb12da043f76dc869b28817c02e69acda6c2145
- **Base / Head:** main / UOR-Foundation:i446-twosided-pipeline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #446. Brings two-sided keying into the real scoring cascade, graded by the pipeline's own Gate C. Artifact format, serving default path, witness and replay contract untouched (witness replay 64/64 ok).

**Single-run Gate C table, natural 500k corpus, 100,306 held-out:**

| scorer | top-1 | bits/token |
|---|---|---|
| graph chain+EXCT (Rule 1+2) | 26.4% | 8.2674 |
| TLA3 store baseline | 26.4% | 7.8832 |
| 1+2 × fwd-anchor (#399 M2) | 31.1% | 8.0296 |
| **1+2 + TWO-SIDED (#446 M1)** | **31.0%** | **6.9411** |
| 1+2 + two-sided SHUFFLED (falsifier) | 8.0% | 9.3104 |

Live slice (98,360): two-sided 31.3% vs Rule 1+2 26.6%, bits **6.8909 vs 8.2433**. Falsifier collapses to 7.1% at identical key cardinality — the gain is right-context information, not cardinality.

**Dilution slice (the mechanism):** 98.9% of held-out resolves as ExactContext at full graded code. The right key subdivides 11,221 full-left-code cells into 192,381 two-sided cells (**17.1× subdivision**), and on exactly that over-diluted population gains **+4.6pp top-1 and −1.32 bits**. This is direct confirmation that fixed code-space dilution is the binding constraint and that right context splits it usefully.

Honest reading: two-sided **ties** the forward-anchor arm on top-1 in-pipeline while beating it decisively on bits (6.94 vs 8.03). It improves the distribution far more than the argmax.

**NOT CAUSAL** — keys on tokens after the target. Infill/analysis measurement, or prospectively a construction-time signal. Printed in the Gate C output and in the schema-23 docs. Never quote as a generation number.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

