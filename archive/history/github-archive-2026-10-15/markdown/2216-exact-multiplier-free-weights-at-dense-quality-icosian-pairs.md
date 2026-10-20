# #2216 — Exact multiplier-free weights at dense quality: icosian pairs with a 2-bit-mantissa norm (KEEP); snap on GPU, 7.8x faster (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T22:34:26Z
- **Updated:** 2026-10-11T22:35:43Z
- **Closed:** 2026-10-11T22:35:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2216
- **Merged:** 2026-10-11T22:35:30Z
- **Merge commit:** 4d56f07fa089d0c1aa1122f8e8b5f8b1d95ad92f
- **Base / Head:** main / UOR-Foundation:claude/snap-gpu
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** geometric weights · **KEEP, count resets** (D22 §2) · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Exact multiplier-free weights at dense quality.** Every quaternion map weight becomes m·2^e·(g₁ + g₂/4): two icosian table entries and an exact 2-bit-mantissa scale, about 19–20 bits against 128.
- **Pre-registration:** #2032 [6114450465](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114450465).
- **Fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114483483), adversarial; its corrections were applied before compute.

| Arm (width 256, 2,000 steps, CUDA) | BPB | vs Q256c |
|---|---:|---:|
| Q256c, float quaternion | 1.4944 | — |
| QPf, pair + f32 norm | 1.5105 | +0.016 |
| **QM2, pair + 2-bit mantissa** (3 seeds) | **1.5148** | **+0.020** |
| D152, dense at equal parameters | 1.5136 | +0.019 |
| QM1, pair + 1-bit mantissa | 1.5312 | +0.037 |
| QP, pair + power of two | 1.5652 | +0.071 |

**What it adds:**
- **The snap on the tensor's own device:** bit-identical to the host loops (20,000 quaternions, four modes; negative gate) and **7.8× faster** training (122k against 15.6k tok/s).
- **\`quaternion_snap=icosian_pair_m1|icosian_pair_m2\`.** Tests: same direction, norm error within half a mantissa step, logits pinned; negative gate.

**Next:** activation snapping, trained in.

Record: `docs/labs/icosian-mantissa-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6114650076 — 2026-10-11T22:34:28Z — Casey-allard

**Exact-head checks at `99d0e1423aaa7f60aa2757efad9a519c00faca00`** (laptop, this worktree's own target):
- `cargo test --release -p uor-r4-training --lib -- icosian snap mantissa quaternion starling`: all pass.
- **Negative gates**, shown during development (#2032): a wrong exponent rounding in the device snap fails the bit-identity test; a wrong mantissa grid fails the mantissa test; dropping the pair dispatch fails the logits test (#2206).
- `cargo build --release --example geometric-stack`: PASS. `check_claim_wording.py`: PASS.
- `cargo fmt --check`: the only `geometric_stack.rs` diffs are the read-binding lines already on main; this PR does not touch them (`grep -c competing` on the diff = 0).
- **CUDA:** the pod bootstrap at this code passed `cuda_stack_ops_parity`, and all 6 runs trained on CUDA at 122k tok/s. That suite has no snap test (FIT correction); the evaluation re-snaps on CPU.

### review 5485323124 — 2026-10-11T22:34:29Z — Casey-allard — COMMENTED

**Self-review at exact head `99d0e1423aaa7f60aa2757efad9a519c00faca00`.**
- The device snap replaces the host loops on every device, and the loops remain as the bit-identity reference.
- The power-of-two and f32 paths are unchanged in value.
- The scoring follows the pre-registered rows and noise rule.
- The bit count is stated for the observed exponent range, and the multiplier-free claim is scoped to the weight side.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6114650076, 5485323124_

