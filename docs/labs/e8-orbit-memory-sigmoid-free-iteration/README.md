# E8 Orbit Memory Sigmoid-Free GPU Iteration — Design, Audit, and Pre-Registration Record (`#2822`)

- **Date**: 2026-10-25
- **Lab**: `antigravity`
- **Roadmap item**: [`e8-orbit-memory-sigmoid-free-iteration`](../../roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md) ([`#2822`](https://github.com/UOR-Foundation/uor-r4/issues/2822))
- **Pre-registration sheet**: [`docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md`](../../harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md)
- **Mechanism**: [`e8-orbit-memory`](../../mechanisms/e8-orbit-memory.md) (`PARKED`)
- **Historical Rung-1 record**: [`archive/labs/e8-orbit-memory-2026-10-11/README.md`](../../../archive/labs/e8-orbit-memory-2026-10-11/README.md) and [`result.json`](../../../archive/labs/e8-orbit-memory-2026-10-11/result.json)

## 1. Executive Summary and Cancel-Probe Audit

In Rung 1 (`archive/labs/e8-orbit-memory-2026-10-11/README.md`, 11 Oct 2026), replacing half of the feed-forward layers (`layers 1, 3, 5, 7`, arm `E8h`) with fixed-$E_8$ product-key orbit memory ($240 \times 240 = 57{,}600$ value slots per layer, $H = 4$ heads, $k = 8$ slots/head) at width 256 for 2,000 CPU steps achieved `1.4406` mean BPB (`1.4406`, `1.4406` across seeds `1` and `2`), staying within the pre-registered $+0.05$ bar against the dense control `D256` (`1.4078` mean; `E8h - D256 = +0.0329`, pooled seed SD `0.0054`) at $\sim 10\times$ fewer FFN multiply-adds (`27k` vs. `270k` per token per replaced layer). However:
1. An equal-capacity learned product-key control `P` (`1.4175` mean; `1.4143`, `1.4207`) beat `E8h` by **`0.0231` BPB** ($4.28\times$ the `0.0054` pooled seed SD), while orbit entropy (`3.85–3.87` of `3.9069` bits) and slot coverage (`83.2%–84.5%`) ruled out routing collapse.
2. The Rung-1 record (`archive/labs/e8-orbit-memory-2026-10-11/README.md:58-61`, `docs/mechanisms/e8-orbit-memory.md:84-87`) named the exact next iteration required to separate whether the `0.0231` gap comes from **slot bandwidth** or **fixed root coordinates**:
   - raise kept slots per head from **$k = 8 \to 16$** ($32 \to 64$ value rows read per token across 4 heads); and
   - add **learned per-orbit residual offsets on the $E_8$ frame** ($\delta_{h,o,m} \in \mathbb{R}^8$ for each head $h \in \{0..3\}$, query half $m \in \{1, 2\}$, and $\mathrm{Cl}(0,6)$ orbit $o \in \{0..14\}$).

Running the cancel probe on `d9dfbdb401f66d749afe4215afdb0a1370168852` (after `#2823` `float-training-module` landed `crates/uor-r4-float-train`) established four concrete facts (`instrument` — scope and dependency barriers, not a mechanism negative):

| Audit Target | Location | Finding | Charter Rule |
|---|---|---|---|
| **1. Item scope** | [`docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md:10`](../../roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md) | Scopes only `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration*`, `docs/labs/e8-orbit-memory-sigmoid-free-iteration/**`, and `docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/**` (`0` crate paths). | `CHARTER.md` §5.10 (scope lock) |
| **2. Banned logistic sigmoid gate** | [`crates/uor-r4-training/src/e8_memory.rs:415-421`](../../../crates/uor-r4-training/src/e8_memory.rs) | `score.broadcast_mul(beta)?.broadcast_add(gamma)?.neg()?.exp()?.affine(1.0, 1.0)?.recip()?` computes $\sigma(\beta_h s + \gamma_h)$. | Violates `CHARTER.md` §3.2 (D23: no sigmoid/squashing anywhere) |
| **3. Banned dense query matmul** | [`crates/uor-r4-training/src/geometric_stack.rs:5995`](../../../crates/uor-r4-training/src/geometric_stack.rs) | `self.linear(u, p.layer(layer, "e8.query.weight")?)?` applies a dense $[64, 256]$ matrix multiplication. | Violates `CHARTER.md` §3.3 (D25/D30: no dense matmul/GEMM) |
| **4. Host CPU Rayon fallback & $k \le 8$ cap** | [`crates/uor-r4-training/src/e8_memory.rs:339, 351, 405`](../../../crates/uor-r4-training/src/e8_memory.rs) | Copies `query` to host (`to_vec1::<f32>()`) and runs `e8_select` on the CPU via Rayon `(0..rows * heads).into_par_iter()`, restricting `k` to `{1, 2, 4, 8}`. Absent from `crates/uor-r4-float-train`. | Violates `CHARTER.md` §3.5 (D24: GPU-only research compute, no host loops) |
| **5. Missing D11 float-trunk exporter** | [`docs/roadmap/items/d11-exporter-float-trunk.md`](../../roadmap/items/d11-exporter-float-trunk.md) | Still `status: ready`; no D11 integer exporter exists yet for `crates/uor-r4-float-train` checkpoints or product-key tables. | Required by `CHARTER.md` §3.6, §3.7 (D37) and [`docs/mechanisms/e8-orbit-memory.md:42, 85-87`](../../mechanisms/e8-orbit-memory.md) |

## 2. Compliant Sigmoid-Free Geometric Architecture (`crates/uor-r4-float-train`)

To run `e8-orbit-memory` lawfully inside the separately scanned D37 module `crates/uor-r4-float-train` (`#2823`), every banned or host-bound component is replaced by an exact geometric counterpart:

### 2.1 Matmul-Free Query Projection via `SpiralcoreFbs` (`CHARTER.md` §3.1, §3.3)
Instead of `self.linear(u, "e8.query.weight")` (`[4 * 16, 256]` dense matmul), each E8-memory layer maps the normalized hidden state $u \in \mathbb{R}^{256}$ ($32$ octonion blocks of $\mathbb{R}^8$) to $H = 4$ heads of 16D queries $q_h = (q_{h,1}, q_{h,2}) \in \mathbb{R}^8 \times \mathbb{R}^8$ ($8$ output blocks of $\mathbb{R}^8$) using `SpiralcoreFbs` (`crates/uor-r4-float-train/src/spiralcore_fbs.rs`, `spiralcore_fbs_fwd` / `spiralcore_fbs_bwd`):
- Forward routes each output block through the 3-stage base-3 FBS binder tree and 64-element signed-axis group $\mathrm{Cl}(0,6)$ with zero matrix multiplications.
- Backward propagates query gradients $\partial \mathcal{L} / \partial q_{h,m}$ into the query route parameters and trunk residual stream via the root-edge secant `spiralcore_fbs_bwd`.

### 2.2 On-Device $O(8)$ $E_8$ Decode, $\mathrm{Cl}(0,6)$ Orbits, and Per-Orbit Residual Frame Offsets (`CHARTER.md` §3.1, §3.5)
- **Exact $O(8)$ $E_8$ decode on CUDA**: Each 8D query half $q_{h,m} \in \mathbb{R}^8$ ($m \in \{1, 2\}$) is decoded in a CUDA device function `e8_nearest_dev(q)` in $O(8)$ operations:
  1. $D_8$ integer candidate: find the two largest $|q_i|$ in a single 8-lane scan, assign $\pm 2$ in doubled coordinates, score $s_{D_8} = 2(|q_{i_1}| + |q_{i_2}|)$.
  2. Half-integer candidate $D_8 + \mathbf{1}/2$: take $\mathrm{sgn}(q_i) \in \{-1, +1\}^8$; if the count of negative signs is odd, flip the coordinate of minimum $|q_i|$; score $s_{\text{half}} = \sum_{i=0}^7 \mathrm{sgn}_i\, q_i$.
  3. Select the higher score and map to root index $r^*_{h,m} \in \{0..239\}$ via constant-memory lookup (`D8_INDEX[8][8][4]` and `HALF_INDEX[128]`), then look up its orbit $o_{h,m} = \mathrm{ORBIT\_OF}[r^*_{h,m}] \in \{0..14\}$ and the 16 orbit members $\mathcal{O}(o_{h,m}) = \mathrm{ORBIT\_MEMBERS}[o_{h,m}][0..16]$.
- **Learned per-orbit residual offset $\delta_{h,o,m} \in \mathbb{R}^8$**:
  For head $h \in \{0..3\}$, half $m \in \{1, 2\}$, and orbit $o \in \{0..14\}$, parameter tensor `e8.orbit_offset` of shape `[4, 2, 15, 8]` (`960` floats per layer, initialized to `0.0`) adds a local frame offset to every candidate root $c \in \mathcal{O}(o_{h,m})$:
  $$\tilde{r}_{h,m}(c) = \frac{1}{2\sqrt{2}}\,\mathrm{coords}[c] + \delta_{h,o_{h,m},m}, \qquad s_{h,m}(c) = \langle q_{h,m},\, \tilde{r}_{h,m}(c) \rangle.$$
  After each optimizer step, $\delta_{h,o,m}$ is radially projected onto $\|\delta_{h,o,m}\|_2 \le \frac{1}{2}$ (half the minimum inter-root distance $\sqrt{2}/2 > 0.5$), guaranteeing that $\tilde{r}_{h,m}(c)$ never leaves the Voronoi cell of root $c$.
- **On-device product-orbit top-$k$ selection (`k = 8` and `k = 16`)**:
  A CUDA warp (`32` threads) per `(row, head)` evaluates the $16 \times 16 = 256$ product-orbit sums $s_h(c_1, c_2) = s_{h,1}(c_1) + s_{h,2}(c_2)$ ($8$ pairs per thread) and performs a warp-register bitonic/tournament top-$k$ reduction for $k \in \{8, 16\}$, writing device tensors `slots: [rows, 4, k]` (`u32`, $p_{h,r} = 240\,c_1 + c_2 \in \{0..57{,}599\}$) and `scores: [rows, 4, k]` (`f32`) with zero host-device copies.

### 2.3 Sigmoid-Free Clamped-Linear Rank Gate (`CHARTER.md` §3.2 / D23)
Replacing the banned logistic sigmoid $\sigma(\beta_h s + \gamma_h)$ (`e8_memory.rs:415-421`), each selected slot $r \in \{0..k-1\}$ is weighted by the piecewise-linear hard-clamped gate:
$$z_{h,r} = \beta_h\, s_{h,r} + \gamma_h, \qquad g_{h,r} = \mathrm{clamp}(z_{h,r},\, 0,\, 1) = \min\!\big(\max(z_{h,r}, 0), 1\big), \qquad w_{h,r} = R[h][r]\cdot g_{h,r},$$
$$y_{\text{row}} = \sum_{h=0}^{3} \sum_{r=0}^{k-1} w_{h,r}\, V\big[p_{h,r}\big] \in \mathbb{R}^{256}.$$
- **Initialization**: $R[h][r] = \frac{1}{H \cdot k}$ (`1/32` for $k=8$, `1/64` for $k=16$), $\beta_h = 0.25$, $\gamma_h = 0.50$, so at step 0 typical unit-norm scores $s_{h,r} \in [-1.5, +1.5]$ land inside the active linear window $z_{h,r} \in (0, 1)$ (`~85%+` active rate).
- **Backward pass (zero squashing)**: For upstream gradient $d y \in \mathbb{R}^{256}$:
  $$d w_{h,r} = \langle d y,\, V[p_{h,r}] \rangle, \qquad d V[p_{h,r}] \mathrel{+}= w_{h,r}\, d y, \qquad d R[h][r] \mathrel{+}= g_{h,r}\, d w_{h,r},$$
  $$d z_{h,r} = R[h][r]\, d w_{h,r}\, \mathbf{1}_{(0, 1)}(z_{h,r}), \qquad d\beta_h \mathrel{+}= s_{h,r}\, d z_{h,r}, \qquad d\gamma_h \mathrel{+}= d z_{h,r}, \qquad d s_{h,r} = \beta_h\, d z_{h,r},$$
  $$d q_{h,m} \mathrel{+}= \sum_{r=0}^{k-1} d s_{h,r}\, \tilde{r}_{h,m}(c_{m,r}), \qquad d \delta_{h,o_{h,m},m} \mathrel{+}= \sum_{r=0}^{k-1} d s_{h,r}\, q_{h,m}.$$
  Wait: notice how `d s_{h,r}` simultaneously trains both the query route (`d q_{h,m}`) and the learned per-orbit residual offset (`d \delta_{h,o,m}`) along the active linear window $(0, 1)$ without a single exponential, logarithm, division, or sigmoid!

### 2.4 D11 Integer Export Representation (`CHARTER.md` §3.6, §3.7)
When exported through `d11-exporter-float-trunk` to `UORLUT01` (`crates/uor-r4-integer`):
1. $E_8$ nearest-root decode uses the exact integer doubled-root rule ($2(|q_{i_1}| + |q_{i_2}|)$ vs. $\sum \pm q_i$ in Q1.15 fixed point — additions, comparisons, and sign flips only).
2. Orbit expansion reads the static 240-byte `ORBIT_OF` and $15 \times 16$-byte `ORBIT_MEMBERS` tables.
3. Per-orbit residual offsets $\delta_{h,o,m}$ are quantized to `i16` (`Q1.14`) and combined with doubled integer root coordinates by shift-add; top-$k$ slot selection is an integer comparison network over 256 candidates.
4. Clamped-linear gate $g_{h,r} = \mathrm{clamp}(\beta_h s_{h,r} + \gamma_h, 0, 1)$ and rank table $R[h][r]$ fold into a shift-and-clamp integer lookup table indexing quantized value rows $V[p_{h,r}]$, passing `scripts/audit_zero_matmul_serving.py --stack` (`0` float, multiply, or divide instructions).

## 3. Why `#2822` Is Held Under D68 Pending Code & Exporter Dependencies

Per `CHARTER.md` §5.1 (D67/D68), a GPU run goes ahead only when every tool, binary, and dependency in the chain exists. Because:
1. `#2822` is scoped to `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration*`, `docs/labs/e8-orbit-memory-sigmoid-free-iteration/**`, and `docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/**`;
2. `crates/uor-r4-float-train` (`#2823`) needs a code-scoped follow-up unit to add `src/e8_memory.rs` and the four CUDA kernels (`e8_select_fwd_k8_f32`, `e8_select_fwd_k16_f32`, `e8_memory_gather_fwd_f32`, `e8_memory_scatter_bwd_f32`); and
3. `d11-exporter-float-trunk` (`docs/roadmap/items/d11-exporter-float-trunk.md`) is still `status: ready` and is required to measure the D37 integer export gap and served `tokens/s`,

this unit completes the pre-registration sheet (`TEST FITNESS: FIT`), records this design and runbook specification, and places `e8-orbit-memory-sigmoid-free-iteration` in `status: held` under a fresh-context D68 decision review until those two code prerequisites land.
