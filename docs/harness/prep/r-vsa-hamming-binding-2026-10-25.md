# Pre-Registration: Discrete Hamming Binding (VSA) on the Current Writer (`r-vsa-hamming-binding`)

- **Issue / Roadmap Item:** [`#2888`](https://github.com/UOR-Foundation/uor-r4/issues/2888) (`docs/roadmap/items/r-vsa-hamming-binding.md`)
- **Milestone / Area:** `research` / `mechanism` (`docs/mechanisms/vsa.md`, `docs/mechanisms/ternary-xor.md`)
- **Author Lab:** `antigravity`
- **Date:** 2026-10-25
- **Upstream Prerequisites:**
  - `m1-a8-full-dose-rerun` (`#2831`, PR `#3192`, merged `cb2b5748905dc77b122325101b62dd7c29fc1413` — `status: done`)
  - `m1-w2-capacity-capture` (`#2845`, PR `#3187`, merged `9dd74d80f077e1d1df523c97dc4a5a0bf5fa1a7b` — `status: done`)
  - `r-ranking-with-admission-intact` (`#2884`, PR `#3213`, merged `79b2df2701f35f54c469c676a308b6021342c3e4` — `status: held`)
  - `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md` — `BVG-K` `K = 16` batched-coordinate training)

---

## 1. Question and Base Artifact

**Question:** On the current `D32` discrete model path (`crates/uor-r4-discrete`, `PrimeZeta` `4,096 / 4,096` injective codebook, `AddressCellsA8` `256 / 256` occupied cells, `D40` admission-anchor controller), does adding a multiplier-free 64-bit XOR/Hamming context-binding term (`DiscreteVsaHammingWriterV1`, using cyclic bit rotation $\rho$, XOR role-filler binding $\oplus$, bitwise majority bundling, and hardware `__popcll` Hamming distance) to `discrete.cu`'s `writer(...)` improve full-panel and conditional admitted `DEV top-1` / `top-8` over the `VSA-Off` baseline (`#2831` / `#2884`) and against the full-panel and admission-matched count tables (`Unigram = 1,226 / 14,287`, `W2 Code Table = 3,414 / 14,287`, `T2 Bigram = 6,300 / 14,287`), and does a frozen E8 root-label $\oplus$ per-token residual codebook (`VSA-Root-Residual-Frozen-64`) match or beat a frozen SplitMix64 hash codebook (`VSA-Fixed-64`) once the 9 Oct 2026 moving-codes confound is removed?

- **Why Earlier VSA Tests Were Configuration Failures Only (`CHARTER §4.1`, `D22`, `vsa.json`):**
  1. **9 Oct 2026 Native VSA Retraining (`archive/labs/vsa-native-test-2026-10-09/README.md`):** Fixed hash codes (`Mode 0`, `4,096` distinct 4,096-bit codes) improved held-out BPB over `VSA-Off` on all four seed-and-slice pairs (`-0.0208` / `-0.0226` on seed `20260917`, `-0.0144` / `-0.0163` on seed `20261009`; within-model ablation CI `[+0.0128, +0.0163]`). However, that run trained on the legacy CPU learner (`train-native-prose`), which is barred by `D24` (`CHARTER §3.5`), and scaled the correlation with a runtime `Q1.15` integer multiply (`(vsa_scale_q15 * corr) >> 16`) unaudited against `D11` (`CHARTER §3.6`).
  2. **Mode 1 (`Icosian-root codes`, 120 distinct codes) & Mode 2 (`Root + readout residual`, refreshed every 1,000 steps):** Mode 1 gave identical codes to all tokens sharing an icosian root (`120` codes for `4,096` tokens, `+0.0003` to `-0.0054` BPB), and Mode 2 rebuilt the codebook every `1,000` steps while training ran (`+0.013` to `+0.028` BPB worse than fixed codes), creating a moving-target confound (`vsa.md:81`).
  3. **Why the Current `D32` Writer Needs a Context-Bound Hamming Term (`#2845`, `#2884`):** In `#2831` / `#2884`, holding admission intact at `Q1` (`4,096` batches, `256 / 256` cells, $\bar{n} = 126.49$, `DEV b_3(Full) = 4,292 / 14,287`) raised `DEV top-1` to `871 / 4,292` (`20.29%` of admitted rows), leaving `3,421 / 4,292` admitted rows (`79.71%`) unranked at `top-1`. Yet `#2845` (`W2`) proved that the 4-block read output $q$ already supports `3,414 / 14,287` (`59.27%` on `P_succ`) via an empirical code table because static L1 distance $d_{\text{L1}}(q, v(t))$ in `discrete.cu:331` has no associative context-to-candidate binding.
