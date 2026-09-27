# Geometric language model lab, phase 2: the route to a chat model, and what hyperbolic attention has and has not shown

2026-09-26 · Requested by the owner · References #820 · Branch `claude/blissful-wozniak-girwwq` (kept separate from `main`)

**Status.** An evidence note and proposed roadmap, not a decision record. It follows [cycle 3](hyperbolic-cycle3-2026-09-26.md) and applies the owner's decisions of 2026-09-26:
- weight transfer from an open transformer is approved, provided the served model performs no matmul;
- hyperbolic geometry is the lead mechanism;
- offline training is in scope;
- paid compute needs the owner's explicit approval.

**Labels.** **Measured** (run in the review sandbox), **Derived**, **Literature** (retrieved in this session; arXiv id given), **Hypothesis**. The lab had five specialist reviewers (mathematics, architecture, training and data, systems, experiment), two adversarial reviewers (feasibility/engineering and science/novelty) and the lead. Their reports and scratch experiments stay outside the repository. The numbers this note relies on are in [the evidence file](../evidence/lab-phase2-2026-09-26.json).

**New in the repository.**
- A Rust conversion tool in the training crate: `kappa_llama`, example `kappa-conversion`, and `scripts/kappa-m1-pilot.sh`, in commits `c67e230`, `50bee85` and `f43aa4e`.
- The M3 integer serving path (§7): the engine crate `uor-r4-lut` with `lut-chat`, the audited SIMD crate `uor-r4-simd`, the exporter `lut_export` with GPTQ and the example `lut-tool`, and `scripts/lut-m1-chat.sh`, in commits `8931b27` to `f5886cc`, with integer sampling in `90e39ce`.
- The M4 hyperbolic cache memory (§8). The float experiment `cache_memory` and its example; the integer Lorentz cache in the engine (`453aa9d`); and the script's `CACHE=1` stage (`bdebc86`).
- *Result.* A learned Lorentz cache beats the Euclidean and dot caches of equal size. Its integer form keeps 99.3% of the float gain.
- The context-256 addendum to [cycle 3](hyperbolic-cycle3-2026-09-26.md#10-addendum-context-256): the Lorentz read's advantage grows with context.
- Cycle 3b, the integer Lorentz read of the native model ([cycle 3 §11](hyperbolic-cycle3-2026-09-26.md#11-addendum-the-integer-lorentz-read)).
  - The integer runtime serves quantized Lorentz models; the trained context-256 models run in it at parity with their F32 emulator.
  - The Lorentz advantage over Dot survives 4-bit integer serving.
  - `scripts/native-lorentz-m1.sh` runs the full-scale native comparison through to integer serving on an M1.

## 0. Findings

1. **The route to a chat model is converting an open instruct model; the adversarial reviewers agree.**
   - *Literature.* Open small talk appears at 100–350M parameters only after 1–11T training tokens (MobileLLM 2402.14905, SmolLM2 2502.02737, LFM2 2511.23404).
   - *Derived.* An M1 trains roughly 0.2B tokens per week at 150M parameters, at an assumed and not yet measured Candle-Metal throughput.
   - Converting SmolLM2-135M/360M-Instruct (Apache-2.0) is the only route to that band inside the owner's budget. Training from scratch gives narrow-domain chat at best.
2. **The dot-to-hyperbolic "exact conversions" are flat-limit reparametrisations.** Three constructions reproduce a trained dot-product head, but only as curvature κ → 0 (§2.1).
   - *Derived, checked numerically.* To first order they are the dot score plus κ times a fixed quartic norm feature.
   - The meaningful measure is the dimensionless curvature `t = κ·mean|k|²`. A head is genuinely hyperbolic only near `t ≈ 1`.
   - *Literature.* Per-head learnable curvature initialised at zero with an exact Euclidean limit already exists (FPS-T 2309.04082; κ-GCN 1911.05076), as does RoPE as a spatial Lorentz rotation (HELM 2505.24722).
   - *Hypothesis: possibly new here.* The polarisation scores that reproduce a *pretrained* head, the hyperbolic norm-completion lift, Gromov-product attention, Busemann-landmark admission, and the flat-basin result in item 3.
3. **Curvature stays in the flat basin unless forced, and forcing it has not helped a converted model yet.**
   - *Measured, stand-in teacher, data objective, two seeds.* With curvature learnable from the exact start, it drifted only to `t ≈ 0.02` and matched the dot control to 0.0001 bits per byte.
   - Forced deep into curvature (one seed each), it was 0.0011–0.0015 bits per byte worse.
   - *Measured, toy retrieval heads on the code-scope tree.* An exact conversion keeps its teacher's blind spot (0.4% non-root). Annealing curvature in lifts it to 31%; a direct curved lift gives 52.5%. That task is hierarchical by construction.
   - The stand-in's heads are local byte mixers (68–98% of attention mass within 3 bytes, per the science reviewer; *Measured*), so the stand-in cannot tell geometries apart.
   - **The deciding test is now a zero-training probe on real SmolLM2 heads with a pre-registered rule (§4, M1a).** The tool implements it.
4. **Hyperbolic geometry measurably helps where heads are *trained* with distance scores.**
   - *Measured.* At 64 key dimensions on this repository's code-scope tree, hyperbolic keys learn the hierarchy (85–90% non-root). Dot keys learn only "attend to the root" (0–2%).
   - *Measured, science reviewer, real cycle-3 read heads.* Distance-trained heads are indexable: query/top-key cosine 0.95, top-1 admitted at 99.2% with 5.5% of keys scored.
   - Real dot heads are not: cosine 0.31–0.33, 79% at 2.5% scored, about 25% scored needed for 99.8%. The synthetic near-copy results (99.8% at 2.8%) do not transfer to them.
   - So the geometry belongs in memory and index layers trained hyperbolic, not in converted dot heads.
5. **The cycle-3 gain is mostly a copy-distance effect.**
   - *Measured, science reviewer.* Controlling for log copy distance, the "enclosing scope" advantage survives in one of three seeds. The Lorentz read's gain grows with copy distance in all three: −0.013 to −0.027 nats per doubling.
   - The mathematics reviewer's 0.01–0.02-nat "oracle ceiling" measures what six scope features add to a small copy model. It is not a ceiling for a real LM.
6. **Energy: at 4 bits and 2K tokens of context a converted model is at about parity with llama.cpp Q4_0; the win needs long context or ternary weights.**
   - *Derived from published constants; nothing was measured on an M1.*
   - Recomputed on matched cores and key/value precision by the feasibility reviewer: 7.7 against 8.6 mJ per token on efficiency cores.
   - About 2× at 8K tokens with 3% of keys admitted.
   - Ternary weights project lower but need about 1.3 H100-days of quantization-aware training per attempt, which is paid compute.
   - The energy case does not depend on the attention geometry (science reviewer): it comes from table-driven low-bit weights, efficiency cores and cache compression, which also work for dot keys.
7. **Negative results.**
   - A power-law (horocycle) positional prior extrapolated worse than exponential decay (3.264 vs 3.226 nats at 2,048 tokens).
   - A conservative Hamiltonian read never settles on a key; with damping it costs about 100 softmaxes.
   - Primes, CRT and Galois-field hashing give no advantage over simple tabulation for exact content memory (all within 0.2% of the ideal collision count).
   - Two-level advertisement cells plateau at 99.6% admission.

   All *Measured*.

## 1. The route to a chat model

| Model | Size / tokens | Result (Literature, arXiv) |
|---|---|---|
| TinyStories | 1–80M | Narrow-domain fluency at 28–80M (2305.07759) |
| MobileLLM | 125M / 350M, 1T | MT-Bench 2.33 / 3.28 (2402.14905) |
| SmolLM2 | 135M (2T) / 360M (4T) | IFEval 29.9 / 41.0 (2502.02737) |
| LFM2-350M | 11T, logit distillation | IFEval 65.1 with 6 attention and 10 convolution layers (2511.23404) |
| Mamba-in-the-Llama | 8B instruct, 20B tokens | MT-Bench 7.35 / 6.86 at 50% / 25% attention; 5.64 for the fully Mamba2 variant (2408.15237) |
| LoLCATs | 8B, 40M tokens, LoRA | MMLU 52.8 vs 66.6; 23.8 without its sliding window (2410.10254) |

Conversions that keep softmax attention over the cache are the cheap regime; fully recurrent conversions lose most. This lab replaces only the attention score. Whether a converted model *also* needs geometric components, and in which layers, is the open question behind §4.

## 2. Converting dot-product attention

### 2.1 Three flat-limit constructions (Derived)

For one head of width `r`, curvature `κ`, and points lifted to the hyperboloid of curvature `−κ` with geodesic distance `d`:

| Construction | Score | Near κ = 0 |
|---|---|---|
| Key-norm homotopy (architecture reviewer) | `(|k|² − d(q,k)²) / (2√r)` | `(⟨q,k⟩ − |q|²/2)/√r + κF/√r`, with `F = (|q|²−|k|²)²/8 + |q−k|⁴/24` |
| Intrinsic polarization (lead) | `(d(o,k)² − d(q,k)²) / (2√r)`, `o` the origin | same, minus `κ|k|⁴/(6√r)` |
| Norm completion (mathematics reviewer) | separate completion coordinates give `q₀ = Q`, `k₀ = C`, `z = QC − q·k` | the dot score as `QC → ∞`, with every key on one hyperbolic sphere |

- *Measured, the science reviewer's check.* With the first-order term the error falls from 7.5×10⁻² to 3.5×10⁻⁵ at κ = 10⁻⁵.
- RoPE rotates spatial coordinates only, a hyperboloid isometry, so the scores stay relative-position scores.
- Norm completion makes the radius constant, so it cannot carry hierarchy.
- The **Gromov product** `(q|k)_o = ½(d(o,q) + d(o,k) − d(q,k))` is not exact at the start. In a rooted tree it is the depth of the lowest common ancestor.

### 2.2 Stand-in conversion (Measured)

Teacher: a 3-layer, width-128, single-head RoPE attention byte model (607,488 parameters), 1.9897 bits per byte on WikiText-2 validation.

| Score | Zero-shot | After 400 steps of Q/K + temperature distillation |
|---|---:|---:|
| dot (drift floor) | 1.9897 | 1.9909 |
| intrinsic polarization (log ε = −6) | 1.9897 | 1.9905 |
| cosine, fitted scale | 2.0511 | 1.9908 |
| Gromov product, fitted scale | 2.0490 | 1.9907 |
| Euclidean distance | 3.9259 | 1.9929 |
| committed Lorentz read `β(δ − d)` | 4.0115 | 1.9946 |

Every score recovers because these heads are local byte mixers: a content-free, distance-only attention pattern alone reaches 2.92 bits per byte against 4.22 with no attention (science reviewer). The table shows that the distillation machinery works. It does not show that geometry swaps are cheap for SmolLM2's sink, induction or retrieval heads.

Drift away from the teacher as curvature grows (mean row KL at log ε = −3, *Measured*):

| Layer | Intrinsic | Key-norm |
|---|---:|---:|
| 0 | 0.032 | 0.024 |
| 1 | 0.056 | 0.24 |
| 2 | 0.086 | 0.53 |

On synthetic sink heads the ordering reverses (intrinsic KL 3.75 at κ = 1, key-norm 0; architecture reviewer), so real heads must choose.

### 2.3 The flat basin (Measured)

Toy retrieval heads on the code-scope tree (mathematics reviewer, one seed). Non-root accuracy at 48 / 96 / 192 stored nodes:

| Arm | Non-root accuracy |
|---|---|
| Dot teacher | 0.0 / 1.7 / 0.4% |
| Exact conversion, curvature learnable | 0.0 / 1.4 / 0.4% |
| Curvature annealed into the curved regime | 35 / 35 / 31% |
| Direct curved lift (raw accuracy first collapses to about 1%) | 59 / 57 / 53% |
| Hyperbolic head trained from scratch | 85.5% at 192 |

Stand-in, data objective, 600 steps. Validation bits per byte:

| Arm | Seed 1 | Seed 2 |
|---|---:|---:|
| dot | 1.9480 | 1.9458 |
| intrinsic, learnable from exact (ends at `t ≈ 0.02`) | 1.9479 | 1.9457 |
| intrinsic, curvature floor raised to log ε = −2 (`t ≈ 3–9`) | 1.9491 | — |
| Gromov product, direct curved conversion | 1.9495 | — |

## 3. The Rust conversion tool

`crates/uor-r4-training/src/kappa_llama.rs` and `examples/kappa-conversion.rs`:

- **Loading.** A Hugging Face Llama checkpoint: BF16 or F32 safetensors, grouped-query attention, tied or untied output head. Biases, RoPE scaling and interleaved RoPE are refused.
- **Scores.** `dot`, `key_norm` and `intrinsic`, plus the exact first-order kinds `*_linear` (the matched-capacity control). Every head has a learnable temperature.
- **Modes:**
  - `tokenize`: the checkpoint's own BPE.
  - `probe`: agreement with dot over a grid of `t`.
  - `drive`: the zero-training test below.
  - `train`: next-token loss or KL to a dot teacher; optional query/key training; gradient accumulation; curvature annealing to a dimensionless `t` whose floor is recomputed from the current keys every step, so shrinking the keys cannot escape it.
  - `sample`: SmolLM2's chat template and default system turn; refuses mismatched saved variables.
- **`drive`, per head of the real checkpoint:**
  - key-norm statistics;
  - the cosine from each query to its top key;
  - the attention mass in the top 2% and 5% of keys, which is the ceiling for any index;
  - the flat-limit curvature drive `dL/dt` of both first-order kinds, from one backward pass, under next-token loss and under KL to a teacher. The self-distillation drive must vanish; it is reported as a check;
  - the zero-shot loss change when one layer's heads are set to `t = 0.1` and `t = 1`.
- **The pre-registered rule.** Curvature training is warranted in a layer only if `−Σ_h dL/dt_h` exceeds the measured zero-shot cost at `t = 1`.
- **Checks.** Ten focused tests: flat-limit and first-order agreement, RoPE invariance, statistics, the drive and its vanishing self-distillation gradient, gradients, the arcosh series switch, the curvature floor, and distillation toward a flat teacher. Every mode ran end-to-end on a synthetic bf16 grouped-query checkpoint.
- **Memory** (measured by the feasibility reviewer; the tool's defaults follow from it). The curved scores keep about thirty `(batch, heads, time, time)` tensors per layer. Training a 135M student with a 360M teacher therefore needs `batch=1` with accumulation at 256 tokens on a 16 GB machine, or 128 tokens on 8 GB. Contexts of 2K or more need a chunked implementation.
- **Not done.** No real checkpoint has been converted. The SmolLM2 weights are not reachable from the review environment (Hugging Face egress is blocked); they are on the owner's Mac.

## 4. Roadmap

| # | Where | Work | Decision |
|---|---|---|---|
| M1a | owner's M1, about 1 h | `scripts/kappa-m1-pilot.sh`: probes, `drive` with SmolLM2-360M as teacher, a chat sample | Pre-registered: if no layer passes the drive rule, skip curvature training, keep dot heads, and report the backbone as a quantized transformer |
| M1c | owner's M1, about 1 h | llama.cpp SmolLM2-135M Q4_0 with a q8_0 cache at 2K and 8K tokens, on a performance core and under `taskpolicy -c background`, joules per token from two run lengths | If Q4_0 on efficiency cores is within 1.5× of the projected converted path at 2K, the energy case rests on M4 or ternary weights |
| M1b | owner's M1, only if M1a passes | Distillation from 360M: dot, `intrinsic_linear`, intrinsic learnable, intrinsic and key-norm annealed to `t = 1`; 3 seeds, paired data | Do curved heads beat *both* controls on held-out KL/NLL beyond seed noise? |
| M2 | M1 days, or GPU if approved | 4-bit QAT distillation including integer activations, table nonlinearities and a ≤4-bit output head | Chat quality at 4 bits. **Gated on owner decision 2** |
| M3 | here and M1 | Integer serving; chat CLI; measured joules per token. **Built and measured here (§7); M1 speed and energy pending `scripts/lut-m1-chat.sh`, which also runs the M1c llama.cpp reference** | Against the M1c baseline |
| M4 | here and M1 | Long context: dump real SmolLM2 queries and keys, test query-aware indexes, and train a native hyperbolic memory/retrieval layer over the converted backbone | The geometry's best-supported role (§0.4). ≥99% of attention mass at ≤5% scored on real heads? |
| M5 | M1 | Parameter memory: none, learned product keys, E8 or sign sub-codebooks, n-gram addressed | ≥0.02 nats or factual-recall gain at equal active parameters |

## 5. What the adversarial review changed

| Defect | Severity | Response |
|---|---|---|
| "Exact conversions" are hyperbolic in name only near κ = 0; curvature stays in the flat basin | Critical (science) | Relabelled as flat-limit reparametrisations with prior art. Added the first-order control and the zero-training drive test with a pre-registered rule. The owner's backbone decision moved before M2 |
| M1b did not fit an M1 (about 27 GB for curved arms at batch 4×256) | Critical (feasibility) | Batch 1 with gradient accumulation by default; memory guidance; drive before training |
| The energy thesis compared unlike configurations | Critical (feasibility) | Restated as parity at 4 bits and 2K; the energy gate moved to M1c/M4 |
| Real dot heads are not indexable like synthetic ones | Major (science) | M4 gated on real dumps; geometry assigned to trained memory/index layers |
| Stand-in cannot discriminate between geometries | Major (science) | §2.2 scoped as a machinery check |
| Enclosing-scope gain confounded with copy distance; "ceiling" mislabelled | Major (science) | §0.5 restated |
| Raw κ is not scale-free; the annealing floor could be escaped by shrinking keys | Major (feasibility) | Curvature stated in `t`; the floor is recomputed from the current keys each step |
| `sample` silently ignored saved weights; missing system turn | Major / minor (feasibility) | Fixed (`f43aa4e`) |
| Latent NaN in the unused series branch | Minor (feasibility) | Fixed |
| Literature labels | Minor | Mamba-in-the-Llama's 5.64 is the fully Mamba2 variant; T-MAC's 70% is its 2-bit best case (20.6% at 4 bits) |

## 6. Owner decisions

Answered by the owner on 2026-09-26 and recorded as [D10](DECISIONS.md) on this branch:

- **Backbone.** Option (a): the converted backbone is accepted, and the rule is restated as no floating point, no dense float matmul, and no multiplier in learned weight maps. D5 is unchanged, so the dense backbone is reported as the interim chat vehicle, not the terminal architecture.
- **SIMD.** The faster option: one audited SIMD crate may use `unsafe` for NEON kernels.
- **Multipliers.** Chosen for runtime speed: hardware integer multiplication is allowed for products of runtime values or fixed non-learned constants. Learned weight maps stay multiplier-free.

Paid compute, which the owner asked to size (not authorized). The estimate is *Derived*:
- **Workload.** Distilling SmolLM2-360M into a quantized 135M costs about 1.5 GFLOP per training token (6·135M for the student plus 2·362M for the teacher).
- **Throughput.** An H100 at 20–40% of its 989 TFLOP/s bf16 peak, which is typical for small models.
- **Prices.** *Literature*, public listings, August–September 2026: H100 80GB about $1.5–2.2 per hour on marketplaces (Vast.ai), $2.0–3.3 on RunPod, about $4 on Lambda, and $4–11 on the large clouds.

| Run | Tokens | H100 hours | Cost at $1.5–4/h |
|---|---:|---:|---:|
| 4-bit QAT, first pass | 1B | 1–2 | $2–8 |
| 4-bit QAT, near saturation (ParetoQ-scale) | 10B | 11–21 | $17–84 |
| Ternary QAT, one attempt | 30B | 32–64 | $48–256 |

- A 360M student costs about 2.7× these figures.
- Setup, evaluation and failed attempts add perhaps 1.5–2×.
- Post-training 4-bit quantization costs nothing and runs on the M1 in minutes. It comes first; QAT is needed only if its measured loss is too large.
- Ternary is worth buying only after M1c shows its energy advantage on the real machine.

Still open:
1. Hugging Face access in this environment. The real checkpoints are on the M1 either way.
2. D5: memory layers addressed by fixed geometric codes (sparse parameter access without a learned gate).
3. Evaluation by event-level metrics alongside mean NLL.

## 7. M3: the integer serving engine

**What exists.** A converted Llama checkpoint is exported offline to a sealed 4-bit artifact and served with integers only ([crate README](../../crates/uor-r4-lut/README.md)).
- Learned weight maps are served by vector table reads (`vpshufb` on AVX2, `tbl` on NEON: sixteen rows per instruction), shifts and additions.
- Runtime products (attention, gating, RoPE, normalization) use the integer multiplier, as D10 allows.
- Nonlinearities are sealed tables, and the key/value cache is 8-bit.
- The exporter quantizes by round-to-nearest or by GPTQ (*Literature*: Frantar et al., arXiv 2210.17323) from input moments on calibration text, into the same artifact layout.
- *Literature for context.* T-MAC (arXiv 2407.00088) serves low-bit weights from bit-plane tables; this engine instead builds per-activation tables of multiples, which suits 4-bit weights. The two have not been compared.

**The exactness chain (Measured).**
1. The AVX2 and NEON kernels reproduce the portable reference bit for bit on random and extreme matrices. AVX2 was run natively; NEON ran under `qemu-aarch64` emulation. The reference in turn matches exact `i128` products.
2. No optimization changed an output. The tiny test model's integer NLL is identical before and after every kernel change. The small Llama below gives an identical NLL and an identical 110-token greedy continuation at 1 and 4 threads.
3. The integer engine reproduces its own dequantized float model in greedy decoding, token for token, for both exports. On this model the remaining gap to the original is therefore the 4-bit weights.

**Disassembly audit (Measured).** In the compiled weight kernels:
- AVX2: 451 instructions, including 32 `vpshufb`, and no vector multiply;
- NEON (inlined): 128 `tbl` and no vector multiply;
- the only multiplies are scalar block-offset computations outside the per-weight loop.

**Fidelity (Measured)** on a real trained model, since the SmolLM2 weights are not reachable here.
- *The model.* A 4.2M-parameter byte-level Llama (6 layers, width 256) trained on WikiText-2 for these tests. A helper trained it with a throwaway JAX script in scratch space, which is not in the repository. The repository's loader agrees with that script's forward pass to 5e-7 nats per token.
- *Protocol.* Evaluation on 4,096 held-out validation positions. GPTQ calibration on 8,192 training positions with damping 0.01, the literature default, not tuned here.

| Integer engine versus | NLL change (nats/token) | KL | top-1 |
|---|---:|---:|---:|
| original bf16, round-to-nearest | +0.0067 | 4.6e-3 | 95.95% |
| original bf16, GPTQ | +0.0034 | 1.8e-3 | 97.95% |
| its dequantized float, round-to-nearest | +0.0007 | 2.6e-4 | 99.32% |
| its dequantized float, GPTQ | +0.0009 | 2.7e-4 | 99.41% |

GPTQ also halves the mean relative output error on the calibration inputs (0.022 against 0.044). In a reduced run (1,024 calibration and 512 evaluation positions), GPTQ still lowered KL and raised top-1, but its NLL change was larger. At that size NLL change is too noisy to rank exports; KL and top-1 are the fidelity measures.

**Throughput (Measured, this 4-vCPU x86-64 container, not an M1).** Random artifact of SmolLM2-135M's shape (86.8 MB), AVX2, other work paused:

| threads | 64 tokens | 1024 tokens |
|---:|---:|---:|
| 1 | 36.5 tok/s | 30.4 tok/s |
| 2 | 47.4 tok/s | 39.7 tok/s |
| 4 | 54.8 tok/s | 42.0 tok/s |

- The scalar table kernel it replaced ran at 5.9 tokens/s on one thread.
- The weight kernel alone takes about 0.10 ns per weight with matrices in cache, and about 0.17 ns when streaming the model.
- At four threads the container's memory bandwidth limits throughput.

**What this does not show.**
- Speed or energy on an M1.
- Fidelity or chat quality of the 135M/360M checkpoints.
- Any advantage over llama.cpp.
- A geometric mechanism: the served model is D10's dense interim vehicle.

The next measurement is the owner's `scripts/lut-m1-chat.sh` run, with `ENERGY=1` and a llama.cpp GGUF for the M1c reference.

## 8. M4a work card: a learned cache memory over a frozen backbone

Written before the measured run; the rules below do not change after it.

- **Deliverable.** A cache memory read over a frozen converted Llama, with measured held-out NLL gain and read concentration for three key geometries of identical parameters: `uor_r4_training::cache_memory` and the example `cache-memory`.
- **Blocker it addresses.** The integer engine serves a dense backbone with a fixed window and no memory. In converted dot heads, curvature stays in the flat basin and the heads are not indexable (§0.3–0.4). Geometry has helped only where scores were *trained* with distances.
- **Mechanism.** A learned continuous cache (*Literature*: Grave, Joulin and Usunier, arXiv 1612.04426).
  - Queries and keys are learned maps (width 256 to 32) of the backbone's final normalized states.
  - Entries point at the token that followed them.
  - A query reads only entries at least 256 positions back, which is exactly what the backbone's window cannot see.
  - A learned gate mixes the cache with the backbone's distribution.
  - Scores: `dot` is `beta q.k`; `euclid` is `-beta |q-k|^2`; `lorentz` is `-beta d_H(q,k)^2` on the hyperboloid of curvature -1. Its maps' scale sets how hyperbolic it is, and it becomes `euclid` at small radius.
- **Fixed conditions.**
  - Backbone: the small Llama of §7 (weights `8c5edbd0…`).
  - Segments of 1,024 tokens with 768 query positions each: 256 training segments (WikiText-2 train) and 64 validation segments (valid).
  - 400 steps, batch 4, AdamW at lr 3e-3.
  - Seeds 1 and 2, with the batch order paired across geometries.
  - Initialization shared.
- **Decisions, fixed in advance.** "Beats" means a lower mean held-out NLL over the two seeds, with both seeds agreeing in sign and the difference larger than the seed spread.
  1. If no arm beats the backbone, the cache does not help this backbone. M4a stops here as a negative result.
  2. If `lorentz` beats both `euclid` and `dot`, Lorentz keys go into M4b (integer serving of the cache read), which needs the integer arcosh table (task 3b).
  3. If `lorentz` and `euclid` are within the spread and both beat `dot`, distance scoring matters but curvature does not. Adopt `euclid`; no arcosh table.
  4. If `dot` is at least as good as both, geometry is not needed for this role. Adopt `dot`.
  5. Secondary: if the adopted arm holds 99% of its read mass in at most 5% of the readable entries, a sparse index is viable for M4b.
- **Checks.** Five unit tests:
  - a closed gate reproduces the backbone;
  - no entry inside the gap is read;
  - the Lorentz score equals `acosh`, and is flat at small radius;
  - gradients reach every variable;
  - the concentration count.

  Plus a smoke run and a one-seed pilot to confirm that the arms train at all.
- **Cost.** About 1 h of implementation. About 40 minutes of CPU for the run: backbone features once, then six arms of 400 steps.
- **Deviation after the pilot, before the measured run.**
  - *Change.* 1,600 training segments instead of 256, read in shuffled passes, so the 400 steps of 4 read each segment once. The decisions and every other condition are unchanged.
  - *Why.* The one-seed pilot trained on 64 segments, about 19 passes, and overfit. Validation NLL change was best mid-training (Lorentz −0.009 at step 100, Euclid −0.008 at step 200). By step 300 it was positive for every arm (+0.0085, +0.0015, dot +0.0037), while training loss kept falling.

### 8.1 M4a result (Measured)

64 held-out validation segments; 768 query positions each, 49,152 in all. The backbone NLL at those positions is 1.1613 nats per token. Change from the backbone:

| Keys | Seed 1 | Seed 2 | Mean | 99% of the read in | Top-8 mass | Gate | Mean `\|k\|^2` |
|---|---:|---:|---:|---:|---:|---:|---:|
| dot | −0.0101 | −0.0092 | −0.0097 | 3.2–3.4% of entries | 0.97 | 0.02–0.04 | 135–141 |
| euclid | −0.0138 | −0.0143 | −0.0141 | 2.2–2.4% | 0.98–0.99 | 0.05 | 111–119 |
| lorentz | −0.0184 | −0.0188 | **−0.0186** | 13.1–13.5% | 0.86–0.87 | 0.07–0.09 | 178–186 |

- **Every arm beats the backbone.** Both seeds agree, and every mean exceeds its seed spread (0.0004–0.0008).
- **Paired, Lorentz beats both other arms.**
  - Against Euclid: −0.0046 and −0.0045.
  - Against dot: −0.0083 and −0.0096.
  - Euclid against dot: −0.0038 and −0.0051.
- **Decision 2 applies.** Lorentz keys go into M4b, the integer serving of the cache read. The integer arcosh kernel it needs now exists (`uor_r4_lut::kernels::arcosh1p_q24`, worst error below 3·10⁻⁷ nats).
- **Decision 5 does not hold for the adopted arm.**
  - The Lorentz read spreads its mass: 99% needs about 13% of the readable entries.
  - The flat arms concentrate theirs (2–3%).
  - So a sparse index is not shown to be viable for the Lorentz cache; M4b reads it densely. At 32 dimensions and a few thousand entries, that is well under 1% of the backbone's per-token work (*Derived*).
- **The learned Lorentz keys are strongly curved.** Mean `|u|^2` is about 180, a hyperbolic radius of about 3.3, close to the cycle-3 read's 3.5.
- **The flat maps grow almost as large** (`|u|^2` 110–140), so scale alone does not separate the arms.
- *Scope.*
  - A 4.2M-parameter byte-level backbone, whose cache holds only what lies beyond its 256-token window. For scale, 0.0186 nats is 0.027 bits per byte (1.6%).
  - Training ran 400 steps. Dot was still improving at step 400 (−0.0052 to −0.0101 over the last 100 steps), while Euclid and Lorentz had levelled off. The ranking is at the fixed budget.
  - The effect on SmolLM2, with BPE tokens, a far stronger backbone and a 2K window, is unmeasured.
- *Cost.* 34 minutes on the container: 815 s of backbone features, then 105–341 s per arm.

### 8.2 M4b: the integer Lorentz cache (Measured)

The chat engine now serves a learned Lorentz cache (commit `453aa9d`).

**How it is served** ([crate README](../../crates/uor-r4-lut/README.md)).
- An artifact may carry the cache: its 4-bit query, key and gate maps, the score scale on the `(16 + m) 2^(e - 4)` grid, and the sealed arcosh table.
- Each decoding step:
  - finalizes the previous position's entry with the token that followed it;
  - keeps this position's key;
  - reads the entries at least `gap` positions back.
- `z - 1 = q0 k0 - <q, k> - 1` is formed exactly at `2^-64` from Q32 time coordinates. The distance comes from the arcosh table, the weights from the exp table, and the gate from a sigmoid built on the exp table.
- The next token comes from `(1 - g) p_model + g p_cache`.
- A full attention window slides: the recent half is re-encoded, and the cache keeps everything older.

**Parity** on the small Llama: all 64 held-out segments of §8.1 (49,152 positions beyond the 256-token window), the saved seed-1 Lorentz cache, and the GPTQ backbone export.

| | Backbone NLL | Gain from the cache | Mean gate |
|---|---:|---:|---:|
| float backbone, float cache | 1.1613 | −0.01840 | 0.0875 |
| integer backbone (GPTQ 4-bit), integer cache | 1.1636 | −0.01828 | 0.0883 |

- The integer cache keeps 99.3% of the float cache's gain.
- Per segment, the two gains differ by 0.0001 on average (standard deviation 0.0009).
- The integer hyperbolic distance matches `acosh` in f64 to 10⁻⁴ relative, for coordinates from 0.01 to 6.
- The exp-table sigmoid matches to 2·10⁻⁶.
- A 900-step `lut-chat` run through the sliding window, with sampling and the cache, completed without error.

**What remains.** Training a cache for SmolLM2 needs the checkpoint, so it runs on the owner's machine.
- The window to pair with the cache is a design choice. The full 2,048-token window gives memory beyond two thousand tokens.
- A shorter attention window, such as 512, would cut attention work at long context. The cache would then cover the rest.
- Which works better for chat is unmeasured.

## 9. Cost of this phase

- **Machine.** One shared review container: 4 cores, 15 GB, no GPU. It ran from about 16:20 to 18:15 UTC, at load 10–30. No paid or external compute.
- **Lead's runs.** The lead's single-threaded stand-in conversion and fine-tune runs recorded 2.9 wall-clock hours in total, under contention: 7,943 s and 2,580 s.
- **Reviewers' runs.** Their costs are in their reports. The experiment reviewer used about 1.2 CPU-hours. The science reviewer's five checks each took under a minute.
- **Context 256.** The four cycle-3 context-256 runs finished; their cost is in the cycle-3 addendum.
- **Cycle 3b.** The integer Lorentz read's build, parity runs and fine-tunes; their cost is in [cycle 3 §11](hyperbolic-cycle3-2026-09-26.md#11-addendum-the-integer-lorentz-read).
- **M4 (§8).**
  - The pilot took about 10 minutes, and the measured run 34 minutes.
  - Re-training and saving the seed-1 Lorentz cache took 20 minutes; the parity runs took 30 s and 134 s.
  - Local end-to-end runs of the M1 script took a few minutes each.
  - Scratch storage grew by under 20 MB.
- **M3 (§7).** Built and measured from about 18:50 to 21:00 UTC on the same container.
  - The small Llama took 1,920 s of single-core training (34 min wall).
  - GPTQ exports take 5–10 s at that size.
  - Benchmarks paused the four context-256 runs for about 17 minutes in total. Eleven of those minutes were an accidental pause, when a process match stopped the benchmarking shell itself. Their wall-clock records include the pauses.
  - Scratch storage:
    - a 269 MB random 135M-shaped checkpoint, removed after benchmarking;
    - 91 MB for its artifact and profiles;
    - 28 MB for the small Llama;
    - 67 MB of its artifacts, dequantized checkpoints and token files.
- **Storage.** Phase-2 scratch outputs are 0.2 GB outside the repository, plus a shared 1.7 GB build cache. The repository gained two Rust source files, one script, this note and a 32 KB evidence file.
