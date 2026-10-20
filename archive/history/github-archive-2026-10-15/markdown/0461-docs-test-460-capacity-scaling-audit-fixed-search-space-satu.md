# #461 — docs,test: #460 — capacity-scaling audit: fixed search space saturates at scale

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T13:23:03Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-06T13:46:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/461
- **Merged:** 2026-08-06T13:46:15Z
- **Merge commit:** 0cbf02ecb4e95e7b8fc057b6d309ec89efd3f4e6
- **Base / Head:** main / UOR-Foundation:i432-capacity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #460 measurement scope (audit + instrumentation; production defaults unchanged pending the scaled re-run).

Traces why a 4.6x larger corpus produced FEWER cover regions (40 -> 8, zero split candidates accepted) and a worse model (Rule 1+2 36.5% -> 26.1%): the split test is an **absolute** entropy-gain floor evaluated on plug-in MLE entropy, whose bias term falls as 1/S — the bar is constant in N, the measured gain is not. Arithmetic, code cites, and per-knob saturation table in `docs/capacity_scaling_432.md`.

Also documents that `RVQ_SAMPLE_CAP = 10_000` silently capped codebook training all along, so **#407's CTX_SAMPLE 6k->50k raise never reached the codebook** — the 500k baseline was itself under-fit.

Ships `capacity_scaling.rs` (ignored): prints regions/depth, records per occupied full key, key-occupancy histograms, FWDA/NGRAM truncation rates and a SATURATION verdict per structure, for any corpus. Measured on both corpora; verdicts at 2.11M: cover observations-per-region SATURATED, entropy-floor rejects 1.0000 SATURATED, codebook sample fraction SATURATED, EXCT supported fraction SATURATED.

**Consequence: the #399 at-scale forward-anchor negative was measured in a degenerate regime — falsifier 2 is INCONCLUSIVE pending the capacity-scaled re-run.**

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

