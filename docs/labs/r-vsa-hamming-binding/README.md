# Lab Record: Discrete Hamming Binding (VSA) on the Current Writer (`r-vsa-hamming-binding`)

- **Issue / Roadmap Item:** [`#2888`](https://github.com/UOR-Foundation/uor-r4/issues/2888) (`docs/roadmap/items/r-vsa-hamming-binding.md`)
- **Pre-Registration Sheet:** [`docs/harness/prep/r-vsa-hamming-binding-2026-10-25.md`](../../harness/prep/r-vsa-hamming-binding-2026-10-25.md)
- **Lab:** `antigravity`
- **Status:** `held` (Cancel probe, mathematical & CUDA writer specification `DiscreteVsaHammingWriterV1`, one-page pre-registration sheet, fresh-context `TEST FITNESS: FIT` review, and `D68` decision review complete; kernel implementation in `crates/uor-r4-discrete` and RTX 5090 GPU execution gated on `#2836` `m1-bvg-k-stage-t-training`)

---

## 1. Cancel Probe and Prior-Test Audit

### 1.1 Why Earlier VSA Runs Were Configuration Failures Only (`CHARTER §4.1`, `D22`)

Per [`docs/mechanisms/vsa.md`](../../mechanisms/vsa.md), [`docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/vsa.json`](../repo-renormalization-2026-10-19/audit/mechanisms/vsa.json), and [`archive/labs/vsa-native-test-2026-10-09/README.md`](../../../archive/labs/vsa-native-test-2026-10-09/README.md):

| Date | Configuration | Measured Result | Why It Does Not Settle Discrete VSA on the Current Writer |
| :--- | :--- | :--- | :--- |
| **8 Oct 2026** (`#820`) | Load-time swap of root and learned VSA codes into a frozen September prose artifact | $\Delta\text{BPB} = -0.0010$ to $-0.0020$, top-1 moved $\le 0.12\text{ pp}$ | Frozen-artifact swap: the surrounding weights and scale never co-adapted with the new codes (`inconclusive`). |
| **9 Oct 2026** (`Arm 2`, `Mode 0`) | Fixed SplitMix64 hash codes (`4,096` distinct 4,096-bit codes) trained vs `VSA-Off` (`Arm 1`), 2 seeds $\times$ 2 slices (`6.8M` tokens/run) | **Positive on its learner:** $\Delta\text{BPB} = -0.0208 / -0.0226$ (seed `20260917`) and $-0.0144 / -0.0163$ (seed `20261009`); within-model VSA ablation $+0.0133$ to $+0.0157$ BPB | Ran on the legacy CPU learner (`train-native-prose`), barred since 12 Oct 2026 by `D24` (`CHARTER §3.5`), and used a runtime `Q1.15` integer multiply `(vsa_scale_q15 * corr) >> 16` unaudited against `D11` (`CHARTER §3.6`). |
| **9 Oct 2026** (`Arm 3`, `Mode 1`) | Icosian-root codes (`120` distinct codes for `4,096` tokens) trained vs `VSA-Off` | $\Delta\text{BPB} = +0.0003 / +0.0001$ (seed 1), $-0.0053 / -0.0054$ (seed 2) | **Identity collapse:** every token sharing an H4 icosian root received the exact same code (`120` codes across `4,096` tokens), so the VSA term could not distinguish within-root rivals. |
| **9 Oct 2026** (`Arm 5`, `Mode 2`) | Root code $\oplus$ 5-D $S^2$ readout residual (`4,096` distinct codes), rebuilt every `1,000` steps (`--vsa-code-refresh 1000`) | Worse than fixed codes by $+0.026 / +0.028$ BPB (seed 1) and $+0.013 / +0.014$ BPB (seed 2); within-model ablation $-0.002$ to $-0.005$ | **Moving-target confound:** rebuilding the codebook every `1,000` steps forced the context heads to chase moving codes (`vsa.md:81`). Freezing the geometric+residual codebook after initialization was explicitly named as the primary untested follow-up (`vsa.md:75`). |

### 1.2 Why the Current `D32` Discrete Writer Needs Context-Bound Hamming Disambiguation

Inspection of [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`root_cost` at line 37, `address_distance` at line 328, `token_distance` at line 331, and `writer` at lines 361–399) alongside the sealed GPU results of `#2831` (`m1-a8-full-dose-rerun`), `#2845` (`m1-w2-capacity-capture`), and `#2884` (`r-ranking-with-admission-intact`) establishes three structural facts:

1. **Hamming is currently used only as a single-block root tie-breaker inside the read, never in the writer:**
   - In `discrete.cu:37-40`, `root_cost` computes both L1 coordinate displacement (`*distance += magnitude(x[d]-y)`) and coordinate Hamming (`*hamming += (x[d]!=y)`).
   - However, `address_distance(q, g, s, cell)` (`discrete.cu:328-330`) discards `ha` and `hb` (`return a+b`), and `token_distance(q, table, g, s, token, ...)` (`discrete.cu:331-335`) computes only the static 4-block L1 distance:
     $$d_{\text{L1}}\big(q,\, v(t)\big) = \sum_{b=0}^{B-1} \sum_{d=0}^{7} \big|q_{b,d} - v_{b,d}(t)\big| \in [0, 8B].$$
2. **Even with admission held intact (`256 / 256` cells), static L1 distance leaves `79.71%` of admitted rows unranked at `top-1`:**
   - In `#2831` / `#2884`, `FrozenZero` at `Q1` (`4,096` batches, `258` updates, `q1-dev.json` SHA-256 `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`) held `256 / 256` cells occupied ($\bar{n} = 126.49$ tokens/row) and admitted `4,292 / 14,287` DEV targets (`30.04%`), yet ranked only `871 / 4,292` (`20.29%`, `679` strict wins + `192` token-ID tie wins) at `top-1`, leaving **`3,421 / 4,292` admitted rows (`79.71%`)** lost to within-cell rivals.
   - Notably, `871 - 679 = 192` of those `871` `top-1` hits (`22.04%` of all `top-1` hits) sit on an **exact L1 distance tie** ($d_{\text{L1}}(q, v(t^*)) = d_{\text{L1}}(q, v(t_{\text{rival}}))$), which `discrete.cu:375` currently resolves solely by static token ID (`token < best_token`).
3. **`#2845` (`W2`) proves that the read output $q$ and prefix context contain `3,414`–`6,300` hits of information:**
   - On the same `14,287`-row DEV panel, the **W2 Code Table** (exact successor frequencies conditioned on the model's 4-block read output $q$) achieves **`3,414 / 14,287` (`23.89%`)** full-panel `top-1` (`2,280 / 3,847 = 59.27%` on `P_succ`), and the **T2 Query-Token Bigram** table achieves **`6,300 / 14,287` (`44.10%`)**.
   - A single static 4-block root vector $v(t) \in \mathcal{R}_{16}^4$ per token cannot represent multiple predecessor-specific transitions $(w_{L-1} \to t)$ without moving $v(t)$ away from its other predecessors. A **role-bound XOR/Hamming context sketch** $B_{\text{ctx}}(\text{row}) \in \{0,1\}^{64}$ supplies a multiplier-free associative transition cue directly inside `writer(...)` while keeping $v(t)$ in its `AddressCellsA8` cell.

---

## 2. Mathematical & Kernel Specification (`DiscreteVsaHammingWriterV1`)

### 2.1 Hypervector Codebooks ($D = 64$ bits = one `W` / `u64` word per token)

To satisfy `CHARTER §3.1–§3.6`, `D11` (zero runtime multipliers), and `D32` (integer-only discrete model path, zero floats), each token $t \in [0, V)$ ($V = 4,096$) carries a frozen 64-bit code $h(t) \in \{0,1\}^{64}$ stored in a read-only device array `const W* vsa_codes` (`4,096` words = `32 KiB`, fitting entirely in L1/constant cache on RTX 5090):

1. **Arm 0 — `VSA-Off` (`s[15] = 0`, Baseline Control):**
   - No hypervector term; `writer` uses `4 * d_L1(q, v(t))` (identical ranking and loss order to `discrete.cu:361-399`).
2. **Arm 1 — `VSA-Fixed-64` (`s[15] = 1`, Frozen Injective SplitMix64 Codebook — Mode 0 on `D32`):**
   - Generated deterministically at initialization via `Hypervector::<1>::from_seed(CANONICAL_TIE_BREAKER_SEED, t as u64)` (`crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs:99`):
     $$h_{\text{fix}}(t) = \operatorname{SplitMix64}\!\big(\texttt{0x5653\_415f\_4255\_4e44},\; t\big) \in \{0,1\}^{64}.$$
   - All `4,096` token codes are distinct (`4,096 / 4,096` injective) with expected pairwise Hamming distance $D/2 = 32$ bits ($\sigma = \sqrt{64}/2 = 4.0$ bits).
3. **Arm 2 — `VSA-Geom-Frozen-64` (`s[15] = 2`, Frozen E8 Orbit-Root Label $\oplus$ Injective Residual — Frozen Mode 2 on `D32`):**
   - For each E8 orbit root $r \in \{0, \dots, 15\}$ (`g[ROOT + r*8 + d]`, $d \in \{0, \dots, 7\}$), define its 16-bit ternary sign-and-support pattern:
     $$\operatorname{pat}_{16}(r) = \sum_{d=0}^{7} \Big(\mathbf{1}[g_{\text{ROOT}}(r, d) \neq 0] \cdot 2^{2d} + \mathbf{1}[g_{\text{ROOT}}(r, d) > 0] \cdot 2^{2d+1}\Big) \in \{0, 1\}^{16}.$$
   - Given token $t$'s initial `AddressCellsA8` (`PrimeZeta`) 4-block root vector $v_0(t) = (r_0, r_1, r_2, r_3) \in \{0, \dots, 15\}^4$, pack all 4 blocks into a 64-bit geometric root code:
     $$c_{\text{root}}\big(v_0(t)\big) = \big(\operatorname{pat}_{16}(r_0)\big) \;\big|\; \big(\operatorname{pat}_{16}(r_1) \ll 16\big) \;\big|\; \big(\operatorname{pat}_{16}(r_2) \ll 32\big) \;\big|\; \big(\operatorname{pat}_{16}(r_3) \ll 48\big).$$
   - XOR $c_{\text{root}}(v_0(t))$ with a low-weight (`16`-bit sparse mask) deterministic per-token residual $\Delta_{\text{res}}(t)$ generated from `PrimeZeta` token rank so that tokens in the same `AddressCellsA8` cell retain positively correlated root bits ($d_H \le 32$) while all `4,096 / 4,096` codes are strictly distinct and **frozen from step 0**:
     $$h_{\text{geom}}(t) = c_{\text{root}}\big(v_0(t)\big) \oplus \Delta_{\text{res}}(t).$$

### 2.2 Causal Role-Filler Binding and Majority Bundling on Device

For each row with causal prefix length $L = \text{plan}[0] \in [1, 243]$ and query vector $q$:

1. **Query Frame Action $a_q \in [0, 64)$:**
   - Let $r_0 = \operatorname{root\_id}(q_0, g) \in [0, 16)$ and $r_{B-1} = \operatorname{root\_id}(q_{B-1}, g) \in [0, 16)$ (or $0$ if unsnapped). Define the cyclic shift offset $a_q = (r_0 \cdot 4 + (r_{B-1} \ \& \ 3)) \ \& \ 63$.
2. **Causal Window Binding ($\otimes$ and $\rho$):**
   - Over the odd causal suffix window of up to $W_{\text{vsa}} = 15$ predecessors $p \in [\max(0, L - 15), L - 1]$ (with $\Delta p = L - 1 - p \in \{0, \dots, 14\}$):
     $$z_p = \operatorname{rotl}_{64}\!\big(h(\text{id}_p),\, (\Delta p \cdot 4 + a_q + 1) \ \& \ 63\big) \oplus h(\text{id}_{L-1}).$$
   - Binding via XOR is self-inverse ($(A \oplus B) \oplus B = A$) and distance-preserving ($d_H(A \oplus C, B \oplus C) = d_H(A, B)$), and cyclic rotation $\operatorname{rotl}_{64}$ makes order non-commutative ($\operatorname{rotl}_{64}(h(a), 4) \oplus h(b) \neq \operatorname{rotl}_{64}(h(b), 4) \oplus h(a)$).
3. **Bitwise Majority Bundling ($+$):**
   - Accumulate the $m \in [1, 15]$ bound 64-bit words $z_p$ using a 4-bit vertical bit-plane adder (`b0, b1, b2, b3` in `u64` registers — 15 bitwise ops per term, zero per-bit loops) and set bit $j$ of $B_{\text{ctx}}(\text{row})$ to $1$ iff the count exceeds $\lfloor m / 2 \rfloor$ (with `CANONICAL_TIE_BREAKER_SEED` breaking even-$m$ ties when $L$ is even).

### 2.3 Multiplier-Free Writer Integration (`discrete.cu:373-399`)

Inside `writer(...)` (`discrete.cu:373-388`) and `discrete_evaluate(...)` (`discrete.cu:535-545`):

```cuda
__device__ U vsa_hamming_cost(W ctx_sketch, const W* vsa_codes, U token, U vsa_mode, U vsa_shift) {
    if (vsa_mode == 0u) return 0u;
    U dh = (U)__popcll(ctx_sketch ^ vsa_codes[token]); // 0..64
    return dh >> vsa_shift; // multiplier-free shift weight (sigma_vsa in 0..3)
}
```

- **Within-cell candidate distance:**
  $$d_{\text{total}}(q, B_{\text{ctx}}, t) = \big(d_{\text{L1}}(q, v(t)) \ll 2\big) + \operatorname{vsa\_hamming\_cost}\big(B_{\text{ctx}}(\text{row}),\, \text{vsa\_codes},\, t,\, s[15],\, s[16]\big).$$
- **Immediate Tie-Break & Margin Lift:**
  - When $d_{\text{L1}}(q, v(t_1)) = d_{\text{L1}}(q, v(t_2))$ (which occurs on $192 / 871 = 22.04\%$ of all `#2831` `Q1` `top-1` wins and thousands of admitted rival pairs), $d_H(B_{\text{ctx}}(\text{row}), h(t))$ breaks the L1 tie using context-bound Hamming similarity instead of static token ID (`token < best_token`).
  - When $\sigma_{\text{vsa}} \in \{0, 1, 2\}$, strong Hamming correlation ($d_H \ll 32$) can also overturn a small 1-to-4 unit L1 coordinate gap inside the admitted cell pool ($\bar{n} \approx 126.5$ tokens/row).
  - Under counterfactual moves (`discrete_counterfactual`), $v(t)$ moves across the 16 E8 orbit roots (`kind = 1`) or `routes`/`gates` move $q$ (`kind = 0` or `3`) so that $d_{\text{L1}}(q, v(t))$ co-adapts with the fixed/frozen Hamming cue $d_H(B_{\text{ctx}}(\text{row}), h(t))$.

---

## 3. Baseline Reference Numbers Against Count Tables (`#2831`, `#2845`, `#2884`)

Every arm in `#2888` is compared against the sealed `#2831` / `#2884` `VSA-Off` baseline and `#2845` (`W2`) count tables on the `14,287`-row DEV panel (`val.bin` SHA-256 `53dfca08...`, `P_succ = 3,847` rows):

| Model / Count-Table Comparator | Admitted Rows $N_{\text{adm}}$ (`Full`) | Full-Panel `DEV top-1` (`/ 14,287`) | Conditional `top-1 / N_adm` (`Full`) | Full-Panel `DEV top-8` (`/ 14,287`) | Conditional `top-8 / N_adm` (`Full`) | `P_succ top-1` (`/ 3,847`) |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| **Count Table: Unigram (`C_uni`)** | `14,287` (`100%`) | **`1,226` (`8.58%`)** | `8.58%` | `3,891` (`27.23%`) | `27.23%` | `400` (`10.40%`) |
| **Count Table: W2 Read-Code Table (`#2845`)** | `14,287` (`100%`) | **`3,414` (`23.89%`)** | `23.89%` | `6,881` (`48.16%`) | `48.16%` | `2,280` (`59.27%`) |
| **Count Table: T2 Query Bigram (`C_T2`)** | `14,287` (`100%`) | **`6,300` (`44.10%`)** | `44.10%` | `10,124` (`70.86%`) | `70.86%` | `2,713` (`70.52%`) |
| **`#2831` `Batch 0` (`AddressCellsA8`, `VSA-Off`)** | `4,244` (`29.71%`) | `133` (`0.93%`) | `3.13%` (`133 / 4,244`) | `513` (`3.59%`) | `12.09%` (`513 / 4,244`) | `64` (`1.66%`) |
| **`#2831` `Q1` (`Batch 4,096`, `FrozenZero`, `VSA-Off`)** | `4,292` (`30.04%`) | **`871` (`6.10%`)** | **`20.29%` (`871 / 4,292`)** | **`1,455` (`10.18%`)** | **`33.90%` (`1,455 / 4,292`)** | `252` (`6.55%`) |

---

## 4. Verification and Next Steps

1. **Pre-Registration & FIT:** [`docs/harness/prep/r-vsa-hamming-binding-2026-10-25.md`](../../harness/prep/r-vsa-hamming-binding-2026-10-25.md) defines the 3 arms (`Arm-0-VSA-Off`, `Arm-1-VSA-Fixed-64`, `Arm-2-VSA-Geom-Frozen-64`), the shift-weight sweep $\sigma_{\text{vsa}} \in \{\text{off}, 3, 2, 1, 0\}$, the finite controls, and the decision bars (`G-VSA-Intact`, `G-VSA-Lift`, `G-VSA-Mode2-Frozen`).
2. **Why `status: held` Is Set Now (`D68`):**
   - `r-vsa-hamming-binding.md` declares `scope: [docs/harness/prep/r-vsa-hamming-binding*, docs/labs/r-vsa-hamming-binding/**, docs/loop/tasks/r-vsa-hamming-binding/**]`, which excludes `crates/uor-r4-discrete/**`.
   - Full `16,404`-batch training on the `D32` discrete path depends on `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`) so the `BVG-K` (`K = 16`) update law avoids `#2831`'s `K = 1` `consecutive_rejections` early stop.
   - Setting `status: held` after the `TEST FITNESS: FIT` and `D68` decision review keeps `loop next` unblocked while staging the complete `DiscreteVsaHammingWriterV1` specification and pre-registration sheet for execution once `#2836` completes.
