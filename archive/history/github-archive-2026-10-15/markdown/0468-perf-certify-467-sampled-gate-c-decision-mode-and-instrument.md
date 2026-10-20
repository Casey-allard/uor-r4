# #468 — perf(certify): #467 — sampled Gate C decision mode and instrument fast path

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-06T18:47:18Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-07T00:17:44Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/468
- **Merged:** 2026-08-07T00:17:44Z
- **Merge commit:** 12083d2e2f2cc1a03cc3a53909e4ed7452c8a75b
- **Base / Head:** main / UOR-Foundation:i467-fastpaths
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #467. Decision runs should not pay census prices: at 402,802 held-out positions the headline's standard error is 0.07pp while every pre-declared exit rule is written at ±2pp.

**Sampled Gate C** (`R4_GATE_C_SAMPLE`, unset = full pass): deterministic stride selection, no RNG. Every rate now carries n and sqrt(p(1-p)/n) so a sampled number can never be quoted as a census. Measured: 10,000 positions gives 35.83% ± 0.48pp against the census 36.55% ± 0.15pp, with Gate C evaluation dropping **597s → 60s**. **Bit-identity with the knob unset was verified end-to-end, not argued** — `r4` built from origin/main and from this branch, both run on the fixture corpus, reports diffed field by field: the only differences in the entire document are `schema` 24→25 and the new fields.

**Instrument** (`R4_CAPACITY_SAMPLE`, default 100k for the one sampled rate; `R4_CAPACITY_TRAIN_SAMPLE`, default census/off): all seven SATURATION verdicts match the full pass, and every exact quantity is identical to the digit.

**The correctness point, and it matters more than the speedup:** only ONE quantity is estimated — `exct.supported_record_fraction`, a mean of a per-record indicator, printed with its standard error. The construction-split statistics are deliberately NOT sampled, because distinct occupied keys is not a mean: a key holding r records survives a rate-f subsample with probability 1-(1-f)^r, and 42% of this corpus's keys hold exactly one record, so a subsample undercounts and dividing by f does not repair it. Forced on via the opt-in knob, **two of seven verdicts flip** (keys 12,193 vs 47,403; records/key 8.20 vs 36.02) — so that mode labels every affected line `BIASED ESTIMATE`.

**Honest miss on the target:** ~10s is not reachable from the instrument side. The residual 468s is the exact code pass — 4 stages × 256 classes × 288 dims of branchy shift-add per record, ~5e11 term applications, already saturating both available cores. That needs a vectorized kernel in `uor-r4-core`, not a smarter instrument. Also confirmed while looking: the compile does **not** persist per-record codes anywhere reusable (only token codes in `hierarchical_codes.json`), so there is no cache to read — filed as follow-up work rather than invented here.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

