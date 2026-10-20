# #143 — feat(compiler): introduce reference semantic state space model and typed graph dynamics (#124)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T14:30:10Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-24T15:33:00Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/143
- **Merged:** 2026-07-24T15:33:00Z
- **Merge commit:** c05e77308ee7692a632bf74b06059842e47c0dec
- **Base / Head:** main / UOR-Foundation:issue-124-semantic-state-model
- **Reviews / inline review comments:** 3 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #124. Implementation follows PDF §§2, 3, 12 and `docs/formal_vocabulary.md` §3.

## Key Changes
1. **Semantic State Model**: Implemented `SemanticState` ($S$) representing continuous latent space projections and Boolean bit signatures, `Region` hyper-ball bounds, `Belief`, `Goal`, and `Constraint` objects.
2. **Typed Transition Evaluator**: Implemented $T: S \times A \to S$ evaluator checking preconditions, postconditions, and invariant constraint violations.
3. **Bounded Trajectories**: Implemented step-limited trajectory execution and deterministic replay.
4. **Cucumber BDD Suite**: Added `features/suites/semantic_state_space.feature` with 5 scenarios and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 30 BDD scenarios passed (96 steps)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (8 entries; chronological: conversation comments, reviews, review comments)

### comment 5070970413 — 2026-07-24T14:30:16Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review 4774395644 — 2026-07-24T15:04:20Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Introduces a reference “semantic state space” model and typed transition evaluator in `uor-r4-graph-compiler`, plus BDD coverage to exercise transitions, constraints, goals/beliefs, and bounded trajectories in a deterministic way (per Issue #124).

**Changes:**
- Added `semantic_state` module with `SemanticState`, `Region`, `Belief`, `Goal`, `Constraint`, `Action`, `TransitionEvaluator`, and `Trajectory`.
- Exposed the new module from `uor-r4-graph-compiler`’s `lib.rs`.
- Added a Cucumber feature suite and step definitions to validate core state/dynamics behaviors.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 2 comments.

| File | Description |
| ---- | ----------- |
| `crates/uor-r4-graph-compiler/src/semantic_state.rs` | New reference semantic state/dynamics implementation with focused error types and unit tests. |
| `crates/uor-r4-graph-compiler/src/lib.rs` | Exports the new `semantic_state` module. |
| `features/suites/semantic_state_space.feature` | Adds BDD scenarios describing expected semantics of transitions/constraints/goals/bounded trajectories. |
| `tests/bdd.rs` | Implements step definitions driving the new semantic state/dynamics types. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3646300908 — 2026-07-24T15:04:20Z — Copilot

_on crates/uor-r4-graph-compiler/src/semantic_state.rs:376_

`TransitionEvaluator::apply` validates `state.vector` vs `action.delta_vector`, but it doesn't validate `state.boolean_signature` vs `action.mask_flip`. Because the signature update uses `zip`, a length mismatch will silently truncate to the shorter length and produce an incorrect signature without an error. Add a second dimension check before computing `new_sig`.

### review-comment 3646300940 — 2026-07-24T15:04:20Z — Copilot

_on features/suites/semantic_state_space.feature:3_

The feature description claims the dynamics/evaluator performs "belief updates", but the implementation in this PR provides belief likelihood evaluation (`Belief::evaluate`) and doesn't implement any belief-update operation as part of transitions. This wording makes the spec misleading relative to the code under test; either implement belief update semantics or adjust the feature text to match what’s actually exercised.

### review 4774530682 — 2026-07-24T15:21:59Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3646405150 — 2026-07-24T15:21:59Z — Copilot

_on crates/uor-r4-graph-compiler/src/semantic_state.rs:376 · in reply to review-comment 3646300908_

Added a length check for `state.boolean_signature` vs `action.mask_flip` before computing `new_sig`, placed alongside the existing vector check to fail fast (bd commit).

### review 4774531239 — 2026-07-24T15:22:03Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3646405608 — 2026-07-24T15:22:03Z — Copilot

_on features/suites/semantic_state_space.feature:3 · in reply to review-comment 3646300940_

Updated the feature description to say "belief likelihoods" instead of "belief updates" to match the `Belief::evaluate` implementation exercised by the tests.


---
_Comment/review ids in this file: 5070970413, 4774395644, 4774530682, 4774531239, 3646300908, 3646300940, 3646405150, 3646405608_

