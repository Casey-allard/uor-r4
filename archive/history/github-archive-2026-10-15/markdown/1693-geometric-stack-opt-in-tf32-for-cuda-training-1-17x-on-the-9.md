# #1693 — geometric-stack: opt-in tf32= for CUDA training (1.17x on the 96M fine-tune) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T17:26:47Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-04T17:27:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1693
- **Merged:** 2026-10-04T17:27:49Z
- **Merge commit:** 80b77ebd7b347161312728aeae6699ceed324cee
- **Base / Head:** main / UOR-Foundation:codex/trainer-speed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds `tf32=true|false` (default `false`) to `geometric-stack train` and `dialogue-train`. It lets CUDA f32 matmuls use TF32 tensor cores, through candle's `set_gemm_reduced_precision_f32`. `tf32=true` on a non-CUDA device is an error. Both run reports record `"tf32"`. Serving is unaffected: this is offline training only.

**Measured on the pod** (RTX 4090 ×2, one arm per GPU, built at this head). The workload is the ~96M chat fine-tune: `dialogue-train`, pointer=32, batch 16, context 384, `mixed-c`, 1,000 steps from the same init and seed.

| tf32 | Seconds to step 1,000 | Train response NLL | Dev response NLL (32 responses) |
|---|---|---|---|
| false | 254.2 | 0.9198 | 0.1002 |
| true | 217.1 | 0.9206 | 0.1049 |

- **Speed:** 1.17× faster.
- **Train loss:** within 0.001.
- **Dev:** +0.005 on a 32-response dev set, within run-to-run noise at this size; not established either way.

The default stays off, so existing recipes are unchanged.

**Checks:** `cargo fmt --check` and `cargo check -p uor-r4-training --example geometric-stack` pass, and both arms above ran at this head.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5982562571 — 2026-10-04T17:26:53Z — Casey-allard

Exact-head self-review at 8cd2327cd9d662ae36fea6c1387fdec1211cca39. The change is in one file, `geometric-stack.rs`: arg parsing in `device_arg`, both argument whitelists, and the two report fields. `cargo fmt` and `cargo check` pass at this head. The pod A/B in the PR body ran at this exact commit.


---
_Comment/review ids in this file: 5982562571_

