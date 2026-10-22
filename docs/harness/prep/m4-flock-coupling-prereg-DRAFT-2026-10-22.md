# M4 C1 — Float-Trained Flock Coupling Exported as Integer Gate Weights (`m4-flock-coupling-prereg`, DRAFT)

- **Status:** Complete specification and re-derived finite-control sheet (**PROPOSED, NOT REGISTERED**; `TEST FITNESS: PENDING` awaiting PM/owner registration of the 5 decisions in [§7](#7-open-pm--owner-decisions-blocking-registration) and [`#2860` comment `6352562539`](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539)).
- **Supersedes draft:** [`m4-c1-flock-coupling-DRAFT-2026-10-18.md`](./m4-c1-flock-coupling-DRAFT-2026-10-18.md) (retained byte-identical).
- **Milestone / Line:** M4 (`#2032`, item [`m4-flock-coupling-prereg`](../../roadmap/items/m4-flock-coupling-prereg.md)); mechanism [`flock-read.md`](../../mechanisms/flock-read.md); notebook [`m1-discrete-attention.md`](../../research/notebook/m1-discrete-attention.md).
- **Lab:** `antigravity` (`antigravity-25`).
- **Pinned source & geometry:** `2987514aa` (`crates/uor-r4-discrete/build.rs`, `OPERATOR_KAPPA = blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b`, `COMPOSITION_KAPPA = blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0`).
- **Host control verification:** [`crates/uor-r4-discrete/tests/flock_coupling_controls.rs`](../../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs) (`7/7` tests PASS).

## 1. Mechanism (`ReadSpec::FlockCoupled { kept, k_max }`)

Today's discrete Flock read ([`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu), `snap` at lines `102–110`, `flock_vector` at lines `125–267`) computes, per row and 8D payload block $b \in \{0, \dots, B-1\}$:

$$u_b = \text{query payload root (8 coords)}, \qquad m_{b,\ell} = \operatorname{snap}_{\text{found}_\ell}\!\left(\sum_{k=0}^{\text{found}_\ell - 1} \operatorname{action}(\text{BFS}[\text{frame}_k][f_q], \text{payload}_k)\right),$$

$$\text{out}_b = \operatorname{snap}_{\text{terms}}\!\left(u_b + \sum_{\ell:\,\text{found}_\ell > 0} g_{b,\ell}\,m_{b,\ell}\right), \qquad g_{b,\ell} \in \{-1, 0, +1\}, \qquad \text{terms} = 1 + \sum_{\ell:\,\text{found}_\ell > 0} |g_{b,\ell}|,$$

$$\operatorname{snap}_n(x) = \operatorname*{argmin}_{r \in \{0,\dots,15\}} \bigl(L_1(x - n\cdot r),\ \operatorname{Hamming}(x, n\cdot r),\ r\bigr).$$

**C1 changes only the coupling:** $g_{b,\ell}$ becomes a learned coupling $K_{b,\ell}$:

- **Training (offline, float, D37):** $K_{b,\ell} \in \mathbb{R}$ (`20` scalars `GateCombine(0..19)` at `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`).
- **Serving (exported, integer, D11):** $K_{b,\ell} \in \{-k_{\max}, \dots, +k_{\max}\}$ ($k_{\max} \in \{1, 2, 3\}$), applied by $|K_{b,\ell}|$ repeated additions of $\pm m_{b,\ell}$, with $\text{terms} = 1 + \sum_{\ell:\,\text{found}_\ell > 0} |K_{b,\ell}|$. A level with no eligible candidate ($\text{found}_\ell = 0$) adds nothing and does not increment $\text{terms}$ (`discrete.cu:250`, which executes `continue` before `++terms` at `:260`).

All other components—neighbor selection, lexicographic rank order, BFS transport, per-level consensus $m_{b,\ell}$, the writer, and the token/route tables—remain fixed and integer at the base checkpoint.

## 2. Offline Float Boundary and Banned-Token Scan (D11, D23, D24, D25, D29, D30, D32, D37)

- **Isolation:** Floating-point arithmetic exists only in an offline GPU training module with ahead-of-time `nvcc`-compiled PTX (zero NVRTC) and its own D29 scanner allowlist. `crates/uor-r4-discrete`'s integer PTX admission (`build.rs:36–98`) and D32 `discrete-float` ratchet (`0` sites) remain untouched.
- **No matmul (D25/D30):** The float read computes elementwise scalar-root scaling $y_b(K) = u_b + \sum_{\ell:\,\text{found}_\ell > 0} K_{b,\ell}\,m_{b,\ell}$ and $n_b(K) = 1 + \sum_{\ell:\,\text{found}_\ell > 0} |K_{b,\ell}|$ over `20` scalar couplings (`4` blocks $\times$ `5` stages). No weight matrix, inner product, or GEMM exists.
- **No squashing or exponential (D23):** The module's sources and PTX are scanned for and must contain zero occurrences of:
  `exp`, `expf`, `__expf`, `log`, `logf`, `softmax`, `tanh`, `sigmoid`, `erf`, `pow`, `wmma`, `mma`, `cublas`, `cutlass`, `cudnn`.
- **Integer export boundary (D11):**
  $$K_{\text{int}} = \operatorname{clamp}\bigl(\operatorname{round\_half\_away\_from\_zero}(K), -k_{\max}, +k_{\max}\bigr).$$
  Served in `ReadSpec::FlockCoupled { kept, k_max }` beside `ReadSpec::Flock { kept }`, with rollback identity `k_max = 1 ≡ Flock` bitwise.

## 3. Piecewise-Linear Float Objective (`NearestAddressMarginV1` Relaxation)

Replacing each block's snapped read $q_b$ by the float mean $\bar{y}_b(K) = y_b(K) / n_b(K)$ in the writer's `NearestAddressMarginV1` cost (`discrete.cu:326–340`, $\text{width} = 8B$):

$$T(t; K) = \sum_{b=0}^{B-1} \|\bar{y}_b(K) - \text{table}_b(t)\|_1, \qquad A(c; K) = \sum_{b \in \{0, B-1\}} \|\bar{y}_b(K) - \text{cellroot}_b(c)\|_1,$$

$$H_\mu(d^*, d_{\text{rival}}) = \max(0, \mu + d^* - d_{\text{rival}}),$$

$$L(K) = \sum_{\text{rows}} \left[ 8\,H_{\text{width}}\!\left(T(\text{target}; K), \min_{t \ne \text{target}} T(t; K)\right) + \text{width}\,H_8\!\left(A(\text{target\_cell}; K), \min_{c \ne \text{target\_cell}} A(c; K)\right) \right].$$

- **First-order update with box projection:** Subgradient step followed by $K \leftarrow \operatorname{clamp}(K, -k_{\max}, +k_{\max})$ (required because the address margin $8$ exceeds the minimum inter-root $L_1$ distance $4$, so the hinge remains strictly decreasing for $K \ge 2$; control `F2`).
- **Pre-run TRAIN proxy-agreement instrument (§7.5):** On the `1,024`-row stratified TRAIN sample (`16` batches $\times 64$ rows), the float proxy's argmin token at the checkpoint's own trits must match the integer writer's `prediction` on at least the registered threshold (**proposed:** $\ge 0.70$). A lower rate halts as a D31 instrument stop, never a mechanism negative.

## 4. Re-Derived Finite Controls at `2987514aa` (`flock_coupling_controls.rs`)

Generated roots from `crates/uor-r4-discrete/build.rs` (`ROOTS`):
- $r_0 = a = A = [2, 2, 0, 0, 0, 0, 0, 0]$, $r_1 = -a = [-2, -2, 0, 0, 0, 0, 0, 0]$;
- $r_3 = -b = [0, 0, -2, 0, -2, 0, 0, 0]$, $r_4 = [0, 0, -2, 0, 2, 0, 0, 0]$, $r_{13} = [0, 0, 2, 0, -2, 0, 0, 0]$, $r_{14} = b = B = [0, 0, 2, 0, 2, 0, 0, 0]$;
- $\text{IDENTITY} = 0$, $\text{BFS}[0][0] = 0$, $\text{BFS}[0][14] = 1$.

**Fixture T′:** `tokens = [[A, A], [A, B]]`, `context = 2, kept_cells = 2, kept = 1`, `input = [identity(1, 0), identity(0, 1)]`, query = token 0 (`[A, A]`), context atom = token 1 (`[A, B]`), `Gate(0) = 0`. In block 1, $u = A = r_0$ and $m_0 = B = r_{14}$.

### 4.0 Cache Fidelity (`C0`)

| ID | Control | Literal Expectation |
|---|---|---|
| `C0` | Cached replay equals live forward | On the `1,024`-row stratified TRAIN sample (and on T′ across all $K \in \{-3,\dots,+3\}^2$), cached-tuple integer replay equals live `discrete_forward` bitwise in both read vectors and writer loss (`0` disagreements). Any row with `ABSENT` $u_b$ or `ABSENT` $m_{b,\ell}$ when $\text{found}_\ell > 0$ refuses with status `8`. |

### 4.1 Integer Export Kernel (`FlockCoupled`, `X1`–`X12`)

| ID | Control | Re-Derived Literal Expectation at `2987514aa` |
|---|---|---|
| `X1` | Zero coupling is inert | $K = 0 \implies \text{out} = [A, A]$, $\text{terms} = 1$, bitwise equal to `Flock` with all gates $0$. |
| `X2` | Trit embedding ($k_{\max}=1 \equiv \text{Flock}$) | For $K \in \{-1, 0, +1\}$, vectors, losses, and full trace match `Flock` bitwise on T′ and on all 6 `flock_qualification.rs` fixtures (`flock_qualification.rs:349–356`). On T′: $+1 \to \operatorname{snap}_2(a+b)$ ties $r_0, r_{14}$ at $(d_{L1}=8, d_H=4) \to r_0 = A$; $-1 \to \operatorname{snap}_2(a-b)$ ties $r_0, r_3$ at $(d_{L1}=8, d_H=4) \to r_0 = A$; $\text{out} = [A, A]$ with $\text{terms} = 2$. |
| `X3` | Magnitude flips the read | $K = +2 \implies \text{residual} = [2,2,4,0,4,0,0,0]$, $n = 3$, $D(r_{14}) = 8$ unique minimum ($D(r_0) = D(r_4) = D(r_{13}) = 16$, others $\ge 20$). $\text{out} = [A, B]$, $\text{terms} = 3$. |
| `X4` | Signed magnitude | $K = -2 \implies \text{residual} = [2,2,-4,0,-4,0,0,0]$, $n = 3$, $D(r_3) = 8$ unique minimum. $\text{out} = [A, -B]$ ($-B = r_3$), $\text{terms} = 3$. |
| `X5` | Saturation | $K = +3 \implies \text{residual} = [2,2,6,0,6,0,0,0]$, $n = 4$, $D(r_{14}) = 8$, $D(r_4) = D(r_{13}) = 20$, $D(r_0) = 24$. $\text{out} = [A, B]$, $\text{terms} = 4$. |
| `X6` | Typed refusals | $|K| > k_{\max}$, $k_{\max} \notin \{1,2,3\}$, or gate count $\ne B\cdot S$ refused with status `10`. Neither `Route` nor `Flock` accepts $|g| > 1$ (status `10`). |
| `X7` | Counterfactual lanes (exact writer losses) | On T′ with `Gate(1) = 0`, $k_{\max} = 3$, and target token `1` (`[A, B]`, cell `14`) against rival token `0` (`[A, A]`, cell `0`), the $2k_{\max}+1 = 7$ active ordinals in order `[CURRENT=0, -1, +1, -2, +2, -3, +3]` yield exact writer losses **`[448, 448, 448, 256, 64, 256, 64]`** (with lanes `7..15` at `u64::MAX`). $K=+1$ stays at `448` (`tm=24, am=16`), $K \in \{-2,-3\}$ drops to `256` (`tm=16, am=8`), and $K \in \{+2,+3\}$ drops to `64` (`tm=8, am=0`); `discrete_choose` picks ordinal `4` ($K = +2$, loss `64`). |
| `X8` | Empty level does not count | On the C7 two-stage length-3 fixture (`context=3, kept_cells=3, kept=2`, `tokens=[[A,A],[NEG_A,A],[A,B]]`, `input=[0,1,2]`), binder node `3` covers atoms `0..=2` ($\text{last}[3]=2 \not< 2$), so level `1` has $\text{eligible}=0, \text{found}_1=0$. Setting `gates = [0, +2, 0, +2]` leaves $\text{terms} = 1$, $n_b = 1$, and $\text{out} = [A, B]$ (`head(0)=[14,0,1,0]`, `head(1)=[0,14,1,14]`) unchanged. |
| `X9` | Two levels at once | On the two-stage length-4 fixture (`context=4, kept_cells=2, kept=1`, `tokens=[[A,A],[A,B]]`, prefix `[0, 0, 1, 0]` = `[[A,A],[A,A],[A,B],[A,A]]`), block `1` has $u = a = r_0$, level-0 recency-winner atom `2` gives $m_0 = b = r_{14}$, and level-1 binder node `3` ($\operatorname{snap}_3(a+a+b)=r_0$) gives $m_1 = a = r_0$. With $K_{1,0}=+2, K_{1,1}=-1$ (`gates = [0, 0, 2, -1]`): $\text{terms} = 4$ and residual $u + 2m_0 - m_1 = [0,0,4,0,4,0,0,0]$, which snaps uniquely to $r_{14} = B$ ($D(r_{14})=8, D(r_4)=D(r_{13})=16, D(r_0)=24$), output `[A, B]`. |
| `X10` | $n = 16$ bound | With $k_{\max} = 3$ and $5$ stages, $\text{terms} = 16$, $|16\cdot 2| = 32$. Residual $a + 15b = [2,2,30,0,30,0,0,0]$ at $n = 16$ snaps uniquely to $r_{14} = B$ ($D(r_{14}) = 8$, $D(r_4) = D(r_{13}) = 68$, $D(r_0) = 120$). |
| `X11` | Relation trits stay trits | Under a Relation variant (`4` blocks, `5` stages), the `20` combine gates validate in $[-k_{\max}, k_{\max}]$ while the `25` relation trits validate in $[-1, 1]$ (status `10` otherwise). |
| `X12` | Trace encoding | Level trace word `0` is $\text{gate} + k_{\max} \in \{0, \dots, 2k_{\max}\}$ under `FlockCoupled` (avoiding `ABSENT` collision at $K = -2$ and matching $\text{gate} + 1$ at $k_{\max} = 1$). |

### 4.2 Offline Float Training Module (`F1`–`F7`, `F5b`, `F5c`)

| ID | Control | Re-Derived Literal Expectation at `2987514aa` |
|---|---|---|
| `F1` | Loss and subgradient at $K = 1$ | On T′ block 1 ($u = a, m = b, r^* = B$): at $K = 1, n = 2$, $D(B) = 4, D(a) = 4$, $L = 8$, $\partial L/\partial K = -16/n^2 = -4$; one SGD step with $\eta = 1/8$ gives $K = 1.5$. |
| `F2` | Box projection at boundary | For $K \ge 2$, $L(K) = 8 - 4K/(1+K)$ and $\partial L/\partial K = -4/(1+K)^2 < 0$. At $K = 3$ ($k_{\max}=3$): $L = 5, \partial L/\partial K = -1/4$; step + clamp returns $K = 3$. At $k_{\max} = 1$ from $K = 1$, step + clamp returns $K = 1$. |
| `F3` | Sign symmetry | Target $r^* = -B$ ($r_3$), start $K = -1, n = 2$: $D(r_3) = 4, D(a) = 4$, $L = 8, \partial L/\partial K = +4$; step with $\eta = 1/8$ gives $K = -1.5$. |
| `F4` | Float/integer agreement | At $K = 2.0$ ($n = 3$): $D(B) = 8/3$ vs $D(a) = 16/3 \implies$ float output $B$ (`14`); export $K_{\text{int}} = 2 \implies$ integer output $B$ (`14`); disagreement `0`. |
| `F5` | Export-gap witness | At $K = 1.4 = 7/5$ ($n = 12/5 = 2.4$): $D(B) = 10/3 = 3.333\dots$, $D(a) = 14/3 = 4.666\dots$, $D(r_4) = D(r_{13}) = 17/3 = 5.666\dots \implies$ float output $B$ (`14`). Export $K_{\text{int}} = 1 \implies$ integer output $A$ (`0`, `X2` tie). Block 1 reports `1` disagreement and block 0 reports `0`. |
| `F5b` | Half-way rounding | $K = +1.5 \to +2$ and $K = -1.5 \to -2$ (`round_half_away_from_zero`). |
| `F5c` | Determinism | Repeated runs at fixed seed produce bitwise-identical $K$ vectors (fixed reduction order, zero unordered atomics). |
| `F6` | D29 banned-token scan | Scanner finds `0` occurrences of the 15 §2 tokens in module source and rejects a seeded `expf(` fixture line in unit test. |
| `F7` | D32 integer export boundary | Exported snapshot passes discrete `validate` under `ReadSpec::FlockCoupled` and loads into the integer path with `0` D32 float sites. |

## 5. Export-Gap Contract (D37 Item 3)

1. **Identical evaluator:** Float and exported integer candidates are scored on the same `14,287` DEV rows (`P_succ` `3,847` rows) in the same order using `eval-corpus` and `attention-cell-reference-scorer` (`T1 / T1b / T2 / U_succ / U_train`).
2. **Offline GPU float forward:** Evaluates real $K$ with real-vector `snap` on the GPU inside the D29-scanned float module, feeding the unchanged integer writer solely to measure the D37 item-3 float-minus-integer export gap. Never served and never used to promote a candidate.
3. **Export-limited flag:** If the float-minus-integer gap on `P_succ` conditional top-1 exceeds arm `A0`'s two-seed range, the candidate is flagged `EXPORT_LIMITED` (permitting at most one successor change to $k_{\max}$ or the rounding map before closing the configuration under D22).

## 6. Arms and Matched-Budget Comparison

| Arm | Coupling Learner | $k_{\max}$ | Isolates |
|---|---|---:|---|
| `A0` | Existing discrete trit proposal learner (`Gate` kind 3) | `1` | Reference trit baseline |
| `A1` | Discrete widened-ordinal counterfactual learner (`X7`, $2k_{\max}+1 = 7$ lanes) | `3` | Effect of coupling magnitude under discrete search |
| `A2` | Offline float-trained (§3), exported to integer via `round_half_away_from_zero` | `3` | Effect of continuous subgradient training vs `A1` |
| `A2-int` | `A2` exported integer couplings polished by `A1` counterfactual search under matched total budget | `3` | Proxy relaxation error vs integer quantization error |
| `A2-prime` (`A2′`) | Offline float-trained (§3), exported to trits | `1` | Float optimization vs discrete search at the trit ceiling (`A0`) |
| `Z` | Zero-coupling ablation ($K \equiv 0$) | `1` | Whether the Flock consensus read contributes over query payload $u_b$ |
| `R` | Random $K_{\text{int}}$ control matched to `A2`'s empirical $|K_{\text{int}}|$ distribution | `3` | Coupling magnitude without learned assignment |

## 7. Open PM / Owner Decisions Blocking Registration

Posted on [`#2860` (`6352562539`)](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539):

1. **D11 admissibility of $k_{\max} > 1$:** Confirm that $k_{\max} \in \{2, 3\}$ applied by $|K|$ repeated additions in `FlockCoupled` and `snap` (zero multiply instructions) is admissible under D11, or restrict to $k_{\max} = 1$.
2. **Line identity & status:** Confirm C1 remains wanted (not superseded) and whether it registers as the M4 starling/murmuration decisive run (`#2032`) or a sub-rung on `m1-discrete-attention` (`#2029`).
3. **Base checkpoint binding:** After the October Q1 diagnostic stops (`attention-rows-q1-diagnosis-2026-10-18` and `a8-dose-2026-10-19`), select whether C1 binds **(a)** the post-admission-guard A8 rerun checkpoint once `#2832` runs, or **(b)** the retained A8 `FrozenZero` 16,404-batch checkpoint (`faed2c0d…`) / initial A8 preflight checkpoint (`1,351 / 3,847` $P_{\text{succ}}$ admitted).
4. **D37 item-3 float export-gap forward:** Confirm the §5.2 offline GPU float forward satisfies D37 item 3.
5. **Registration numbers:** Confirm the proposed `C0` sample size (`1,024` stratified TRAIN rows, `0` disagreements), pre-run TRAIN proxy-agreement threshold ($\ge 0.70$), and numeric keep bar below.

## 8. Pre-Registration Blocks (PROPOSED, NOT REGISTERED; $0)

### Bar

**PROPOSED, NOT REGISTERED (awaiting PM/owner confirmation on `#2860`):**
To earn `KEEP`, an exported integer coupling arm (`A1`, `A2`, `A2-int`, or `A2-prime`) must beat the reference trit arm `A0`, the zero-coupling ablation `Z` ($K \equiv 0$), and the distribution-matched random control `R` on **both** registered seeds (`seed 1` and `seed 2`) on the `14,287`-row DEV panel (`P_succ` `3,847` rows) by at least:
- **Primary metric (`P_succ` conditional top-1 on `3,847` rows):** $\Delta_{\text{top1}}(P_{\text{succ}}) \ge \max\bigl(24\text{ rows},\ |A_0(\text{seed }1) - A_0(\text{seed }2)|\bigr)$ over `A0`, `Z`, and `R` on every seed;
- **Full-panel guard (`14,287` rows):** $\Delta_{\text{top1}}(\text{full DEV}) \ge \max\bigl(24\text{ rows},\ |A_0(\text{seed }1) - A_0(\text{seed }2)|\bigr)$ over `A0`, `Z`, and `R`, with target-admitted rows non-decreasing ($\text{admitted} \ge \text{admitted}(A_0)$);
- **Sealed fresh transfer:** Every `KEEP` candidate on DEV must also pass the sealed fresh transfer draw (`V2`) before any capability claim is made.

### Base artifact

**PROPOSED, NOT REGISTERED (awaiting PM checkpoint selection in §7.3):**
- **Executable / geometry base:** `origin/main` at `2987514aa` (`crates/uor-r4-discrete`, `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`, `ObjectivePolicy::NearestAddressMarginV1`, `OPERATOR_KAPPA = blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b`).
- **Candidate base checkpoints (PM to bind one SHA-256 at registration):**
  1. Post-admission-guard A8 decision checkpoint produced after `#2832` (`m1-a8-registration-and-runbook-staging`), or
  2. Retained A8 `FrozenZero` checkpoint from `docs/labs/a8-dose-2026-10-19/README.md` (plan `faed2c0d…`, initial $P_{\text{succ}}$ admission `1,351 / 3,847`, Q1 `857 / 3,847`).

### Data split

- **TRAIN:** Pinned discrete corpus (`2,425,925` eligible response-masked targets; `archive/labs/discrete-driver-cost-2026-10-13/README.md`).
- **Pre-run `C0` and proxy-agreement sample:** Stratified deterministic sample of `1,024` TRAIN target rows (`16` batches $\times 64$ rows, stride `151,620` across the `2,425,925` eligible TRAIN targets).
- **DEV:** Pinned `14,287` eligible development targets (`3,847` copy-successor `P_succ` rows and `10,440` complement `P_comp` rows).
- **Transfer (`V2`):** Sealed fresh held-out slice (`SEALED_PENDING_PAIRED_DEV_DECISION`), opened only once for an arm that passes the DEV bar.

### Steps

1. **Host finite controls ($0, completed in `antigravity-25`):** Run `cargo test -p uor-r4-discrete --test flock_coupling_controls --locked` (`7/7` tests verifying `C0`, `X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`).
2. **PM registration & non-author FIT:** Record PM answers to §7.1–§7.5, bind the exact base checkpoint SHA-256, and obtain a non-author `TEST FITNESS: FIT` review on the registered sheet.
3. **Owner hold release (D40):** Stage the exact commands for Casey to type personally with `UOR_LAB` unset.
4. **Device finite qualification (`NOT_RUN`):** Execute CUDA controls `X1`–`X12` and `F1`–`F7` on the pod; halt on any control failure (D31).
5. **Pre-run gates `C0` and proxy-agreement (`NOT_RUN`):** Verify `0 / 1,024` tuple-cache replay mismatches (`C0`) and $\ge 0.70$ proxy-writer argmin agreement on the `1,024`-row TRAIN sample.
6. **Tuple cache & 7-arm execution (`NOT_RUN`):** Produce the traced `(u_b, m_{b,\ell}, \text{found}_\ell, \text{target})` cache once per seed (`seed 1`, `seed 2`), run arms `A0`, `A1`, `A2`, `A2-int`, `A2-prime`, `Z`, and `R` under matched writer-evaluation budgets, and score all candidates plus the D37 item-3 float export-gap report on DEV (`14,287` rows).

### Stop

- **Finite control failure:** Any mismatch on `C0`, `X1`–`X12`, or `F1`–`F7` stops the run immediately as a D31 instrument fault (`HARD_STOP` / `DIAGNOSTIC_STOP`), never a mechanism negative.
- **Pre-run proxy-agreement stop:** If TRAIN proxy-writer argmin agreement on the `1,024`-row sample is $< 0.70$, halt before arm training as a D31 instrument stop.
- **Export-limited stop:** If `A2` or `A2-prime` misses the integer DEV bar while its float-minus-integer `P_succ` top-1 gap exceeds `A0`'s two-seed range, flag `EXPORT_LIMITED`; at most one successor adjustment to $k_{\max}$ or the rounding map is allowed before closing the configuration (D22).
- **Budget & hold stop:** All training and M2/M4 compute holds remain in force until released by Casey; pod spend is capped at the registered sheet budget below.

### Cost

- **This task (`antigravity-25`):** **$0.00** (host-only design, literal re-derivation, and `flock_coupling_controls` verification; `0` GPU pods, `0` GPU-hours).
- **Projected GPU run after registration and owner release:** Using the measured A8 `rows_v1` rate of `0.12 s/batch` on `1 × RTX 5090` at `$0.908/h` (`docs/labs/a8-dose-2026-10-19/README.md:61`), caching `1,024`–`16,404` batches plus optimizing `20` scalar couplings across `2` seeds and scoring `7` arms on `14,287` DEV rows requires approximately **25–45 minutes on `1 × RTX 5090` ($\approx \$0.40\text{–}\$0.75$, hard ceiling $\$1.50$)**. Zero spend is authorized by this sheet until registered, FIT-reviewed, and released by Casey.

### Minimum test power

- **Seeds:** $\ge 2$ paired seeds (`seed 1`, `seed 2`) across all learned arms (`A0`, `A1`, `A2`, `A2-int`, `A2-prime`, `R`).
- **Evaluation panel size:** `14,287` DEV rows (`3,847` `P_succ` copy-successor rows and `10,440` `P_comp` rows), exceeding the CHARTER §4.3 minimum of `200` rows by $> 19\times$ on `P_succ` and $> 71\times$ on full DEV.
- **Noise floor:** Binomial standard error on `P_succ` (`3,847` rows) near `p = 0.05..0.35` is `8.4..14.8` rows; the `24`-row minimum floor plus `max(24, two-seed range of A0)` ensures the keep threshold is never narrower than two-seed noise (CHARTER §4.3, D22).
- **Controls:** Explicit zero-coupling ablation (`Z`, $K \equiv 0$), distribution-matched random integer coupling control (`R`), trit baseline (`A0`), and D37 item-3 float-to-integer export-gap witness.

TEST FITNESS: PENDING
