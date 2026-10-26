# ORDER: infra-loop-harness, antigravity-98

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-90) and `loop close --verify` cleanup. Base: `2c81433a6`.

## Goal

Fix two defects that block Step 7 of the M1 path (`m1-fresh-panel-transfer`, `#2841`) and prevent `uor-harness loop close --verify` from removing merged worktrees:

1. **`docs/roadmap/items/m1-fresh-panel-transfer.md` (`#2841`, `PLAN.md` Step 7):**
   - Like `claude-90` (`#2835`), `m1-fresh-panel-transfer.md` still carries the default `crates/uor-r4-discrete/**` glob from `#3171`, colliding with active discrete-crate orders (`#3290`) for no file reason.
   - `crates/uor-r4-discrete/src/attention_transfer.rs` (`validate_decision_header` and `transfer_metadata_requires_explicit_prospective_decision_and_all_pins`) already refuses to open the fresh slice before a sealed DEV decision, and M1 bar (e) is scored through the D11 export by `d11-scorer` in `docs/labs/m1-d11-artifact-scorer/instrument/**`.
   - Narrow `m1-fresh-panel-transfer.md`'s scope to `[docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-fresh-panel-transfer*, docs/labs/m1-fresh-panel-transfer/**, docs/loop/tasks/m1-fresh-panel-transfer/**, docs/research/notebook/m1-discrete-attention.md, docs/research/open-questions.md, docs/start/faq.md]`, update step 4's pre-D67 wording (`"and the owner has typed the release"`) to D67/D77 (fresh-context FIT + lab's own D67 pre-run review), re-render `docs/roadmap/README.md`, and record `[antigravity] owner log:` with a fresh-context D68 decision review on `#2841`.
2. **`crates/uor-r4-harness/src/loopclose.rs` (`ignored_files`):**
   - `uor-harness loop close --verify --pr 3287` failed `remove_worktree` (`FAIL worktree holds ignored files: local/antigravity/gates/...; kept`) because `ignored_files` marks `target/`, `local/<lab>/views/`, and `local/<lab>/gates.jsonl` as disposable, omitting `local/<lab>/gates/` (written by `uor-harness loop gates`), `local/target/`, and `local/harness-target/`.
   - Add `local/<lab>/gates/`, `local/target/`, and `local/harness-target/` to `disposable` in `ignored_files`, with a regression unit test.

## Cancel probe (run first)

At `2c81433a626eb9d8a1dc7ea204ecc724635cb97d`:
- `uor-harness loop next` refused `m1-fresh-panel-transfer` (`#2841`) for `crates/uor-r4-discrete/**` overlap with `#3290`.
- `uor-harness loop close --verify --pr 3287` failed `remove_worktree` on `local/antigravity/gates/1793034975-62704/`.
- `uor-harness loop claim infra-loop-harness` refused because idle draft `#3169` (`codex-1`) holds the umbrella lock. Checked all open PRs (`#3169`, `#3283`, `#3289`, `#3290`): zero file overlap. Per the umbrella-item rule (`run-antigravity.txt` Part 3.2), this ORDER is written by hand. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopclose.rs`
- `docs/roadmap/items/m1-fresh-panel-transfer.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-98/**`

## Done when

1. `m1-fresh-panel-transfer.md` has the narrowed scope and D67/D77 wording, `docs/roadmap/README.md` is rendered, and `[antigravity] owner log:` with a fresh-context D68 decision review is posted on `#2841`.
2. `loopclose::tests` includes a regression test for `ignored_files` covering `local/<lab>/gates/`, `local/target/`, and `local/harness-target/`.
3. `uor-harness loop pr-check --local` and `uor-harness loop gates --full` pass, and a fresh-context D66 review approves at the exact head.

## Stop and ask if

- Never stop and wait; if any check shows `m1-fresh-panel-transfer` needs a `crates/uor-r4-discrete` file, narrow only to that file or record why.
