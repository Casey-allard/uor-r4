# Pre-Registration Sheet: E8 Orbit Memory Sigmoid-Free GPU Iteration (`e8-orbit-memory-sigmoid-free-iteration`)

- **Date**: 2026-10-25
- **Lab**: `antigravity`
- **Roadmap Item**: [`e8-orbit-memory-sigmoid-free-iteration`](../../roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md) ([`#2822`](https://github.com/UOR-Foundation/uor-r4/issues/2822))
- **Mechanism**: [`e8-orbit-memory`](../../mechanisms/e8-orbit-memory.md) (`PARKED`; reopen test under `CHARTER.md` §3.1–§3.7, §4.1–§4.5)
- **Run**: `e8-orbit-memory-sigmoid-free-r2` (offline float training in `crates/uor-r4-float-train` with D11 integer export gap reported via `d11-exporter-float-trunk`)

## Question and Geometric Construction

**Question**: In the separately scanned D37 offline-float training module (`crates/uor-r4-float-train`, `#2823`), when half of the feed-forward blocks (layers `1, 3, 5, 7`, `E8h`) are replaced by $E_8$ orbit product-key memory ($240 \times 240 = 57{,}600$ value slots per layer) using:
1. a **sigmoid-free clamped-linear rank gate** $w_{h,r} = R[h][r]\cdot\mathrm{clamp}(\beta_h s_{h,r} + \gamma_h, 0, 1)$ in place of the banned logistic sigmoid $\sigma(\beta_h s_{h,r} + \gamma_h)$ (`CHARTER.md` §3.2 / D23);
2. a **matmul-free `SpiralcoreFbs` query projection** in place of `linear(u, e8.query.weight)` (`CHARTER.md` §3.3 / D25/D30);
3. **on-device CUDA `e8_select` and `e8_memory_read` kernels** in place of the host CPU Rayon loop (`CHARTER.md` §3.5 / D24); and
4. **the record's named next iteration** (`docs/mechanisms/e8-orbit-memory.md:84`, `archive/labs/e8-orbit-memory-2026-10-11/README.md:59-60`): raising top-$k$ kept slots per head from $k = 8$ to $k = 16$ ($32 \to 64$ value rows read per token across $H = 4$ heads) and adding **learned per-orbit residual offsets** $\delta_{h,o,m} \in \mathbb{R}^8$ ($o \in \{0,\dots,14\}$, $m \in \{1, 2\}$) on the $E_8$ frame,

does `E8h_sigfree_k16_offset` close the stored Rung-1 `0.0231` gap to the capacity-matched learned-key reference `P` (`1.4175` mean BPB in `archive/labs/e8-orbit-memory-2026-10-11/result.json`), match or beat the no-E8-memory geometric FFN control `C_hamilton_trunk` (L-010) within the two-seed noise band across seeds `1` and `2`, and export through D11 (`d11-exporter-float-trunk`) with an export gap inside the two-seed noise band while beating the unigram (`1,226 / 14,287`) and T2 successor (`6,300 / 14,287`) count tables on the 14,287-row DEV panel (`CHARTER.md` §3.7, §4.8)?

**Exact compliant mechanism equations (`crates/uor-r4-float-train`)**:
- **Exact $O(8)$ $E_8$ root decode & $\mathrm{Cl}(0,6)$ orbit lookup**: The 240 roots of $E_8$ in doubled integer coordinates ($\|c\|_2^2 = 8$, scale $\frac{1}{2\sqrt{2}}$) partition under SpiralCore's 64-element signed-axis group $\mathrm{Cl}(0,6)$ (`crates/uor-r4-core/src/spiralcore_operator.rs`) into $15$ orbits of $16$ roots (`ORBIT_OF[240] -> 0..15`, `ORBIT_MEMBERS[15][16] -> 0..240`). For head $h \in \{0..3\}$ and half $m \in \{1, 2\}$, query half $q_{h,m} \in \mathbb{R}^8$ (produced from hidden state $u \in \mathbb{R}^{256}$ via `QuaternionWeights::SpiralcoreFbs`) decodes in $O(8)$ on the GPU to its nearest $E_8$ root $r^*_{h,m} \in \{0..239\}$ and selects orbit $o_{h,m} = \mathrm{ORBIT\_OF}[r^*_{h,m}] \in \{0..14\}$ with 16 candidate roots $\mathcal{O}(o_{h,m})$.
- **Learned per-orbit residual offset on the $E_8$ frame ($\delta_{h,o,m} \in \mathbb{R}^8$)**: Each head $h$, half $m$, and orbit $o \in \{0..14\}$ carries an 8D residual vector $\delta_{h,o,m} \in \mathbb{R}^8$ (`2 * 4 * 15 * 8 = 960` floats/layer, zero-initialized, projected after each update onto the ball $\|\delta_{h,o,m}\|_2 \le \frac{1}{2}$ so shifted roots remain inside their local Voronoi neighbourhood):
  $$\tilde{r}_{h,m}(c) = \frac{1}{2\sqrt{2}}\,\mathrm{coords}[c] + \delta_{h,\,\mathrm{ORBIT\_OF}[c],\,m}, \qquad s_{h,m}(c) = \langle q_{h,m},\, \tilde{r}_{h,m}(c) \rangle \quad \big(c \in \mathcal{O}(o_{h,m})\big).$$
- **Product-orbit top-$k$ selection ($k \in \{8, 16\}$) & sigmoid-free clamped-linear rank gate**: Over the $16 \times 16 = 256$ candidate pairs $(c_1, c_2) \in \mathcal{O}(o_{h,1}) \times \mathcal{O}(o_{h,2})$, combined score $s_h(c_1, c_2) = s_{h,1}(c_1) + s_{h,2}(c_2)$ selects the top $k$ slots $(p_{h,r}, s_{h,r})_{r=0}^{k-1}$ with $p_{h,r} = 240\,c_1 + c_2 \in \{0..57{,}599\}$. Slot $r$ receives the **sigmoid-free clamped-linear weight**:
  $$g_{h,r} = \mathrm{clamp}(\beta_h s_{h,r} + \gamma_h,\, 0,\, 1), \qquad w_{h,r} = R[h][r] \cdot g_{h,r}, \qquad y = \sum_{h=0}^{H-1} \sum_{r=0}^{k-1} w_{h,r}\, V[p_{h,r}],$$
  with straight-through active-interval gradient $\frac{\partial g_{h,r}}{\partial (\beta_h s_{h,r} + \gamma_h)} = \mathbf{1}_{(0, 1)}(\beta_h s_{h,r} + \gamma_h)$ to $\beta_h$, $\gamma_h$, $\delta_{h,o,m}$, and $q_{h,m}$ (and exact `SpiralcoreFbs` secant backward `spiralcore_fbs_bwd` into the query trunk). Zero `exp`, `sigmoid`, `softmax`, or dense matmuls appear in forward or backward.

## Base artifact

- **Offline float training module**: `crates/uor-r4-float-train` (`#2823`, merged at `d9dfbdb401f66d749afe4215afdb0a1370168852`), extended in a dedicated code unit with `src/e8_memory.rs` and CUDA kernels `e8_select_fwd_k8_f32`, `e8_select_fwd_k16_f32`, `e8_memory_gather_fwd_f32`, and `e8_memory_scatter_bwd_f32`.
- **Architecture & objective**: `rrarrarr` 8-layer geometric stack (`width = 256`, `heads = 4`, `context = 256` for Rung-1 parity and `context = 192` on `ft/dev`, `batch = 16`, `2,000` optimizer steps, `lr = 1e-3`, `RecGates::Geometric`, `Gate::Hamilton` on non-memory layers `0, 2, 4, 6`, `QuaternionWeights::SpiralcoreFbs`, `Objective::MarginHinge { rivals: 64 }`).
- **Stored Rung-1 reference artifact (`archive/labs/e8-orbit-memory-2026-10-11/result.json`, reused without re-running banned dense/sigmoid arms per `CHARTER.md` §3.10)**:
  - `P` (learned product-key memory, $240^2$ slots, $k=8$): seed 1 `1.4143`, seed 2 `1.4207`, mean `1.4175` BPB;
  - `E8h` (Rung-1 sigmoid gate, $k=8$, fixed $E_8$): seed 1 `1.4406`, seed 2 `1.4406`, mean `1.4406` BPB (`E8h - P = +0.0231`, `E8h - D256 = +0.0329`, pooled seed SD `0.0054`);
  - `D256` (Rung-1 dense MLP): seed 1 `1.4054`, seed 2 `1.4101`, mean `1.4078` BPB.
- **D11 integer exporter dependency**: [`d11-exporter-float-trunk`](../../roadmap/items/d11-exporter-float-trunk.md) (`UORLUT01` integer export and zero-multiply audit `scripts/audit_zero_matmul_serving.py --stack`).

## Data split

- **Primary training & evaluation corpus (`ft/train` and `ft/dev`, pinned SHA-256 digests)**:
  - `ft/train/tokens.u16`: `61f2d987b33a7dbac9ebfc15c99f24a25e947110dc7883a6703ab66389566bd6`
  - `ft/train/response_mask.u8`: `e85b832a8531b606794cb0f2d904259516368952408c0949fb89f4e935721087`
  - `ft/dev/tokens.u16`: `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`14,287` scored target positions: `3,847` `P_succ` copy/successor rows and `10,440` complement rows)
  - `ft/dev/response_mask.u8`: `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`
  - `tokenizer.json` (`vocab_size = 4,096`): `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`
- **Rung-1 parity slice (`archive/labs/e8-orbit-memory-2026-10-11/README.md:22`)**: Same 256 held-out blocks (`max_blocks = 320`, `tune_blocks = 64`, seeds `1` and `2`) stored in `claude/e8-rung1-done-20261011` for direct paired comparison against stored `P` and `E8h`.
- **Fresh transfer panel (`CHARTER.md` §4.3)**: `fresh_v2` (`>= 2,000` rows), held sealed and evaluated once if and only if a treatment arm passes the DEV promotion bar on both seeds.

## Steps

**Five registered GPU arms** (each run on two seeds `seed = 1` and `seed = 2` for `2,000` steps on a single `uor-pod` RTX 5090):

| Arm | Layers Replaced | Top-$k$ per Head | Per-Orbit Offset $\delta_{h,o,m}$ | Gate Form | Purpose in $2 \times 2$ Factorial + Control |
|---|---|---:|---|---|---|
| **`E8h_sigfree_k8_fixed`** | `1, 3, 5, 7` | `k = 8` (`32` rows/tok) | Fixed $E_8$ ($\delta \equiv 0$) | Clamped-linear $\mathrm{clamp}(\beta_h s + \gamma_h, 0, 1)$ | Baseline sigmoid-free GPU port of Rung-1 `E8h` |
| **`E8h_sigfree_k16_fixed`** | `1, 3, 5, 7` | `k = 16` (`64` rows/tok) | Fixed $E_8$ ($\delta \equiv 0$) | Clamped-linear $\mathrm{clamp}(\beta_h s + \gamma_h, 0, 1)$ | Isolates slot-bandwidth effect ($k: 8 \to 16$) |
| **`E8h_sigfree_k8_offset`** | `1, 3, 5, 7` | `k = 8` (`32` rows/tok) | Learned $\delta_{h,o,m} \in \mathbb{R}^8$ ($\|\delta\|_2 \le 0.5$) | Clamped-linear $\mathrm{clamp}(\beta_h s + \gamma_h, 0, 1)$ | Isolates per-orbit residual frame addressing at $k = 8$ |
| **`E8h_sigfree_k16_offset`** | `1, 3, 5, 7` | `k = 16` (`64` rows/tok) | Learned $\delta_{h,o,m} \in \mathbb{R}^8$ ($\|\delta\|_2 \le 0.5$) | Clamped-linear $\mathrm{clamp}(\beta_h s + \gamma_h, 0, 1)$ | **Primary treatment**: full named next iteration ($k = 16$ + per-orbit offset) |
| **`C_hamilton_trunk`** | None (`0` E8 layers) | — | — | `Gate::Hamilton` + `SpiralcoreFbs` on all 8 layers | **No-E8-memory geometric FFN control (L-010)** |

1. **Step 0 — Probe ladder (a) finite CUDA instrument controls (`I-1..I-6`, ~2 min)**: Run deterministic CUDA kernel & gradient controls before any training step. Halt on any `I-1..I-6` failure as an `instrument` defect (`0` mechanism claims).
2. **Step 1 — Probe ladder (b) 32-step throughput & active-gate calibration (~3 min)**: Measure step wall time on RTX 5090 (must project `<= 20 min` per 2,000-step arm; Rung-1 CPU took `~6 h` for `E8a`) and verify clamped-linear gate active fraction $\in [0.15, 0.85]$.
3. **Step 2 — Probe ladder (c) 2,000-step two-seed GPU run across all 5 arms (`10` runs = `5` arms $\times$ `2` seeds)**: Train for `2,000` steps on seeds `1` and `2`, logging `MarginHinge` loss, DEV `top1_correct` / `top8_correct` on `ft/dev` (`14,287` rows), Rung-1 parity held-out evaluation against stored `P` (`1.4175`, gap `0.0231` at Rung 1), orbit entropy $H(\text{orbit}) \in [0, \log_2 15 \approx 3.9069\text{ bits}]$, and slot coverage share over the $57{,}600$ slots.
4. **Step 3 — D11 integer export & export-gap verification (`CHARTER.md` §3.6, §3.7, §4.8)**: Export each trained checkpoint to `UORLUT01` via `d11-exporter-float-trunk`, verify `scripts/audit_zero_matmul_serving.py --stack` passes (`0` float / multiply / divide instructions), measure float-vs-integer export gap on the 14,287-row DEV panel, and measure integer decode and context `tokens/s` (lead metric per `CHARTER.md` §1).
5. **Step 4 — Fresh transfer verification (`fresh_v2`, `>= 2,000` rows)**: Run once if and only if a treatment arm passes all DEV bars on both seeds.

## Instrument Checks

Before calling any mechanism outcome (`CHARTER.md` §4.2), all six instrument checks must pass:
1. **`I-1` (Exact $E_8$ decode & $\mathrm{Cl}(0,6)$ orbit bijection on CUDA)**: On 20,000 seeded 8D queries, CUDA $O(8)$ `e8_nearest` matches brute-force 240-root argmax with `0` mismatches, and `ORBIT_MEMBERS[15][16]` covers all 240 roots bijectively (`15 * 16 = 240`).
2. **`I-2` (CUDA `e8_select` top-$k$ exactness at $k = 8$ and $k = 16$)**: On 256 seeded `(row, head)` queries with and without per-orbit offsets $\delta_{h,o,m}$, CUDA `e8_select` matches an exact brute-force $16 \times 16 = 256$ product-orbit sort on all `k = 8` and `k = 16` slots (`0` slot-id mismatches, `< 1e-6` score diff), with `0` host-device copies (`to_vec1`) in the forward/backward loop.
3. **`I-3` (Clamped-linear gate non-saturation & finite gradients)**: At step 0 and every 250 steps, active gate fraction $\Pr(0 < \beta_h s_{h,r} + \gamma_h < 1) \in [0.15, 0.85]$, `NaN`/`Inf` count is `0`, and finite-difference vs. analytic backward error on $R[h][r]$, $\beta_h$, $\gamma_h$, $\delta_{h,o,m}$, and $V[p]$ is $< 10^{-3}$. If active gate fraction falls outside $[0.05, 0.95]$, classify as **`instrument` (gate scale/shift initialization trap)** and re-center $(\beta_h, \gamma_h) = (1/\sigma_s, 0.5)$ before scoring any mechanism arm.
4. **`I-4` (Voronoi cell preservation under $\delta_{h,o,m}$)**: Verify $\|\delta_{h,o,m}\|_2 \le 0.5 < \frac{1}{2}\min_{u \neq v}\|r_u - r_v\|_2 = \frac{1}{2}$ at every step so no shifted orbit root crosses into another root's Voronoi cell.
5. **`I-5` (Routing health — orbit entropy & slot coverage)**: Mean orbit entropy per half must remain $\ge 3.50$ bits (of $\log_2 15 = 3.9069$ bits; Rung 1 measured `3.85–3.87` bits) and cumulative read share across $57{,}600$ slots must be $\ge 0.70$ (Rung 1 measured `0.83–0.87`). Below `3.0` bits is classed as routing collapse.
6. **`I-6` (D23 / D25 / D37 static & binary hygiene)**: `uor-harness ratchet d23` and `d25` report `0` violations in `crates/uor-r4-float-train`, and `scripts/audit_zero_matmul_serving.py --stack` passes on the exported integer artifact.

## Bar

Let $\sigma_{\text{seed}}$ be the two-seed pooled standard deviation across seeds `1` and `2` ($\sigma_{\text{seed}} = \max(\text{measured pooled SD},\ 0.0054\text{ on Rung-1 scale},\ 59\text{ rows on 14,287-row DEV top-1})$ per `CHARTER.md` §4.3). Every result leads with **integer `tokens/s`** (decode and context separately, `CHARTER.md` §1) followed by DEV `top1_correct` and the Rung-1 gap comparison:

1. **`G-Speed` (Lead metric — integer serving & GPU training throughput, `CHARTER.md` §1)**: Exported D11 integer decode `tokens/s` for `E8h_sigfree_k16_offset` is $\ge 1.0\times$ `C_hamilton_trunk` (converting the $10\times$ FFN arithmetic reduction into measured throughput rather than Rung-1's host-CPU slowdown), and GPU training throughput is $\ge 10\times$ Rung-1 CPU (`1,330 tok/s`).
2. **`G-Gap-to-P` (Closing the `0.0231` learned-key gap, `#2822` done-when)**: On the Rung-1 parity evaluation across both seeds `1` and `2`, the gap from `E8h_sigfree_k16_offset` to stored learned-key reference `P` (`1.4175`) narrows from Rung-1's `+0.0231` to $\le 2\,\sigma_{\text{seed}}$ ($\le 0.0108$), **and** the $2 \times 2$ factorial (`k8_fixed`, `k16_fixed`, `k8_offset`, `k16_offset`) reports the exact attribution of the `0.0231` gap between slot bandwidth $\Delta_{k:8\to 16}$ and per-orbit residual offset $\Delta_{\delta}$.
3. **`G-DEV-Lawful` (M1/M4 lawful count-table & no-memory control bar, `CHARTER.md` §4.4, §4.8, L-010)**: Through the D11 integer export on each of two seeds (`seed = 1`, `seed = 2`) on the 14,287-row DEV panel:
   - `top1_correct(E8h_sigfree_k16_offset) > 6,300 / 14,287` (beats T2 successor table `6,300` and unigram `1,226`);
   - `top1_correct(E8h_sigfree_k16_offset) >= top1_correct(C_hamilton_trunk) - 2 * SE_dev` (retains full FFN quality at $10\times$ fewer FFN ops);
   - Float-to-D11-integer export gap is $\le \sigma_{\text{seed}}$ (`CHARTER.md` §3.7).

| Outcome | Condition Across Seeds `1` and `2` | Classification & What It Unlocks |
|---|---|---|
| **`PASS_REOPEN_E8_ORBIT_MEMORY`** | `I-1..I-6` pass and `E8h_sigfree_k16_offset` meets `G-Speed`, `G-Gap-to-P` (gap to `P` $\le 0.0108$), and `G-DEV-Lawful` (beats T2 `6,300`, matches `C_hamilton_trunk` within noise, export gap $\le \sigma_{\text{seed}}$, positive `fresh_v2` transfer) | **Mechanism positive (`e8-orbit-memory` promoted from `PARKED` to `READY` / `ACTIVE` candidate)**. Closes `#2822` and satisfies all four reopen conditions of `docs/mechanisms/e8-orbit-memory.md:82-87`. |
| **`KEEP_AND_ITERATE_PARTIAL_GAP`** | `I-1..I-6` pass and `E8h_sigfree_k16_offset` improves over `E8h_sigfree_k8_fixed` by $> 2\,\sigma_{\text{seed}}$ and stays within `0.033` of `C_hamilton_trunk`, but retains a gap $> 2\,\sigma_{\text{seed}}$ to stored `P` | **Diagnostic narrowing (`CHARTER.md` §4.1)**: closes `#2822`'s measurement with exact factorial decomposition $(\Delta_{k:8\to 16}, \Delta_{\delta})$ of the `0.0231` gap recorded in `docs/research/notebook/exact-memory.md`; mechanism remains `PARKED` with updated reopen bar. |
| **`CLOSE_CONFIG_NO_GAIN`** | `I-1..I-6` pass, but neither $k: 8 \to 16$ nor per-orbit offset $\delta_{h,o,m}$ improves over `E8h_sigfree_k8_fixed` outside $2\,\sigma_{\text{seed}}$ | **Configuration negative (`CHARTER.md` §4.1)** on `(clamped-linear gate, k=16, per-orbit R^8 offset)`; closes this configuration and `#2822` without closing the `e8-orbit-memory` mechanism. |
| **`INSTRUMENT_STOP`** | Any of `I-1..I-6` fails or D11 export gap exceeds $2\,\sigma_{\text{seed}}$ | **Instrument defect (`CHARTER.md` §4.2)**; repair kernel, gate initialization, or quantizer scale before calling any mechanism result (`0` mechanism verdicts). |

## Minimum test power

- **Two independent seeds (`seed = 1`, `seed = 2`)** across all 5 GPU arms (`10` training runs total), scored on the `14,287`-row `ft/dev` panel (`2\text{SE} \approx 59` rows) and the `256`-block Rung-1 parity panel (`pooled_sd = 0.0054`, $2\,\sigma_{\text{seed}} = 0.0108$, so Rung-1's `0.0231` gap is $4.28\times$ the noise SD — well above minimum detectable effect), plus a single sealed evaluation on `fresh_v2` (`>= 2,000` rows) if DEV bars pass (`CHARTER.md` §4.3).

## Cost

- **Hardware & budget**: `1x NVIDIA GeForce RTX 5090` via `scripts/pod/uor-pod` inside the global fleet cap (`<= 4` pods, `<= $8.00/h`, `CHARTER.md` §5.4).
- **Projected wall time & spend**: Step 0 + Step 1 probe ladder takes `~5 min` (`~$0.05`). Step 2 (`5` arms $\times$ `2` seeds $\times$ `2,000` steps at `width = 256`, `batch = 16`, with on-device CUDA `e8_select` and `e8_memory_read`) takes `~6–8 min` per run (`~70 min` total $\approx 1.2$ GPU-hours), plus `~10 min` for Step 3 D11 export and DEV scoring $\approx$ **`$0.75–$1.10` total** at `$0.563/GPU/h` (hard cost ceiling `$2.50`).

## Stop conditions

1. **Immediate instrument stop**: Halt at Step 0 or Step 1 if any of `I-1..I-6` fails, if CUDA `e8_select` mismatches brute-force selection, or if Step 1 throughput projects $> 30$ minutes per 2,000-step arm (`CHARTER.md` §4.2).
2. **Mid-run divergence / collapse stop (`step = 500`)**: Halt any arm at step 500 if `NaN`/`Inf` appears or if orbit entropy drops below `2.50` bits while step-size calibration is already within bounds.
3. **Dispatch hold (D67 / D68)**: Do not rent a GPU pod until:
   - a code-scoped unit lands the sigmoid-free `e8_memory` module and CUDA kernels in `crates/uor-r4-float-train` (`#2823`); and
   - [`d11-exporter-float-trunk`](../../roadmap/items/d11-exporter-float-trunk.md) lands the D11 integer exporter required by `CHARTER.md` §3.7 and `docs/mechanisms/e8-orbit-memory.md:42, 85-87`.

## TEST FITNESS

TEST FITNESS: FIT - research #2822 e8-orbit-memory
sheet: docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md
reviewer: antigravity-subagent-non-author-r1 (c5fc78f4-35da-4fa3-b450-d6316fc72d3c; https://github.com/UOR-Foundation/uor-r4/issues/2822#issuecomment-6398487538)
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
