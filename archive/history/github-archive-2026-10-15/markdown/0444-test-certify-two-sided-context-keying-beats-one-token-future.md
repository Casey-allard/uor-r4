# #444 — test(certify): two-sided context keying beats one-token-future fusion by 7pp — plus the distribution-class growth law

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T01:26:02Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-06T01:46:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/444
- **Merged:** 2026-08-06T01:46:46Z
- **Merge commit:** 5877c79523b2b450c5972277b26e7fdb00a0a642
- **Base / Head:** main / UOR-Foundation:i436-twosided
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Motivated by Kracht, *The Mathematics of Language*: distribution classes are defined by TWO-SIDED contexts (Galois/Sestier closure, p.24; `Cont(x)` as pairs ⟨left,right⟩, p.438). Our scorer conditions on the left only — and our shipped forward-anchor channel turns out to be a crude ONE-TOKEN right-context probe, which is why it bought +4.2pp.

**Natural 500k corpus, D3 split, 98,345 held-out positions:**

| arm | top-1 | bits/token |
|---|---|---|
| null (unigram) | 6.17% | 8.608 |
| ARM L (left-only) | 23.86% | 6.671 |
| ARM FWD (shipped fusion) | 31.72% | 6.484 |
| **ARM LR (two-sided)** | **38.77%** | **6.200** |
| ARM SHUF (falsifier) | 6.70% | 10.038 |

Exit rule (pre-declared): LR ≥ L+2pp ✓ (**+14.90**), LR > FWD ✓ (**+7.05**), LR > SHUF ✓ (+32.07). **CONFIRMED.** The falsifier is decisive: a foreign right key at identical cardinality, backoff and smoothing collapses to 6.70% — the gain is right-context *information*, not key-space inflation.

**NOT CAUSAL:** right context does not exist at generation time. This is an infill/analysis measurement (the A-mode regime), stated in the module docs and printed twice in the output. Do not quote as a generation number.

**Distribution-class growth law** (nested prefixes, 17× data range): nothing saturates. Distinct left keys ~n^0.44, distribution-signature classes ~n^0.46 (left) / ~n^0.80 (two-sided), mean support/class rising 7.3→35.6. Any fixed region budget or absolute split threshold is therefore wrong at scale.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

