# CS/ML architecture and scaling review (agent: arch)

Scope: the D8 joint learner (`crates/uor-r4-training`) and the integer serving crate (`crates/uor-r4-integer`), training efficiency on an M1, a breakdown of the capacity gap, quaternion versus Householder transport, contextual access under D0-b, the architecture I recommend, and distillation.
Experiment scripts are in `<scratchpad>/exp/arch/`: `params_flops.py`, `budgets.py`, `quat_recurrence_check.py`, `codebook_attention.py`, and `mulbench/{main2,main3,qsq}.rs`.
Labels: [SOURCE file:line], [MEASURED], [LITERATURE url], [DERIVED], [HYPOTHESIS].
Before hand-off, I re-matched every [LITERATURE] number and quote against the text actually retrieved this session, using `exp/arch/verify_clean.py` on this agent's transcript. Four wordings were corrected: the Zoology bound, Mamba-3 and parity, MobileLLM's token count, and SmolLM2's corpus types.

## 1. Executive verdict

- **What the model is.** It is a one-layer GRU-variant RNN (d=256) with one softmax read head over all earlier states (256 tokens) and a pointer-sentinel copy mixture. It has **1,678,466 parameters** (lead's figure confirmed). **1,048,576 of them (62.5%) are the tied embedding.** Only **629,890 are non-embedding**. The output head is **62.7% of forward FLOPs** and **67% of serving MACs**. [SOURCE+DERIVED]
- **Poor language is mainly a training-exposure (compute) problem, secondarily a capacity problem, and the architecture is third.** At the same ~30M tokens, the project's own 7.16M-parameter transformer (#1014) scored **2.131 dev / 2.127 sealed**. That is no better than these learners: **2.092 (ordinary) / 2.117 (quaternion)** on the full 249,856-target development population. Given 5× more tokens, the same transformer reached **1.580 / 1.573**. [SOURCE]
- **The quaternion-versus-Householder result says nothing either way.** The gap is 0.025 nats with one seed per arm, below the typical run-to-run variation (Kaplan reports about 0.05 at 3M parameters). Also, inside a *nonlinear* GRU a rotation transport cannot add expressive power. A negative Goal-R result was therefore expected, and it does not bear on geometry in general. [SOURCE+LITERATURE+DERIVED]
- **Training throughput is the binding constraint.** Each arm achieves about **14 GFLOP/s**, roughly **28 GFLOP/s for the whole M1**. The #1014 transformer achieved **≈0.32 TFLOP/s on M1 Metal (MPS)**: 12.5% of the 2.6 TFLOPS peak and about **12× more machine-level compute**, 5.2× the tokens/s at 4.3× the parameters. A step-by-step nonlinear unroll with hundreds of tiny ops per token cannot use the GPU. [DERIVED from SOURCE]
- **Integer serving mostly pays for emulated multiplication.** The project's shift-add `checked_mul` costs **~45–120×** a hardware multiply (measured). A full-context token needs **97,217 software products and 9,472 software divisions** (counted from source). On this VM that arithmetic is **~92%** of a synthetic per-token budget. A D0-b-legal exact **quarter-square lookup multiply** is **30–45× faster**. On a general-purpose CPU the multiplier ban costs energy rather than saving it. [MEASURED+SOURCE]
- **Recommended path: a parallel-trainable *linear* quaternion recurrence.** I show exactly (and checked numerically to ~1e-13) that it equals **Mamba-2/RetNet decayed linear attention on queries and keys rotated by a data-dependent cumulative quaternion frame** ("quaternionic RoPE"). It therefore trains with existing chunked matmul kernels. Around it: ternary channel mixing, exact pointer/n-gram memory, and optionally 1–2 lookup-table attention layers. [DERIVED+MEASURED+LITERATURE]
- **This is where geometry becomes provably load-bearing.** With q_t in the binary icosahedral group 2I (the 600-cell vertices; closure verified), one layer solves a word problem that is NC¹-complete by Barrington's theorem. Diagonal (real or complex/Mamba-3) SSMs and constant-depth transformers cannot solve it unless TC⁰=NC¹. The "4-D instead of 2-D" intuition has an exact meaning here: 2-D rotations commute, 4-D (SU(2)) rotations do not. [DERIVED+LITERATURE]
- **Realistic M1 targets.** The compute-optimal model for one base-M1 week (≈1.8e17 FLOP) is about **40M parameters on about 0.8B tokens**. About **20–30M** parameters on 1–2.5B tokens is a plausible TinyStories-class coherence target. **Models of 100M+ parameters cannot be trained from scratch to useful general quality on an M1.** General conversation or coding needs curated synthetic data, distillation, or weight transfer from an open model. [DERIVED+LITERATURE]
- **Has the project drifted?** In CS/ML terms, yes. Effort has gone into admission sparsity at 256 tokens, where attention is about 5% of compute, and into integer emulation of a 0.63M-parameter core. The dominant costs are elsewhere: compute exposure, the output head, and sequential training. A breakthrough claim needs a re-base onto the parallel linear-recurrence family, with geometry tested where theory says it can win (state tracking, codebooks), not in next-token loss of a GRU. [HYPOTHESIS]

## 2. Findings

**F1 — Exact model** [SOURCE joint_model.rs:938-1130, 1314-1340, 1707-1780; config.rs:59-85]
For input token x_t, with E ∈ R^{4096×256} tied, and rms(·) the parameter-free RMS normalization:

- f_t = W_in E[x_t] + W_s rms(h_{t-1}) + b, with W_in, W_s ∈ R^{768×256}.
- c_t = tanh(f[0:d]); z_t = σ(f[d:2d]); raw_t = f[2d:3d].
- Transport, per 4-D lane k (64 lanes):
  - Quaternion arm: T_k = q_k ⊗ h_{t-1,k}, where q_k = normalize(e0 + 0.1·raw_k).
  - Householder arm: T_k = H(v_k)H(e0)h_{t-1,k}, where v_k = normalize(e0 + 0.0707·raw_k).
- Provisional state: p_t = (1−z_t)⊙T + z_t⊙c_t; n_t = rms(p_t).
- Read, over all earlier events i<t:
  - query and null: q = W_q n_t + b_q (64-d); null = w_∅·n_t + b_∅.
  - keys and values: k_i = W_k rms(h_i) + b_k; v_i = tanh(W_v rms(h_i) + b_v).
  - masses: [a_∅, a_i] = softmax([null, q·k_i/8 + age[t−1−i]]); read_t = Σ_i a_i v_i.
- Update, where ρ is a *single scalar* gate: u = [p_t; read_t]; h_t = (1−ρ)p_t + ρ·tanh(W_u u + b_u), with ρ = σ(w_ρ·u + b_ρ).
- Copy gate: g = σ(w_g·[h_t; read_t] + b_g).
- Output: P = (1 − g(1−a_∅))·softmax(E·(rms(h_t)⊙γ) + b_o) + g·Σ_i a_i·onehot(x_i), then mixed with a 1e-8 uniform distribution.

This is a one-layer RNN with one attention head plus a **pointer-sentinel mixture**; the NoRead slot is the sentinel [LITERATURE https://arxiv.org/abs/1609.07843].

Parameters and costs [DERIVED, `params_flops.py`]:

| Component | Parameters |
|---|---:|
| Recurrent maps | 393,984 |
| Read (Q, K, V, age, null) | 99,200 |
| Update and gates | 132,354 |
| Output norm and bias | 4,352 |
| Embedding | 1,048,576 |

- Forward cost: 3.43 MFLOP/token, of which the read is 81.6k (averaged over the window).
- Training cost: ≈10.3 MFLOP/token (6N rule: 10.07).
- Serving cost at full context: 1.56M MACs/token if the E·W_in product is tabulated. The output head is 1.05M of these (67%); the read at t=255 is 81.6k (5%); transport is 1k (0.1%).

**F2 — Training recipe** [SOURCE joint_optimizer.rs:34-37,107-116,389-391; joint_campaign.rs:384-425,976-1011; docs/integration/joint-fit256-householder_pair-2026-09-24.json]

- AdamW with β=(0.9, 0.999), global clip 1.0, and weight decay 0.01 applied to every tensor that receives a gradient, with no exclusion for biases, norms or the embedding.
- **Constant LR 1e-3: no warmup and no decay/cooldown.**
- 4,096 targets per step, 7,324 steps, 29,999,104 visits in total. The first 2,104 steps used B64/T64, the rest B16/T256.
- Full BPTT through 256 steps on two CPU threads per arm.

The tune NLL was still falling fast: 2.288→2.214 (quaternion) and 2.264→2.182 (ordinary) between 19.3M and 30M tokens [SOURCE joint-recurrent-result-2026-09-25.md:75-76]. The model is under-trained and was never annealed. The literature shows the cooldown phase gives a sharp loss drop and is needed to match cosine [LITERATURE https://arxiv.org/abs/2405.18392]. Ternary/low-bit models show an "S-shaped" drop as the LR goes to 0 [LITERATURE https://arxiv.org/abs/2406.02528].

**F3 — Training efficiency** [DERIVED from SOURCE joint-recurrent-result:172-173 and r4_softmax_end_to_end_attention_1014.md:52,97]

| Run | Tokens/s | Achieved compute |
|---|---:|---:|
| Recurrent learner, sustained per arm | 1,362 | 14.0 GFLOP/s |
| Recurrent learner, whole machine (2 arms) | — | 28 GFLOP/s |
| Recurrent learner, Metal | 823 | 8.5 GFLOP/s |
| #1014 transformer, 45.6 MFLOP/token (29,999,104 tokens in 4,220 s on MPS) | 7,108 | 0.324 TFLOP/s |

The #1014 figure is 12.5% of the M1 8-core GPU's 2.617 TFLOPS [LITERATURE https://lowendmac.com/2025/apple-silicon-m1-chip-specs]. For comparison, M1 Pro is 5.2 TFLOPS [LITERATURE https://www.extremetech.com/computing/328337-apple-unveils-new-m1-pro-monster-m1-max-socs].

Cause [HYPOTHESIS, consistent with source]: `core_step` issues about 120 small tensor ops per time step and about 3× that including backward, which is about 90k ops per worker step. The matmuls have only 8 rows, and `append_history` re-concatenates the whole K/V history at every step (O(T²) copies). All of this is serialized across 256 steps, so dispatch overhead dominates and Metal loses to the CPU.

A linear recurrence is parallelizable instead: an associative scan, or the chunkwise form used by GLA, Mamba-2 and DeltaNet [LITERATURE https://arxiv.org/abs/2412.06464; https://arxiv.org/abs/2312.00752]. Its cost is dominated by large dense matmuls, the same profile as the #1014 transformer, so MFU of about 10–25% on M1 Metal is a reasonable planning assumption [HYPOTHESIS; must be benchmarked in Rust/Candle-Metal].

Training budget for one week (6ND FLOPs + 10%) [DERIVED, `budgets.py`]:

| Parameters | Current design (28 GFLOP/s) | M1 at 0.3 TFLOP/s | M1 at 0.6 TFLOP/s | M1 Max, assumed 1.2 TFLOP/s | Chinchilla-optimal (20N) |
|---|---:|---:|---:|---:|---:|
| 10M | 0.26B | 2.75B | 5.5B | 11B | 0.2B |
| 30M | 0.09B | 0.92B | 1.8B | 3.7B | 0.6B |
| 100M | 0.03B | 0.27B | 0.55B | 1.1B | 2B |
| 300M | 0.01B | 0.09B | 0.18B | 0.37B | 6B |

**F4 — Capacity decomposition** (non-embedding parameters N, tokens D)

| Model (same TinyStories distribution) | N | D | NLL (nats/token) |
|---|---:|---:|---:|
| This learner, ordinary / quaternion [SOURCE joint-recurrent-result:95-97] | 0.63M | 30M | 2.085 / 2.110 on the comparison tail |
| #1014 transformer, 6L × 288 [SOURCE r4_softmax_end_to_end_attention_1014.md:98,107] | 5.98M | 30M | 2.131 dev / 2.127 sealed |
| #1017, the same model continued [SOURCE r4_softmax_quality_capacity_continuation_1017.md:19,32] | 5.98M | 150M | 1.580 / 1.573 |
| llama2.c stories15M (same 6×288 shape, 32K vocab), far longer training on 4×A100 [LITERATURE https://github.com/karpathy/llama2.c] | ≈6M | ≫150M | 1.072 |
| llama2.c stories42M / stories110M [same source] | — | — | 0.847 / 0.760 |

The llama2.c README states that a 4,096-token vocabulary trained on TinyStories gives about the same sequence length as the 32K Llama tokenizer. Per-token NLLs are therefore roughly comparable, assuming the project's BPE-4096 behaves similarly [HYPOTHESIS].

What this shows [DERIVED]:
- At 30M tokens, size and architecture barely matter: a model with 9.5× the non-embedding parameters did no better.
- The 0.51-nat gap to #1017 is mostly data and compute exposure.
- Even #1017 is exposure-limited: the same shape reaches about 1.07 with more training.
- Capacity becomes binding later. In TinyStories, consistency emerges only as hidden size grows from 64 to 128 and with depth; example scores were 1M: 2/10, 8.3M: 5/10, 28M: 9/10 [LITERATURE https://arxiv.org/abs/2305.07759]. A 0.63M non-embedding single-layer model will plateau above coherent-story loss [HYPOTHESIS].
- Architecture matters too: LSTMs "plateau after <100 tokens" of context while transformers keep improving [LITERATURE https://arxiv.org/abs/2001.08361].

Verdict: the "semantic unreliability" is first compute/exposure, then capacity, then architecture (single layer, weak recall).

**F5 — Quaternion versus Householder transport** [SOURCE joint_model.rs:1707-1780; DERIVED]

- **They are genuinely different families.** Both have 3 degrees of freedom per lane, are near the identity (scale 0.1 versus 0.1/√2), and have matched local Frobenius scale.
  - Quaternion: a *left-isoclinic* rotation L_q ∈ SU(2)_L. Every vector is rotated by the same angle and no plane is fixed. Compositions stay inside the 3-dimensional subgroup.
  - Householder pair H(v)H(e0): a *simple* rotation by twice the angle between e0 and v, in the plane span(e0, v). It fixes a 2-plane and always involves coordinate 0. The set is not a group, and its compositions generate all of SO(4).
- At initialization, raw ≈ N(0, 0.5) per coordinate (from Xavier W_s and unit-RMS state), giving about 7° of rotation per step per lane [DERIVED]. This is moderate state-dependent "phase noise" that the model must learn to use or suppress.
- Why the ordinary arm could be ahead [HYPOTHESIS]:
  1. Most likely noise: n=1 seed per arm, versus about 0.05 run-to-run variation [LITERATURE https://arxiv.org/abs/2001.08361, App. D.6].
  2. The fixed 2-plane keeps half of each lane untouched, which gives a cleaner memory path.
  3. Its compositions explore all of SO(4) rather than SU(2)_L.
- **Transport is not needed for expressive power here.** The candidate path already mixes the state densely through W_s. A nonlinear RNN can already express finite-state tracking in one layer [LITERATURE https://arxiv.org/abs/2404.08819]. Transport only makes the retained path block-orthogonal.
- Clean ablation, in two steps:
  1. **Free:** on the saved checkpoints, force raw→0 (identity transport) at evaluation and measure ΔNLL; also histogram the learned per-lane angles.
  2. **Cheap:** three arms (quaternion / Householder / identity) × 3 seeds × about 4M tokens at B16/T256, with confidence intervals on the paired differences.
  Geometry should instead be tested where theory predicts separation (F8).

**F6 — Integer serving arithmetic** [SOURCE uor-r4-integer/src/math.rs:59-83,113-134; model.rs:297-392; MEASURED]

- `product()` is a shift-add loop of up to 128 iterations with a data-dependent branch. It is used for every activation-by-activation product: scores (16,320), value read (65,280), mixture (12,288), normalization, blend and transport. Together with divisions this is 97,217 products and 9,472 long divisions per full-context token.
- Weight maps use the signed-4 16-multiples table (1.67M weight reads per token).
- Measurements (`mulbench/main2.rs`, `main3.rs`, `qsq.rs`; shared Xeon VM, so noisy):

| Operation | Software (project code) | Alternative |
|---|---:|---:|
| Multiply | 90–300 ns | 1.5–2.5 ns hardware i64 multiply (black_box-bound) |
| Divide | 260–300 ns | 35–40 ns (u128 `/`) |

  Per synthetic token: software products 19–23 ms, divisions 4.2 ms, table matmuls 2.2–2.4 ms, so **software arithmetic is about 92%**.
- **Exact quarter-square LUT** (ab = ⌊(a+b)²/4⌋ − ⌊(a−b)²/4⌋, 1 MiB table for 16-bit operands): **4.4–6.5 ns per product**, verified exact on 97k samples, which is 0.43–0.64 ms per token. It uses table reads and adds, so it is D0-b-legal.
  - Caveat: my binary builds the table in-process. A compliant kernel must load a precomputed table and pass `serving_multiplier_check.py`.
- I have not profiled the M1. Its measured step is 3.7 ms [SOURCE current-state.md:47], which suggests software arithmetic is the largest cost there too [HYPOTHESIS].
- Energy: on a CPU core, instruction and data-movement overheads dominate the ALU. Horowitz's 45nm numbers are: int32 add 0.1 pJ, multiply 3.1 pJ, 32KB SRAM read 5 pJ, DRAM read 640 pJ [LITERATURE http://arxiv.org/pdf/1602.01528, Table I]. Replacing a 3 pJ multiply with hundreds of instructions is energy-negative. Multiplier-free wins are real only when a product becomes about one add, as with ternary weights, or on custom silicon; for example, the 370M MatMul-free LM on Loihi 2 ran at 70.8 mJ/token, versus 785–912 mJ/token for Qwen2-500M on a Jetson Orin Nano depending on generation length [LITERATURE https://arxiv.org/abs/2406.02528].

**F7 — Contextual access under the serving constraints** [LITERATURE; MEASURED]

- **Recall needs state that grows with content.** Associative recall explains 82% of the gap between gated convolutions and attention. Gated convolutions need a model dimension that grows with sequence length to solve MQAR (their Thm 4.4), and hybrids with fewer than 10% attention layers, or sparse attention on repeated-bigram positions, close most of the gap [https://arxiv.org/abs/2312.04927]. Any recurrent model needs Ω(N) bits of state for MQAR, and 64–128-token sliding windows plus linear attention recover 90.8% of softmax recall [https://arxiv.org/abs/2402.18668]. Fixed-state models provably cannot copy beyond their state size [https://arxiv.org/abs/2402.01032]. Griffin's single local-attention layer retrieves perfectly within its window [https://arxiv.org/abs/2402.19427].
- **Cost at T=256.** Full attention is 81.6k MACs per token, about 5% of serving. Bounded admission at this length is premature optimization; it only matters at long context. The project's exact-cache64 intervention scored 2.147/2.136 versus full 2.080/2.070 [SOURCE bounded-admission-result-2026-09-25.md].
- **Geometric polar codebook for keys (the owner's "preserve the radius, use 4-D" idea).** Test: d_k=64, 256 keys, planted target, 400 trials (`codebook_attention.py`). The 600-cell direction plus a 4-bit log-radius at **2.73 bits/dim** gives top-1 1.000 and KL 0.032, about as good as int3 scalar at 3 bits (0.993, KL 0.023). It beats a random 120-point code (KL 0.048; covering radius 22.2° versus 40.1°). E8 roots plus radius at 1.49 bits/dim gives KL 0.196. [MEASURED]
- **Scoring cost with this codebook.** Scores become 16 table reads, shifts and adds per key. Tables cost 16×120 small sums per query, multiplier-free because 600-cell coordinates lie in ½·Z[φ] [DERIVED].
- **Relevant precedents.** TurboQuant stores the norm separately rather than discarding it, so "preserving the radius" is not itself new [https://arxiv.org/abs/2504.19874]. The E8 lattice is the best 8-D codebook in QuIP#: 2-bit E8P beats a 4-D D4 codebook (W2 4.156 versus 4.408 perplexity on Llama-2-70B) [https://arxiv.org/abs/2402.04396]. That gives the project's icosian E8 a literature-backed role as a *weight or KV codebook*.

**F8 — Proposed core: linear quaternion recurrence** [DERIVED; MEASURED by `quat_recurrence_check.py`]

- **Vector form.** h_t = r_t (q_t ⊗ h_{t−1}) + b_t, where r_t ∈ (0,1] is a learned radial decay (the "preserved radius") and q_t is a unit quaternion.
  - Elements (a, b) with a = r·q compose as (a₂,b₂)∘(a₁,b₁) = (a₂a₁, a₂b₁ + b₂). This is composition of affine maps, so it is associative and a parallel prefix scan applies (error 5e-15 versus the sequential form).
  - **Frame decomposition.** With U_t = q_t U_{t−1}, h_t = U_t ⊗ g_t, where g_t = r_t g_{t−1} + Ū_t ⊗ b_t is a *scalar-decay* recurrence (error 3e-14).
- **Matrix-state (recall) form.** S_t = r_t L_{q_t} S_{t−1} + k_t v_tᵀ and o_t = S_tᵀ q'_t, so o_t = Σ_{j≤t} (Π r) ⟨Ū_j k_j, Ū_t q'_t⟩ v_j (error 2e-13).
  - This is exactly decayed linear attention (Mamba-2 SSD / RetNet) on keys and queries rotated by a *data-dependent non-commutative frame*. Existing chunked kernels therefore apply after an O(T·d) rotation.
  - Mamba-3 shows the complex (2-D) special case is equivalent to a data-dependent rotary embedding (RoPE) and solves synthetic state-tracking tasks that earlier linear models fail; the paper cites parity as the standard failure case [LITERATURE https://arxiv.org/abs/2603.15569].
- **Expressivity.** 2I (verified group of order 120, minimum angle 36°) is non-solvable, so its word problem is NC¹-complete (Barrington). Diagonal/complex SSMs and constant-depth transformers are in TC⁰ [LITERATURE https://arxiv.org/abs/2404.08819]. Non-diagonal Householder-product transitions also escape TC⁰ [https://arxiv.org/abs/2411.12537; RWKV-7 https://arxiv.org/abs/2503.14456]. The advantage is therefore over *diagonal* linear RNNs, not over Householder/DeltaNet ones. The quaternion's edge there is cost: 3 parameters per 4-D block and an exact integer group.
- **Integer serving.**
  - U_t is one 120×120 Cayley-table lookup.
  - Rotating a 4-vector by an element of 2I uses Z[φ] adds and shifts (multiplying by φ maps (a,b) to (b, a+b)).
  - Decay r = 1−2^−k is computed as x − (x>>k).
  - In the frame form, only q and k are rotated per token, not the state.
  - The remaining products (k vᵀ, Gᵀq̃) need 4-bit or ternary keys/queries (a 16-entry multiples table) or the quarter-square LUT.
- **Rest of the block:** ternary GLU channel mixing (BitNet/MatMul-free style), a lookup-table readout, an exact pointer/n-gram memory (the owner's content-addressed store-and-recall), and optionally 1–2 local attention layers with polar/PQ keys for recall.

Three scales [DERIVED, `budgets.py`]:

| | S | M | L |
|---|---|---|---|
| Shape | d=384, 12 layers, 6×64 heads, GLU 1024, V=4096 | d=768, 16 layers, V=16k | d=1024, 24 layers, V=32k |
| Parameters | 22.8M | 126M | 342M |
| Ternary weight size | 5.7 MB | 31.5 MB | 85.5 MB |
| Recurrent state | 0.6 MB | 1.6 MB | 3.1 MB |
| Tokens per base-M1 week (0.3 / 0.6 TFLOP/s) | 1.2 / 2.4B (≈50–100 tokens/param) | 0.22 / 0.44B | 0.08–0.16B |
| Expected quality | TinyStories-class coherence plausible: target NLL about 1.0–1.3 on this evaluation, versus 2.09 now [HYPOTHESIS, anchored on llama2.c and TinyStories] | Undertrained (≈2–4 tokens/param). General text would be GPT-2-small-class or worse; 125M general models "rarely generate coherent text" per TinyStories [LITERATURE] | Not trainable from scratch; usable only via weight transfer |

Literature quality bands require far more compute:
- MatMul-free 370M on 15B tokens: 40.3 average versus 41.1 for Transformer++ [https://arxiv.org/abs/2406.02528].
- BitNet b1.58 700M: perplexity 12.87 versus 12.33 [https://arxiv.org/abs/2402.17764].
- BitNet 2B4T needs 4T tokens [https://arxiv.org/abs/2504.12285].

Serving speed: bitnet.cpp ran 125M/350M/700M ternary models at 593/282/194 tok/s on an M2 Ultra [https://arxiv.org/abs/2410.16144]. An M1 has 68 GB/s of memory bandwidth and 4 P-cores, so expect lower figures [HYPOTHESIS].

Hybrids: 1–2 local-attention layers fix recall at low cost [LITERATURE Zoology, Based, Griffin]. The alternative is no attention plus exact pointer memory; its recall must be verified on MQAR.

**F9 — Distillation**

- Evidence for:
  - Distillation beats supervised pretraining when a teacher already exists and student compute is below a size-dependent threshold, which is the M1 regime [LITERATURE https://arxiv.org/abs/2502.08606].
  - Gemma-2 2B: distilled 67.7 versus 60.3 from scratch at 500B tokens [https://arxiv.org/abs/2408.00118].
  - MiniLLM: reverse-KL, on-policy distillation reduces exposure bias for students from 120M to 13B [https://arxiv.org/abs/2306.08543].
  - Cross-architecture conversion: MOHAWK distilled Phi-1.5 into Mamba-2 with 3B tokens, under 1% of the teacher's data [https://arxiv.org/abs/2408.10189]. LoLCATs linearized Llama-3-8B with 40M tokens and LoRA [https://arxiv.org/abs/2410.10254].
- Evidence against: in MobileLLM, KD slowed training 2.6–3.2× and gave comparable or worse accuracy than label-based training [https://arxiv.org/abs/2402.14905].
- Recommendation, in order:
  1. Use more of the already-distilled data. TinyStories is GPT-3.5/4-generated, and only 30M–150M tokens of it have been used. Add open curated corpora of the kind SmolLM2 built for this: SmolTalk for dialogue, Stack-Edu for code (educationally filtered, not synthetic), FineMath, and the synthetic Cosmopedia-v2 [https://arxiv.org/abs/2502.02737].
  2. Online logit KD from #1017, which shares the tokenizer and costs about 14 MFLOP/token to run. This speeds early learning but caps quality near 1.57.
  3. MOHAWK/LoLCATs-style conversion of a small open model into the quaternion-frame architecture. This is the only M1-feasible route to general quality, but it transfers transformer weights and needs an explicit owner decision under D8's "no transformer backbone".

## 3. What is strong / novel versus what is weak

**Strong:**
- Honest, reproducible measurement discipline.
- A working integer end-to-end serving path.
- The pointer-sentinel read is a sound small-model design.
- The quaternion and 2I/Z[φ] machinery maps cleanly onto the frontier linear-RNN family, and it offers a *provable* state-tracking separation with exact integer serving (novel as far as I found) [HYPOTHESIS on novelty].
- The polar 600-cell codebook works (about int3 quality at 2.7 bits/dim).

**Weak or wrong:**
- Sequential nonlinear training, which starves compute.
- Constant LR with no cooldown.
- 62% of parameters in the embedding.
- A single scalar update gate.
- Geometry comparisons run with n=1 seeds.
- Admission sparsity work at 256 tokens.
- The software multiplier.
- D5 (per-token parameter sparsity) conflicts with the owner's "no MoE/sparse routing". Its energy rationale is weak for cache-resident models: a 23M ternary model is 5.7 MB and fits in the 12 MB L2 [LITERATURE lowendmac specs; DERIVED].

## 4. Recommendations (ranked)

1. **Re-base on the quaternion-frame linear recurrence (F8) with chunked training on Metal from Rust.**
   - Impact: at least 10× tokens per M1-hour, 10–30× feasible model size, a provable geometric role.
   - Cost: 2–4 weeks of engineering.
   - Falsify: a Rust microbenchmark must reach at least 0.2 TFLOP/s and at least 10× the current tokens/s at equal parameters. Then S-scale versus matched (a) diagonal-decay (Mamba-2-like) and (b) small transformer controls, 3 seeds × 0.3B tokens. Abandon if it trails the diagonal control by more than 0.05 nats.
2. **Cooldown now: linear LR decay to 0 over about 1,000–1,500 steps from the step-7,324 checkpoints.**
   - Impact: likely −0.05 to −0.2 nats [HYPOTHESIS].
   - Cost: about 1–2 hours per arm.
   - Falsify: ΔNLL on the development tail smaller than 0.02.
3. **Replace `checked_mul` and `div_round` in hot loops with quarter-square/reciprocal LUTs, or allow integer MUL (the owner's wording targets floating point).**
   - Impact: about 5–10× faster integer steps [HYPOTHESIS for M1].
   - Cost: days.
   - Falsify: M1 profile plus `powermetrics` J/token versus F32.
4. **Transport ablation (F5).**
   - Cost: hours for the evaluation intervention. For 3 arms × 3 seeds × 4M tokens, about 4–6 M1 hours (9 runs at about 1,362 targets/s per arm, two arms at a time) [DERIVED].
   - Falsify: identity transport within the seed confidence interval means transport is not load-bearing.
   - Add a 2I/A5 word-problem probe comparing a linear quaternion recurrence, a diagonal recurrence, and the GRU.
5. **Scale data and parameters only after (1).** Target the S scale for TinyStories coherence, with stories scored as in the TinyStories paper.
6. **Distillation in the order given in F9.** Falsify: KD student at matched tokens is not at least 0.05 nats better.
7. **Put D5 (per-token parameter sparsity) back to the owner** given the "no sparse routing" constraint. If it is kept, use a deterministic hierarchical vocabulary head (about 32× less output access) rather than expert routing.
8. **Geometric codebooks (600-cell polar KV; icosian-E8 weights, QuIP#-style) for long context and bigger models.** Falsify: they must beat int3/ternary at equal bits by more than 10% in KL/perplexity.

## 5. Open questions

- Achievable MFU for Candle-Metal (or mlx-rs) chunked kernels on the owner's actual M1 (base, Pro or Max?). Not measured.
- M1 profile of the integer step; my 92% share was measured on an x86 VM.
- Tokens per byte of the project's BPE-4096 on TinyStories. This is needed to make the llama2.c comparison rigorous.
- Seed variance of the current learners at this scale. It is unknown, since only one seed per arm exists.
- Whether quantizing q_t to 2I (6.9 bits) keeps language-modeling quality. This needs a straight-through estimator (STE) or Gumbel training test.
- Whether exact pointer memory alone matches 1–2 local-attention layers on MQAR at S scale.
