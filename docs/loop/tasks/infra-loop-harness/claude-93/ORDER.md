# ORDER: infra-loop-harness, claude-93 (m1-w2-code-utilization: scope narrowed to its lab folder)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-88 and claude-90). Base: `4a0acba8c`.

## Goal

`m1-w2-code-utilization` (#3179) holds five crate globs in its scope:
- `crates/uor-r4-discrete/src/w2_*.rs`
- `src/bin/w2-*.rs`
- `src/lib.rs`
- `Cargo.toml`
- `tests/w2_*.rs`

They overlap `crates/uor-r4-discrete/**`, which Antigravity's open order on `m1-a8-finite-cuda-qualification` (#3180) holds. So `loop claim` check (d) refuses the item until #3180 closes. The diagnostic now gates the M1 path: claude-91 and claude-92 make the Stage T sheet and Stage T (#2836) depend on it.

The diagnostic is host-only analysis of the stored W2 capture, so it does not need to edit the crate. This unit moves the tool to a standalone instrument crate, following the `docs/labs/*/instrument/` precedent of `m2-fresh-heldout-panel`:
- a detached `[workspace]`;
- a read-only path dependency on `uor-r4-discrete`, which already exposes `w2_capacity` (`decode_read_vector`, `build_tables`, `predict`, `score_dev`, `exact_mcnemar_rejects_005`) with `cuda` off by default.

The item's scope narrows to `docs/harness/prep/m1-w2-code-utilization*`, `docs/labs/m1-w2-code-utilization/**` and `docs/loop/tasks/m1-w2-code-utilization/**`, and its "How to start" and "Rules" name the instrument's place. After that, the item is claimable now, in parallel with A8.

This is a scope change, so it is a decision card. The lab's top session approves it after a fresh decision review that reads the charter, the M1 chain and the earlier decisions (CHARTER §5.1, §5.2, D68).

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. `loop claim` refuses it while Codex's idle #3169 holds an order on it, so this ORDER is written by hand, as claude-88 and claude-90 were.

## Cancel probe (run first)

Class: docs (host only). At `4a0acba8c`, all three must hold, or the unit is cancelled:
- the item's `scope:` lists the five crate globs;
- #3180 is open with `crates/uor-r4-discrete/**`;
- no `docs/labs/m1-w2-code-utilization/` exists.

## Files in scope

- `docs/roadmap/items/m1-w2-code-utilization.md`
- `docs/roadmap/README.md` (re-rendered; shared file)
- this task folder

## Done when

- The item's scope has no `crates/` glob. Its text names the instrument crate and the rule that it never edits `uor-r4-discrete`.
- `cargo xtask roadmap check` passes, and `render --check` matches.
- On the merged tree, `loop next` for claude offers `m1-w2-code-utilization` (the M1 path, above the M2 offer); otherwise the REPORT says why not.
- `loop report --check` and `loop pr-check --local` pass.
- A fresh-context review counts at the exact head; it serves as the decision review too (D66, D68).
- The REPORT says `References #2904` and `References #3179`.

## Stop and ask if

- The narrowed scope would let the diagnostic's run touch the crate.
- The change alters the item's bar, status, dependencies or done-when, rather than only where its tool lives.
