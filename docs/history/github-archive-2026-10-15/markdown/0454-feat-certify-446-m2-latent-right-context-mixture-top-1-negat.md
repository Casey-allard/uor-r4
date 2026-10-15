# #454 — feat(certify): #446 M2 — latent right-context mixture: top-1 NEGATIVE, bits strongly positive (causally legitimate)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T05:54:27Z
- **Updated:** 2026-08-06T13:37:41Z
- **Closed:** 2026-08-06T13:37:41Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/454
- **Merged:** 2026-08-06T13:37:41Z
- **Merge commit:** c0233d9b2085b75ae7f7547cc63c0b8c0f0c3050
- **Base / Head:** main / UOR-Foundation:i446-m2-latent-right
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #446. Implements P(next | left) = Σ_c P(c | left)·P(next | left, c), where c is a right-context class observed only during CONSTRUCTION and marginalized away at serving. **Serving reads the left key only — this is causally legitimate and quotable as a generation number**, unlike M1.

In-pipeline (schema 24), natural 500k corpus, 100,306 held-out. Prior rows reproduced exactly.

| arm | top-1 | bits/token |
|---|---|---|
| Rule 1+2 (baseline) | 26.42% | 8.2674 |
| TLA3 store baseline | 26.41% | 7.8832 |
| 1+2 + TWO-SIDED (M1, not causal) | 31.03% | 6.9411 |
| **1+2 + LATENT-MIX (M2, causal)** | **25.91%** | **7.3707** |
| 1+2 + latent ORACLE-RIGHT (not causal) | 31.82% | 7.3077 |
| 1+2 + latent SHUF-CLASS (falsifier) | 21.76% | 7.5333 |

**Exit rule NOT MET: −0.51pp top-1, needed +2.0pp. Recorded negative.** Headroom fraction −9.5%: marginalization gives up all of the two-sided top-1 gain.

Two findings inside the negative:
1. **Bits are a real causal win: 7.3707 vs 8.2674 = −0.897 bits/token**, also beating the TLA3 store baseline (7.8832). The mixture is far better *calibrated* even though its argmax is slightly worse. This is the first causally-legitimate distributional improvement we have measured.
2. The falsifier separates cleanly (25.91% vs 21.76%), so P(c|left) carries genuine left-conditional information — the machinery is not vacuous, the information is simply insufficient to move the mode.
3. **ORACLE-RIGHT at class depth 1 reaches 31.82%, above full M1 two-sided (31.03%)** — one byte of right class captures the entire two-sided top-1 gain. The loss is *entirely* in marginalization, not in coarsening.

Class = graded prefix of the right key at depth 1 (≤256 classes, 6.70 classes per full left code, retaining 6.7× of M1's 17.1× subdivision); Jeffreys smoothing over observed classes; deepest-populated-left-prefix backoff at the EXCT support gate.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

