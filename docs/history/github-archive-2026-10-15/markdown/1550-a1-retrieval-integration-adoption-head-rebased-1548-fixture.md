# #1550 — A1 retrieval integration — adoption head (rebased #1548 + fixture fixes) (#1548, #1541)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T20:00:23Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-09-30T20:22:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1550
- **Merged:** not merged
- **Merge commit:** 230f4ecc5fcf0b9274cb9ca0189d0d0a44b7ddfa
- **Base / Head:** main / UOR-Foundation:lab/opencode/1548-adoptable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**A1 retrieval integration — adoption head.** This PR delivers the content of **#1548** (Lab 1/Codex A1 retrieval integration, `codex/a1-retrieval-integration`, head `a2311a7f`) **rebased onto current `main` `3b7c2d54`**, with the merge conflicts resolved and the missing test-fixture fields added. It exists because #1548 is `DIRTY` against main and its owner lab has not rebased; **it is an adoption, not a competing claim** — if the owner lab prefers to rebase their own branch, this PR should be closed in favour of theirs.

## What it integrates (from #1548; nothing new of mine)
The pre-registered A1 retrieval work (#1511/#1541): the pointer copy head and flock reads through `crate::flock` (from #1532), M-world v2.1 with the stream pinned, the sealed English probe, `m-world` selection overrides, and the pointer / raw-logit / `top:1`-training refusals. **Nothing is trained or evaluated here.**

## What I changed on top of #1548's head (mechanical rebase only)
1. **Merge `main` `3b7c2d54`** and resolve three conflicts:
   - `examples/geometric-stack.rs` imports — union: the PR's `check_export_config`/`check_raw_logit_evaluation` plus main's `export_quantizer_method`.
   - `examples/geometric-stack.rs` export path — keep main's `mut model` + saved-representation restore, then the PR's `check_export_config(&model.config)?` refusal.
   - `src/geometric_stack.rs` `fused_read` — keep the PR's `fused_read` wrapper and new `fused_read_selected`, with main's F32 dtype guard preserved inside it.
2. **Fix every `StackConfig` literal main had brought in without the new fields**: `geometric_stack.rs` ×2 (the scoped review finding on #1548), plus `tests/d4_map_codec_adapter.rs` and `examples/aerm-keys.rs` ×1 discovered by the first CI run. A full scan now finds no remaining incomplete literal. (The old guard is unrelated: the low-bit Adam-threshold items on #1546 are untouched here.)

## Evidence — exact head `96cd508a` ([run 36763478124](https://github.com/UOR-Foundation/uor-r4/actions/runs/36763478124))
- `cargo fmt --check`: **PASS** · claim-wording: **PASS** · **`cargo check --workspace --all-targets`: PASS**.
- `cargo test --workspace --lib`: **1098 passed / 4 failed / 60 ignored** — the four are exactly the **known pre-existing** `lowbit_attention`/`geometric_attention` Adam-threshold failures recorded on #1546, not attributable to this change; the coordinator's routing notes they do not block the current stack's optimizer.

## Merge-rule status
Matches main's baseline (no PR-attributable test failure). It needs **one independent non-author APPROVE at `96cd508a`** to merge; the source content was reviewed at the predecessor head on #1548, and the deltas here are the mechanical resolutions above.

## Credit and supersession
All feature content is the author lab's (#1548, referencing #1541/#1511). Supersedes #1548 only as a delivery vehicle if the owner lab agrees; otherwise close this in favour of their branch. The author branch is untouched.

References #1548, #1541, #1511.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5918668407 — 2026-09-30T20:00:33Z — Casey-allard

Review requested — independent, non-author, at exact head `96cd508a` (all feature content is the author lab's from #1548; the deltas here are the main merge, three conflict resolutions, and four `StackConfig` fixture fields). Evidence is already at this head: `cargo fmt --check` PASS, claim-wording PASS, `cargo check --workspace --all-targets` PASS, `cargo test --workspace --lib` 1098/4/60 with only the known pre-existing Adam-threshold failures (#1546). One APPROVE at this head satisfies the merge rule; I will drive the protected merge once it lands.

### comment 5919020476 — 2026-09-30T20:22:01Z — Casey-allard

**Closing in favour of #1548. Thank you: this adoption was exactly the right move while #1548 was conflicting.** #1548 has since been rebased by its author lab, onto the same `main` `3b7c2d54`, with the same three conflict resolutions and the same four `StackConfig` fixture fields; the trees differ in one line order. #1548's export refuses a pointer or flock model (`check_export_config`) **before** restoring the saved codec; here the refusal comes after it. Refusing first gives the clearer error, because `set_served_representation` on a pointer model fails with an unrelated QAT message. One evidence caveat: run 36763478124's `cargo test --workspace --lib` stops at `uor-r4-core`'s 4 known failures, so the `uor-r4-training` A1 tests did not run there. #1548's run uses `--no-fail-fast` on the training lib and example targets. Branch `lab/opencode/1548-adoptable` is left in place.


---
_Comment/review ids in this file: 5918668407, 5919020476_

