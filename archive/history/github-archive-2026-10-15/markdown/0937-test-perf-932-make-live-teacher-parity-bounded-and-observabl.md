# #937 — test/perf(#932): make live teacher parity bounded and observable

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T16:50:57Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-25T17:35:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/937
- **Merged:** 2026-08-25T17:35:19Z
- **Merge commit:** 7c37e2d6642d1975038253109b9d56b7907db23e
- **Base / Head:** main / UOR-Foundation:issue-932-exact-parity-multicore
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #932

## Outcome

Replaces the open-ended serial live-teacher parity path with bounded exact multicore execution, shared immutable weights, private stream state, canonical reductions, durable heartbeats/events, hard deadlines, and fail-closed fixture and report validation.

The current canonical local bundle is truthfully `UNAVAILABLE`, so the live tuner and full parity run remain `NOT_RUN`; no threshold was relaxed and no production-performance claim is made.

## Scope

- exact row-partition executor and adaptive W=4/W=8 tuner
- shared S2/S3/S7 transcript and bounded adaptive S4 continuation
- content-bound production-envelope and tokenizer validation
- durable JSONL progress plus machine-readable report/evidence schemas
- deterministic, negative-control, observability, and BDD coverage
- configuration, roadmap, research, baseline, and conformance documentation

No deployed serving behavior, runtime artifact format, multiplication-free kernel, or parity floor changes.

## Local verification

- `cargo test -p uor-r4-model-source --lib --offline`: 155 passed, 4 fixture-gated ignored
- `cargo test --test parity_determinism_932 --offline`: 4 passed
- `cargo test --test parity_observability_932 --offline`: 49 passed
- `cargo fmt --all -- --check`
- `python3 scripts/check_claim_wording.py`
- `cargo run -p xtask --offline -- check-model`: 33 ids
- graph-format no-std and alloc ladders
- wasm32 library check
- strict workspace clippy passed on the final source state

The protected merge queue is the binding full-workspace verification. The expensive local workspace rerun was intentionally not repeated after the focused gates because CI runs the complete speculative-merge ladder.

## Evidence

Current teacher-free preflight: `UNAVAILABLE`; missing canonical `release-bundle.json`; teacher source unopened; zero teacher forwards. Ordinary BDD evidence is structurally complete and records zero work rather than treating absent prerequisites as PASS. Exact CIDs and the superseding run contract are recorded in `docs/teacher_parity_parallelism_932.md`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

