# #296 — feat(compiler): #243 Phase C — 1-term TLA6 default emission + κ re-pin (era note)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T04:17:53Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-07-30T04:53:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/296
- **Merged:** 2026-07-30T04:53:09Z
- **Merge commit:** f6314aa5ad638ac9d60e27d646a4fa8d179b03f9
- **Base / Head:** main / UOR-Foundation:issue-243-phase-c
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the Phase C maintainer decision (Casey, 2026-07-29, recorded on #243): default emission = **1-term TLA6**, single κ re-pin now, TLA5 era note; #265 explicitly not bundled (re-pins in its own era).

- `DOT_TERMS = 1`: one power-of-two term per dot-table entry; u16 slot unchanged, decode skips zero-flag terms, Phase B two-term artifacts decode bit-exactly.
- TLA6 is the default emission (`R4_TLESS_TLA6=0` opts back to TLA5 for era-comparison runs). Loaded TLA5 artifacts keep the sign-Hamming path.
- κ re-pin, 3rd era (discipline per the fixture's own precedent): **only the container pin moves** (462,092 → 1,051,916 bytes; `blake3:81db4406…` → `blake3:cf64fe74…`) — all non-container κs byte-identical to the TLA5 era, verified by the re-pin dump. Fixture container regenerated; PROOF.md P5(a) era note.

## Decision-row evidence (census-verified fresh binary)

| | 2-term (Phase B, shipped) | **1-term (this PR)** |
|---|---|---|
| A row (shipped path) | 30.7 / 34.3 | 30.5 / 34.0 |
| WB bits/token | 9.0242 | **9.0107** |
| store keys | 47,476 | 48,321 |
| op census add/shift | 612,766 / 612,414 | **317,850 / 317,502** |

Accuracy holds within row noise with WB slightly better; the dot-scan op budget halves, resolving the ⚑ 13× flag from PR #291 (now ~6.8× vs sign path, with stage early-exit and #277 VP-tree still open as further reductions). First run of this evidence was discarded as a stale-binary artifact (census unchanged at 612k = two-term); the shipped numbers come from a rebuilt binary with a census freshness assert in the run script.

Remaining Phase C scope (follow-ups on #243): serving/score signature-path adoption with its own evidence run, BASELINE table; closes #230 on completion.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

