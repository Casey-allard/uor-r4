# #1116 — #973: Context-bind retained keys and values

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T14:56:15Z
- **Updated:** 2026-09-04T14:59:59Z
- **Closed:** 2026-09-04T14:57:38Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1116
- **Merged:** 2026-09-04T14:57:38Z
- **Merge commit:** 4d3b720bdd4fb443b470dd0d056b34017ca23011
- **Base / Head:** main / UOR-Foundation:codex/973-context-key-value
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Problem and result

The delivered contextual-value path stored prompt-conditioned values under token-local keys. This adds a versioned `R4ContextualKeyValueWriteLanguagePathV1` recurrence in which the current token first reads the transported strict-prior state, then forms `c_t = RMSNorm(x_t + retained)` and writes both `Wk(c_t)` and `Wv(c_t)` through the existing identity slot.

The existing value-only path and its artifacts remain unchanged. The successor preserves the token-local read query, exact-H4 transport, read-before-write causality, rho/eta gates, occupancy, recurrent-state shapes, 252,160 learned parameters, and tied vocabulary head. Generator loading now selects fitted semantics only from an exact allowlist bound to the adjacent fit schema, completion status, policy, formulas, artifact CID, and geometry CID. Metadata-free loading is restricted to the canonical pinned V1 artifact.

## One bounded comparison

The fixed `fit-contextual-key-value` command warm-started from the qualified V1 artifact and completed exactly once:

- 128 updates, batch 16
- 2,048 open training windows / 245,760 causal targets
- four CPU threads
- 90.369271 seconds under the 840-second wall
- 24/24 parameter tensors with finite nonzero first-step gradients
- all 252,160 parameter values covered by the optimizer
- artifact: `blake3:1bc5bb5fc2daae7d67d3b7e748a583ca00310245c4289dfb20c1655bf24c6050`
- validation, sealed, teacher, and source-model file reads: zero

The two declared 16-token direct generations ran once each:

- `A purple turtle found a clock in the garden` → `.\n"Thank you you, "Yes, I is there happened. He`
- `Albert Einstein was born in` → ` the dog. He said, "Hi, I are I were pretty."\n`

This bounded result is negative: contextualizing both stored keys and values did not recover factual/entity binding or coherent continuation. It does not erase the prior value-only negative and does not justify another fit, retry, or broader geometry change.

## Focused verification

- `python3 -m py_compile` on the five modified Python modules
- `git diff --check`
- independent read-only review of causal ordering, artifact compatibility, fit/CLI bounds, and loader binding; its fail-closed metadata findings were fixed before the run

No broad QA, proof work, replay campaign, validation reveal, or additional model run was performed for this decision.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

