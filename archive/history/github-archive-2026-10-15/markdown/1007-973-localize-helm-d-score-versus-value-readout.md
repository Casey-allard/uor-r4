# #1007 — #973: localize HELM-D score versus value readout

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T05:22:56Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-30T05:24:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1007
- **Merged:** 2026-08-30T05:24:20Z
- **Merge commit:** df6e2dcba0222937ff76d594c5c64a596d987855
- **Base / Head:** main / UOR-Foundation:issue-973-helm-d-score-readout-localization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements the frozen `HelmDScoreCentroidLocalizationR4V1` successor from the protected learned-manifold V2 negative.

- separates Lorentz/Euclidean scoring from normalized-Lorentz/tangent value readout;
- freezes the disjoint 8-fit/8-audit construction-only population and locally held target commitment;
- adds the two-document identity preflight and the admitted four-arm 8/8 localization decision;
- preserves D3 as `NOT_RUN` and leaves resonance, recurrence, lowering, scale, generation, correctness, reasoning, and #954 blocked;
- adds the protected-revision runner with exclusive evidence paths and an 80-minute decision watchdog.

The decision run is intentionally `NOT_RUN` until this implementation is protected on `main`.

## Verification

- `cargo test --workspace --offline --no-fail-fast`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- focused HELM-D core and integration tests: 10/10 and 8/8, with 4 decision/freezer tests intentionally ignored
- `cargo fmt --check`
- graph-format no-default and alloc builds
- wasm32 library build
- claim-wording, diff, runner syntax, and executable-mode checks
- independent read-only contract and implementation review: no remaining P0/P1 blocker

References #973.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