- **Base Artifact & Pinned Inputs:**
  - **Codebook & Cell Layout:** `PrimeZeta` injective codebook (`4,096 / 4,096` unique vectors, $B = 4$, $d = 32$) + `AddressCellsA8` (`256 / 256` occupied cells at `16` tokens/cell at init, `b_3(P_succ) = 1,351 / 3,847`, `b_3(Full) = 4,244 / 14,287` at $K = 8$).
  - **Active Admission Controller (`D40`):** Fixed 3,847-row TRAIN anchor manifest `f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de` (`margin = 24`, `minimum_baseline = 500`, `floor = 1,187 / 3,847` at $K = 8$).
  - **Pinned Corpus Shards (BPE `d36d3e87`, $V = 4,096$, $L = 128$):**
    - `train.bin`: SHA-256 `49263fe24ecc3ad2d8bfabbc6dc1fbd95fb5fbecbefdfbe0fa0ee6bfefcf0d6f` (`19,165` sequences, `2,425,925` targets)
    - `val.bin` (`DEV` panel): SHA-256 `53dfca085fa4be04875e424eefeb43952d0b44ab6d5d6ec2d590ae7cf5ff7e71` (`14,287` evaluation positions; `P_succ = 3,847` rows)
    - `fresh_v2` transfer slice (`crates/uor-r4-discrete/fixtures/fresh_panel_v2.bin`): SHA-256 `c1f40ab0578d20ca337282b9efbe12b29f21f99a11cb17689708ee46f2181167` (`384` rows; sealed until a DEV gate passes)

---

## 2. Arms, Data Split, and Steps

All arms run on a single NVIDIA GeForce RTX 5090 (`sm_120`) under the Stage T `BVG-K` (`K = 16` coordinates/step) + `AddressCellsA8` + `D40` admission-anchor recipe ($K_{\text{cells}} = 8$) across two deterministic seeds (`seed in {1, 2}`) for `16,404` batches (`64` rows/batch):

| Arm ID | Writer Distance $d_{\text{total}}(q, B_{\text{ctx}}, t)$ | Hypervector Codebook $h(t) \in \{0,1\}^{64}$ | Distinct Codes | Shift Weight $\sigma_{\text{vsa}}$ |
| :--- | :--- | :--- | ---: | :--- |
| **`Arm-0-VSA-Off`** (Control) | $4\,d_{\text{L1}}(q, v(t))$ (`s[15] = 0`) | None (existing `discrete.cu` `writer`) | — | Off (`0` VSA weight) |
| **`Arm-1-VSA-Fixed-64`** (Mode 0 on `D32`) | $4\,d_{\text{L1}}(q, v(t)) + (d_H(B_{\text{ctx}}, h_{\text{fix}}(t)) \gg 1)$ (`s[15] = 1`) | Frozen `SplitMix64(0x5653415f42554e44, t)` | `4,096 / 4,096` | $\sigma_{\text{vsa}} = 1$ (weight $1/2$; sweep $\sigma_{\text{vsa}} \in \{0,1,2,3\}$ at eval) |
| **`Arm-2-VSA-Geom-Frozen-64`** (Frozen Mode 2 on `D32`) | $4\,d_{\text{L1}}(q, v(t)) + (d_H(B_{\text{ctx}}, h_{\text{geom}}(t)) \gg 1)$ (`s[15] = 2`) | Frozen E8 root-label code $c_{\text{root}}(v_0(t)) \oplus \Delta_{\text{res}}(t)$ | `4,096 / 4,096` | $\sigma_{\text{vsa}} = 1$ (weight $1/2$; sweep $\sigma_{\text{vsa}} \in \{0,1,2,3\}$ at eval) |

