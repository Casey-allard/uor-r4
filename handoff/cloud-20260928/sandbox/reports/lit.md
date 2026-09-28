# Literature & novelty scout report (agent: lit), 2026-09-25

**Scope.** This report maps prior work onto the eight ideas in the brief, adds one small measurement of the owner's original quantization idea, and gives a novelty map and a reading list.

**Labels.**
- `[LITERATURE arXiv NNNN]`: I retrieved the paper this session. The URL is `https://arxiv.org/abs/NNNN`, and the reference table in §9 lists every source.
- "(abstract)": I read only the metadata and abstract, not the full text.
- `[SOURCE file:line]`, `[MEASURED]`, `[DERIVED]`, `[HYPOTHESIS]`: as defined in CONTEXT.md.

**Overlap with an earlier brief.** An earlier literature brief already covers BitNet, T-MAC, MatMul-free LM, RWKV-7, xLSTM, memory layers, Engram and Apple-silicon throughput [SOURCE docs/integration/review-2026-09-16/05-literature-brief.md:12-172]. I cite it and do not repeat it; this report concentrates on what is new.

---

## 1. Executive verdict

- **The original idea rests on a mistaken premise, and its 4-D form was published in 2026.** TurboQuant, PolarQuant and QJL all *keep* the vector norm (radius) in full precision. None of them discards it. What they remove is the per-block normalisation constants (zero point and scale). The 4-D version, with each 4-element chunk treated as a quaternion, the radius quantized, and the direction taken from a 24-cell/Hurwitz-group codebook, appeared in May 2026 (HQMQ). The companion idea of SO(4) quaternion-pair block rotations appeared in March 2026 (IsoQuant). The owner's idea is therefore not a novel contribution. [LITERATURE arXiv 2504.19874, 2502.02617, 2406.03482, 2605.27646, 2603.28430]
- **4-D is a weak operating point for rate-distortion, and a literally stored radius is expensive.** Published ablations and my own measurement agree on this. Storing an fp16 radius per 4-D block with 600-cell directions costs 5.73 bits/dim and still gives MSE 0.071, worse than a 3-bit scalar quantizer [MEASURED]. At 2 bits/dim on a Gaussian source:

  | Quantizer | MSE per dim | Source |
  |---|---|---|
  | Scalar Lloyd-Max | 0.117 | measured |
  | 2-D polar | 0.119 | measured |
  | 4-D gain-shape, 600-cell shapes | 0.107 | measured |
  | Best unconstrained 4-D VQ | 0.098 | measured |
  | E8P (8-D) | 0.089 | QTIP paper [LITERATURE arXiv 2406.11235] |
  | 256-D trellis | 0.069 | QTIP paper [LITERATURE arXiv 2406.11235] |
  | Shannon bound | 0.0625 | theory |

  On a real LLM, QuIP#'s own 4-D D4 lattice is clearly worse than 8-D E8P (Llama-2-70B at 2 bits, WikiText-2 perplexity 4.41 vs 4.16) [LITERATURE arXiv 2402.04396].
- **The 600-cell is essentially the best 4-D shape codebook at about 7 bits per block (measured).** The geometry is right; the dimension is the constraint. At 3 bits/dim a fixed 120-point polytope cannot absorb the extra bits: MSE 0.071, which is worse than scalar at 0.035 [MEASURED].
- **The strongest literature-backed role for the quaternion/H4 transport is state tracking, not TinyStories perplexity.**
  - A5 is the canonical NC¹-complete word problem, and neither transformers nor diagonal SSMs can track it at fixed depth [LITERATURE arXiv 2404.08819].
  - A5 is the icosahedral rotation group, and its double cover 2I is exactly the 120 unit icosians, i.e. the 600-cell [LITERATURE errorcorrectionzoo.org/c/600cell; DERIVED].
  - Published changes to recurrent transitions that enable state tracking leave language-model perplexity essentially unchanged (DeltaNet 18.57 vs 18.54 WikiText perplexity at 1.3B) [LITERATURE arXiv 2411.12537].
  - So D4's null result on next-token loss (quaternion 2.110 vs Householder 2.085 nats) is what the literature predicts. It is not evidence either way about the geometry's real niche [DERIVED].
- **At ~1.68M parameters (~0.63M non-embedding), "semantically unreliable" text is the expected outcome of model size.** TinyStories transformers at 1–2.5M parameters score about 1–6/10 on consistency; consistency needs roughly ≥8M parameters and width ≥128 [LITERATURE arXiv 2305.07759; MEASURED parameter count from SOURCE crates/uor-r4-integer/src/config.rs:59-85].
- **Multiplier-free serving, addressed memory and hyperbolic LMs all have strong prior art.** They give high leverage as components the project can adopt, and little room for a novelty claim [LITERATURE; see F4, F5, F6].
- **Zeta-zero phases and prime addressing have no ML precedent as model features.** They are novel, but no literature suggests they help, and learned RoPE frequencies beat fixed ladders (LeRoPE) [LITERATURE arXiv 2607.10134; negative search results in F6.5].
- **Local inference is not automatically low-energy.** In a measured study, an NVIDIA B200 delivers 1.6–2.3× more "intelligence per joule" than an M4 Max on identical models at batch size 1, and batching adds another 12–20× [LITERATURE arXiv 2511.07885]. Any energy claim for the project has to come from moving fewer bytes and doing fewer operations per token at equal quality [DERIVED].

