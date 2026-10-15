# #717 — ci: isolate Kani from native matmul dependencies

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T21:18:48Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-08-14T21:39:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/717
- **Merged:** 2026-08-14T21:39:48Z
- **Merge commit:** 4b1513ebd206b2a9c5f5ce1c844f7abc6bf415c2
- **Base / Head:** main / UOR-Foundation:issue-712-kani-minimal-deps
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- split `uor-r4-proof-model` into a default-on full executable-spec surface and a minimal Kani surface
- keep only `uor-r4-graph-format` and `uor-r4-graph-runtime` in the Kani dependency graph
- fail CI if compiler, certifier, model-source, core, or `uor-matmul` re-enters that graph
- make Kani select only the library target with default features disabled
- make the ScoreQ proof exercise `ScoreQ::saturating_add` itself and assert its semantics

## Root cause

The proof crate unconditionally depended on `uor-r4-core`, which now reaches `uor-r4-model-source -> uor-matmul-kernels`. Kani rejects that crate's global assembly before either proof harness runs. Suppressing global assembly would weaken the verification result, even though neither harness needs the teacher/compiler stack.

## Impact

Normal builds and proof-matrix tests retain the complete API through the default `full-model` feature. Kani now compiles only the integer format/runtime surfaces used by its two harnesses and does not ignore unsupported assembly.

## Validation

- `cargo kani -p uor-r4-proof-model --no-default-features --lib` — 2/2 harnesses verified; runtime-state checks and the ScoreQ semantic assertion pass
- `cargo test -p uor-r4-proof-model --offline` — 25 tests pass
- `cargo test -p uor-r4-proof-model --no-default-features --offline` — minimal library profile passes
- default and minimal-profile package clippy with `-D warnings` — pass
- both graph-format no_std ladder commands — pass
- `cargo fmt --check`, claim-wording gate, and `git diff --check` — pass
- broader Apple-aarch64 workspace test/clippy reach pre-existing stale `dot_neon` test code and a fixture-enabled one-second observation assumption; both are recorded on #704 and are outside this diff

Closes #712

Generated with Codex


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

