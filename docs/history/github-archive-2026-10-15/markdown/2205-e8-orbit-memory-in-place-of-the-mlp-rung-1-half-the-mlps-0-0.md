# #2205 — E8 orbit memory in place of the MLP, rung 1: half the MLPs +0.033 BPB at ~10x fewer ops; learned keys beat E8 addresses; row 3 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T18:04:50Z
- **Updated:** 2026-10-11T18:06:25Z
- **Closed:** 2026-10-11T18:06:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2205
- **Merged:** 2026-10-11T18:06:04Z
- **Merge commit:** aad19ace885cbb35698312d5edb33013b6fd9d84
- **Base / Head:** main / UOR-Foundation:claude/e8-memory
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** E8 orbit memory · count 1/3 · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Result: row 3, keep and iterate.** Half the MLPs become E8 orbit memories at +0.033 BPB, inside the 0.05 bar, with about 10× fewer multiply-adds per replaced layer. A learned-key memory of equal capacity is 0.023 better, so the E8 geometry is not credited yet.
- **Pre-registration:** #2032 [6105777437](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105777437), amended in 6105824579. FIT is recorded on #2032.

| Arm (width 256, 2,000 steps, 2 seeds) | BPB | vs D256 |
|---|---:|---:|
| D256, dense MLPs | 1.4078 | — |
| P, learned product keys, equal capacity | 1.4175 | +0.010 |
| **E8h**, half replaced | **1.4406** | **+0.033** |
| E8a, all replaced | 1.5237 | +0.116 |

Routing is healthy: 3.85 of 3.91 bits of orbit entropy, and 83–87% of slots read.

**What it adds:**
- `e8_memory.rs`: an exact O(8) E8 decode, SpiralCore's 15×16 Cl(0,6) orbits, and a softmax-free rank-table × gate read. Usage is logged.
- `set_e8_memory`, CLI `e8_memory=`, `evaluate` writing `e8_usage`, save/load, and an export refusal.
- **CPU AdamW in place** (bit-identical, pinned with a negative gate) plus parallel E8 slot selection: E8 steps are 1.8× faster.
- The mechanism brief `docs/mechanisms/e8-orbit-memory.md`.

The branch was squashed and rebased onto main once. The pre-squash commits are kept locally until merge.

Record: `docs/labs/e8-orbit-memory-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6112036424 — 2026-10-11T18:04:52Z — Casey-allard

**Exact-head checks at `1bdac148ac4ab7bb7ced9968e6cccdb17a6ea0d7`** (laptop; this worktree's own `target`; an earlier shared-target build was discarded):
- `cargo test --release -p uor-r4-training --lib -- e8 adam quaternion starling icosian snap`: **57 passed**, 1 ignored. This includes the 4 `e8_memory` tests (O(8) decoder = full scan, 15×16 orbits, selection = brute force, read gradient = finite differences), the E8 stack test (trains and round-trips), and the AdamW bit-identity test (which fails with a sabotaged moment).
- `cargo build --release --example geometric-stack --example mqar-bench`: PASS.
- `check_claim_wording.py`: PASS.
- `cargo fmt -p uor-r4-training --check`: the remaining diffs are in read-binding code already on main and in other files this PR does not touch. None are in this PR's hunks.
- **Rebase:** squashed onto main. 19 both-sides conflicts were resolved by keeping main's side and re-inserting the E8 methods and test verbatim from the pre-squash branch.

### review 5484348509 — 2026-10-11T18:04:53Z — Casey-allard — COMMENTED

**Self-review at exact head `1bdac148ac4ab7bb7ced9968e6cccdb17a6ea0d7`.**
- The E8 layers replace only the listed MLPs.
- The export refuses an E8 model, as it does a starling one.
- The scoring follows the pre-registered rows and amendment: noise = max(pooled SD, paired SE per target), the geometry credit is checked against P, and the usage is reported.
- The record states that the 10× is design arithmetic, and that CPU training was slower.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6112036424, 5484348509_

