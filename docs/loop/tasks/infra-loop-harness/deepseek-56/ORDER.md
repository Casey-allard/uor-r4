# ORDER: infra-loop-harness, deepseek-56 (banned-presence pr-check rule)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (tooling). Base: `3df3798e4`. `References #2904`.

## Goal

Close the PM-accepted harness-gap card from #2028 (Claude 16:28Z, PM ruling 16:37Z): the D23/D25/D32 ratchets count banned source sites but cannot see them start compiling when a PR removes or weakens a `#[cfg(any())]` gate. Add a diff-level `loop pr-check` rule that compares the banned sites that compile per changed file between the merge base and the head, follows gated out-of-line modules, and fails when the head compiles more of any family.

## Cancel probe (run first)

Class instrument/tooling (host-only; no GPU). At the base, `crates/uor-r4-harness/src/loopprcheck.rs` has no `banned-presence` entry in `RULES`, and PR #3079's old heads `1ddfa1cb` / `54de128d` pass pr-check while 19 D23 sites change from gated to compiled. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopprcheck.rs` (shared registration file: always allowed), `crates/uor-r4-harness/tests/loop_prcheck.rs` (item scope), `docs/harness/README.md` (shared registration file), this task folder.

## Done when

- `loop pr-check` runs a `banned-presence` rule: for each changed source file it counts D23 (word + identifier families with the ratchet's rope exception), D25 (`.matmul(`, `MatMul`, cublas, the word gemm, tensor-core intrinsics), and D32 (float types and literals, in the ratchet's scan scope) sites in that file's compiled code at the merge base and the head, and fails when the head's count rises for any family.
- Compiled code means a structure view (comments, plain strings, char literals and raw strings blanked) with every `#[cfg(any())]`-gated span removed: outer gates inline or before an item, gates on parent modules, inner `#![cfg(any())]` attributes, and gated out-of-line `mod NAME;` declarations followed into their module files transitively; `cfg(test)` is not an exemption.
- An item-start parser covers visibility (`pub(crate)`, `pub(super)`, `pub(in ...)`), modifiers, `use`, `extern crate`, `extern "C"`, macros and the usual items; finding the next item is precomputed and binary-searched.
- Planted fault: against PR #3079's violating heads `1ddfa1cb` / `54de128d` the rule FAILS (`relation_compiler.rs`: D23 0 -> 19) and against the re-gated head `8f36239a` it passes.
- Regression tests cover: un-gated banned item, `cfg(test)` move, rename, benign/`properties`/char-literal items, deletion, parent-module gate, D32 float forms, `__half`, `pub(crate)`, inner gate, comment gates, out-of-line module moves, raw-string braces, and D25 helper-name false positives.
- Docs updated; fmt/clippy/full harness suite/ratchets green; counted non-author review (with an owner-directed subagent-review fallback recorded in the PR).

## Stop and ask if

- The rule would need a charter change, an owner path, or a change outside the shared/loop files in scope.