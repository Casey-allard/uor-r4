# #2202 — Icosian weights: +0.141 BPB post-hoc, +0.140 trained in; the 120 directions are too coarse; row 4 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T15:55:11Z
- **Updated:** 2026-10-11T16:16:22Z
- **Closed:** 2026-10-11T15:56:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2202
- **Merged:** 2026-10-11T15:56:16Z
- **Merge commit:** 88966b521c3fbfa3b4cb04ac23a476f50768f125
- **Base / Head:** main / UOR-Foundation:claude/icosian-qat
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** geometric weights · count 1/3 · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Result: row 4.** Icosian weights cost +0.14 BPB even when trained in. The 120 directions are too coarse.
- **Pre-registration:** #2032 [6106445338](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6106445338), amended in 6106573972.
- **Fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6106609174).
- **Device amendment:** [6110389805](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6110389805).
- **Also carries:** the spin-snap probe record (6106370054).

**What it adds:**
- `quaternion_snap=icosian|icosian_f32`: a straight-through icosian snap of every compact quaternion weight. `icosian` also rounds the norm to a power of two.
- The snap is saved in the config.
- `data_parallel` refuses it, on resume too.
- Tests: the snapped forward values, the exact straight-through gradient, logits pinned with a negative gate, and save/load.

| Arm (width 256, 2,000 steps, trained on a 4090, evaluated on the laptop) | BPB mean | Δ vs Q256c |
|---|---:|---:|
| Q256c, float quaternion | 1.4944 | — |
| QSe, icosian + power-of-two norm (3 seeds) | 1.6344 | +0.140 |
| QSf, icosian + f32 norm (2 seeds) | 1.5976 | +0.103 |
| Post-hoc snap (probe) | 1.6369 | +0.141 |

Noise is 0.0041. The trained norms need 3 bits.

**Next:** a finer exact set (residual icosian pairs). This closes the configuration only (D22 §1).

Record: `docs/labs/icosian-qat-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6110841686 — 2026-10-11T15:55:14Z — Casey-allard

**Exact-head checks at `8a57ee2ee3d07d034515a97da9dd92bc1c6885b5`:**
- `cargo test --release -p uor-r4-training --lib -- quaternion starling icosian snap`: **38 passed**, 1 ignored. This includes the starling tests that sit in the same file after the rebase.
- `cargo build --release --example geometric-stack`: PASS.
- `check_claim_wording.py`: PASS.
- `cargo fmt -p uor-r4-training --check`: the only diffs are in the read-binding code already on main (`examples/geometric-stack.rs` around 4638/4694, `src/geometric_stack.rs` around 4730/9536, and four files this PR does not touch). None of them are in this PR's hunks.
- **CUDA:** the pod bootstrap at 27732022b (this code before the rebase) passed `cuda_stack_ops_parity`, and all 7 runs trained on CUDA.
- **Rebase:** two both-sides conflicts were resolved by keeping both. Two seams were fixed by hand: a duplicated `if`, and a starling test's `Ok(())` restored exactly as on main.

### review 5483931487 — 2026-10-11T15:55:15Z — Casey-allard — COMMENTED

**Self-review at exact head `8a57ee2ee3d07d034515a97da9dd92bc1c6885b5`.**
- The snap is applied only in `params()`; float weights stay the trained latent.
- The scoring follows the amended rows, with noise = max(pooled SD, paired SE) and the device amendment's Q256c base. The per-block NLL is normalised per target, as in the study's scorer.
- Row 4 is stated for its configuration only.
- The FIT review's recorded items are reported: the exponent range is 3 bits, and "no multiplier" holds on the weight side only.

APPROVE (self-review is allowed under the owner's exact-head rule).

### comment 6111039798 — 2026-10-11T16:16:22Z — Casey-allard

Read-only exact-merge review at88966b521. Configuration negative/trained-in multi-seed evidence/count1/3 are retained correctly underD22; do not close H4/E8 or geometric weights. One mathematical/lowering scope needs explicit conditioning: geometric_stack.rs377-399 snaps only weights via host f32/nearest root; params() then expands them with quaternion_dense_weight. Current activations remain general vectors, so a weight-side product is NOT yet a Cayley-table read+shift unless the activation operand is also represented in a closed admitted group/scale form or another exact decomposition is specified. Neither current dense expansion nor weight-only snapping measures table-serving throughput/energy. Separate measured quality of this configuration from proposed10-bit representation and future exact table lowering. Likewise QSf/QSe identifies losses for this tested snap/STE/recipe, not proof that all120-direction learned operators intrinsically require finer codebooks; capacity/training/operand-state contracts remain relevant. The finer exact set is a hypothesis with its own prospective FIT, not a mechanism closure. No peer source/jobs/leases/criteria changed by this audit.


---
_Comment/review ids in this file: 6110841686, 6111039798, 5483931487_

