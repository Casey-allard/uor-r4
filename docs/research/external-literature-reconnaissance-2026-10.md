# External Literature Reconnaissance — UOR-R4 Geometric Language Model

**Date:** 2026-10-01 · **Scope:** external literature only; no project claims adjudicated
**Method:** primary-source retrieval (`arxiv.org/abs`, `arxiv.org/html`, conference proceedings, Apple tech specs, Zenodo records). Every arXiv ID below was fetched and title-checked unless marked otherwise.

> **Sourcing caveat.** Several 2026 hits in this exact space are self-published Zenodo / preprints.ru
> preprints with no peer review (GeoLLM, Q-Jamba, Sovereign-Lila-E8, "Geometric Attention"). They
> are labelled **NON-PEER-REVIEWED** wherever cited and must not be treated as feasibility evidence.
> The volume of unrefereed material in this niche raises the bar for internal evidence discipline.

---

## 0. Executive summary — the findings that most change design decisions

1. **The hardest external constraint is a complexity-class theorem, not an engineering limit.** Merrill, Petty & Sabharwal (ICML 2024, [arXiv:2404.08819](https://arxiv.org/abs/2404.08819)) prove SSMs "cannot express computation outside TC⁰" and therefore **cannot solve permutation composition**. Mamba-3 (ICLR 2026) treats this as a defect to engineer around and adds a **complex-valued state update** to recover state tracking. Any fixed-size recurrent geometric state is in the same class unless something outside the affine/diagonal family is added.
2. **There is now a sharp solvable-groups theorem.** Shakerinava, Khavari, Ravanbakhsh & Chandar (ICLR 2026) prove that a *single-layer* input-dependent complex diagonal SSM **cannot express state tracking of any non-Abelian group at finite precision**, and that *k*-layer DCD SSMs express exactly the groups with a subnormal series of length *k* with Abelian factors — i.e. the **solvable** groups. They also find empirically that multi-layer models often *fail to learn* non-Abelian tracking, "highlighting a gap between expressivity and learnability."
3. **A second, independent 2026 failure mode: error control.** Chung, Choi & Kim ([arXiv:2605.07755](https://arxiv.org/abs/2605.07755)) prove affine recurrent networks — "a class of models encompassing State-Space Models and Linear Attention" — **cannot correct errors along state-separating subspaces**. Tracking degrades to a finite horizon set by accumulated error. This is a *dynamical* limit, not a complexity-theoretic one.
4. **Mamba-3 maps onto the project's mechanisms almost one-to-one, and it is peer-reviewed.** Complex-valued state update (≈ a zeta-phase / S³ state), SSM-discretisation-derived recurrence, and MIMO for quality at constant decode latency: **+1.8 pp downstream accuracy at 1.5 B over Gated DeltaNet**, and **Mamba-2-equal perplexity at half the state size** ([arXiv:2603.15569](https://arxiv.org/abs/2603.15569)). This is the single most useful external reference.
5. **Multiplier-free inference is real, and it is bandwidth-bound.** T-MAC (EuroSys 2025, [arXiv:2407.00088](https://arxiv.org/abs/2407.00088)) delivers **30 tok/s single-core / 71 tok/s 8-core for BitNet-b1.58-3B on M2-Ultra**, **11 tok/s on Raspberry Pi 5**, 4× throughput and 70 % energy reduction vs `llama.cpp`. The paper itself states that multi-threaded mpGEMV is "primarily constrained by memory bandwidth." The honest claim is **energy and instruction-count reduction**, not a throughput multiple.
6. **Associative recall is exactly where attention-free architectures lose.** Zoology (ICLR 2024, [arXiv:2312.04927](https://arxiv.org/abs/2312.04927)): **82 % of a 2.1-perplexity-point gap** is in-context recall, and a **70 M**-parameter attention model beats a **1.4 B**-parameter gated convolution on associative recall. This is the precise capability the project's exact addressed memory must beat — and it is a fair, measurable target.
7. **Zeta zeros do not connect to language or neural computation in the reviewed literature.** Searched hard; see Section E. There is legitimate work on *p*-adic and ultrametric networks, but no peer-reviewed body linking Riemann/Selberg zeta zeros to language modelling.
8. **But *p*-adic ML is now a real, well-resourced line — and it is the strongest external support for the project's core arithmetic thesis.** Google DeepMind published the **first native continuous gradient descent for *p*-adic-parameter models** ([arXiv:2609.25501](https://arxiv.org/abs/2609.25501), Sept 2026; code `google-deepmind/padic-ml`), showing that a linear model over ℚ_p **solves modular arithmetic — "an XOR-like task not expressible by linear models in ℝ."** Separately, **v-PuNNs** ([arXiv:2508.01010](https://arxiv.org/abs/2508.01010)) use neurons that are characteristic functions of *p*-adic balls with every weight a *p*-adic number, and reach **99.96 % WordNet leaf accuracy** on 52,427 leaves in 16 min CPU-only, with a **perfectly ultrametric** learned metric.

---

## A. Alternatives to dot-product attention

### A.1 The two dual forms, stated exactly

Mamba-2's *state space duality* ([arXiv:2405.21060](https://arxiv.org/abs/2405.21060), ICML 2024) shows SSMs are semiseparable matrices and linear attention is the same object in tensor-contraction form. Base recurrence (their Eq. 1):

```
h_t = A h_{t-1} + B x_t          (1a)
y_t = Cᵀ h_t                     (1b)
```

selective (time-varying) form (their Eq. 2):

```
h_t = A_t h_{t-1} + B_t x_t      (2a)
y_t = C_tᵀ h_t                   (2b)
```

Mamba-2's key simplification is restricting `A_t = a_t I` (**scalar × identity**), making the operator *1-semiseparable* and admitting both a linear recurrent mode and a quadratic dual mode. Reported: SSD is **2–8× faster** than Mamba's selective scan, supports **8× larger state** with minimal slowdown, beats FlashAttention-2 crossover at 2 K and is **6× faster at 16 K**; Mamba-2 2.7 B (300 B Pile tokens) outperforms Mamba-2.8B, Pythia-2.8B **and Pythia-6.9B**.

Dao & Gu also prove a **converse: any kernel-attention method possessing a fast recurrent form must be an SSM.** This is a closure result and a genuine design fork — a new recurrent scheme either lies in the classified space (and inherits its limits) or it does not have O(1) decode.

### A.2 Exact updates, costs, and integer-only feasibility

| Method | Update equation | State | Train | Decode/token | Needs exp? | Integer/add-only feasible? |
|---|---|---|---|---|---|---|
| **Linear attention** ([2006.16236](https://arxiv.org/abs/2006.16236)) | `S_t = S_{t-1} + φ(k_t) v_tᵀ`; `y_t = φ(q_t)ᵀ S_t` | `d×d_v` | O(n) | O(1) | only if φ=exp | **Yes** for φ = identity/relu — the cleanest candidate |
| **Performer/FAVOR+** ([2009.14794](https://arxiv.org/abs/2009.14794)) | `exp(qᵀk) ≈ E[φ(q)ᵀφ(k)]`, random features | `m×d_v` | O(n) | O(1) | **Yes** | **No** (exp + trig features) |
| **S4** ([2111.00396](https://arxiv.org/abs/2111.00396)) | diagonal/DPLR `A`, HiPPO init, Cauchy kernel (Woodbury) | `N`/channel | O(n log n) | O(1) | `exp(ΔA)` | **No** as specified |
| **Mamba/S6** ([2312.00752](https://arxiv.org/abs/2312.00752)) | `h_t = exp(Δ_t A) h_{t-1} + Δ_t B_t x_t` | `N` | O(n) | O(1) | **Yes**, input-dependent | **No** (exp of data-dependent Δ) |
| **Mamba-2/SSD** ([2405.21060](https://arxiv.org/abs/2405.21060)) | `A_t = a_t I`; chunked semiseparable matmul | `N` large | O(n) matmul | O(1) | only scalar `a_t` | **Partially** — narrows the exp to one scalar/token |
| **RWKV v4–v7** ([2305.13048](https://arxiv.org/abs/2305.13048), [2307.09288](https://arxiv.org/abs/2307.09288)) | WKV recurrence, data-dependent decay `w` | `d` | O(n) | O(1) | yes (decay) | **No** unless decay is a fixed table |
| **RetNet** ([2307.08621](https://arxiv.org/abs/2307.08621)) | `D ⊙ (QKᵀ)V` with decay γ; chunkwise recurrent | `d×d_v` | O(n) | O(1) | no | **Yes**, with fixed γ table |
| **Hyena** ([2302.10866](https://arxiv.org/abs/2302.10866)) | implicit long conv (FFT) + gating | filter | O(n log n) | O(n log n) | no | multiply-heavy |
| **xLSTM** ([2405.04517](https://arxiv.org/abs/2405.04517)) | sLSTM exp gating + normaliser; mLSTM `C_t = f_t C_{t-1} + i_t v_t k_tᵀ` | `d×d` | O(n) | O(1) | **Yes** | **No** (exp gate + stabiliser) |
| **DeltaNet** ([2102.11174](https://arxiv.org/abs/2102.11174), [2412.06464](https://arxiv.org/abs/2412.06464)) | `W_t = W_{t-1}(I − β_t k_t k_tᵀ) + β_t v_t k_tᵀ`; WY/UT for parallelism | `d×d` | O(n) | O(1) | No | **Yes** — all rational; closest peer-reviewed analogue to a typed-operator memory |
| **GLA** ([2312.06635](https://arxiv.org/abs/2312.06635)) | `S_t = diag(α_t)S_{t-1} + k_t v_tᵀ`, two-level blocked | `d×d` | O(n) | O(1) | α via exp/sigmoid | Partially |
| **AFT** ([2105.14103](https://arxiv.org/abs/2105.14103)) | `Y = σ(Q) ⊙ (Σ_j exp(K_j + w_{t,j}) ⊙ V_j)` | — | O(n) | O(n) | **Yes** | **No** |
| **Modern Hopfield** ([2008.02217](https://arxiv.org/abs/2008.02217)) | `ξ_new = X softmax(β Xᵀ ξ)`; one step = one attention update; exponential capacity | patterns | — | — | **Yes** | **No** |
| **TTT layers** ([2407.04620](https://arxiv.org/abs/2407.04620)) | inner-loop SGD on a self-supervised reconstruction loss; mini-batch dual form | `d×d` | O(n) | O(1) | depends on loss | Partially (L2 + fixed-point lr) |
| **Titans** ([2501.00663](https://arxiv.org/abs/2501.00663)) | associative-memory loss with **surprise** metric, momentum, weight decay | `d×d` | O(n) parallel | O(1) | yes | Partially |
| **Mamba-3** ([2603.15569](https://arxiv.org/abs/2603.15569), ICLR 2026) | improved SSM-discretisation recurrence + **complex-valued state** + MIMO | `N` (**half** Mamba-2 for equal ppl) | O(n) | O(1) | complex arithmetic | Partially — complex state is phase-like |

### A.3 What is integer-only feasible, and what is fundamentally blocked

Three distinct obstacles separate these methods from an additive/lookup kernel:

1. **Transcendentals in the recurrence** (`exp(ΔA)`): the hard blocker for Mamba/S4/RWKV/xLSTM. A fixed-rate exponential is a table; Mamba's `Δ` is *input-dependent*, so the table index is a learned value. **Mamba-2's scalar-identity `A` narrows this to one scalar per token** — a materially easier problem.
2. **Softmax / normaliser division.** One reciprocal per token, not a matmul — the *least* serious obstacle, but still a divide unless implemented as a reciprocal table.
3. **Data-dependent multiplicative scaling of large states** (xLSTM stabiliser, GLA `diag(α_t)`): many small multiplies.

**Cleanly additive-feasible:** linear attention with identity/relu φ; RetNet with a fixed decay table; **DeltaNet's delta rule** (all-rational — a scaled outer-product subtraction); and the *read* side of any addressed memory. This matters: the project's "no multiplier in the declared numerical kernel" target **is** satisfiable for a delta-rule-style typed-operator memory, which is the closest peer-reviewed mechanism to what the project is building.

---

## B. Vector Symbolic Architectures / Hyperdimensional Computing

### B.1 Exact operators

| VSA | Binding | Unbinding | Bundling |
|---|---|---|---|
| **HRR** (Plate 1995) | circular convolution `z_j = Σ_k b_k a_{(j−k) mod d}` | circular correlation `a ⊛ b†` | addition |
| **MAP** (Gayler 1998/2003) | elementwise product | same product (self-inverse) | addition |
| **BSC** (Kanerva 1995/1996) | XOR | XOR | majority rule |
| **Sparse Block Codes** (Laiho 2015; [2310.16158](https://arxiv.org/abs/2310.16158)) | block-local circular convolution (⇒ modular index addition for maximally sparse blocks) | block-local correlation | block sum |

Sequence encoding uses a **permutation** `ρ` per position: an ordered n-let is `a₁ ⊛ ρa₂ ⊛ ρ²a₃ ⊛ …`. This is the standard VSA idiom (Plate 1995; Gayler 1998) and is exactly the project's "ordered n-lets as addresses" construction. The project's prime-address idea is a special case of role–filler binding with distinct hash identities as roles.

### B.2 Capacity — the arithmetic that matters

**(Correction: the HRR-learning paper is [arXiv:2109.02157](https://arxiv.org/abs/2109.02157), not `2309.07364`. `2309.07364` is *Hodge-Aware Contrastive Learning*.)**

Plate's HRR superposition capacity follows from an SNR argument. With components `~N(0,1/d)`, unbinding one of `m` superposed pairs gives signal power `1/d` and noise power `≈ m/d`, hence

```
SNR = d/m  per component,   amplitude SNR = √(d/m)
```

consistent with the `√(d/m)` form quoted across secondary sources. BSC majority-rule bundling of `n` vectors yields expected normalised Hamming distance `½ − (1−2p)/2ⁿ · C(n−1, (n−1)/2)` → `½ − 1/√(2πn)` for `p=0`, i.e. the **same `√(d/m)` law** ([Kleyko, arXiv:2003.11458](https://arxiv.org/abs/2003.11458)). Frady, Kleyko & Sommer's detection-theoretic accuracy expression is the general theory, and the survey states capacity "increases roughly linearly with the hypervector dimension." A 2023 information-rate bound ([Neural Computation 35(7):1159](https://doi.org/10.1162/neco_a_01590)) improves achievable density from 1.20 → **1.40 bits/dimension** (small codebooks) and 0.60 → **1.26 bits/dimension** (large codebooks). Resonator networks ([arXiv:1906.11684](https://arxiv.org/abs/1906.11684)) achieve operational capacity **quadratic in *N***: `M_max ≈ 8.078 N² / F^1.268`.

> **UNVERIFIED:** the literal printed theorem form in Plate (1995)/2003. The `SNR = √(d/m)` relation and the `m ∝ d/log D` consequence are derived from verified noise statistics, not quoted from Plate's text. Treat the constants as approximate.

**Concrete capacity table**

| Representation | Dim *d* | Items @90 % | Items @99 % | Source |
|---|---|---|---|---|
| HRR/MAP real, `D=10³` | 10,000 | 523 | **341** | derived from `SNR = √(d/m)` |
| HRR/MAP real, `D=10³` | 4,096 | 214 | 140 | same |
| BSC binary majority, `D=10³` | 10,000 | 115 | **87** | [Kleyko 2003.11458](https://arxiv.org/abs/2003.11458) Eq. 11 |
| BSC binary majority, `D=10³` | 4,096 | 47 | 35 | same |
| HRR + projection, error-free | 256 | **1024 bindings** | — | [Ganesan et al. 2109.02157](https://arxiv.org/abs/2109.02157) App. D |
| Resonator (factorisation) | *N* | `M_max ≈ 8.078N²/F^1.268` | same | [Kent et al. 1906.11684](https://arxiv.org/abs/1906.11684) Eq. 26 |

Under a stricter union-bound criterion (beat `m−1` other pairs *and* `D−1` distractors), the same formula gives only **m = 35 (90 %)** and **m = 16 (99 %)** at `d = 10⁴`. Binary codes are **~4.5× worse than real HRR** at fixed *d* (87 vs 341 at 99 %).

> **CONTRADICTION FLAG (important, and it supports the project's design choice).** At `d = 10⁴`, superposition retrieves on the order of **10² reliably-addressable bound pairs** — versus a single Llama-3.1-8B layer holding a 128 K-token KV cache across 32 heads (~10⁵ keys per head) with exact key→value addressing. The gap is **4–5 orders of magnitude**, and Plate's capacity is *linear in d*, i.e. ~3 orders of magnitude less parameter-efficient per association than an exact KV store (a *d*-float VSA holds ~341 pairs ≈ 3×10⁻⁴ pairs/float; a KV cache stores 1 pair per 2 floats exactly). **Conclusion: superposition cannot be the primary store.** This is direct external support for the project's choice of **exact addressed memory over distributed superposition** — and a refutation of any plan in which "binding replaces attention."

### B.3 Can binding substitute for attention? — No, and here is why

- **Binding is not selection.** Attention is *content-addressable, data-dependent* routing with learned query/key projections; VSA binding is a fixed algebraic operation. A bound pair is retrievable **if you already know the role** — no VSA operation reads content and emits different weights per token position.
- **No competitive language-scale demonstration exists.** The recent VSA/HDC-for-language literature is probes and read-outs, not models:
  - **Hyperdimensional Probe** ([arXiv:2509.25045](https://arxiv.org/abs/2509.25045), Sep 2025; v3 Jul 2026) **decodes** LLM representations with a `D=4096` bipolar codebook of 2,996 concepts and a 55–71 M-parameter encoder, applied to models from 355 M to 109 B parameters. A **read-out layer**, ~1 GB.
  - **Learning with HRR** ([arXiv:2109.02157](https://arxiv.org/abs/2109.02157), NeurIPS 2021) — extreme multi-label text classification (up to Amazon-670K labels), compression plateauing at `d' = 2500`. **No language modelling.**
  - **HDC classifier heads** (Ayar et al.) replace BERT/GPT-2 *heads* with hypervectors: 83 % on IMDb sentiment with a 9 KB model.
  - **"Attention as Binding"** ([arXiv:2512.14709](https://arxiv.org/abs/2512.14709), Dec 2025) argues attention ≈ soft unbinding — but it is a **position paper under review with no experiments and no scale**.

  **The largest VSA-side artifact found is a 71 M-parameter probe. No paper trains a VSA/HDC language model on token-scale corpora.**
- **Structural limits.** Tensor-product binding needs `O(dⁿ)` space; the CSUR survey (§VI-A2) concedes exact-reconstruction guarantees hold only for "data structures of limited size," beyond which representations are "lossy."

### B.4 The result that *supports* a VSA-flavoured design

Bricken & Pehlevan (NeurIPS 2021, [arXiv:2111.05498](https://arxiv.org/abs/2111.05498)) show transformer attention relates closely **under certain data conditions** to Kanerva's Sparse Distributed Memory, and **verify those conditions hold in pre-trained GPT-2**. Modern Hopfield ([arXiv:2008.02217](https://arxiv.org/abs/2008.02217)) shows attention is one Hopfield update step with exponential storage capacity.

**So: attention *is* associative memory.** The project's exact addressed memory is not a rejection of attention's function — it is a different implementation of the same primitive. Framed that way, the external literature supports the memory design and refutes only the idea that superposition/binding alone replaces attention.

---

## C. Discrete, lattice and quantized inference

### C.1 The E8 line is real and state of the art

**QuIP#** (ICML 2024, [arXiv:2402.04396](https://arxiv.org/abs/2402.04396)) combines the **randomised Hadamard transform** for incoherence, **E8-lattice-derived codebooks** ("E8P"), and fine-tuning. E8 is chosen because it achieves **the densest unit-ball packing in ℝ⁸** (Viazovska 2017). The construction is worth studying as a design pattern:

```
E₈ = D̂₈ ∪ (D̂₈ + ½),   D̂₈ = { x ∈ ℤ⁸ + ½ | 1ᵀx even }
```

Sign extension: flipping an even number of signs of a `D̂₈` element stays in `D̂₈`, so only **7 of 8 sign bits** are needed (the 8th follows from parity). Bit budget: 8 bits index a 256-entry absolute table, 7 sign bits, 1 bit for the ±¼ shift ⇒ a `2¹⁶ = 65,536`-entry 8-D codebook decodable from a `256×8` table = **1 KiB**, fitting GPU L1. Higher rates use residual VQ (4-bit = E8P twice; 3-bit = E8P + 1-bit E8).

**QTIP** ([arXiv:2406.11235](https://arxiv.org/abs/2406.11235)) separates codebook size from bitrate via a **stateful trellis** decoder, giving ultra-high effective dimension. Its quantitative "beyond E8" claim, quantising i.i.d. Gaussian at `k=2`: scalar Lloyd-Max **0.118**, QuIP# 8-D E8P **0.089**, QTIP 256-D bitshift-trellis **0.069**, against the infinite-length distortion-rate lower bound **0.063**. So E8 is beaten by *rate-distortion theory*, not by another lattice. Node transition is `j = (i·2^{kV} mod 2^L) + c` — decodable with a `kV`-bit shift per group of `V` weights, parallelisable, and lookup-free variants (1MAD/3INST) avoid storing the codebook entirely.

> **ID correction:** QuIP# is **arXiv:2402.04396**, *not* `2308.16692`. Any internal note citing `2308.16692` for QuIP# is wrong.

**Quality-vs-bits (Llama-2, WikiText-2 PPL ↓).** ⚠️ Context length differs across papers — **QuIP#'s headline Table 2 is ctx 2048; AQLM/QTIP and QuIP#'s appendix are ctx 4096.** AQLM re-ran QuIP#'s fine-tuning and got 6.19 for 7B, not QuIP#'s own 6.66/8.22. Cross-paper comparison is unsafe.

| Method | Bits | 7B | 13B | 70B | Source |
|---|---|---|---|---|---|
| FP16 (ctx 2048 / 4096) | 16 | 5.47 / 5.12 | 4.88 / 4.57 | 3.32 / 3.12 | 2402.04396; 2401.06118 |
| **QuIP# 2-bit** | ~2.0 | 6.66 / 8.22 | 5.74 / 6.06 | **4.16** | 2402.04396 |
| QuIP# 3-bit / 4-bit | 3 / 4 | 5.79 / 5.56 | 5.10 / 4.95 | 3.56 / 3.38 | 2402.04396 |
| **AQLM 2-bit** | 2.02/1.97/2.07 | 6.59 | 5.60 | **3.94** | 2401.06118 |
| AQLM 2-bit + e2e FT | ~2.0 | 6.14 | 5.33 | 3.83 | 2401.06118 Tbl 4 |
| **QTIP 2-bit** (hybrid) | 2.00 | 5.86 | 5.11 | **3.70** | 2406.11235 Tbl 5 |
| QTIP 3-bit / 4-bit | 3 / 4 | 5.28 / 5.17 | 4.69 / 4.61 | 3.26 / 3.16 | 2406.11235 |
| **BitNet b1.58 3B** | **1.58** | 9.91 (3B) | — | — | 2402.17764 Tbl 1 |
| LLaMA FP16 3B | 16 | 10.04 (3B) | — | — | 2402.17764 Tbl 1 |

Reading: at 2 bits, the best methods reach 70B PPL ≈ **3.7–4.2 vs 3.1–3.3 FP16** — a real but bounded degradation. **No 1-bit LLM perplexity result exists in QTIP**; its LLM experiments are 2/3/4 bits only.

### C.2 BitNet b1.58 — the exact function, and the honest scope

```
W̃ = RoundClip(W / (γ + ε), −1, 1),   RoundClip(x,a,b) = max(a, min(b, round(x))),
γ = (1/nm) Σ_ij |W_ij|        ← absmean, not max, not std
```

Activations are 8-bit, per-token scaled to `[−Q_b, Q_b]` with no zero-point. The "1.58 bits" figure is `log₂3 ≈ 1.585`.

**Scope correction — the parity claim is narrower than commonly repeated.** BitNet b1.58 reports: 700 M **12.87 vs 12.33** (worse); 1.3 B 11.29 vs 11.25 (tie); **3 B 9.91 vs 10.04** (better); 3.9 B 9.62 vs 10.04. So parity holds **from 3 B upward**, not below. **No 7B/13B/30B/70B perplexity is reported at all** — those sizes appear only in latency/memory trend figures. And the energy claim (71.4× arithmetic-operations energy for matmul on 7 nm) is a *modelled* figure from Horowitz/PokeBNN energy tables, with speeds measured using a 2-bit GPU kernel, **not** a CPU measurement.

**Independent evaluation** ([arXiv:2407.09527](https://arxiv.org/abs/2407.09527), accepted to DeLTA): 1.58-bit QAT for 100 K–48 M models "provides state-of-the-art performance for small language models **when doubling hidden layer sizes**," and robustness to learning rate and weight decay shows "different patterns for small language and vision models than previously reported for large language models." **Interpretation: ternary is viable at small scale, but the capacity has to be paid back in width.** That is a direct, quantified input to the project's parameter budget.

### C.3 Multiplier-free kernels — T-MAC is the reference implementation

T-MAC (EuroSys 2025, [arXiv:2407.00088](https://arxiv.org/abs/2407.00088)) transforms mpGEMM into **bit-wise table lookup**, "simultaneously eliminating multiplications and reducing additions," with kernels that **scale linearly in weight bit-width** via `A×W = Σ_i 2^i (A×W_i)`. Verified numbers: **30 tok/s single-core and 71 tok/s 8-core for BitNet-b1.58-3B on M2-Ultra; 11 tok/s on Raspberry Pi 5; up to 4× throughput and 70 % energy reduction vs llama.cpp**; kernel speedups up to 6.6× (avg 3.6×).

**LUT sizing:** a table holds a `[1,g]×[g,2^g]` one-bit sub-GEMM ⇒ `2^g` entries, "grows exponentially with the group size *g*." But **bit-width scaling is linear**, not exponential because each additional bit is one more one-bit LUT pass. `g=4` fits exactly one ARM NEON `.TBL` register; `g=5` needs two registers and slower `.TBL2`. Mirror consolidation, table quantisation and width narrowing recover up to a **quarter** of the original footprint.

### C.4 The roofline — the decisive caveat

**Memory-bound, stated by the authors:** T-MAC §5.2 — "T-MAC's performance is primarily constrained by memory bandwidth." Device bandwidths used in the paper: **M2-Ultra 819.2, Jetson AGX Orin 204.8, Surface Book 3 58.2, Raspberry Pi 5 17.1 GB/s.** Section 5.7 says they pick "the minimum number of CPU cores that can fully leverage the memory bandwidth." Single-thread GEMV speedups of 11.2×/5.8×/4.7×/3.1× for 1/2/3/4 bits show single-core is *not* bandwidth-limited.

**Arithmetic.** 3 B params at 1.58 bits:

```
size = 1.58 × 3×10⁹ / 8 = 5.925×10⁸ bytes ≈ 0.59 GB per token (weights only)
at 100 GB/s  → T_max ≈ 169 tokens/s
```

| Measurement | tok/s | Effective GB/s | % of 100 GB/s reference | % of M2-Ultra 819.2 GB/s |
|---|---|---|---|---|
| T-MAC, 1 core, M2-Ultra | 30 | ~17.8 | 17.8 % | 2.2 % |
| T-MAC, 8 cores, M2-Ultra | 71 | ~42.1 | 42 % | **5.1 %** |
| T-MAC, Raspberry Pi 5 | 11.1 | ~6.6 | 38 % | — |
| bitnet.cpp, 3.8 B, M2-Ultra, unlimited threads | 26.75 | ~20.1 | 15.8 % | — |

> **CONTRADICTION FLAG.** Removing multipliers does **not** by itself produce frontier-class throughput. Bit-width reduction helps because it divides bytes-per-token (FP16 3 B = 6.00 GB → 1.58-bit = 0.59 GB, a **10.1× reduction**); multiplier elimination is a necessary second step so the dequantise-and-multiply path does not eat that gain. But **wall-clock tokens/s is bounded by bandwidth × bytes-per-token** once compute stops being the limiter, and measured systems sit at 5–42 % of the roofline. The correct project claim is **energy and instruction-count reduction** (T-MAC: 70 % energy; bitnet.cpp: −70.0 % on 70 B), plus the 10× smaller memory footprint that makes laptop-scale models fit at all — **not** a large throughput multiple.

**Apple Silicon bandwidth** (Apple tech-spec pages, Wayback snapshots): M1 Pro 200, M1 Max 400, M1 Ultra 800; M2 Max 400, **M2 Ultra 800**; M3 100, M3 Pro 150, M3 Max 300/400; M4 120, M4 Pro 273, M4 Max 410/546; M5 153, M5 Pro 307, M5 Max 460/614 GB/s.

> **⚠ The widely quoted "M1 = 68 GB/s" is UNVERIFIED** — Apple does not publish a base-M1 figure on those pages. An M-series HPC study ([arXiv:2502.05317](https://arxiv.org/abs/2502.05317)) *measures* "up to 100 GB/s" for M1–M4. Any internal projection resting on 68 GB/s should be re-based.

**Integer-only training.** [arXiv:2412.04787](https://arxiv.org/abs/2412.04787) (ACML 2025) shows "training with only low-precision weights is feasible even when they are constrained to ternary values," using stochastic rounding instead of straight-through estimation to avoid keeping high-precision shadow weights — but finds that **extending to 8 bits is what achieves performance on par with BitNet b1.58**, i.e. pure ternary training carries a cost. This is directly relevant to the project's Rust offline-training / integer-serving split.

---

## D. Geometric deep learning theory — what justifies and what refutes

### D.1 The blueprint, and its limit for language

Bronstein, Bruna, Cohen & Veličković ([arXiv:2104.13478](https://arxiv.org/abs/2104.13478)) derive architectures from symmetry: **symmetry → equivariance → weight sharing**. The programme yields a strong prior for grids, groups, graphs, geodesics and gauges, and a **weak or absent** geometric prior for language. Read correctly, this says a geometric prior is a *modelling choice that must be earned empirically* — **it refutes any argument of the form "the geometry must help because the geometry is natural."** It does not refute the architecture itself.

### D.2 Gauge equivariance and the icosahedron — the strongest supportive precedent

Cohen, Weiler, Kicanaoglu & Welling ([arXiv:1902.04615](https://arxiv.org/abs/1902.04615)) build a CNN on the **icosahedron** with exact gauge equivariance, reporting that the gauge constraint restricts the network enough to work with **few filters**. Two things matter: the **icosahedron** is the 2-D face of exactly the 600-cell / binary-icosahedral structure the project uses, and the mechanism is **parallel transport with a learned connection** — the same mathematical object as the project's "retained fiber and torsion" language. This is the closest peer-reviewed analogue to the project's R4/S³ transport machinery, with a *measured* parameter-efficiency benefit.

### D.3 Clifford / geometric algebra nets

CGENN ([arXiv:2305.11141](https://arxiv.org/abs/2305.11141)), CliffordNet ([arXiv:2306.08370](https://arxiv.org/abs/2306.08370)), Geometric Algebra Transformer (Brehmer et al., [arXiv:2305.18415](https://arxiv.org/abs/2305.18415)), Clifford Neural Layers ([arXiv:2209.04934](https://arxiv.org/abs/2209.04934)). These establish that geometric-algebra layers are implementable and provably equivariant — **but the reported wins are on structured physical/robotics tasks, not language.** Verdict: **neutral-to-supportive for mechanism, unsupported for language.**

### D.4 Quaternion / hypercomplex expressivity — the testable question

The project assumes quaternion state is advantageous. External evidence is mixed:

- Quaternion RNNs ([arXiv:1806.07789](https://arxiv.org/abs/1806.07789)), quaternion CNNs ([arXiv:1712.04604](https://arxiv.org/abs/1712.04604)), QuatE ([arXiv:1902.10197](https://arxiv.org/abs/1902.10197)) report parameter-efficiency gains, all arising from **weight sharing** (a quaternion layer has 4× fewer free parameters than the real matrix it generates).
- **The critical caveat, externally sourced:** Q-Jamba ([DOI 10.5281/zenodo.18673701](https://doi.org/10.5281/zenodo.18673701), Feb 2026, **NON-PEER-REVIEWED**, 547 K–813 K params, 45 experiments on one consumer GPU) reports 3.4× parameter compression and on WikiText-2 Q-Linear matching a 3.4× larger standard model (BPC 2.102 vs 2.105) — **but its own controlled dual-axis ablation states that "structured coupling — not the algebraic rules of the quaternion algebra — drives these gains for feed-forward weights, while Hamilton algebra remains essential for recurrent state transitions."**

  Treat as a hypothesis generator (tiny, unreviewed), but it draws precisely the distinction the project needs: **quaternion algebra buys expressivity in the *recurrent state transition*, and mostly mere parameter coupling elsewhere.** This is cheaply falsifiable internally.
- **RoPE** ([arXiv:2104.09864](https://arxiv.org/abs/2104.09864)) encodes position by **rotation** in 2-D subspaces, `⟨R_Θ,q t, R_Θ,k s⟩`, with theory in ["Round and Round We Go"](https://arxiv.org/abs/2410.06205) and ["Base of RoPE Bounds Context Length"](https://arxiv.org/abs/2405.14591). It is the **strongest existing evidence at frontier scale that rotation-based encoding works.** If any external result supports the project's rotation/phase thesis, it is RoPE's ubiquity.

### D.5 HARD CONTRADICTIONS

**(i) TC⁰ limits on fixed-state recurrence.** Merrill, Petty & Sabharwal (ICML 2024, [arXiv:2404.08819](https://arxiv.org/abs/2404.08819)): SSMs "cannot express computation outside TC⁰"; "they cannot solve simple state-tracking problems like **permutation composition**"; and "SSMs are provably unable to accurately track chess moves with certain notation, evaluate code, or track entities in a long narrative." Supplemented by experiments showing Mamba-style SSMs struggle with state tracking. Related: log-precision transformers ([arXiv:2207.00729](https://arxiv.org/abs/2207.00729)); saturated transformers as threshold circuits ([arXiv:2106.16213](https://arxiv.org/abs/2106.16213)); chain-of-thought expressivity ([arXiv:2310.07923](https://arxiv.org/abs/2310.07923)).

**(ii) The solvable-groups theorem (ICLR 2026).** A single-layer input-dependent complex diagonal SSM cannot express state tracking of **any non-Abelian group at finite precision**; *k*-layer DCD SSMs express exactly the groups with a subnormal series of length *k* with Abelian factors — the **solvable** groups. Empirically, multi-layer models often fail to *learn* non-Abelian tracking.

> **This is the sharpest single constraint on the project.** The project's geometry includes S₃ (order 6) and the **binary icosahedral group 2I (order 120)**. S₃ is solvable, so a multi-layer stack could in principle express it — but **2I is not solvable**, so *no* finite stack of DCD/affine layers can track it. Crucially, **adding quaternion (non-commutative) algebra to the state does not by itself escape this**, because the theorem constrains the *class of update rules*, not whether the algebra is commutative. Escaping requires a non-affine / non-diagonal state update, or serial computation outside the state.

**(iii) Error control (2026, newer than most internal knowledge).** Chung, Choi & Kim, ["Rethinking State Tracking in Recurrent Models Through Error Control Dynamics"](https://arxiv.org/abs/2605.07755) (May 2026): affine recurrent networks — "a class of models encompassing State-Space Models and Linear Attention" — **cannot correct errors along state-separating subspaces once they preserve state representations**. Consequently "practical affine trackers do not learn robust state tracking; rather, they learn finite horizon solutions governed by accumulated state-relevant error," readable only while within-class spread stays small relative to between-class separation. The predicted crossing point matched the observed failure horizon empirically. This is a second, independent, *dynamical* reason a fixed affine recurrent state fails at long horizon.

**(iv) An apparent tension worth reading carefully.** Mamba-3 claims richer state tracking from a *complex* state update, while Shakerinava et al. prove limits for *complex diagonal* SSMs. The resolution is that Mamba-3 is not purely diagonal/affine. The project should read both together: **complex/phase state is helpful but not sufficient; the update rule's class is what matters.**

---

## E. Primes and zeta zeros — what exists, and what does not

Searched hard, including deliberately unusual phrasings.

### E.1 What does NOT exist

- **No peer-reviewed body connects Riemann zeta zeros to language modelling or neural computation.** Searches for "zeta function machine learning", "Riemann zeta neural network", "zeta zeros neural network", "explicit formula machine learning", "Mertens function neural network" surfaced only (a) legitimate mathematics on zeta-zero distributions using ML *tooling*, and (b) **crank-adjacent self-published material** — e.g. a Zenodo preprint on "Softmax Thermodynamics and the Four Eigenvalue Laws of the Prime Gas", and a tribonacci/`η³−η²−η−1` framework deriving the Standard Model, microtubule coherence and a "GeoLLM". **NON-PEER-REVIEWED**; not citable as support.
- **No literature uses prime numbers as positional encodings or token addresses** in a competitive model. Searches for "prime positional encoding", "prime number embedding neural network", "coprime positional encoding", "prime factorization embedding" returned nothing substantive.
- **"Geometric Attention" (Zenodo)**, **"Sovereign-Lila-E8" (Zenodo)**, **"GeoLLM over ℤ₁₃[η]" (Zenodo)** exist but are **NON-PEER-REVIEWED** with no independent replication. The density of unrefereed claims here should raise the internal evidence bar, not lower it.

> **Blunt verdict on the zeta-zero-phase mechanism:** external literature provides **zero** supporting evidence and **zero** refuting evidence. It is an untested internal hypothesis. Treat fixed zeta-zero phases as a *fixed deterministic orthonormal phase code* — mathematically legitimate as a basis — and do **not** cite external work as motivating a semantic role for zeta zeros.

### E.2 What DOES exist and is genuinely relevant: *p*-adic and ultrametric computation

This is the real find of Section E, and it is recent.

- **v-PuNNs** (["van der Put Neural Networks for Transparent Ultrametric Representation Learning"](https://arxiv.org/abs/2508.01010), Aug 2025, rev. Jan 2026): the first architecture whose neurons are **characteristic functions of *p*-adic balls in ℤ_p**, with every weight itself a *p*-adic number. A **Finite Hierarchical Approximation Theorem** shows a depth-*K* v-PuNN with `Σ_{j=0}^{K−1} p^j` neurons **universally represents any K-level tree**. Because gradients vanish in the discrete space, optimisation uses Valuation-Adaptive Perturbation Optimisation. Reported: **WordNet nouns (52,427 leaves) 99.96 % leaf accuracy in 16 min CPU-only**; GO molecular function 96.9 % leaf / 100 % root in 50 s; NCBI Mammalia Spearman ρ = −0.96 vs true taxonomic distance; **learned metric perfectly ultrametric (zero triangle violations)**.

  **Why this matters:** a *discrete, integer-valued, hierarchy-native* network beating Euclidean baselines on a genuinely hierarchical **linguistic** resource. The "gradients vanish → use discrete perturbation optimisation" finding is directly relevant to the project's typed-operator / table-lookup training problem.

- **Continuous Optimisation for *p*-adic Models** (Salazar, Kanevsky, Harvey, Getreuer & Dixon, [arXiv:2609.25501](https://arxiv.org/abs/2609.25501), Sept 2026, Google DeepMind; library `google-deepmind/padic-ml`): the **first native continuous gradient descent for *p*-adic parameters**, via the **Berkovich affine line** — a canonical path-connected expansion of ℚ_p preserving isometries and uniquely extending analytic maps, a metric tree with interpretable points and local derivatives enabling backprop. They learn linear models with coefficients in ℚ_p to do **modular arithmetic, "an XOR-like task not expressible by linear models in ℝ"**, plus momentum/Adam, linear regression, and classification on **binary-encoded hierarchies (Quillian semantic networks)**.

  **Why this matters:** the strongest external support in this report for the project's central arithmetic thesis — that there exist representational tasks which linear models over ℝ **provably cannot** express but which linear models over a *p*-adic/modular number system can. Combined with the project's exact `Z[φ]` ring and prime addresses, this is the closest peer-adjacent justification for "exact discrete arithmetic is not merely an efficiency choice but can be an **expressivity** choice."

- **A *p*-adic Perspective on Low-Bit Training** (Bershatskiy, Munkhoeva & Oseledets, ICML 2026 workshop, [OpenReview `KPcwE6qwAh`](https://openreview.net/forum?id=KPcwE6qwAh)): models the network as a **polynomial system over ℤ/p^N**, replaces activations and losses with piecewise polynomial approximations, and recasts training as root-finding, using **Hensel's lemma** to lift seed roots **digit-by-digit**. Demonstrated on linear regression and shallow polynomial networks.

  **Why this matters:** digit-by-digit Hensel lifting is *exactly* the shape of an integer-only, table-lookup-friendly procedure. This is the most directly transposable optimisation idea found for the project's discrete regime.

### E.3 Number-theoretic transforms — exact but expensive

A 2026 paper ([arXiv:2603.29129](https://arxiv.org/abs/2603.29129)) computes split component convolutions **exactly** using the **NTT (an FFT over a finite field) plus the Chinese Remainder Theorem**, with an upper bound on the number of splits and NTT-domain accumulation (at most 96 NTT calls, or 64 with accumulation). Exact integer convolution is therefore a **solved primitive**.

> **CONTRADICTION FLAG.** The same paper reports execution time of **approximately 107–1315× that of FFTW's double-precision FFT** for lengths 2¹⁰–2¹⁸, with NTTs accounting for ~80 % of total time. **Any project plan expecting exact modular-transform convolution to be a performance win at scale is unsupported.** It is viable only where exactness justifies 100×+ overhead, or at small transform sizes.

The **Fast Walsh–Hadamard Transform** is exact, add/subtract-only, O(n log n); the **randomised Hadamard transform** is already used *inside* QuIP# for incoherence — evidence that Hadamard machinery is practically useful in modern quantised inference.

### E.4 Residue Number Systems

RNS represents integers as tuples of residues mod coprime moduli, making multiplication carry-free and per-lane with CRT reconstruction. There **is** an established hardware line — multiplier-free RNS-based CNN accelerators exploiting bit-level sparsity, and affine-quantised CNNs in RNS — but the reviewed work is **FPGA/ASIC accelerator literature, not language-model results.** RNS is a credible implementation substrate for the project's multiplier-free kernel; it is **not** evidence of language capability.

### E.5 Exact addressed memory — the peer-reviewed precedents

The relevant comparators are NTM/DNC ([arXiv:1410.5401](https://arxiv.org/abs/1410.5401), [arXiv:1605.05273](https://arxiv.org/abs/1605.05273)), Memory Networks ([arXiv:1410.3916](https://arxiv.org/abs/1410.3916)), product-key memory ([arXiv:1907.05242](https://arxiv.org/abs/1907.05242)) and RETRO ([arXiv:2112.04426](https://arxiv.org/abs/2112.04426)). Measured trade-offs are in Section F.

---

## F. Compression, memorization, and what exact memory can recover

### F.1 A language model is a very good compressor — with an internal inconsistency worth knowing

Delétang et al., "Language Modeling Is Compression" ([arXiv:2309.10668](https://arxiv.org/abs/2309.10668), ICLR 2024). Table 1 raw compression rates (compressed/raw):

| Compressor | enwik9 | ImageNet | LibriSpeech |
|---|---|---|---|
| gzip (∞ ctx) | 32.3 % | 70.7 % | 36.4 % |
| LZMA2 | 23.0 % | 57.9 % | 29.9 % |
| PNG | 42.9 % | **58.5 %** | 32.2 % |
| FLAC (chunked) | 88.9 % | 60.9 % | **30.3 %** |
| **Chinchilla 70B** | **8.3 %** | 48.0 % | 21.0 % |
| Llama 2 7B | 8.9 % | 53.4 % | 23.1 % |
| Chinchilla 1B | 11.3 % | 62.2 % | 24.9 % |

> **⚠ Verified internal inconsistency.** The abstract/intro claim Chinchilla 70B compresses ImageNet to **43.4 %** and LibriSpeech to **16.4 %**, beating PNG (58.5 %) and FLAC (30.3 %). The numbers **43.4 and 16.4 appear nowhere in any table** — Table 1 reports **48.0 %** and **21.0 %**. This was checked in both the HTML and PDF of v2. Cite the table, not the abstract.

**Adjusted rates (including model size) are catastrophic:** Chinchilla 70B = **14,008 %** on enwik9 — a 140 GB (fp16) model cannot pay for itself on a 1 GB file. **Tokenisation:** "Transformers are trained on compressed data, with tokenizers acting as the compressor"; surprisingly, "Transformers compress better with **simpler** tokenizers" (ASCII 6.4 % vs BPE-20K 9.0 % at 38 M params). **Scaling ↔ compression:** larger models compress *larger* datasets better but *smaller* datasets worse — "scaling laws are, in fact, dependent on the size of the test set."

### F.2 Scaling laws, exactly

**Kaplan et al.** ([arXiv:2001.08361](https://arxiv.org/abs/2001.08361)): `L(N) = (N_c/N)^{α_N}`, `α_N ≈ 0.076`, `N_c ≈ 8.8×10¹³`; `L(D) = (D_c/D)^{α_D}`, `α_D ≈ 0.095`, `D_c ≈ 5.4×10¹³`; joint `L(N,D) = [(N_c/N)^{α_N/α_D} + D_c/D]^{α_D}` (Table 2 refit: `α_N = 0.076`, `α_D = 0.103`, `N_c = 6.4×10¹³`, `D_c = 1.8×10¹³`). Compute-optimal: `N ∝ C^0.71`, so data grows only as `C^0.29`.

**Chinchilla** ([arXiv:2203.15556](https://arxiv.org/abs/2203.15556)):

```
L(N,D) = E + A/N^0.34 + B/D^0.28,   E = 1.69, A = 406.4, B = 410.7
N_opt ∝ C^a, D_opt ∝ C^b,   a ≈ 0.50, b ≈ 0.50  (Ansatz 1: 0.488/0.501; IsoFLOP: 0.49/0.51; parametric: 0.46/0.54)
```

The ~20 tokens/parameter ratio is Chinchilla itself (70 B params, 1.4 T tokens = 20.0). Kaplan's values were `a = 0.73, b = 0.27` — the discrepancy is the whole story.

> **Correction to a widely repeated claim.** The assertion that the optimal token/parameter ratio "depends on the learning-rate schedule" is **not supported by the strongest 2024 replication**. Porian et al. ([arXiv:2406.19146](https://arxiv.org/abs/2406.19146), NeurIPS 2024 spotlight) identify the three causes of the Kaplan/Chinchilla gap as **last-layer compute cost, warmup duration, and scale-dependent optimizer tuning** — and state explicitly: "Counter to a hypothesis of Hoffmann et al., we find that careful learning rate decay is not essential for the validity of their scaling law." Fitted `a` moves 0.699 → 0.518 as corrections are applied. Data-constrained scaling ([arXiv:2305.16264](https://arxiv.org/abs/2305.16264)): `D' = U_D + U_D R*_D(1 − e^{−R_D/R*_D})` with `R*_D ≈ 15` (≈16 epochs); ≤4 epochs of repetition is loss-neutral, and its data-constrained optimum has **27 % fewer parameters** than the Chinchilla optimum at the same loss.

### F.3 Entropy of English vs SOTA bits-per-byte — is there headroom?

- **Shannon (1951)**, Bell Syst. Tech. J. 30:50–64 — human prediction game, **0.6–1.3 bits/char** (conventionally ~1.3). *UNVERIFIED from primary text* (image scan, no extractable text).
- **Cover & King (1978)**, IEEE Trans. IT-24:413 — gambling estimate, conventionally ~1.25 bits/char. *UNVERIFIED* (paywalled).
- **Grassberger** ([arXiv:physics/0207023](https://arxiv.org/abs/physics/0207023), 2002) — NSRPS on ~135 GB of Project Gutenberg: effective **1.82 bits/char**, extrapolated true entropy **0.7 ± 0.2 bits/char**.

Converting compression rate `r` → `8r` bits/byte (**derived arithmetic**): Chinchilla 70B on enwik9 = **0.664 bits/byte**; Llama 2 7B = 0.712; Chinchilla 7B = 0.816; Chinchilla 1B = 0.904. Pile bits-per-byte ([RETRO Table 15](https://arxiv.org/abs/2112.04426)): GPT-3 175B ranges 0.566 (uspto) to 1.371 (dm_mathematics); Gopher 0.506–1.135; RETRO 7.5B 0.199 (github) to 1.178 (ubuntu_irc).

> **CONTRADICTION FLAG for any "there is lots of headroom" assumption.** Chinchilla 70B's **0.664 bits/byte already sits below Shannon's 1951 human estimate (~1.3 bits/char) and below Grassberger's extrapolated 0.7–0.8 bits/char** for Wikipedia-style English. Remaining theoretical headroom on this corpus is plausibly **< 0.1 bit/byte (~10–15 %)**. Caveats: enwik9 is Wikipedia XML, not conversational English, and bits/char ↔ bits/byte is approximate. **Practical implication:** the project should not assume that a geometrically superior architecture has a large loss-reduction reservoir available on standard text — the easy entropy is largely captured already.

### F.4 Memory and retrieval — how much capability exact addressed memory recovers

| Method | Corpus | Result | Source |
|---|---|---|---|
| **kNN-LM** (247 M + 103 M entries) | WikiText-103 | **18.65 → 16.12 ppl** (−13.6 %) | [1911.00172](https://arxiv.org/abs/1911.00172) |
| **kNN-LM** (100 M + 3 B entries) | WIKI-3B | **19.59 → 13.73 ppl** — beats a model trained on **30× more unique data** (15.17) | 1911.00172 |
| **RETRO** 7.5 B + 2 T-token DB | Pile | ≈ GPT-3 175B "despite using **25× fewer parameters**"; github **0.199** vs 0.420 baseline vs 0.645 GPT-3 | [2112.04426](https://arxiv.org/abs/2112.04426) |
| **Memorizing Transformer** (2048 ctx + 65 K mem) | arXiv / GitHub | 2.69 → **2.26** / 2.22 → **1.80** ppl (−16 % / −19 %) | [2203.08913](https://arxiv.org/abs/2203.08913) |
| **Unlimiformer** | GovReport | ROUGE-1 48.7 → **56.6** | [2305.01625](https://arxiv.org/abs/2305.01625) |
| **gzip + kNN** | 7 datasets | paper mean 0.520 → **0.820** after correction, vs BoW 0.878 | [2212.09410](https://arxiv.org/abs/2212.09410) / [2307.15002](https://arxiv.org/abs/2307.15002) |
| **GPT-family capacity** | measured | **3.51 (bf16) / 3.83 (fp32) bits per parameter** | [2505.24832](https://arxiv.org/abs/2505.24832) |

**The "Less is More" gzip result does not survive replication.** Opitz ([arXiv:2307.15002](https://arxiv.org/abs/2307.15002)) reports bag-of-words **0.878** and BERT **0.881** vs gzip **0.820** across 8 datasets. The dataset audit found train/test leakage (`DengueFilipino` **100.0 %**, `KirundiNews` **90.4 %**, `KinyarwandaNews` **23.8 %** of test present in train) and that the paper used **top-2 accuracy**. With standard `k=1`, AG News 0.937 → **0.876**, YahooAnswers 0.638 → **0.485**, Ohsumed 0.521 → **0.365**. **Do not use gzip-compression-distance as a baseline without re-deriving it.**

**Memorization scaling** ([arXiv:2202.07646](https://arxiv.org/abs/2202.07646), ICLR 2023): **10× model size → +19 pp** of extractable sequences (log-linear, R² = 99.8 %); extractability rises log-linearly from 2 to 900 duplications; GPT-J 6B memorizes **≥1 % of The Pile**. Prompt length: **33 %** extractable at 50 tokens of context vs **65 %** at 450.

### F.5 The blunt quantified answer

Using the two best-measured constants — **3.6 bits/param** ([arXiv:2505.24832](https://arxiv.org/abs/2505.24832)) and **20 tokens/param** (Chinchilla) — a compute-optimal model carries

```
3.6 bits/param ÷ 20 tokens/param = 0.18 bits of verbatim storage per training token
English carries ≈0.7 bits/byte ≈ 2.8 bits/token (at ~4 bytes/token)
→ verbatim capacity covers ≈6% of the information in its own training stream
```

For Chinchilla 70B: `7×10¹⁰ × 3.6 bits ≈ 2.5×10¹¹ bits ≈ 31.5 GB` of maximum recoverable content against ~5.6 TB of training tokens ⇒ **≤0.6 % of the corpus is even in principle verbatim-recoverable.**

> **CONTRADICTION FLAG (and the most useful framing in this report).** A language model is **not** mostly a compressed corpus. It is a *small learner plus statistics*, storing ~6 % of its training stream verbatim and compressing the rest into generalisation. **Exact addressed memory recovers the verbatim fraction essentially for free** — and the measured payoff is large: **~25× in parameters** (RETRO), **~10–30× in training tokens** (kNN-LM), **13.6–19 % perplexity** on recall-dominated tasks. But on reasoning-heavy Pile subsets the retrieval win shrinks to ~1×, and RETRO is *worse* than GPT-3 on several (arxiv 0.714 vs 0.838). **Memory is worth roughly 10–25× on recall, and roughly 1× on reasoning.** The project should expect exact addressed memory to buy knowledge and copying, and to buy *nothing* for the capabilities in Section D.5.

---

## G. Newest work (2025–2026) — newer than a naive internal knowledge cutoff

| Topic | Result | Venue / ID | Date |
|---|---|---|---|
| Diagonal SSM state-tracking limits | *k*-layer DCD SSMs express exactly solvable groups with subnormal series length *k*; 1 layer cannot do any non-Abelian group at finite precision | **ICLR 2026** (no arXiv ID located) | 2026 |
| State-tracking error dynamics | Affine recurrent nets (SSM & linear attention) cannot correct errors along state-separating subspaces | [arXiv:2605.07755](https://arxiv.org/abs/2605.07755) | May 2026 |
| **Mamba-3** | SSM-discretisation recurrence + **complex-valued state** + MIMO; **+1.8 pp @1.5 B** over Gated DeltaNet; Mamba-2 perplexity at **half state size** | [arXiv:2603.15569](https://arxiv.org/abs/2603.15569) / ICLR 2026 | Mar 2026 |
| ***p*-adic continuous optimisation** | First native continuous GD for *p*-adic parameters via Berkovich affine line; ℚ_p linear models do modular arithmetic, provably not expressible in ℝ | [arXiv:2609.25501](https://arxiv.org/abs/2609.25501) (DeepMind) | Sep 2026 |
| ***p*-adic low-bit training** | Network as polynomial system over ℤ/p^N; **Hensel lifting** digit-by-digit | ICML 2026 workshop `KPcwE6qwAh` | 2026 |
| **v-PuNNs** | Neurons = characteristic functions of *p*-adic balls; universal K-level tree representation; **99.96 % WordNet leaf acc.**, perfectly ultrametric metric | [arXiv:2508.01010](https://arxiv.org/abs/2508.01010) | Aug 2025 / rev. Jan 2026 |
| NTT exact convolution | Exact integer cyclic convolution via NTT+CRT, **107–1315× slower than FFTW** | [arXiv:2603.29129](https://arxiv.org/abs/2603.29129) | Mar 2026 |
| T-MAC | LUT-based mpGEMM; **30/71 tok/s** BitNet-3B on M2-Ultra; 4× throughput, 70 % energy | [arXiv:2407.00088](https://arxiv.org/abs/2407.00088), EuroSys 2025 | v2 Mar 2025 |
| BitNet.cpp | 2.37–6.17× x86, **1.37–5.07× ARM**, energy −55.4 to −70.0 % (M2) | [arXiv:2410.16144](https://arxiv.org/abs/2410.16144) | Oct 2024 |
| BitNet b1.58 Reloaded | 1.58-bit QAT SOTA for small models **when doubling hidden sizes**; LR/WD robustness differs from large models | [arXiv:2407.09527](https://arxiv.org/abs/2407.09527) | Jun 2024 |
| Direct quantized training | Ternary-only training feasible via stochastic rounding; **8-bit needed to match BitNet b1.58** | [arXiv:2412.04787](https://arxiv.org/abs/2412.04787), ACML 2025 | Dec 2024 |
| LM capacity | **3.5–3.6 bits/param**; "grokking" begins when capacity fills | [arXiv:2505.24832](https://arxiv.org/abs/2505.24832) | May 2025 |
| Kaplan/Chinchilla reconciliation | Gap caused by last-layer cost, warmup, **optimizer tuning** — *not* LR schedule | [arXiv:2406.19146](https://arxiv.org/abs/2406.19146), NeurIPS 2024 | Jun 2024 |
| Q-Jamba | Quaternion-native hybrid LM, 3.4× param compression; *structured coupling, not quaternion algebra, drives FFN gains* | [DOI 10.5281/zenodo.18673701](https://doi.org/10.5281/zenodo.18673701) **NON-PEER-REVIEWED** | Feb 2026 |
| GeoLLM (cubic *p*-adic ring) | Language model over ℤ₁₃[η] | Zenodo **NON-PEER-REVIEWED** | 2026 |
| E8-root-system attention | "Sovereign-Lila-E8" | Zenodo **NON-PEER-REVIEWED** | 2026 |
| "Attention as Binding" | Position paper arguing attention ≈ soft unbinding; **no experiments** | [arXiv:2512.14709](https://arxiv.org/abs/2512.14709) | Dec 2025 |

**Is the field moving toward or away from the project's thesis?** Split verdict, and the split is informative.

*Away, on architecture.* Transformers + quantisation + scale remain dominant. The strongest 2026 architecture result (Mamba-3) is a **hybrid SSM with complex state and explicit attention components**, framed as buying back state tracking *within* the recurrence paradigm, not as abandoning it. Hybrids with input-dependent sparse attention are what close the recall gap (Zoology: 97.4 %).

*Toward, on arithmetic and geometry.* Discrete/*p*-adic/ultrametric learning is now a real, well-resourced line with DeepMind tooling and a first-class optimiser; exact-integer convolution is a solved primitive (at a cost); lattice quantisation with **E8** is state of the art; LUT-based multiplier-free kernels ship with measured ARM numbers; and 3.6 bits/param is a measured capacity constant the project can design against.

---

## H. Consolidated contradiction list (ranked by threat)

1. **Non-solvable group state tracking is provably out of reach for diagonal/affine recurrent state** (ICLR 2026; [arXiv:2404.08819](https://arxiv.org/abs/2404.08819)). **The binary icosahedral group (order 120) is not solvable.** Quaternion-valued state does **not** by itself escape this, because the constraint is on the *update-rule class*, not commutativity. **Action: state explicitly which update rule the recurrent state uses and check it against this theorem before claiming S₃ / 2I / 600-cell state tracking.**
2. **Affine recurrences cannot correct state-separating error** ([arXiv:2605.07755](https://arxiv.org/abs/2605.07755)) — tracking degrades to a finite horizon. Second, independent failure mode.
3. **Associative recall is the measured gap** ([arXiv:2312.04927](https://arxiv.org/abs/2312.04927)): 82 % of the perplexity gap is recall; 70 M attention beats 1.4 B gated convolution. **Any attention-free parity claim must be evaluated on MQAR-style recall, not only authored panels.**
4. **A language model is ~6 % verbatim corpus, not a compressed corpus** ([arXiv:2505.24832](https://arxiv.org/abs/2505.24832) + Chinchilla). Exact memory buys 10–25× on recall and ~1× on reasoning.
5. **Superposition capacity is ~10² items at d = 10⁴** — 4–5 orders of magnitude below a single 8B model's KV cache. Supports exact addressed memory; refutes "binding replaces attention."
6. **Multiplier elimination is not a throughput win** — decode is bandwidth-bound (T-MAC's own §5.2); measured systems sit at 5–42 % of roofline. The real wins are energy (−70 %) and a 10.1× smaller footprint.
7. **Exact modular transforms are 100–1300× slower than optimised FFT** ([arXiv:2603.29129](https://arxiv.org/abs/2603.29129)). Budget for exactness or avoid it.
8. **Little entropy headroom remains on standard text** — Chinchilla 70B already at **0.664 bits/byte**, below Shannon's 1951 human estimate. A better architecture should not be expected to find a large loss reservoir on Wikipedia-like English.
9. **No external evidence for zeta-zero or prime-address mechanisms.** Not refuted; simply unevidenced. Justify internally.
10. **The geometric prior for language is weak** ([arXiv:2104.13478](https://arxiv.org/abs/2104.13478)). Geometry must be earned empirically.
11. **Quaternion gains may be parameter coupling, not algebra** (Q-Jamba ablation, non-peer-reviewed) for feed-forward weights; the claimed exception is recurrent state transitions.

## I. Supportive findings worth acting on

1. **Mamba-3 is the blueprint to study** ([arXiv:2603.15569](https://arxiv.org/abs/2603.15569)): complex-valued state, SSM-derived recurrence, MIMO for quality at constant decode latency, half the state size for equal perplexity — peer-reviewed.
2. ***p*-adic linear models do modular arithmetic that ℝ-linear models provably cannot** ([arXiv:2609.25501](https://arxiv.org/abs/2609.25501), DeepMind) — strongest support for exact discrete arithmetic as an *expressivity* choice. **Hensel lifting** (ICML 2026) is a concrete digit-by-digit integer training procedure.
3. **v-PuNNs** ([arXiv:2508.01010](https://arxiv.org/abs/2508.01010)): a fully discrete, integer-valued, hierarchy-native network beating Euclidean baselines on **WordNet**, with the finding that gradients vanish and discrete perturbation optimisation is required.
4. **Gauge-equivariant icosahedral CNN** ([arXiv:1902.04615](https://arxiv.org/abs/1902.04615)): peer-reviewed precedent for icosahedral symmetry + learned parallel transport, with measured parameter efficiency.
5. **RoPE** ([arXiv:2104.09864](https://arxiv.org/abs/2104.09864)): frontier-scale evidence that rotation-based encoding works.
6. **Attention *is* associative memory** (Hopfield equivalence [2008.02217](https://arxiv.org/abs/2008.02217); attention-approximates-SDM [2111.05498](https://arxiv.org/abs/2111.05498), conditions verified in GPT-2). The project's exact addressed memory is a legitimate re-implementation of the same primitive.
7. **E8 is independently chosen for optimal 8-D packing** in QuIP#, with a 1 KiB, 65,536-entry codebook decodable from a 256×8 table ([arXiv:2402.04396](https://arxiv.org/abs/2402.04396)) — a concrete, LUT-friendly design pattern.
8. **T-MAC proves the LUT numerical-kernel thesis implementable** with measured ARM numbers and NEON `.TBL` register-level detail ([arXiv:2407.00088](https://arxiv.org/abs/2407.00088)).
9. **RETRO shows exact retrieval substitutes for ~25× parameter scale** ([arXiv:2112.04426](https://arxiv.org/abs/2112.04426)).
10. **DeltaNet's delta rule is all-rational and additive-feasible** ([arXiv:2102.11174](https://arxiv.org/abs/2102.11174), [arXiv:2412.06464](https://arxiv.org/abs/2412.06464)) — the closest peer-reviewed mechanism to a typed-operator memory that respects the project's numerical-kernel contract.
11. **Hybrid sparse input-dependent attention closes 97.4 % of the recall gap at sub-quadratic cost** ([arXiv:2312.04927](https://arxiv.org/abs/2312.04927)) — a bounded-routing precedent matching the project's language.

---

## J. Corrections to likely-internal citation errors

| Wrong | Correct | Notes |
|---|---|---|
| `2308.16692` for QuIP# | **arXiv:2402.04396** | ICML 2024, Tseng et al. |
| `2307.09288` for RetNet | **arXiv:2307.08621** | `2307.09288` is RWKV |
| `2309.07364` for "Learning with HRR" | **arXiv:2109.02157** | `2309.07364` is *Hodge-Aware Contrastive Learning* |
| "Pythia-6.9B" as an SSM comparison | verified as written in [2405.21060](https://arxiv.org/abs/2405.21060) | Mamba-2 2.7B > Pythia-6.9B |
| "Chinchilla compresses ImageNet to 43.4 %" | **48.0 %** per Table 1 | abstract/table mismatch in [2309.10668](https://arxiv.org/abs/2309.10668) |
| "M1 bandwidth = 68 GB/s" | **UNVERIFIED** | Apple publishes no base-M1 figure; M-series HPC study measures ≤100 GB/s |
| "optimal token/param ratio depends on LR schedule" | **not supported** | [arXiv:2406.19146](https://arxiv.org/abs/2406.19146): it is last-layer cost, warmup, optimizer tuning |
| "4-bit LUT blows up exponentially" | **linear in bit-width** | [arXiv:2407.00088](https://arxiv.org/abs/2407.00088); only group size `g` is exponential |

## K. Residual open items

Not fully resolved in this pass, and therefore to be treated as leads rather than facts:
(i) Plate (1995/2003) literal theorem statement and constants — the `SNR=√(d/m)` law is derived from verified noise statistics, not quoted; (ii) per-task numbers from "When Are 1.58 Bits Enough?" (SciTePress 2025, DOI 10.5220/00133824); (iii) whether an arXiv version of the ICLR 2026 diagonal-SSM paper exists; (iv) exact T-MAC LUT footprint at 4 bits on ARM; (v) Grassberger/Brown entropy figures confirmed only via secondary citation. **Everything marked UNVERIFIED above should be independently re-checked before entering a design calculation.**