- **Multiplier-Free Context Binding & Writer Law (`DiscreteVsaHammingWriterV1`):**
  - Over the odd causal prefix window $p \in [\max(0, L - 15), L - 1]$ ($m \le 15$ terms), each context token code $h(\text{id}_p)$ is bound by cyclic 64-bit rotation $\operatorname{rotl}_{64}(h(\text{id}_p), (\Delta p \cdot 4 + a_q) \bmod 64)$ and XORed with the query token code $h(\text{id}_{L-1})$:
    $$z_p = \operatorname{rotl}_{64}\!\big(h(\text{id}_p),\, (\Delta p \cdot 4 + a_q) \bmod 64\big) \oplus h(\text{id}_{L-1}), \qquad \Delta p = L - 1 - p \in \{1, \dots, 15\}.$$
  - Bitwise majority bundling across the $m$ terms produces $B_{\text{ctx}}(\text{row}) \in \{0,1\}^{64}$, and candidate scoring inside admitted cells adds hardware `__popcll(B_ctx ^ h(t)) >> sigma_vsa` using shifts and adds only (`0` float ops, `0` multiplies, `D11 R1–R5` compliant).
- **Execution Steps (Probe Ladder):**
  1. **Step 1 — Finite Algebraic & CUDA Equivalence Controls (~4 min):** Verify on RTX 5090 that (a) `s[15] = 0` is bit-identical to `Arm-0-VSA-Off`, (b) both $h_{\text{fix}}$ and $h_{\text{geom}}$ have `4,096 / 4,096` unique 64-bit codes, (c) XOR self-inverse $(A \oplus B) \oplus B = A$ and cyclic rotation distance invariance $d_H(\rho_k(A), \rho_k(B)) = d_H(A, B)$ hold on device, and (d) the shift-weight sweep $\sigma_{\text{vsa}} \in \{\text{off}, 3, 2, 1, 0\}$ on the `#2831` `Q1` checkpoint executes in $< 2$ s with zero `d23`/`d25`/`d32` or `D11` multiplier violations.
  2. **Step 2 — 256-Batch Throughput & Stability Probe (~5 min):** Run `256` batches of `Arm-1-VSA-Fixed-64` and `Arm-2-VSA-Geom-Frozen-64` (seed 1) on RTX 5090, confirming $< 5\%$ throughput overhead vs `Arm-0-VSA-Off` and projected wall time $\le 35$ min/run.
  3. **Step 3 — Paired 3-Arm $\times$ 2-Seed GPU Training & Shift-Sweep Evaluation (`6` runs):** Run `Arm-0-VSA-Off`, `Arm-1-VSA-Fixed-64`, and `Arm-2-VSA-Geom-Frozen-64` on seeds `1` and `2` through `16,404` batches, reporting `DEV top-1`, conditional admitted `top-1`, within-model VSA-off ablation $\Delta\text{top-1}$, shift-weight sweep $\sigma_{\text{vsa}} \in \{\text{off}, 3, 2, 1, 0\}$, and decode `tokens/s` against the count tables.

---

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum Test Power (CHARTER §4.3):**
  - `2` independent seeds (`seed in {1, 2}`) per arm across all `14,287` DEV positions (`3,847` `P_succ` rows, $\ge 4,244$ admitted rows at $K = 8$) and `384` `fresh_v2` transfer rows.