## 2. Findings by idea

### Idea 1: "TurboQuant, but keep the radius and use 4 dimensions"

**F1.1: What the Google methods actually do.**
- **TurboQuant** (Zandieh, Daliri, Hadian, Mirrokni, 2025; ICLR 2026) [LITERATURE arXiv 2504.19874]:
  - It applies a random rotation, so each coordinate follows a Beta distribution, then quantizes each coordinate with an optimal Lloyd-Max scalar quantizer.
  - It assumes unit norm; for other data it says to "compute and store the L2 norms in floating-point precision and rescale".
  - The inner-product variant uses (b−1)-bit MSE quantization plus 1-bit QJL on the residual, and stores the residual norm γ.
  - Bounds: D_mse ≤ (√3π/2)·4^−b against a lower bound of 4^−b. The ratio is ≈2.7, falling to ≈1.45 at b=1.
  - KV-cache results: quality-neutral at 3.5 bits per channel, marginal degradation at 2.5 bits. The 2.5-bit setting uses 32 outlier channels at 3 bits and 96 channels at 2 bits.
- **The Google blog** (March 2026) frames TurboQuant as "PolarQuant for the main compression plus 1-bit QJL" and describes "two pieces of information: the radius … and the angle" [LITERATURE research.google/blog/turboquant-…].
- **PolarQuant** (arXiv 2502.02617, verified; AISTATS 2026) [LITERATURE arXiv 2502.02617]:
  - It converts coordinate pairs to 2-D polar form and recursively polar-transforms the radii.
  - It keeps the final radius in full precision.
  - The practical setting uses L=4 levels, one fp16 radius per 16 coordinates, 4-bit first-level angles and 2-bit higher-level angles, for 3.875 bits per coordinate.
  - LongBench average: 48.37 vs 48.63 exact and 46.70 for KIVI.
- **QJL** [LITERATURE arXiv 2406.03482] stores sign(Sk) together with the key norm ‖k‖₂ in full precision, and gives an unbiased asymmetric estimator. At 3 bits per floating-point number it cuts memory by more than 5×.
- **Conclusion** [DERIVED]: none of these methods discards the radial direction; all of them store it. The two ideas that remain open are (a) *quantizing* the gain instead of storing it as a float, and (b) using 4-D blocks. Both are taken below.

**F1.2: The 4-D and quaternion version already exists (2026).**
- **HQMQ** (MIT/IBM, May 2026) [LITERATURE arXiv 2605.27646]:
  - It treats "each 4-element chunk of K or V as a quaternion". The radius is quantized with b_r bits: 3–6 bits works well, ≤1 bit breaks the model, and 2 bits is marginal.
  - The direction is the product q_p·q_s, where q_p is one of the 24 elements of the binary tetrahedral group 2T (the 24-cell) and q_s comes from S random unit quaternions.
  - Results: within 0.02–0.03 perplexity of fp16 at about 5 bits on Mistral-7B and Qwen3-8B. At 3.79 bits it matches KIVI-4 (about 4.5 bits) to within about 1 point on CoQA.
  - A preliminary 8-D E8/octonion extension *underperformed* the 24-cell at matched bits.
- **IsoQuant** (March 2026) [LITERATURE arXiv 2603.28430]:
  - It rotates each 4-D block with an SO(4) isoclinic map q_L·v·q̄_R, stores the norm separately, and then applies scalar Lloyd-Max.
  - Its fused kernels run 4.5–4.7× faster than RotorQuant's 3-D Clifford rotors.
  - Its evaluation covers only the first quantization stage on synthetic vectors.
- **TurboAngle** [LITERATURE arXiv 2603.27467, abstract] quantizes angles of coordinate pairs in the Walsh–Hadamard domain and quantizes norms separately (8 bits for K, 4 bits log-space for V).
- **Verdict:** already done (concurrent). **Leverage:** high as a reference implementation. **Use:** do not claim priority; cite HQMQ and IsoQuant.

**F1.3: Gain-shape and lattice prior art says dimension beats structure.**
- **Classical gain-shape VQ.** Sabin & Gray, "Product code vector quantizers…", IEEE TASSP 1984 [LITERATURE doi 10.1109/TASSP.1984.1164346, metadata only]. The DOI printed in GSRQ's reference list (…1164367) resolves to an unrelated paper; I checked this in Scite.
- **4-D lattice and polytope constellations** have long been standard in communications [LITERATURE ntrs.nasa.gov 19830028029 (1983 NASA report on 4-D modulation, D4); publications.lib.chalmers.se 234967].
  - The 600-cell is a universally optimal spherical code and a unique 11-design.
  - Its 120 vertices are the unit icosians, and together with their φ-conjugates they give the 240 minimal vectors of E8 [LITERATURE errorcorrectionzoo.org/c/600cell]. This is exactly the project's "E8 = H4 ⊕ φH4" shorthand [SOURCE AGENTS.md:154].
- **QuIP#** [LITERATURE arXiv 2402.04396]:
  - Its E8P codebook is 8-D with 2^16 entries, decoded from a 256-entry table plus sign bits (1 KiB).
  - Its ablation on Llama-2-70B at 2 bits: **D4 (4-D) 4.408 vs E8P 4.156** WikiText-2 perplexity (FP16: 3.120).
