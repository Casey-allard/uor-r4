# #1555 — Ignore the four legacy low-bit learner threshold tests (D18 §9, #1546)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T21:40:44Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-09-30T21:43:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1555
- **Merged:** 2026-09-30T21:43:04Z
- **Merge commit:** dd457291e5919d8842d60ad5a52233256b93aa9c
- **Base / Head:** main / UOR-Foundation:codex/d18-legacy-tests-ignore
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

The owner ruled on #1546 (D18 §9): the Adam `v_hat` guard removed in #1298 is not restored. That leaves four legacy low-bit learner threshold tests in `uor-r4-core` failing on main. They are marked `#[ignore = "…"]`, which names the ruling. They are not deleted, so they stay runnable with `--ignored`:
- `geometric_attention::…::corruption_in_the_objective_changes_the_filter`;
- `lowbit_attention::…::learns_induction_at_short_delay`;
- `lowbit_attention::…::learns_induction_at_long_delay`;
- `lowbit_attention::…::normalisation_moves_off_the_uniform_collapse`.

No other change: four attribute lines.

## Evidence at the exact head `18cec272`

GitHub run [36777691983](https://github.com/UOR-Foundation/uor-r4/actions/runs/36777691983) on `codex/ci/v2/legacy-ignore` (this head plus the evidence workflow):
- `cargo fmt --check`: success;
- `cargo check --workspace --all-targets --keep-going` (workbench excluded): success;
- `cargo test -p uor-r4-core --lib`: **1098 passed, 0 failed, 64 ignored**. Main had these four failing.

References #1546, #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5920215048 — 2026-09-30T21:40:58Z — Casey-allard

**Claude lab, exact-head self-review at `18cec272`: APPROVE** (owner rule, #820). Four `#[ignore]` attributes, one on each of the four tests the #1546 ruling covers (checked by function name in the diff); no code or assertion changes. The exact-head run 36777691983 completed with fmt, all-targets check and `uor-r4-core` lib tests all passing (1098/0/64 ignored).

### comment 5920418559 — 2026-09-30T21:56:22Z — Casey-allard

Merged as dd457291e5919d8842d60ad5a52233256b93aa9c. Verified: `git merge-tree --write-tree e4a6771f 18cec272` equals the squash tree (164d4009). The four legacy tests are now ignored on main; `uor-r4-core` lib stands at 1098 passed / 0 failed / 64 ignored.


---
_Comment/review ids in this file: 5920215048, 5920418559_

