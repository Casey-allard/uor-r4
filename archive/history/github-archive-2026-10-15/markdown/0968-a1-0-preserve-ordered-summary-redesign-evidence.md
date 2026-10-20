# #968 — A1.0: preserve ordered-summary redesign evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T05:25:10Z
- **Updated:** 2026-10-09T14:17:20Z
- **Closed:** 2026-08-27T05:26:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/968
- **Merged:** 2026-08-27T05:26:20Z
- **Merge commit:** 9a04fe9705a16ba1de87566da9f17181e346f4d7
- **Base / Head:** main / UOR-Foundation:issue-952-recursive-geometric-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Executes the publicly frozen #952 A1.0 ordered-state/value-reachability gate and preserves its predeclared terminal result:

`REDESIGN_ORDERED_ROUTE_SUMMARY`

The recursive attention scorer is **not implemented**. This PR does not claim attention, generation, correctness, reasoning, or product readiness.

## Evidence

- 3/3 matched earlier-order contrasts collide across all seven hierarchy levels.
- 46/46 exposed non-digest fields are censused at each level, including plain boundary and chain identities: 966/966 comparisons are equal.
- Exact identities differ 3/3, but digest distance is not used as semantic geometry.
- The real schema-2 path naturally admits both `ll` and `rr`: union size 2, ceiling 8, no exact-row leakage, target injection, truncation, or future events.
- 12/12 admitted-candidate addresses invert to exact payloads; 6/6 incremental next states reproduce.
- Attention-selected output is `NOT_RUN`; no scorer exists.
- Source-model weights, teacher, transformer, MoE, learned-router, dense-matrix, Ollama, and hosted-provider counts are all zero.

Independent report kappa:

`blake3:23e07a17e897abb701c09354d35a3113a1b075ba1d60ee634067fa3ed3fd1904`

## Implementation

- Adds the fixed A1.0 probe and `recursive-attention-a1-probe` CLI surface.
- Adds the exact selected/admitted-address-to-payload value view needed for the later #953 handoff.
- Exhaustively closes the concrete scaled 120-root H4 table under exact binary-icosahedral multiplication: 14,400 products, 120 inverses, 1,728,000 associativity checks, no rounding.
- Compiles the existing 64-state SpiralCore group into a bounded exact composition/inverse table: 4,096 products and 262,144 associativity checks. It remains an optional control with no semantic claim.
- Makes the predeclared decision mapping fail closed by separating ordered-state collision from candidate absence, admission truncation, and invalid leakage/support conditions.
- Adds the omitted #961/#952 core modules to the exact local source-closure contract.
- Reconciles current programme/lifecycle/research/ADR status and records narrow successor #967. The two additional rustfmt hunks are mechanical pinned-formatter reconciliation only.

## Focused verification

- `cargo test -p uor-r4-core --test recursive_geometric_attention_952 --offline -- --nocapture` — 2 passed, 0 failed.
- `cargo test -p uor-r4-model-source --lib exact_executor_contract_binds_complete_local_source_closure --offline` — 1 passed, 0 failed.
- `cargo check -p uor-r4-core --lib --offline` — passed.
- `cargo check --bin r4 --offline` — passed.
- `cargo fmt --check` — passed.
- `python3 scripts/check_claim_wording.py` — passed.
- `git diff --check` — passed.

The protected queue is transport only; its acknowledgement statuses are not cited as tests.

## Successor state

#967 preserves the working candidate/value path, replaces only the order-erasing reusable summary with an associative noncommutative fold, and reruns this frozen gate before any scorer work. #953 is natively blocked by #967 and cannot become eligible when #952 closes.

Closes #952


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