- **QTIP** [LITERATURE arXiv 2406.11235]:
  - MSE at 2 bits on an i.i.d. Gaussian: Lloyd-Max 0.118, E8P 0.089, 256-D trellis 0.069, rate-distortion bound 0.063.
  - Its lookup-free "computed" codes need 2–4 instructions per weight, which fits the project's D0-b serving rule.
- **LLVQ, Leech lattice** (Qualcomm, 2026) [LITERATURE arXiv 2603.11021]:
  - 24-D shape-gain retains 92.1% of the Shannon SQNR at 2 bits, vs 82% for E8 and 77% for Lloyd-Max.
  - The best gain allocation is about 1 bit per 24-D block. The high-resolution rule is "about 1/n of the bits go to the gain", so a 4-D block should give about a quarter of its bits to the radius.
  - Llama-2-7B at 2 bits without fine-tuning: 6.83 vs QTIP 7.28 and QuIP# 7.96.
- **HIGGS** [LITERATURE arXiv 2411.17525]:
  - It stores each 1024-group norm and quantizes with Gaussian-MSE-optimal grids.
  - Its grid-dimension ablation on Llama-3.1-8B at 3.25 bits: **p=2: 7.110, p=3: 6.807, p=4: 6.643** WikiText-2 perplexity (FP16 5.607).
  - This is the closest weight-side precedent for "4-D instead of 2-D".
- **PVQ** [LITERATURE arXiv 2410.16926] uses gain-shape with pyramid codes. Llama-3-70B at 3.25 bits per weight retains about 98% accuracy.
- **GSRQ** (ICML 2026) [LITERATURE arXiv 2607.01065] applies gain-shape k-means inside residual quantization. For a 1-bit KV cache it raises the LongBench average from 11.34 to 33.54 over VQLLM.
- **Verdict** for "gain-shape with 4-D polytope shape codes for weights/KV": already done or incremental. **Leverage:** high. These papers give the bit-allocation rule, rotation preprocessing and kernels.

**F1.4: My measurement of the owner's idea** [MEASURED].
- Script: `exp/lit/vq4d.py`, run as `OMP_NUM_THREADS=1 PYTHONPATH=…/pylib python3 vq4d.py`; output in `exp/lit/vq4d_output.txt`.
- Setup: i.i.d. N(0,1) source; 400k training and 400k test vectors; every quantizer uses its MSE-optimal (cell-centroid) decoder. Values are MSE per dimension.

| Quantizer | 2 bits/dim | 3 bits/dim |
|---|---|---|
| Scalar Lloyd-Max | 0.1172 | 0.0346 |
| 2-D polar (best radius/angle bit split) | 0.1194 | 0.0350 |
| 2-D unconstrained VQ (k-means) | 0.1094 | 0.0298 |
| 4-D gain-shape, 24-cell (24 dirs) × radii | 0.2036 | 0.2037 |
| 4-D gain-shape, **600-cell (120 dirs)** × radii | **0.1074** | 0.0706 |
| 4-D gain-shape, best learned spherical code | 0.1062 (128×2) | 0.0304 (1024×4) |
| 4-D unconstrained VQ (k-means) | 0.0977 | ≤0.0277 (upper bound; reduced budget of 150k points, 10 iterations) |
| E8P / 256-D trellis / Shannon bound (QTIP paper) | 0.089 / 0.069 / 0.0625 | – / – / 0.0156 |
| *Literal "preserve the radius"*: fp16 radius per 4-D block + 600-cell direction | 0.0707 at **5.73** bits/dim | |

Run notes:
- The full run was stopped at about 11.5 CPU-minutes, over the 10-minute budget, before its last cell. The ≤0.0277 value and the literal-radius row come from `exp/lit/vq4d_followup.py`, with output in `vq4d_followup_output.txt`.

What the numbers show [DERIVED]:
- Storing the radius exactly costs 4 bits/dim of overhead in 4-D. At 5.7 bits/dim it gives *worse* MSE than a 3-bit scalar quantizer. Keeping the radius only makes sense if the radius itself is quantized with about 1/n of the bits, as LLVQ and HQMQ do.
- 2-D polar product coding is no better than scalar quantization.
- The 600-cell is within 1% of the best learned 4-D shape code at 2 bits.
- Gain-shape product structure costs about 10% MSE relative to unconstrained 4-D VQ.
- The 24-cell alone has too few directions to be useful.
- High-resolution theory gives the fixed-rate MSE factor G_k·2π·((k+2)/k)^((k+2)/2)·2^−2R: 2.72 for k=1, about 2.0 for k=2 (A2), about 1.62 for k=4 (D4), about 1.37 for k=8 (E8) and about 1.17 for k=24 (Leech). Going from 2-D to 4-D gains about 0.9 dB, and 8-D and 24-D gain another 0.7 dB each.

### Idea 2: Quaternion, hypercomplex and geometric-algebra networks; Lie-group RoPE

**F2.1: Earlier quaternion and hypercomplex networks.**
- QRNN/QLSTM reached better speech recognition with up to 3.3× fewer parameters [LITERATURE arXiv 1806.04418, abstract].
- Quaternion Transformers cut parameters by "up to 75%" without significant loss [LITERATURE arXiv 1906.04393, abstract].
- PHM layers *learn* the multiplication rule and include the Hamilton product as a special case, using 1/n of the parameters [LITERATURE arXiv 2102.08597, abstract].
- Implication [DERIVED]: in NLP, the demonstrated value of quaternion structure is parameter sharing, not a unique inductive bias.

