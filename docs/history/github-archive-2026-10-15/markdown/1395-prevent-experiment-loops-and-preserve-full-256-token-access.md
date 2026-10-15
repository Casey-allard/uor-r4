# #1395 — Prevent experiment loops and preserve full 256-token access

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T18:45:26Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-09-25T18:46:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1395
- **Merged:** 2026-09-25T18:46:34Z
- **Merge commit:** 37b244cd5093a85fa23d5e93b466faa1d3cfe00f
- **Base / Head:** main / UOR-Foundation:codex/progress-control-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Owner-directed correction

The owner challenged the 256/256 configuration and required a system to prevent repeated narrow experiments and test-fixing loops. Training, evaluation and session contexts remained 256, but orthant64 restricted reads to at most 64 selected events. That admission experiment failed source retention; neither candidate was promoted.

## Changes

- Add mandatory progress control to root AGENTS, the machine/readable execution policy, the research skill and DeepSeek workflow: one integrated deliverable, an observed causal blocker, fixed context/access conditions, distinct decisions, necessary checks and whole-cycle costs.
- Require new causal evidence and a decision-changing mechanism before repeating model compute. Park a sound failed branch without that evidence. End validation when its declared risks are resolved; defer unrelated failures. Preserve frozen acceptance and all negative evidence.
- Record D9, withdraw the unexecuted recent64 fit, and preserve accepted full-256 parents. Quantized transport and the integer execution bridge are the next integrated implementation. Candidate pruning is a deferred optimization; full fixed-window access does not claim terminal sparsity.
- Reconcile active roadmap/front doors and preserve the original recommendation explicitly as superseded history. The rules govern agents; they are not an OS or direct-CLI interlock.

## Validation

- `git diff --check`: pass.
- `python3 scripts/check_claim_wording.py`: pass.
- Machine policy JSON and research issue-form YAML parse: pass.
- Reviewed the changed docs/policy and actual saved model settings (B16/T256, evaluation/context256, read width64, state width256, admission orthant64).
- RDC validation process exited 0 in 0.54s. No model training, cargo build or Rust test campaign for this documentation/policy correction. No Rust source or model artifact changed.

References #973 and #820. Protected delivery; historical queue statuses remain compatibility acknowledgements, not executed model tests.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

