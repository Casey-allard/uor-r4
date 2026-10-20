# #1024 — feat: add and bound R4 source-span pointer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T21:26:15Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-08-31T21:27:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1024
- **Merged:** 2026-08-31T21:27:27Z
- **Merge commit:** be500ed51caf63dc5b6bcfed14e60411e9f8c5d0
- **Base / Head:** main / UOR-Foundation:issue-954-source-span-pointer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #954

## Outcome

Implements the frozen `R4SourceSpanPointerV1` research seam over the established #1017 causal R4/Spin states, records the sole bounded run, and fails closed before product wiring.

- Adds exact final normalized residual capture through all six R4/Spin layers.
- Adds strict canonical pointer-head loading, candidate-relative weighted-cosine scoring, explicit answer/abstain/conflict decisions, deterministic source copying, and a compact public parity report.
- Adds the two-stage frozen Python trainer/data contract and focused tests.
- Updates the CLI and project documentation to state that no qualified final head exists.
- Records the terminal `FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`; no retry, product probes, browser wiring, or release claim.

## Evidence

- 12/12 overfit preflight: PASS
- Python/Rust parity: max score delta `1.234420776e-7`; max logit delta `1.428717041e-6`; tolerance `0.01`
- Sole full fit: answer `89/128`, abstain `114/128`, conflict `117/128`, supported pointer `121/128`; all frozen gates required at least 95%
- Final artifact: not emitted
- Product probes: `NOT_RUN`

## Focused verification

- `cargo fmt --all -- --check`
- `cargo check -p uor-r4-wasm-router --offline`
- `cargo clippy -p uor-r4-wasm-router --lib --offline -- -D warnings`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- focused Rust policy-boundary test
- 11 focused Python tests
- `python3 scripts/check_claim_wording.py`
- JSON and diff validation


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

