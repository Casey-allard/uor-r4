# #127 — Make predictive entropy and information bottleneck first-class compiler objectives

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:45Z
- **Updated:** 2026-07-24T17:02:09Z
- **Closed:** 2026-07-24T17:02:09Z
- **Labels:** graph-compiler
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/127
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Extends the predictive-sufficiency work in #24.
Source: PDF §§6 and 8.

Define the offline compiler objective around future predictive utility and compact reusable state, rather than geometric clustering alone.

## Scope

- Add objective terms for predictive entropy `H(A|R)` and/or future-state entropy `H(S_future|R)`.
- Specify an information-bottleneck-inspired objective `I(Z;X) - beta I(Z;Y_future)`, including practical estimators and declared approximations.
- Combine teacher loss, runtime cost, artifact size, bytes read, and structural complexity into a versioned objective configuration.
- Define split-safe estimation: objective fitting uses training data and all reports use held-out data.
- Emit objective components and tradeoffs in compile reports so region split/merge/remove decisions are auditable.

## Acceptance criteria

- Compiler-side types represent objective configuration, component values, weights, and estimator metadata without leaking floats into the deployed runtime.
- A deterministic mini-corpus test verifies objective decomposition and stable tie-breaking for region decisions.
- Reports show entropy, teacher loss, artifact cost, and information-retention proxies separately; no single composite score hides regressions.
- The existing predictive-sufficiency and Gate C reports remain reproducible, with a documented migration path for objective version changes.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