- **Registered Bars:**
  1. **Bar `G-VSA-Intact` (Admission & Determinism Bar):** On **both** seeds (`1` and `2`), `occupied_cells >= 192 / 256`, DEV $b_3(\text{Full}) \ge 4,244 / 14,287$, two-build evaluation outputs are byte-identical, and zero `d23`/`d25`/`d32` or `D11` multiplier instructions exist.
  2. **Bar `G-VSA-Lift` (Paired Lift Over `VSA-Off` and Count-Table Comparison):** On **both** seeds at `Q4`, at least one VSA arm (`Arm-1-VSA-Fixed-64` or `Arm-2-VSA-Geom-Frozen-64`) beats `Arm-0-VSA-Off` by **$\ge +143$ `DEV top-1` hits** ($\ge +1.0\%$ of `14,287` DEV rows, exceeding $2\times$ the `#2831` `Q1` strict-vs-tie tie-resolution band), its within-model VSA ablation $\Delta\text{top-1} = \text{top-1}(\text{VSA on}) - \text{top-1}(\text{VSA off}) > 0$ on both seeds, and full-panel `DEV top-1` beats full-panel `Unigram` (`1,226 / 14,287`) with decode `tokens/s` within $95\%$ of `Arm-0-VSA-Off`.
  3. **Bar `G-VSA-Mode2-Frozen` (Frozen Geometric Codebook vs Fixed Hash):** Tests whether `Arm-2-VSA-Geom-Frozen-64` matches or beats `Arm-1-VSA-Fixed-64` on both seeds once the codebook is frozen after `AddressCellsA8` init (`batch = 0`).
- **Decision Table:**

| Outcome | Condition | Decision & What It Unlocks |
| :--- | :--- | :--- |
| **`KEEP_VSA_GEOM_FROZEN`** | `Arm-2-VSA-Geom-Frozen-64` passes `G-VSA-Intact`, `G-VSA-Lift`, and `G-VSA-Mode2-Frozen` on both seeds, with positive `fresh_v2` transfer | Promote `DiscreteVsaHammingWriterV1` (`s[15] = 2`) from `PARKED` to `ACTIVE`, close `#2888` (`status: done`), and feed into `m1-export-gap-and-toks` (`#2839`). |
| **`KEEP_VSA_FIXED_HASH`** | `Arm-1-VSA-Fixed-64` passes `G-VSA-Intact` and `G-VSA-Lift` on both seeds, while `Arm-2` trails `Arm-1` | Promote `DiscreteVsaHammingWriterV1` (`s[15] = 1`) to `ACTIVE`, close `#2888` (`status: done`), confirming fixed injective 64-bit Hamming binding transfers to the `D32` GPU writer. |
| **`NULL_REDUNDANT_ON_D32`** | Both `Arm-1` and `Arm-2` pass `G-VSA-Intact` across all $\sigma_{\text{vsa}} \in \{0,1,2,3\}$ scales, but neither clears `G-VSA-Lift` ($\Delta\text{top-1} < +143$) | Close `#2888` (`status: done` on its comparison contract) and record a `D22` configuration negative for 64-bit prefix bundling on `D32` (`vsa.md` remains `PARKED`). |
| **`INSTRUMENT_STOP`** | Step 1 or Step 2 fails any finite control or throughput check | Classify as `instrument` (CHARTER §4.2) and repair before Step 3. |

---

## 4. Cost and Stop Conditions

- **Estimated Compute & Cost:**
  - `1 x NVIDIA GeForce RTX 5090` (`~$0.65/h`–`$1.05/h` on Vast.ai, inside the 4-pod / `$8/h` fleet cap).
  - Step 1 (finite controls + zero-retrain shift sweep on `#2831` `Q1`) + Step 2 (`256`-batch probe): `~9 min`.
  - Step 3 (`6` runs $\times$ `~22 min/run` under Stage T `BVG-K` `K = 16`): `~2.2 hours` (`~$1.50`–`$2.40` total).
- **Stop Conditions:**
  - **Immediate Instrument Stop:** Abort before Step 3 if Step 1 fails bit-identity at `s[15] = 0` or if Step 2 exceeds $10\%$ runtime overhead.
  - **Scope & Dependency Gate:** Kernel changes in `crates/uor-r4-discrete/**` are outside this item's `docs/**` scope and gated on `m1-bvg-k-stage-t-training` (`#2836`, Step 4 of `PLAN.md`), so `#2888` is set to `status: held` until `#2836` completes.

---

## 5. Review Verdict

- **Reviewer:** Fresh-context non-author `research` subagent `d662d613-31f5-4bd2-825f-212acec9efbe` (`antigravity`, CHARTER §4.5, §5.1, D68)
- **Posted Record:** Issue [`#2888` comment `6397106918`](https://github.com/UOR-Foundation/uor-r4/issues/2888#issuecomment-6397106918)

TEST FITNESS: FIT