**F2.2: Equivariant geometric-algebra networks.** GATr (E(3), 16-D projective algebra) [LITERATURE arXiv 2305.18415, abstract] and Clifford-group networks (O(n)/E(n)) [LITERATURE arXiv 2305.11141, abstract] help when the *data* carries that symmetry (physics, robotics). Text has no known E(3) or SO(4) symmetry, so this line of work offers little leverage for language [DERIVED].

**F2.3: RoPE generalizations.**
- LieRE learns dense skew-symmetric generators [LITERATURE arXiv 2406.10322, abstract].
- ComRoPE proves that commuting generators are needed for shift-consistency [LITERATURE arXiv 2506.03737, abstract].
- GRAPE unifies SO(d) rotations with additive unipotent biases, recovering ALiBi and FoX [LITERATURE arXiv 2512.07805, abstract].
- CARE/QuatRo builds quaternion rotary embeddings [LITERATURE arXiv 2511.11665, abstract].
- **LeRoPE**: learned per-frequency scalars beat fixed RoPE; at 2.5B, plain RoPE needs 3.4% more compute to match [LITERATURE arXiv 2607.10134, abstract].
- **Verdict:** the project's per-lane left-quaternion transport is one instance of input-dependent orthogonal or unitary state transitions (orthogonal/unitary RNNs, AUSSM, U(d)-subgroup RNNs, DeltaProduct; see F3.1), and the analogous position-encoding designs exist too (GRAPE, PaTH). It is incremental [DERIVED]. **Leverage:** medium.

### Idea 3: Input-dependent rotation recurrences and state tracking

**F3.1: Theory.**
- The Illusion of State [LITERATURE arXiv 2404.08819]:
  - S4 and Mamba lie in TC⁰, like transformers, so they cannot solve S5 or A5 word problems at fixed depth.
  - A minimal input-dependent-transition SSM (IDS4) learns A5 with *one* layer.
  - The paper uses **A5** (60 elements, the smallest non-solvable group) as its hard task.
- Grazzi et al., ICLR 2025 [LITERATURE arXiv 2411.12537]:
  - Parity needs a negative (or non-real) eigenvalue, and counting mod 3 needs non-real eigenvalues.
  - Products of generalized Householder matrices with eigenvalues in [−1,1] recognize every regular language.
  - **Language modeling was essentially unchanged:** DeltaNet 1.3B WikiText perplexity 18.57 with the extended range vs 18.54 without; the gains appeared on code and math perplexity.
- Further work: DeltaProduct (n_h Householder factors per token) [LITERATURE arXiv 2502.10297, abstract]; PaTH, a data-dependent Householder position encoding that beats RoPE [LITERATURE arXiv 2505.16381, abstract]; LRU [LITERATURE arXiv 2303.06349, abstract]; uRNN [LITERATURE arXiv 1511.06464, abstract]; "Transformers learn shortcuts to automata" [LITERATURE arXiv 2210.10749, abstract]; adaptive unitary SSMs [LITERATURE arXiv 2507.05238, abstract]; U(d)-subgroup RNNs [LITERATURE arXiv 2602.18417, abstract]; state tracking from code [LITERATURE arXiv 2602.14814, abstract].
- **"The Automaton Underneath"** (2026, pre-registered, 202 runs) [LITERATURE arXiv 2609.18966, abstract]:
  - With an additive input pathway, Householder linear RNNs collapse out of distribution on A5 and S5.
  - Without that pathway they learn the exact automaton and reach 1.00 accuracy at 16× the training length.
  - The minimum number of reflections per token equals the generators' reflection length in the task's representation (4 for A5 and S5).

**F3.2: The icosian connection** [DERIVED; I found no prior use, so novelty is HYPOTHESIS].
- 2I = the 120 unit icosians = the 600-cell, and 2I/{±1} ≅ A5.
- A single 4-D lane updated by exact left multiplication with q ∈ 2I, s_t = q_t·s_{t−1} in Z[φ]/2 coordinates, solves the A5 word problem exactly for any length with no numerical drift.
- In source, the project's quaternion arm is exactly a left Hamilton product per lane [SOURCE crates/uor-r4-training/src/joint_model.rs:1741-1771]. Its ordinary control is H(v)H(e0) [SOURCE joint_model.rs:1773-1777], a *simple* rotation in a plane that contains e0 (reflection length ≤2). The quaternion step is left-isoclinic (reflection length 4).
- By the reflection-length law above, the current control cannot realize every A5 element in one step in this 4-D representation, but the quaternion arm can.
- Caveats:
  - A free 4-reflection DeltaProduct control, or an SO(3) realization, can also represent A5.
  - Training currently quantizes unit coordinates onto a grid with "no H4 codebook" [SOURCE joint_model.rs:1642], so exact icosians are not reachable today.
  - The transport is q = normalize(e0 + 0.1·raw) [SOURCE joint_model.rs:28,1653], which is near-identity at initialisation [DERIVED].
- **Leverage:** high. This is a cheap, falsifiable test aligned with a *provable* capability rather than with perplexity.

