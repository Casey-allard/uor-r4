# #1801 — Holographic phase binding on the read (opt-in read_lineage=phase_bind, Step 9; References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T16:40:16Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-10-06T16:48:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1801
- **Merged:** 2026-10-06T16:48:17Z
- **Merge commit:** 57497bb9530276c33d0260c1f161226cdb17a8e7
- **Base / Head:** main / UOR-Foundation:claude/phase-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820 (Step 9).

Adds `ReadLineage::PhaseBinding { snap }` (`read_lineage=phase_bind|phase_bind_free`), default off. On each read layer:
- Key phase `P_j` and query phase `Q_t`: learned unit quaternions per head and per 4-channel block. With `snap`, they are straight-through snapped to the 120 unit icosians.
- Write `w_j = g_j (P_j ⊗ v_j)`.
- Strictly causal decay-averaged trace `S_t = Σ_{j<t} a_h^{t-1-j} w_j / Σ a_h^{t-1-j}`, with `a_h = 1 − 2^-(2 + h mod 8)`.
- Unbind `r_t = conj(Q_t) ⊗ S_t`, then add `merged += mix ⊙ r` before read.out.

`mix` starts at 0, so a model with the lineage computes the plain read bit for bit until trained. Saved and loaded like the Step 7a lineages. Refused when the head width is not a multiple of 4, or when a geometric address is set (both orders). Candle ops only. Adds 2.12M parameters per read layer at width 1024 / 16 heads.

Unlike 7a `rot` (keys and queries rotated, so the softmax lands elsewhere), the value itself is bound to the key phase and recovered by unbinding: a content-addressed, position-free channel.

Tests (`phase_binding*`, 7): starts as the plain read (both snap values); exact retrieval and rotation on a mismatch; decay-weighted average over 5 steps; causality; save/load plus gradients on the maps, gate and mix; icosian snap. Negative gates:
- mix nonzero at init → T1 fails;
- the trace including j = t → the retrieval test fails;
- reading j = t+1 → the causality test fails;
- decay exponent `a^j` → the decay test fails.

A fresh-context review found no correctness, causality or safety defects. Its low findings are fixed (decay test, comment), apart from performance: the decay matrix is built on the host per forward, which is fine at context 384.

Not run here: the CUDA parity entries (`test_step7a_read_lineage_parity` now lists both variants). The pod bootstrap's parity run covers them before Step 9 trains.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6021059021 — 2026-10-06T16:47:25Z — Casey-allard

Exact-head self-review at `683e10b5458db41094522d59a7a0f11ef4489057` (owner rule: self-review allowed).
- Laptop at this head: `cargo check -p uor-r4-training --lib --tests --example geometric-stack` 0 errors; rustfmt clean on touched files; `phase_binding` lib tests 7/7; `step7a` 3/3. Negative gates as in the description (decay exponent swap → the new decay test fails, restored → 7/7).
- Pod (RTX 5090, sm_120) bootstrap at this exact sha: release build + `cuda_stack_ops_parity` **PASS 34 passed, 0 failed** incl. `test_step7a_read_lineage_parity` (now covering both PhaseBinding variants, mix perturbed via the `.read.lineage_` prefix).
- Fresh-context review: no correctness/causality/safety defects; low findings fixed.
- Off by default; Step 9 (#820) trains it now. APPROVE.


---
_Comment/review ids in this file: 6021059021_

