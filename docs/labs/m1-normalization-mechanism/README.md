# M1 Lawful Normalization Mechanism (`OrbitRootSnapNormV1`, `#2842`)

**Date:** 2026-10-22
**Lab:** Antigravity (`antigravity-12`)
**Item:** [`m1-normalization-mechanism`](../../roadmap/items/m1-normalization-mechanism.md) (`#2842`)
**Design Note:** [`#2842` comment 6338588907](https://github.com/UOR-Foundation/uor-r4/issues/2842#issuecomment-6338588907)
**Pre-Registration Draft:** [`docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`](../../harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`)

## 1. Stabilized Block Invariants & Construction

In the discrete M1 carrier ([`crates/uor-r4-discrete/src/normalization_mechanism.rs`](../../../crates/uor-r4-discrete/src/normalization_mechanism.rs)), each 8D block root $\rho_r \in \mathcal{O}_{16} \subset \mathbb{Z}^8$ ($r \in \{0, \dots, 15\}$) is a signed permutation of $(2, 2, 0, 0, 0, 0, 0, 0)$. When $n \ge 1$ signed root contributions are summed across tree aggregation or residual stages ($x_b = \sum_{k=1}^n s_k \rho_{r_k}$), `OrbitRootSnapNormV1` evaluates the exact integer cost triple against each scaled root $n\rho_r$ (built by repeated addition, zero multiply instructions):
$$C_n(x, r) = \bigl(d_1(x, n\rho_r),\; d_H(x, n\rho_r),\; r\bigr), \qquad r^\star = \arg\min_{r \in \{0,\dots,15\}} C_n(x, r), \qquad \hat{x}_b = A_{a_b}\rho_{r^\star} \in \mathcal{O}_{16}.$$

Every normalized block $\hat{x}_b$ satisfies:
- $\|\hat{x}_b\|_1 = 4$, $\|\hat{x}_b\|_2^2 = 8$, $\|\hat{x}_b\|_\infty = 2$, and support size $= 2$.
- **Exact information loss:** retains $\log_2(16) = 4\text{ bits}$ per 8D block ($4B\text{ bits}$ across $B$ blocks, $16\text{ bits}$ at $B = 4$), discarding radial multiplicity $n$ and off-orbit residual components $x_b - n\rho_{r^\star}$.
- **Whole-number D11 export:** exports as 4-bit root indices `0..16` and `NormFrameTable::export_words()` (`B` whole numbers in `0..16`).

## 2. Six Finite CPU Controls (`cargo test -p uor-r4-discrete --lib normalization_mechanism`)

| Control Test | Property Verified | Result |
|---|---|---|
| `norm_stability_and_bounded_range` | $\|\hat{x}\|_1 = 4$, $\|\hat{x}\|_2^2 = 8$, $\|\hat{x}\|_\infty = 2$, support $= 2$ across arbitrary inputs and $n \in 1..=16$ | PASS |
| `idempotence_two_pass_stability` | $\operatorname{Norm}_1(\operatorname{Norm}_n(x)) = \operatorname{Norm}_n(x)$ and $\operatorname{Norm}_1(\rho_r) = \rho_r$ ($d_1 = 0, d_H = 0$) | PASS |
| `scale_equivariance_on_orbit_multiples` | $\operatorname{Norm}_n(n\rho_r) = \rho_r$ ($d_1 = 0, d_H = 0$); on $3\rho_0 + \rho_{\text{orth}}$, scale-matched `snap_4` keeps majority root `0` ($d_1 = 8$) while un-scaled `snap_1` flips to minority root `r_orth` ($d_1 = 12, d_H = 2 < 4$) | PASS |
| `deterministic_tie_breaking_on_zero_and_midpoints` | $x = \mathbf{0} \mapsto \text{root } 0$; all $\binom{16}{2} = 120$ pairwise sums $\rho_{r_1} + \rho_{r_2}$ break ties deterministically and commute under addition | PASS |
| `cl06_frame_equivariance_across_all_64_actions` | $\operatorname{Norm}_n(A_a(n\rho_r)) = A_a(\operatorname{Norm}_n(n\rho_r))$ across all 64 `Cl(0,6)` frames, 16 roots, and $n \in \{1, 4, 8\}$; `NormFrameTable` exports 4 bits/block | PASS |
| `multistage_residual_accumulation_vs_unnormalized_drift` | Across $S = 16$ residual stages, `UnnormalizedControlV1` drifts to $\|x\|_1 = 68$ and `address_distance > 16`, while `ScaleMatchedSnapV1` holds $\|\hat{x}\|_1 = 4$ and `address_distance` $\in \{0, 4, 8, 12, 16\}$ at every stage | PASS |

## Non-author FIT review (deepseek-37, 2026-10-22)

Non-author `TEST FITNESS` review of the draft sheet: **NOT_FIT**, recorded in the sheet's `## Verdict`. No compute is cleared. The sheet was later updated (`deepseek-40`, 2026-10-23) after the owner's **2026-10-23T02:36:36Z** standing-direction registration (`#2028` comment `6356034555`, card `6338596024`): the bar is now quoted as registered and §5 carries the full TRAIN/DEV code-pin digests. The two original prerequisites:
1. owner-registered bar (the linked `#2028` comment is antigravity's decision card `6338596024`, not an owner answer);
2. the full DEV token/mask pin — the sheet must use the authoritative code pin in `attention_learning.rs` (`0919e90f953e…` / `a838bb1a67e9…`; the staging sheet's `0919e90f813b…` / `a838bb1ae20c…` pair appears nowhere else and needs an errata).

Verified at the review head: `cargo test -p uor-r4-discrete --lib normalization_mechanism` = 6 passed; no float, dense matmul or squashing form on the path; `4 × B`-bit norm-table export; the parent A8 sheet is `TEST FITNESS: FIT`; all 8 blocks present.

## `C_unscaled` device wiring (claude-60, PR #3071)

- **What changed.** The GPU read kernel takes the `C_unscaled` control (`NormalizationMode::UnitSnapControlV1`) through spec word `s[12]`.
  - `1` sets `n := 1` at the three snap sites: tree, consensus and output.
  - `0` keeps `snap_n`, so the default path is unchanged.
  - `C_raw` is refused on the device path, because it needs a ℤ⁸ carry.
- **Carried end to end (claude-61, same PR).**
  - Snapshots record `normalization_mode`. The field is omitted when it is the default, so existing snapshots and their hashes are byte-identical.
  - `DriverConfig.normalization_mode` selects the arm (`unit_snap_control_v1`). A checkpoint whose mode differs from the config is refused.
  - `export_d11` refuses a non-default mode, because the integer engine has no unit snap.
- **GPU-verified on 2026-10-23,** on one RTX 5090 with Casey's approval. The run was zero-update: no training.
  - `device::unit_snap_device_tests::unit_snap_device_word_runs_and_refuses` passed.
  - At this head, the default path reproduced the A8 checkpoint-0 identity exactly: `b3_psucc` 1,351, `b3_full` 4,244, top-1 133, 256 occupied cells, 4,096 distinct vectors, tie mass 0.
  - The results are in the HF store at `results/claude/unit-snap-3071-20261023/`, with `SHA256SUMS`. Details are in the claude-61 REPORT.
