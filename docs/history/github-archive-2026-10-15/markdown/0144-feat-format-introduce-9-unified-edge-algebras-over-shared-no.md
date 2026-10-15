# #144 — feat(format): introduce 9 unified edge algebras over shared node space (#125)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T14:34:50Z
- **Updated:** 2026-07-24T14:58:55Z
- **Closed:** 2026-07-24T14:58:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/144
- **Merged:** not merged
- **Merge commit:** 88cbb34bc751056e69bf82b1f295c73993f24784
- **Base / Head:** main / UOR-Foundation:issue-125-shared-node-edge-algebras
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #125. Implementation follows PDF §§4, 9 and `docs/formal_vocabulary.md` §3.

## Key Changes
1. **Edge Algebra Discriminants**: Implemented `EdgeKind` (`u8` discriminants 0..8) covering Semantic, Causal, Temporal, Constraint, GoalProgress, Evidence, Refinement, Forward, and Reverse edge algebras.
2. **Zero-Allocation Packed Layout**: Created 16-byte `PackedMultiEdge` representation compatible with `no_std` R4G1 binary container formats.
3. **Causal DAG & Graph Validation**: Implemented Kahn's topological sorting algorithm on Causal edges to reject causal cycles (`FormatValidationError::CausalCycleDetected`), check dangling node references, and prevent double counting via contribution IDs.
4. **Cucumber BDD Suite**: Added `features/suites/multi_edge_algebras.feature` with 4 scenarios and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 29 BDD scenarios passed (91 steps)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean (`no_std` verified)
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5071020822 — 2026-07-24T14:34:55Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).


---
_Comment/review ids in this file: 5071020822_

