# #43 — feat: compile forward transitions and reverse indexes from teacher corpus (closes #18)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T01:11:50Z
- **Updated:** 2026-07-22T01:23:52Z
- **Closed:** 2026-07-22T01:23:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/43
- **Merged:** 2026-07-22T01:23:12Z
- **Merge commit:** fcc4dc2e61ccf339dd3f03088c6a24ea7252edeb
- **Base / Head:** main / UOR-Foundation:feature/issue-18-compile-transitions-and-reverse-index
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #18: Compiles forward region transition edges (E_f) and reverse edge indexes (E_b) from teacher corpus stream, enforcing Theorem 7 consistency by construction.

## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### comment 5040718199 — 2026-07-22T01:11:53Z — Casey-allard

@github-copilot review

### comment 5040718348 — 2026-07-22T01:11:54Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3626756668 — 2026-07-22T01:19:27Z — Copilot

_on crates/uor-r4-core/src/transformerless/transitions.rs:162_

`transitions_by_src` is a `HashMap`, so iterating `for (src, mut dsts) in transitions_by_src` makes canonical edge ID assignment depend on randomized hash iteration order. That makes edge IDs (and thus `reverse_index`) non-deterministic across runs, which is risky for reproducible compilation artifacts.

### review 4750196854 — 2026-07-22T01:19:28Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds a new `transformerless::transitions` module for compiling forward region-transition edges and constructing a reverse edge index intended to satisfy Theorem 7, along with integration tests and module wiring.

**Changes:**
- Introduces `TransitionGraph` with forward edges and a reverse index (`reverse_index` + `reverse_offsets`) plus a Theorem 7 verifier.
- Implements `compile_transitions_from_corpus` to aggregate per-(src,dst) transition weights from the teacher `Corpus`, apply a per-node cap, and build the reverse index.
- Adds integration tests covering manual graph construction, synthetic corpus compilation, and per-node transition bounding.

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 2 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/transitions.rs | New transition-graph data structures, reverse indexing, and corpus-based compilation. |
| crates/uor-r4-core/tests/transitions_test.rs | New tests validating reverse-index construction, Theorem 7 checks, and bounding behavior. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new `transitions` module on non-wasm targets. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626756682 — 2026-07-22T01:19:28Z — Copilot

_on crates/uor-r4-core/src/transformerless/transitions.rs:126_

`verify_theorem_7` currently only checks that offsets point to edges with the expected `dst`, but it does not actually verify the stated/specified property that `reverse_index` is sorted by destination (nor that `reverse_offsets` covers the whole `reverse_index`). This can return Ok even if `reverse_index` is unsorted or partially indexed.


---
_Comment/review ids in this file: 5040718199, 5040718348, 4750196854, 3626756668, 3626756682_

