# #455 — feat(certify): #446 M3 — hard-select and top-k arms plus class predictability: closes the latent-class direction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T06:12:25Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-06T13:39:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/455
- **Merged:** 2026-08-06T13:39:53Z
- **Merge commit:** ce088264bc36efa6c9989701de7e4c7e4b5bf785
- **Base / Head:** i446-m2-latent-right / UOR-Foundation:i446-m3-hard-select
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #446. Stacked on #454. Tests whether SELECTING a right class beats marginalizing over it — and the diagnostic settles the whole direction.

| k | arm (all causal) | top-1 | bits |
|---|---|---|---|
| 1 | hard-select | 22.81% | 8.8856 |
| 3 | top-k | 24.54% | 8.1535 |
| all | marginalize (M2) | 25.91% | 7.3707 |
| — | Rule 1+2 baseline | 26.42% | 8.2674 |
| — | oracle right class (not causal) | 31.82% | 7.3077 |

**The curve is monotone the wrong way — more averaging is strictly better on BOTH top-1 and bits.** My 'marginalization flattens the mode' hypothesis is refuted: selecting sharpens the *wrong* mode.

**Class predictability, the diagnostic that closes it:** `argmax_c P(c|left)` hits the true right class **19.55%** of the time (19.83% at full depth, 12.00% backed-off) against **1.95%** uniform chance — a 10× lift, so the left key genuinely carries right-class information, which is why every latent arm beats the shuffled falsifier (21.76%). But mean posterior entropy is **4.25 bits over ~51 classes**: hard-select routes to the wrong class-conditional four times in five, and a sharp distribution on the wrong class is worse than a diluted one on the right one. **You cannot select what you cannot predict** — the oracle's +5.4pp is unreachable from the left key, and the latent right-class family is measured dead for top-1.

**The one survivor, worth keeping:** LATENT-MIX at **7.3707 bits/token is −0.897 vs baseline and beats the TLA3 store baseline (7.8832), causally legitimate, with top-1 at parity (−0.51pp)** — our first causally-quotable distributional improvement.

Schema 25; all pre-existing rows reproduce byte-identically (artifact κ `blake3:ec9391ce…`).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

