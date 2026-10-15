# #572 — feat(#510): R5 tranche — api engine InferenceError cluster to ObservedBound/total (api 13->2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T04:02:21Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T04:18:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/572
- **Merged:** 2026-08-10T04:18:59Z
- **Merge commit:** 8f7ff4e951edc0d412eeb9567d3833f51584aefd
- **Base / Head:** main / UOR-Foundation:r5/api-inference-observedbound
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): api engine `InferenceError` cluster → `ObservedBound` / total

Eliminates the composite `InferenceError` enum across the 11-fn serving path. Its
two variants split cleanly by R5 kind: `TokenOutOfVocabulary` is a **measured
input bound** → `ObservedBound`; `Scorer(String)` was a `None` from a validated
scorer scoring a live signature — a **self-produced defect** → panic.

The token-out-of-vocabulary boundary carries the only real error:
- `check_window` → `Result<(), ObservedBound>` (observed token id vs the
  `token_rows` bound)
- `signature_for_window` / `predict_decision` / `predict_decision_with_witness` /
  `predict_next_into` / `generate_into` / `generate_into_with_witness` →
  `Result<_, ObservedBound>` (they window-check their input)

The signature-based fns have only the scorer-defect path, so they become total:
- `score_sig` / `score_sig_witness` → **bare** (scorer `None` → `.expect`)
- `predict_signature_status` / `predict_signature_status_with_recent` → **bare**
  `PredictDecision`

`verify_witnesses` (Option, #571) drops its now-redundant `Ok`/`Err` match on the
now-total `score_sig_witness`. The `InferenceError` enum and its `Display`/`Error`
impls are removed and the crate re-export drops it.

### Ripple
The root package's `r4g1` `predict_signature_status` wrapper wraps the now-total
call in `Ok(...)`; its `predict_window_status` / `signature_for_window` /
`generate_into_status` wrappers keep `.map_err(to_string)` (`ObservedBound`
Displays `"observed X past bound Y"`). The status-policy out-of-vocabulary test
asserts the sanctioned `"past bound"` message. `serving_eval` keeps its own
`ObservedBound → ServingEvalError` mapping until its R5 conversion.

### Audit
api audit-limits: **13 → 2** (only `load`/`LoadError` and `serving_eval` remain).
No non-sanctioned surface added; `ObservedBound` is sanctioned.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (api,
all-targets), the api lib + integration suites, `status_policy` (+census), and the
full root-package `bdd` suite (100 scenarios / 333 steps) — all green. (api is not
a wasm target.)

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

