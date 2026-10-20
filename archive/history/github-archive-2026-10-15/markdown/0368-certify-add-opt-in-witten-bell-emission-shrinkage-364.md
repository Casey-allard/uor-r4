# #368 — certify: add opt-in Witten-Bell emission shrinkage (#364)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T04:38:54Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-03T04:53:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/368
- **Merged:** 2026-08-03T04:53:26Z
- **Merge commit:** 0aa005a9ddfbe36c1f1bdcc36ddac003d811f124
- **Base / Head:** main / UOR-Foundation:issue-364-emission-shrinkage
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements the next measurement-backed step for #364: an opt-in Witten–Bell shrinkage policy for region emission residuals.

## Change

- Adds `--emission-shrinkage none|witten-bell` to the scoring CLI.
- Keeps `none` as the default, preserving the shipped artifact path and runtime/ABI.
- `witten-bell` scales each region residual by `λ = n / (n + T)`, where `n` is region evidence and `T` is distinct observed token types.
- Adds bounded-weight, CLI parsing, and default-preservation tests.
- Compiler-side only; no deployed-kernel operations or format changes.

## Fixture ablation

Probability-ranked emissions, same fixture and Gate C slice:

| metric | shrinkage off | Witten–Bell |
| --- | ---: | ---: |
| graph chain+EXCT top-1 | 36.6% | 36.6% |
| graph chain+EXCT bits/token | 7.7933 | **7.7927** |
| mean λ | — | 0.9992 |
| mean region evidence | — | 3.1M counts / 2,198 types |

The effect is intentionally small because this corpus has very high region evidence. The policy is retained as an opt-in calibration control for genuinely sparse regions; it is not promoted to the default based on this result.

## Verification

- Full workspace tests: green
- Full workspace clippy with `-D warnings`: green
- `cargo fmt --check`: green
- graph-format no-std and alloc ladders: green
- Focused certifier, CLI, and runtime suites: green

Related: #364, #367, #362.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

