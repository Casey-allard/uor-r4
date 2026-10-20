# #1870 — Land the hidden-versus-projected pre-projection dump that was never merged (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T02:48:03Z
- **Updated:** 2026-10-09T13:20:31Z
- **Closed:** 2026-10-08T02:50:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1870
- **Merged:** 2026-10-08T02:50:18Z
- **Merge commit:** 0dc1ac05c17f87ddebe222b7ea978e1f372db80f
- **Base / Head:** main / UOR-Foundation:deepseek/hidden-probe-land
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this lands

`deepseek/hidden-probe` was completed but never delivered. It captures the two **pre-projection normalized states** in the read-layer dump so the hidden-space and projected-space readouts are taken from one forward — identical rows, positions and candidate sets, differing only in the map. That is what makes the hidden-versus-projected comparison a controlled one.

- `crates/uor-r4-training/src/geometric_stack.rs` — `ReadQkLayer` gains `qk_input` (the exact normalized state the query/key projections were computed from, before key shift, lineage and latched identity) and `value_input` (the state the value and NoRead maps were computed from), cloned at the dump site. No new parameter, no new arithmetic.
- `crates/uor-r4-training/examples/mqar-bench.rs` — the stage-0 readout dumps both states as `layer{N}.qk_input.f32` / `layer{N}.value_input.f32` beside the existing query/key/aux dumps.

## Verification at the exact head

- `cargo check -p uor-r4-training --lib --example mqar-bench` — clean.
- `git rebase origin/main` applied without conflict; only the two files above differ from `main`.

## Note on an unrelated pre-existing break (not from this branch)

`cargo check -p uor-r4-training --all-targets` on current `main` fails in `crates/uor-r4-training/examples/geometric-native-compiler.rs:1086` with `E0433: too many leading super keywords` (`pub(super) struct ReferenceRule` at file scope of an example crate root). This branch does not touch that file and both of its targets compile; the failure is inherited from `main` and is being reported separately so it can be fixed at its own scope.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

