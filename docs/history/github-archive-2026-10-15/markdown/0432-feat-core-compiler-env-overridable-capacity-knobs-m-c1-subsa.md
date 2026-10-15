# #432 — feat(core,compiler): env-overridable capacity knobs + M-C1 subsample control (capacity-starvation hypothesis)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T14:37:22Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-08-05T15:25:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/432
- **Merged:** 2026-08-05T15:25:43Z
- **Merge commit:** c72b61f99a3d4425803dcb8be85e230f31d09101
- **Base / Head:** main / UOR-Foundation:i399-capacity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399 #393. The falsifier-2 scale run collapsed the whole substrate (Rule 1+2 26.1% vs 36.5%; cover 15 nodes vs 39; harness store 0.3%; M-R2 negative) — diagnosis (Casey's hypothesis, confirmed in code): every capacity knob is a fixed constant while data grew 4.2×. Root causes located: cover split-acceptance uses an ABSOLUTE 0.25-bit entropy-gain floor that broad heterogeneous regions cannot clear (the cover anti-scales); real RVQ k-means cap is 10k samples (the '100k' in logs is a progress-bar total).

This PR: every capacity knob env-overridable (R4_CTX_SAMPLE, R4_RVQ_SAMPLE_CAP, R4_COVER_{K0,DEPTHS,REGIONS_BUDGET,MEMORY_BUDGET_MB,MIN_SUPPORT,ENTROPY_GAIN_BITS}, R4_{CONTEXT_ENTRIES,EXCT_TOP_X,ROOT_TOP_B,EMISSION_ENTRIES,FWDA_ENTRY_CAP,TRANSITION_OUT_DEGREE}) — **all default to the pinned constants; κ-neutral when unset, verified: kappa_reproduction passes bit-identical (canonical mode, 93.7s)**. Invalid values panic loudly. Plus scripts/mc1_subsample_corpus.py (499,985-record story-boundary subsample of the 10k corpus) and unit tests.

M-C1 (subsample control, defaults) and M-C2 (full corpus, ratio-scaled knobs) measurements are running; results follow on #399/#393.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