### Idea 4: Multiplier-free, low-bit and LUT language models

**F4.1: Headline numbers** (fuller detail in the Sept-16 brief).
- **BitNet b1.58 2B4T** [LITERATURE arXiv 2504.12285]:
  - Quality: average 54.19 vs 55.23 for Qwen2.5-1.5B; MMLU 53.17; GSM8K 58.38.
  - Efficiency: 0.4 GB non-embedding memory; 29 ms/token on an i7-13800H with 8 threads.
  - Its 0.028 J/token is an *estimate* from an arithmetic-energy model, not a measurement.
  - Attention, RoPE and norms stay in higher precision.
- **MatMul-free LM** [LITERATURE arXiv 2406.02528]:
  - Average score: 46.2 vs 48.0 for Transformer++ at 1.3B/100B tokens, and 49.9 vs 50.7 at 2.7B.
  - **Ternarizing Q and K in attention failed to converge**, which is why the paper uses an element-wise GRU token mixer. Element-wise products, sigmoid and SiLU remain.
  - Energy on Loihi 2: 59.4 tokens/s at 70.8 mJ/token (370M model). Measured on a Jetson Orin Nano: Qwen2-500M at 13.4–14.1 tokens/s and 785–912 mJ/token.
- **T-MAC** [LITERATURE arXiv 2407.00088]:
  - It performs mixed-precision matrix-vector products by bit-serial table lookup.
  - BitNet-3B runs at 30 / 71 tokens/s on 1 / 8 cores of an M2 Ultra, and 11 tokens/s on a Raspberry Pi 5.
  - Energy measured with powermetrics falls 20.6% / 61.2% / 51.3% vs llama.cpp.
- bitnet.cpp gives 1.37–5.07× speedups on ARM [LITERATURE arXiv 2410.16144, abstract].
- Related multiplier-free work:
  - ShiftAddLLM re-parameterizes pretrained models post-training into shift-and-add form [LITERATURE arXiv 2406.05981, abstract].
  - AdderNet is a vision/CNN method [LITERATURE arXiv 1912.13200, abstract].
  - SpikeGPT goes up to 216M parameters [LITERATURE arXiv 2302.13939, abstract].
  - Differentiable logic-gate networks are demonstrated at MNIST scale [LITERATURE arXiv 2210.08277, abstract].
  - The Tsetlin and VSA language models I found are character-level or small sequence models, and none is competitive [LITERATURE arXiv 2408.16620; dl.acm.org/doi/10.1007/978-3-031-36822-6_12].
