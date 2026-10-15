# #76 — Canonical bits/token definition across harnesses

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:35Z
- **Updated:** 2026-07-23T04:10:02Z
- **Closed:** 2026-07-23T03:30:47Z
- **Labels:** graph-compiler
- **Milestone:** Phase 5: Allocation-free graph runtime
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/76
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: none. The repo currently reports bits/token in at least two accountings: the P2 certificate's 6.54 (WB metric) and the Gate C scorer's 11.88 — different metric families that will confuse the M.V.G. checkpoint. Pick one canonical definition (documented formula, nats→bits conversion, which distributions it covers), implement it in the Gate C harness and the certificate path, and annotate the historical values as their own families. Acceptance: one formula in GLOSSARY.md + all future reports cite it; BASELINE.md updated.


---

## Definition of Done
- One canonical bits/token definition (formula, nats→bits conversion, covered distributions) is documented in `docs/transformerless/GLOSSARY.md`.
- The Gate C harness and the certificate path both emit the canonical metric; historical values are annotated as their own families.

## Acceptance criteria
- [ ] The formula appears verbatim in GLOSSARY.md and is referenced by every subsequent report.
- [ ] score_report.json and the certificate schema carry the canonical field name.
- [ ] BASELINE.md marks 6.54 (P2 WB) and 11.88 (scorer) as their historical families with the mapping note.

## Comments (1)

### comment 5053990538 — 2026-07-23T03:30:46Z — auser

Complete. DoD status: (1) canonical definition documented in docs/transformerless/GLOSSARY.md — mean cross-entropy of the true next token under the scorer's predicted distribution, (1/N) Σ −log2 P_scorer(v_i|c_i) with floor mass included, implemented in score.rs::outcome_bits (Gate C harness). (2) Comparability rule explicit: values comparable only within same scorer AND same distribution; the historical 'families' annotated as scorer/distribution differences in BASELINE.md (6.54 = P2 WB legacy corpus, 11.88 = Gate C baseline fixture, 9.86 = Rule 1+2 fixture), with the requirement that reports name scorer + distribution. (3) Field name verified consistent: bits_per_token across GateCMetrics, per-status metrics, and score_report.json schema; the certificate path references external metric CIDs per its schema (no divergent local field).


---
_Comment/review ids in this file: 5053990538_

