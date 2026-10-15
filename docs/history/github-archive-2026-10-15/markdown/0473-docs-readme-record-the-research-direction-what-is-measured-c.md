# #473 — docs(readme): record the research direction — what is measured, closed, and open

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T05:11:33Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-07T05:36:38Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/473
- **Merged:** 2026-08-07T05:36:38Z
- **Merge commit:** 9a7feb8a49f1f61e357b449e196a506cc2b9a57c
- **Base / Head:** main / UOR-Foundation:readme-state
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Documentation only; no code or behaviour changes. Shipped as a PR because the `main` ruleset (required checks + merge queue) has no bypass actors and no path exemptions, so even a README edit has to go through the queue.

Adds a **Research direction** section to the README so the programme's state is legible without reading issue history:

- the measurement discipline (pre-declared exit rules, nulls, falsifiers, and the run-contract rules from AGENTS.md);
- **what works and is load-bearing** — the scorer order, the induced-cover store with observed evidence, de-banding (#434), the two-sided calibration gain (#446), and A-mode infill serving (#399);
- **what is closed and why**, each with its numbers — standalone two-pass generation (refuted twice, and again at scale), code-space subdivision (#460, negative in its strongest form: we bought the subdivision and the headline fell), stratification (#435, algebraic close), Hopf sector transport (#422/#306), and the earlier dead terms (#400, #290, #393, #395);
- **the pattern those results draw** — every lever that added key resolution failed; the only two that helped improved evidence quality per key. That is why the open work concentrates on evidence and estimation rather than subdivision;
- **a table of open issues with their defined remainder**, matching the level-set just applied to the board;
- the measurement infrastructure (sampled decision runs, the κ-keyed code sidecar, the saturation instrument) and the precision arithmetic that justifies sampling.

Claim-wording gate passes.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