- **Verdict:** multiplier-free serving at 2–3B scale is already done. **Leverage:** very high. D0-b (≤4-bit additive/LUT maps) is exactly T-MAC's regime, and T-MAC's layout should be adopted as-is [HYPOTHESIS: the project's "signed4 product-table" is a special case; not checked].

### Idea 5: Addressed memory, store and recall

**F5.1: The main results.**
- PKM: a 12-layer model with memory beat a 24-layer baseline and ran 2× faster [LITERATURE arXiv 1907.05242, abstract].
- **Memory Layers at Scale** [LITERATURE arXiv 2412.09764]:
  - At 1.3B parameters with 1M memory slots, it roughly matches dense models with 2–4× the compute on QA.
  - With 64M keys (128B memory parameters) it approaches Llama2-7B.
  - An 8B model plus 64B memory parameters trained on 1T tokens approaches Llama-3.1-8B, which used 15T tokens.
  - It beats MoE at matched parameters and compute.
  - About 3 memory layers is optimal; replacing more FFNs hurts.
- kNN-LM improves WikiText-103 perplexity by 2.9 points (to 15.79) with no retraining [LITERATURE arXiv 1911.00172, abstract].
- Memorizing Transformers keeps improving with memory up to 262K tokens [LITERATURE arXiv 2203.08913, abstract].
- RETRO matches GPT-3 on the Pile with 25× fewer parameters [LITERATURE arXiv 2112.04426, abstract].
- Modern Hopfield networks show attention is one-step energy-minimizing retrieval with exponential capacity [LITERATURE arXiv 2008.02217, abstract].
- Associative-memory theory: HRR binding by circular convolution [LITERATURE doi 10.1109/72.377968, metadata/abstract]; sequence-indexing capacity of VSA-style RNNs with orthogonal recurrence [LITERATURE arXiv 1803.00412, abstract]; phasor associative memory, TPAM [LITERATURE arXiv 1901.07718, abstract].
- Test-time-learned memory: Titans [LITERATURE arXiv 2501.00663, abstract]; TTT [LITERATURE arXiv 2407.04620, abstract].

**F5.2: Compatibility with "no MoE / no sparse routing"** [DERIVED].
- **Compatible** (dense, or deterministic addressing with no learned router): Hopfield or attention over all slots, HRR/FHRR superposition memory, Titans/TTT fast weights, kNN-LM/RETRO exact-occurrence retrieval, Engram's hashed n-gram addresses (Sept-16 brief).
- **Conflicting** (learned, input-dependent top-k selection of *parameters*): PKM, Memory Layers and PEER.
- **Tension to flag:** D5's "per-token parameter sparsity" is precisely the PKM/memory-layer mechanism. If the owner's ban covers learned sparse selection, D5 conflicts with it. I am flagging this, not resolving it.
- The project's copy/pointer gate is essentially the **pointer-sentinel mixture** [LITERATURE arXiv 1609.07843, abstract]. The learned NoRead slot is a relative of **attention sinks** [LITERATURE arXiv 2309.17453, abstract] and of Memory Layers' "softmax sink".

### Idea 6: Geometric and hyperbolic routing, H4, primes, zeta zeros, golden ratio

**F6.1: Hyperbolic embeddings and LLMs.**
- Poincaré embeddings [LITERATURE arXiv 1705.08039, abstract] and hyperbolic neural networks [LITERATURE arXiv 1805.09112, abstract] are the foundations.
- **HELM** (NeurIPS 2025) [LITERATURE arXiv 2505.24722] trains fully hyperbolic LLMs up to 1B parameters on 5B tokens, but:
  - Accuracies are near chance: at 1B, MMLU is 25.9 vs 23.6 and CommonsenseQA 19.8 vs 19.5 (4-way chance is 25%).
  - The best variant is **MoE** ("mixture-of-curvature experts").
  - It runs 1.43–1.55× slower.
- HypLoRA [LITERATURE arXiv 2410.04010, abstract] adapts existing LLMs with hyperbolic low-rank updates.
- **Verdict:** already done; weak evidence of value; the best variant conflicts with the no-MoE rule.

**F6.2: RiLM: geodesic decoding under 1M parameters** (Sept 2026) [LITERATURE arXiv 2609.10305].
- This is the closest published "geometric small LM". It reports WikiText-2 (2k vocabulary) perplexity of 54.2 at 289k parameters vs 113–147 for tied LSTM, Transformer and SSM baselines.
- I am sceptical [DERIVED]. On flat space its decoder, −‖h−e_w‖², equals tied dot-product decoding plus a per-word −‖e_w‖² bias. A ≈2× perplexity gap from that change points to under-trained baselines: tied LSTM at 118–214 perplexity, 10 epochs, context 64.
- Use it as a cheap replication target with properly tuned baselines, not as evidence.

**F6.3: H4/icosahedral structure in ML.** The icosahedral gauge-equivariant CNN handles spherical signals [LITERATURE arXiv 1902.04615, abstract]. I found no language-model use of the 600-cell or H4 (negative search result).

**F6.4: Primes and residue number systems.**
- **Prime Fourier Embeddings** (2026) [LITERATURE arXiv 2606.23044, abstract] encode integers as prime-indexed (cos, sin) pairs.
  - They prove that equivariant linear maps must be block-diagonal, one block per prime.
  - They reach perfect in-distribution modular arithmetic.
  - This is the closest analogue of the project's prime/phase addresses. It helps *arithmetic*, and there is no evidence it helps semantics.
- RNS appears in hardware arithmetic (Mirage photonic accelerator) [LITERATURE arXiv 2311.17323, abstract].

**F6.5: Zeta zeros.** The only ML uses I found predict zeta zeros [LITERATURE arxiv.org/pdf/2312.01507, search excerpt] or link RH to a class of networks [LITERATURE arXiv 2309.09171, search excerpt]. None uses zeta zeros as features or frequencies (negative search result). LeRoPE (F2.3) supports learning frequencies rather than fixing them.

**F6.6: Golden-ratio or quasi-random positional encodings.** Nothing substantive found (negative result).

### Idea 7: Small-model quality and distillation

**F7.1: TinyStories** [LITERATURE arXiv 2305.07759].
- Setup: GPT-Neo architecture, top-10K tokens, context 512.
- Models under 10M parameters (embedding width 256) produce fluent stories.
- Consistency emerges as hidden size goes from 64 to 128.
- 1-layer models struggle with instructions.
- GPT-4-graded consistency on the paper's examples: 1M/8-layer 1–3/10, 2.5M/8-layer 3–6/10, 8.3M/8-layer 5–8/10, 28M/8-layer 8–9/10.

**F7.2: Where the project sits.**
- The project's model has 1,676,799 parameters in listed weight shapes plus about 1.7k biases and norms, i.e. ~1.68M total, with 1,048,576 in the tied embedding and ~0.63M non-embedding [MEASURED from SOURCE crates/uor-r4-integer/src/config.rs:59-85].
- Interpretation [DERIVED]: this is the regime where TinyStories *transformers* are incoherent. The observed role confusion and drift do not discriminate between architectures at this size.

**F7.3: Other small-model references.**
- SmolLM2-1.7B, trained on 11T tokens, beats Qwen2.5-1.5B and Llama-3.2-1B [LITERATURE arXiv 2502.02737, abstract].
- MobileLLM finds deep-thin designs best at 125M/350M (+2.7 / +4.3 points) [LITERATURE arXiv 2402.14905, abstract].
- MiniLLM does reverse-KL on-policy distillation from 120M to 13B [LITERATURE arXiv 2306.08543, abstract].
- TinyLlama is 1.1B trained on 1T tokens [LITERATURE arXiv 2401.02385, abstract].
- **Leverage:** high. Distillation from a teacher into the geometric student is allowed offline under the project's D-rules as a "training source", and it is the most reliable way to lift coherence at small scale [HYPOTHESIS].

### Idea 8: Local inference baselines on Apple M1

**F8.1: Measured throughput** (llama.cpp Metal, LLaMA-7B) [LITERATURE github.com/ggml-org/llama.cpp/discussions/4167].

| Chip | Q4_0 generation (tok/s) | Q4_0 prompt processing (tok/s) | F16 generation (tok/s) |
|---|---|---|---|
| M1, 68 GB/s | 14.2 | 108–118 | – |
| M1 Pro, 200 GB/s | 35.5–36.4 | 233–266 | 12.75 |

Generation is bandwidth-bound [DERIVED]: 3.56 GiB × 36.4 tok/s ≈ 139 GB/s, about 70% of the M1 Pro's peak. The project's model at 4 bits is about 0.84 MB, so its per-token cost is compute- and latency-bound rather than bandwidth-bound. The right competitor is a *same-quality small transformer* on the same M1, not a 7B model.

**F8.2: Measured energy.**
- **Intelligence per Watt** (Stanford) [LITERATURE arXiv 2511.07885]:
  - Method: GPU-subsystem power from powermetrics sampled every 50 ms, 3–5 runs.
  - On identical models, a B200 achieves 1.6–2.3× higher intelligence per joule than an M4 Max.
  - Batch size 64 cuts per-query energy on the B200 by 12–20×.
- **GreenBench** [LITERATURE arXiv 2608.28667] is **weak**. Its 0.41–0.47 W package power during 7B inference is implausible, and its 0.09–0.25 J/token assumes a 10 W system power rather than measuring it.
- T-MAC (F4.1) is the best peer-reviewed powermetrics measurement on Apple silicon I found.

## 3. What is genuinely strong or novel vs weak or irrelevant

**Strong or novel:**
1. Exact icosian (2I) state transport as an A5 state tracker. I found no precedent. It is cheap to test and tied to a provable NC¹ capability.
2. The project's discipline of matched ordinary controls. RiLM and HELM illustrate what happens without it.
3. The 600-cell as a shape codebook: measured near-optimal at about 7 bits per 4-D block.

**Weak, wrong or irrelevant:**
1. The claim "TurboQuant ablates the radius". This is factually wrong.
2. A novelty claim for 4-D gain-shape quantization (HQMQ and IsoQuant exist) and for 4-D as a sweet spot (8-D, 24-D and trellis codes dominate).
3. Judging geometry by TinyStories NLL at 1.7M parameters.
4. Fixed zeta frequencies as a performance lever.
5. Hyperbolic LLMs as evidence (near-chance, MoE).
6. "Local = green" without measurement.

## 4. Novelty map

| Project idea | Closest prior work | Novelty | Leverage |
|---|---|---|---|
| Keep the radius, quantize the direction | TurboQuant, PolarQuant, QJL (all store the norm) | Already done | High |
| 4-D quaternion chunks with polytope shape code | HQMQ (2026), IsoQuant (2026), HIGGS p=4, QuIP# D4 | Already done (concurrent) | High |
| 600-cell / E8 = H4⊕φH4 as codebook | QuIP# E8P, LLVQ Leech, 600-cell code theory | Incremental | Medium |
| Per-lane quaternion state rotation | QRNN, PaTH, GRAPE, DeltaProduct | Incremental | Medium |
| Exact 2I/icosian A5 state tracking | Illusion of State, Grazzi, Automaton Underneath | Probably novel as a demonstration (HYPOTHESIS) | High |
| ≤4-bit additive/LUT serving (D0-b) | T-MAC, BitNet, bitnet.cpp, MatMul-free LM | Already done | Very high |
| Soft read over all events + NoRead slot | Attention with sinks / softmax sink | Incremental | Low |
| Copy/pointer gate | Pointer-sentinel mixture (2016) | Already done | Low |
| Exact addressed memory | kNN-LM, RETRO, Engram, memory layers | Already done (the learned-sparse variants conflict with the owner's rule) | High |
| Prime addresses | Prime Fourier Embeddings (2026) | Incremental | Low–medium (arithmetic only) |
| Fixed zeta-zero phases | None found | Novel, unmotivated | Low |
| Hyperbolic / "least-energy" manifold routing | HELM, RiLM, Hopfield energy view, NRGPT | Incremental | Low–medium |
| Geometric LM on M1 | BitNet 2B4T + T-MAC; TinyStories scale | Incremental | High (baselines) |

## 5. Recommendations (ranked)

1. **Re-target the geometric transport to state tracking.**
   - Test: an A5 word problem (one group element per token), a 1–4 lane model, exact Z[φ] arithmetic, and **no additive input path**.
   - Arms: quaternion left-multiplication, the current H(v)H(e0) control, a free 2- and 4-reflection DeltaProduct, and a diagonal ±1 model.
   - Prediction [DERIVED]: the quaternion arm and the free 4-reflection arm reach 1.00 at 16× the training length; the fixed-e0 pair fails.
   - Impact: the first capability where the geometry can win or lose decisively. Cost: CPU-minutes.
   - Falsification: the quaternion arm is no better than a matched free-reflection control at equal parameters and exactness.
2. **Retire the quantization novelty claim and adopt the known recipe.**
   - Recipe: rotate, then use a high-dimensional shape-gain code (E8/Leech/trellis) or a HIGGS 4-D grid, giving about 1/n of the bits to the gain.
   - Keep the 600-cell only where a 4-D block has a structural role.
   - Test: MSE and loss on the project's real weight and state tensors at 2, 3 and 4 bits vs the current signed-4 format.
3. **Separate scale from mechanism.**
   - Train the ordinary #1017 recipe at 1.7M parameters and at 8M parameters on the same tokens.
   - If the 1.7M transformer is equally incoherent, stop attributing incoherence to geometry.
   - Consider offline distillation (reverse-KL, MiniLLM).
4. **Serve with T-MAC-style bit-serial LUTs and measure J/token.** Use powermetrics on an M1 against a quality-matched small transformer running under llama.cpp or bitnet.cpp.
5. **Use deterministic addressing for memory** (exact-occurrence kNN, hashed n-grams) to stay clear of learned sparse selection, and record the D5 tension explicitly.
6. **Treat zeta and prime phases as hyperparameters.** Ablate them against learned frequencies (LeRoPE-style) before any claim.
7. **Optionally replicate RiLM's geodesic decoding** with tuned tied baselines. It costs about an hour and tests a cheap "geometric decoder" claim.

## 6. Top 10 papers to read this week

1. **HQMQ** (arXiv 2605.27646): the owner's original idea, already published, with its limits.
2. **LLVQ / Leech** (arXiv 2603.11021): shape-gain done properly; bit allocation; 24-D beats 8-D beats 4-D.
3. **QTIP** (arXiv 2406.11235): dimension matters; lookup-free codes at 2–4 instructions per weight fit D0-b.
4. **The Illusion of State** (arXiv 2404.08819): A5 word problem and TC⁰ limits; the right target for the transport.
5. **Negative-eigenvalue linear RNNs** (arXiv 2411.12537): reflection products for state tracking, and why LM loss does not move.
6. **The Automaton Underneath** (arXiv 2609.18966): the additive input path breaks exact automata; the reflection-length law.
7. **T-MAC** (arXiv 2407.00088): LUT kernels on Apple CPUs with measured energy; the D0-b serving blueprint.
8. **MatMul-free LM** (arXiv 2406.02528): the closest end-to-end design; ternary attention did not train.
9. **TinyStories** (arXiv 2305.07759): calibrates what 1–10M parameters can do.
10. **Intelligence per Watt** (arXiv 2511.07885): measured-energy methodology; local inference is not automatically efficient.

## 7. Open questions

- Whether the project's "signed4 product-table" kernel is equivalent to T-MAC's layout. I did not inspect it.
- Whether the lookup-free QTIP codes can be written without any multiply instruction. Their LCG step uses a MAD, which may conflict with D0-b.
- Full texts of FibQuant and Spherical KV (cited by HQMQ; not retrieved).
- Whether any 2026 work already uses 2I transitions for A5 tracking. My searches found none; that is not proof of absence.

## 8. Retrieval notes

- Full text read: 2504.19874, 2406.03482, 2502.02617, 2605.27646, 2607.01065, 2603.28430, 2603.11021, 2410.16926, 2402.04396, 2406.11235, 2411.17525, 2411.12537, 2404.08819, 2504.12285, 2407.00088, 2406.02528, 2412.09764, 2505.24722, 2609.10305, 2305.07759, 2608.28667, 2511.07885.
- Abstract or metadata only: all items marked "(abstract)" above.
- Web pages: Google Research blog; llama.cpp discussion #4167; Error Correction Zoo 600-cell page; NASA NTRS 19830028029; Chalmers 234967.

## 9. References

All arXiv items are at https://arxiv.org/abs/ID:
2504.19874, 2502.02617, 2406.03482, 2605.27646, 2603.28430, 2603.27467, 2607.01065, 2603.11021, 2410.16926, 2402.04396, 2406.11235, 2411.17525, 2401.06118, 2404.00456, 2405.16406, 1806.04418, 1906.04393, 2102.08597, 2305.18415, 2305.11141, 2406.10322, 2506.03737, 2512.07805, 2511.11665, 2607.10134, 2404.08819, 2411.12537, 2502.10297, 2505.16381, 2303.06349, 1511.06464, 2210.10749, 2507.05238, 2602.18417, 2602.14814, 2609.18966, 2504.12285, 2406.02528, 2407.00088, 2410.16144, 2406.05981, 1912.13200, 2302.13939, 2210.08277, 2408.16620, 1907.05242, 2412.09764, 1911.00172, 2203.08913, 2112.04426, 2008.02217, 1803.00412, 1901.07718, 2501.00663, 2407.04620, 1609.07843, 2309.17453, 1705.08039, 1805.09112, 2505.24722, 2410.04010, 2609.10305, 1902.04615, 2606.23044, 2311.17323, 2309.09171, 2305.07759, 2502.02737, 2402.14905, 2306.08543, 2401.02385, 2511.07885, 2608.28667.

Non-arXiv sources:
- https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression
- https://doi.org/10.1109/TASSP.1984.1164346
- https://doi.org/10.1109/72.377968
- https://errorcorrectionzoo.org/c/600cell
- https://ntrs.nasa.gov/api/citations/19830028029/downloads/19830028029.pdf
- https://publications.lib.chalmers.se/records/fulltext/234967/local_234967.pdf
- https://github.com/ggml-org/llama.cpp/discussions/4167
- https://arxiv.org/pdf/2312.01507
- https://dl.acm.org/doi/10.1007/978-3-031-36822-6_12
