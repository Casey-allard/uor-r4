# #947 — Compile and compose full-prefix trajectory routing regions (#946)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T22:25:32Z
- **Updated:** 2026-10-09T14:16:57Z
- **Closed:** 2026-08-25T22:55:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/947
- **Merged:** 2026-08-25T22:55:26Z
- **Merge commit:** 43edd2de1639651a66d94e75c717a0e6d827136b
- **Base / Head:** main / UOR-Foundation:issue-946-trajectory-routing
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #946

## Outcome

- compiles production-identical full-prefix TokenHistorySignature observations and explicit trajectory ROUT regions
- validates node lane flags and builds separate context/trajectory indexes at admission
- composes a deterministic allocation-free 4+4 active set under the existing ceiling of eight
- preserves legacy flag-zero and no-session behavior
- records the bounded exact #933 product result: trajectory routes reached all 20 fallback cases, but changed 0/512 candidate lists and 0/512 served tokens, so the truthful verdict is INERT and #841 remains NOT_RUN

## Focused verification

- cargo fmt --check
- cargo check for every touched package with all targets, offline
- focused graph compiler, format, runtime, certifier, CLI, and API tests
- graph runtime no-default-features check
- WASM boundary check
- teacher-free 4,096-position equal-budget preflight: PREFLIGHT_POSITIVE
- exact schema-2 production admission and 72,130-position deployed-quality census: PASS
- exact 512-case product canary: INERT
- python3 scripts/check_claim_wording.py

BDD, teacher parity, frozen #841, fuzz, audit, Kani, and other certification suites were intentionally NOT_RUN locally per #946 and the repository fast-gate policy.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

