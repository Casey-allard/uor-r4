# Secant Surrogate Descent Check for Recurrence Gates and SpiralCore/FBS — 2026-10-25

- **Roadmap item:** [`r-secant-surrogate-descent`](../../roadmap/items/r-secant-surrogate-descent.md) (#2886), prerequisite for [`hamilton-gate-offline-float-sheet`](../../roadmap/items/hamilton-gate-offline-float-sheet.md) (#2824), [`recurrence-gates-six-recurrence-fit`](../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (#2890), and [`spiralcore-fbs-offline-float-sheet`](../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (#2891).
- **Lab:** `antigravity`. **Milestone:** `research` (offline float training gate under `CHARTER.md` §3.7 / D37).
- **Dependency gate:** [`float-training-module`](../../roadmap/items/float-training-module.md) (#2823, `crates/uor-r4-float-train`, merged), the separate scanned D37 offline-float crate carrying the 24-kernel extract with zero dependency on `crates/uor-r4-training` (`CHARTER.md` §3.7).

## 1. Question

Do the two declared geometric secant surrogates — (A) `rate_select_bwd` on the 16-entry dyadic fade/keep table (`crates/uor-r4-float-train/src/cuda_stack.cu:259-291`, `geometric_stack_native.rs:1480-1566`) for `RecGates::Geometric`, and (B) `spiralcore_fbs_bwd` on the 16-root $E_8$ anchor orbit and base-3 FBS binder tree (`crates/uor-r4-float-train/src/spiralcore_fbs.cu:124-183`, `spiralcore_fbs.rs`) for `QuaternionWeights::SpiralcoreFbs` — produce genuine loss descent under `Objective::MarginHinge { rivals: 8 }` (`kept_cells = 8`) across two seeds (`seed in {1, 2}`), beating both a zero-credit frozen-gate/query control and a matched-norm random-sign surrogate control once step-size and table-plateau instrument faults are ruled out?

## 2. Mechanisms, arms, steps, and minimum test power (defined exactly)

Both tests run on a leased `uor-pod` RTX 5090 inside the separate scanned D37 module `crates/uor-r4-float-train` (#2823) at the rung-0 micro-probe shape (`pattern = "rrrrrr"`, `width = 256`, `heads = 4`, `mlp_hidden = 384`, `batch = 4`, `context = 192`, `vocab = 4096`, `kept_cells = 8`, `Objective::MarginHinge { rivals: 8 }`, `Gate::Hamilton`, `256` optimizer steps cycling `4` epochs over the pinned 64-batch / 256-row diagnostic slice of `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16`, seeds `1` and `2`, zero softmax/squashing/matmul). Within each seed $s \in \{1, 2\}$, all arms start from the identical initial model state $L_0^{(a,s)} = L_0^{(s)}$; to prevent the 1,048,576-parameter `embedding.weight` skip-connection and global gradient clipping (`cuda_gradient_clip`) from confounding the tested secant surrogates, Part A updates only the recurrence rate parameters (`layers.*.rec.decay`, whose backward path flows exclusively through `rate_select_bwd`) and Part B updates only the SpiralCore/FBS query tensors (`layers.*.*.spiralcore_fbs`, whose backward path flows exclusively through `spiralcore_fbs_bwd` and downstream layers' exact inverse input adjoint $A_o^{-1} d y_o$), holding all other parameters fixed at their seed-$s$ initialization:

### Part A — Recurrence-gate dyadic table secant (`rate_select_bwd`)
Forward selects $i = \min(\lfloor 16 s \rfloor, 15)$ for clamped opening $s = \mathrm{clamp}(g + d, 0, 1)$, emitting $\lambda_i = 2^{-(i+1)}$ (`RATE_TABLE`) and $\kappa_i = \sqrt{1 - \lambda_i^2}$ (`KEEP_TABLE`). Backward computes:
$$d s = 16 \cdot \frac{1}{|\mathcal{N}(i)|} \sum_{j \in \{i-1, i+1\} \cap [0, 15]} \mathrm{sgn}(j - i)\Big(d\lambda\cdot(\lambda_j - \lambda_i) + d\kappa\cdot(\kappa_j - \kappa_i)\Big).$$

| Arm | Definition | Seeds | Steps |
|---|---|---:|---:|
| `RecSecant_Active` | `RecGates::Geometric` with `rate_select_bwd` active on all recurrence layers (`layers.*.rec.decay` updated by `StackAdamW`). | `1, 2` | `256` |
| `RecZeroCredit_Null` | Identical forward (`rate_select_fwd`), but $d s \equiv 0$ (`rate_select_bwd` zeroed so rate parameters receive zero table credit and stay frozen at initialization, $\Delta L = 0$). | `1, 2` | `256` |
| `RecRandomRefit_Null` | Identical forward, but $d s$ is multiplied per lane by an independent deterministic Rademacher sign $\xi \in \{-1, +1\}$ (preserving $\|d s\|_2$ and $\|d(\text{rec.decay})\|_2$ while destroying secant direction). | `1, 2` | `256` |

### Part B — SpiralCore/FBS root-edge secant (`spiralcore_fbs_bwd`)
Forward routes each query $q_{o,m} \in \mathbb{R}^8$ to the nearest root $r_{\text{goal}(m)}$ of the 16-root $E_8$ orbit, composing a 64-state $\mathrm{Cl}(0,6)$ signed-axis action $A_o$ applied to leaf atom $x_{\sigma(o)}$: $y_o = A_o x_{\sigma(o)}$. Backward resolves counterfactual sibling displacements along the root edge:
$$d q_{o,m} \mathrel{+}= \sum_{b \neq c_m} \frac{\langle d y_o,\, y_o^{(m \leftarrow b)} - y_o \rangle}{\|r_{\text{goal}(m,b)} - r_{\text{goal}(m,c_m)}\|_2^2} \Big(r_{\text{goal}(m,b)} - r_{\text{goal}(m,c_m)}\Big).$$

| Arm | Definition | Seeds | Steps |
|---|---|---:|---:|
| `FbsSecant_Active` | `QuaternionWeights::SpiralcoreFbs` with `spiralcore_fbs_bwd` query secant active and exact inverse input adjoint $A_o^{-1} d y_o$ (`layers.*.*.spiralcore_fbs` updated by `StackAdamW`). | `1, 2` | `256` |
| `FbsZeroCredit_Null` | Identical forward and exact input adjoint $A_o^{-1} d y_o$, but $d q_{o,m} \equiv 0$ (queries frozen at initialization, $\Delta L = 0$). | `1, 2` | `256` |
| `FbsRandomRefit_Null` | Identical forward and exact input adjoint, but $d q_{o,m}$ is multiplied per 8D block by a deterministic orthogonal Rademacher sign-flip $\mathrm{diag}(\pm 1, \dots, \pm 1) \in O(8)$ of matched Euclidean norm. | `1, 2` | `256` |

## 3. Instrument checks (`CHARTER.md` §4.2 — run before calling any mechanism result)

1. **`I-Finite-CUDA`:** Device error flag `error == 0` on every step (`spiralcore_fbs_bwd` error codes `1..4` all zero), zero `NaN`/`Inf` in `d_score`, `d_queries`, `d_input`, and `MarginHinge` loss, and step-0 $\|d s\|_2 > 0$ and $\|d q\|_2 > 0$.
2. **`I-Boundary-Crossings` (step-size vs. cell-width check):** Over 256 steps, at least $1.0\%$ of active recurrence lanes must cross a dyadic table threshold ($\Delta i \neq 0$) in `RecSecant_Active`, and at least $1.0\%$ of active `(output, stage)` queries must change selected $E_8$ root ($\Delta c_m \neq 0$) in `FbsSecant_Active`. If $< 1.0\%$ cross a boundary, the hard selection never changed state — classed **`instrument` (learning rate below cell width)**, not a mechanism negative; calibrate the gate/query learning-rate multiplier so updates cross $\ge 1.0\%$ of boundaries before scoring `G-Rec-Descent` / `G-Fbs-Descent`.
3. **`I-Keep-Plateau` (`f32` precision diagnostic for `KEEP_TABLE[12..15]`):** In `KEEP_TABLE`, entries $i \in \{12, 13, 14, 15\}$ all equal `1.0f` in `f32` because $1 - 2^{-26} < \mathrm{ulp}(1.0\text{f})$, making $\kappa_j - \kappa_i = 0$ on the keep channel for $i \ge 12$. Log the histogram of selected indices $i \in 0..16$ at steps $0, 64, 128, 256$; if $> 80\%$ of lanes initialize or trap in $i \ge 12$ with $|d\lambda| < 10^{-7}$, classify as **`instrument` (`f32` keep-table plateau)** and test with exact analytic $\Delta\kappa_i \approx -\frac{1}{2}\Delta(\lambda_i^2)$ before calling a mechanism verdict.

## 4. Bars and decision table

Let $L_t^{(a,s)}$ be the 256-row `MarginHinge(rivals=8)` loss of arm $a$ on seed $s \in \{1, 2\}$ at step $t \in \{0, 256\}$, let $\Delta L^{(a,s)} = L_0^{(a,s)} - L_{256}^{(a,s)}$ be the paired 256-step loss reduction from the shared seed-$s$ initialization $L_0^{(a,s)} = L_0^{(s)}$, and let the two-seed noise band for each comparison be $\Delta_{\text{noise}} = \max_{a} |\Delta L^{(a,1)} - \Delta L^{(a,2)}|$ (`CHARTER.md` §4.3).

| Gate | Bar | Unlocks on PASS | Action on FAIL (with `I-1..3` green) |
|---|---|---|---|
| `G-Rec-Descent` | On both seeds $s \in \{1, 2\}$: $L_{256}^{(\text{RecSecant},s)} < L_0^{(\text{RecSecant},s)}$ AND $L_{256}^{(\text{RecSecant},s)} \le \min\big(L_{256}^{(\text{RecZero},s)}, L_{256}^{(\text{RecRand},s)}\big) - \Delta_{\text{noise,Rec}}$ | Unblocks [`recurrence-gates-six-recurrence-fit`](../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (#2890) and [`hamilton-gate-offline-float-sheet`](../../roadmap/items/hamilton-gate-offline-float-sheet.md) (#2824). | Closes the 1-step ($1/16$) neighbour secant configuration (`CHARTER.md` §4.1); holds `#2890` and `#2824` until a multi-step secant or discrete counterfactual rate selector is substituted. |
| `G-Fbs-Descent` | On both seeds $s \in \{1, 2\}$: $L_{256}^{(\text{FbsSecant},s)} < L_0^{(\text{FbsSecant},s)}$ AND $L_{256}^{(\text{FbsSecant},s)} \le \min\big(L_{256}^{(\text{FbsZero},s)}, L_{256}^{(\text{FbsRand},s)}\big) - \Delta_{\text{noise,Fbs}}$, with step-256 route flip rate $< 0.50$ | Unblocks [`spiralcore-fbs-offline-float-sheet`](../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (#2891). | Closes the unnormalized root-edge secant configuration (`CHARTER.md` §4.1); holds `#2891` and keeps SpiralCore/FBS on the ACTIVE discrete counterfactual path (`crates/uor-r4-discrete`). |

## 5. Base artifact, data split, checkpoints, and pins

- **Training slice:** First `256` rows (`64` batches of `4` rows, `context = 192`, contiguous non-overlapping causal windows `r * 192 .. r * 192 + 193` after the 64-byte `UORT` header) of `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16` (SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`) and `w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8` (SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`), tokenized with `w2-inputs/2026-10-25/driver-data/tokenizer.json` (SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, `vocab_size = 4096`) in `caseyallard/uor-r4-store`.
- **Source module:** Runs inside `crates/uor-r4-float-train` (#2823) with its own scanned allowlist entry (`CHARTER.md` §3.7, D37 item 6) and `cargo tree -p uor-r4-float-train` verified free of `uor-r4-training`.

## 6. Cost, stop rule, and dispatch gate

- **Cost:** 1 leased `uor-pod` RTX 5090 instance (`--count 1 --gpu 5090`) inside the global `<= 4` pods and `<= $8.00/h` cap (`CHARTER.md` §5.4), `12` runs (`6` arms $\times$ `2` seeds $\times$ `256` steps at micro-probe shape) replicated across two independent `nvcc` builds, `<= 25` minutes total wall time, target `~$0.60` (`<= $1.20` hard stop).
- **Stop rule:** Stop immediately if `I-Finite-CUDA` fails (`error != 0` or `NaN`/`Inf`), or if after one learning-rate calibration pass `I-Boundary-Crossings` remains $< 1.0\%$.
- **Dispatch gate:** [`float-training-module`](../../roadmap/items/float-training-module.md) (#2823) is merged on `main`; runs on the lab's own D67 pre-run review after fresh-context `TEST FITNESS: FIT` and D68 decision review on the reconciled sheet.

## 7. Non-author `TEST FITNESS` review (`CHARTER.md` §4.5)

TEST FITNESS: FIT

- Reviewed by fresh-context non-author reviewer (`antigravity` subagent `1fc66c6c-04f7-4fda-ba1e-02d53c860756`, [`#2886#issuecomment-6395161953`](https://github.com/UOR-Foundation/uor-r4/issues/2886#issuecomment-6395161953), 2026-10-25); re-reviewed after pre-run pin/rivals reconciliation in `antigravity-77`.
