# #119 — test(quantum): BDD suite confirming density operator and cover criterion (#108)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T18:51:13Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-07-23T19:27:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/119
- **Merged:** 2026-07-23T19:27:16Z
- **Merge commit:** da6565ebc28bf23697631ec56f1bafec1a847a0f
- **Base / Head:** main / UOR-Foundation:issue-108-quantum-cover-bdd
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #108

## What

Behavior-level confirmation of the quantum cover work: new cucumber suite `features/suites/quantum_cover.feature` (+ step defs in `tests/bdd.rs`) covering the issue's acceptance criteria as externally visible behavior:

- maximum-entropy operator in dimension 8 attains S = ln 8 (the analytical bound)
- a pure distribution carries zero entropy
- an aligned split of observations whose halves predict disjoint tokens gains exactly ln 2 nats and is **accepted** by the default criterion
- the interleaved (noise) split gains ~0 and is **rejected**

## Verification

- `cargo test --test bdd`: 4 features, 20/20 scenarios, 58/58 steps
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
- `cargo fmt --check`: clean
- `cargo test --workspace --offline`: green


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

