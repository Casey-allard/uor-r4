# #2206 — Residual icosian pairs: the direction is fixed (+0.016 BPB with an f32 norm); the power-of-two norm is the remaining cost; row 3 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T18:12:16Z
- **Updated:** 2026-10-11T18:13:29Z
- **Closed:** 2026-10-11T18:13:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2206
- **Merged:** 2026-10-11T18:13:20Z
- **Merge commit:** d6c5f02c123d1aad7a1219466a108e55fa963c9c
- **Base / Head:** main / UOR-Foundation:claude/icosian-pair
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** geometric weights · count 2/3 · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Result: row 3.** Residual icosian pairs fix the direction. With an f32 norm the cost is +0.016 BPB (inside the bar), level with dense at equal parameters. The power-of-two norm adds +0.055, for +0.071 in total against +0.140 for one icosian.
- **Pre-registration:** #2032 [6111364163](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6111364163), amended in 6111530535 after a NOT FIT first review.
- **Fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6111540766).
- **Owner cap approval:** 6111589309.

| Arm (width 256, 2,000 steps, trained on 4090s) | BPB | vs Q256c |
|---|---:|---:|
| Q256c, float quaternion | 1.4944 | — |
| QPf, pair direction + f32 norm (2 seeds) | **1.5105** | **+0.016** |
| QP, pair direction + power-of-two norm (3 seeds) | 1.5652 | +0.071 |
| QSe, one icosian + power-of-two norm (#2202) | 1.6344 | +0.140 |

**What it adds:**
- `quaternion_snap=icosian_pair|icosian_pair_f32`: 2^e (g₁ + g₂/4) with a straight-through gradient.
- Tests: exact pair form, closer than one icosian, logits pinned for both modes (with a negative gate).

**Finding:** the host-side snap made training 11× slower, so the next snap rung moves it onto the GPU first.

**Next:** a finer exact norm set. Count is 2/3 on this line.

Record: `docs/labs/icosian-pair-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6112107014 — 2026-10-11T18:12:17Z — Casey-allard

**Exact-head checks at `77f0424ed2e25d3847f5e57f8d954cbf77a9fe90`** (laptop, this worktree's own target):
- `cargo test --release -p uor-r4-training --lib -- icosian snap quaternion`: **33 passed**, 1 ignored. This includes the module-level pair test and the logits test over `icosian`, `icosian_f32`, `icosian_pair` and `icosian_pair_f32`.
- **Negative gate:** mapping `IcosianPair` to the unsnapped map fails the logits test (shown before the amendment).
- `cargo build --release --example geometric-stack`: PASS. `check_claim_wording.py`: PASS.
- `cargo fmt --check`: the only diffs in `geometric_stack.rs` are the read-binding lines already on main (around 4922/9728), not in this PR's hunks.
- **CUDA:** the pod bootstrap at e0eed80f5 (the same code before the docs rebase) passed `cuda_stack_ops_parity`, and all 5 runs trained on CUDA.

### review 5484371558 — 2026-10-11T18:12:18Z — Casey-allard — COMMENTED

**Self-review at exact head `77f0424ed2e25d3847f5e57f8d954cbf77a9fe90`.**
- The scoring follows the amended rows, with QPf's noise including its own seed SD as the FIT nit asked.
- The record reports the measured exponent range (3 bits) before the bit claim, scopes "multiplier-free" to the weight side, and states D152's device.
- The 11× throughput cost is disclosed.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6112107014, 5484371558_

