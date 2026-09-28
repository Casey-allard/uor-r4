# Geometric language model lab, phase 2: converting attention, and where hyperbolic geometry pays

2026-09-26 · Requested by the owner · References #820 · Branch `claude/blissful-wozniak-girwwq` (kept separate from `main`)

**Status.** An evidence note and proposed roadmap, not a decision record. It follows [cycle 3](hyperbolic-cycle3-2026-09-26.md) and applies the owner's decisions of 2026-09-26: weight transfer from an open transformer is approved provided the served model performs no matmul; hyperbolic geometry is the lead mechanism; offline training is in scope; paid compute needs the owner's explicit approval.

Labels: **Measured** (run in the review sandbox), **Derived**, **Literature** (retrieved in this session; arXiv id given), **Hypothesis**. The lab consisted of five specialist reviewers (mathematics, architecture, training and data, systems, experiment), two adversarial reviewers and the lead. Their reports and scratch experiments stay outside the repository. The numbers this note depends on are in [the evidence file](../evidence/lab-phase2-2026-09-26.json). The Rust tool it introduces is in the training crate (`kappa_llama`, example `kappa-conversion`).

## 0. Findings

1. **Chat quality is bought with training tokens, so the practical route to a chat model is converting an open instruct model.**
   - *Literature.* Open small talk appears at 100–350M parameters only after 1–11T training tokens (MobileLLM 2402.14905, SmolLM2 2502.02737, LFM2 2511.23404).
   - *Derived.* An M1 trains roughly 0.2B tokens per week at 150M parameters (assumed Candle-Metal throughput, not yet measured).
   - Converting SmolLM2-135M/360M-Instruct (Apache-2.0) is therefore the only route to that band inside the owner's budget. Training from scratch gives narrow-domain chat at best.
2. **A trained model's attention geometry can be swapped cheaply.**
   - *Measured on a stand-in teacher* (3-layer 0.6M-parameter byte-level attention LM, WikiText-2, 1.990 bits per byte): every tested score geometry came within 0.004 bits per byte of the teacher after 400 distillation steps (about 1.6 MB of text) that updated only the query/key projections and one temperature per layer (§2).
   - Scores that start far off also recover: the committed Lorentz read starts at 4.01 bits per byte and ends at 1.995.
3. **Three constructions make a hyperbolic head reproduce a trained dot-product head in the flat limit.**
   - *Derived.* The three constructions (§2.1):
     - a key-norm curvature homotopy;
     - an intrinsic polarization score;
     - norm completion.
   - *Measured.* On the stand-in teacher's real heads, the intrinsic score at log-scale −6 changed validation bits per byte by 5×10⁻⁶.
   - *Measured.* In the new Rust tool, on a synthetic SmolLM2-shaped checkpoint, the largest logit difference was 8×10⁻⁶ at curvature 4×10⁻¹¹.
4. **An exact start is not enough: learned curvature stays in the flat basin and inherits the teacher's blind spots.**
   - *Measured, toy retrieval heads on this repository's code-scope tree.* The dot teacher gets 0.4% of non-root queries right. The same teacher converted exactly and fine-tuned with learnable curvature stays at 0.4%.
   - Annealing curvature upward lifts it to 31%. Converting directly into the curved regime reaches 52.5%, after a transient collapse. A hyperbolic head trained from scratch reaches 85.5%.
   - The conversion tool therefore has a curvature-annealing schedule (§2.3).
5. **Hyperbolic geometry wins on sparse, hierarchy-determined retrieval, by learnability rather than capacity.**
   - *Measured.* At 64 key dimensions on the code-scope tree, dot keys learn only "attend to the root" (0.0–1.7% non-root, also with three times the training). Hyperbolic keys reach 85–90%.
   - *Measured.* A learned per-key potential on dot keys recovers only a third of that gap.
6. **The cycle-3 gain is concentrated where the geometry predicts; the ceiling for mean loss is low, and context length is the big lever.**
   - *Measured, per-event rebuild of the saved cycle-3 models.* The Lorentz read wins by 0.060–0.067 nats per token on copies whose last occurrence lies in an enclosing scope. This holds in all three code seeds (95% intervals exclude zero), on 5.4% of targets.
   - *Measured.* A copy oracle given exact scope relations gains only 0.010–0.020 nats at 128–2,048-token windows. Widening the window from 128 to 2,048 tokens gains 0.40 nats.
7. **Hyperbolic geometry is most valuable as the long-context index.**
   - *Measured, synthetic flat recall, 16,384 stored keys.* Content-addressed cells admitted ≥99.8% of targets while scoring 1.2% of keys (2.8% of keys compared, counting the advertisement comparisons). Storage-order chunks, the cycle-1 design, never passed 55%.
   - *Measured.* Dot-product advertisements must also publish their squared norm: right cell at one probe 19.5% without it, 98.2% with it. The Lorentzian score already carries that term.
   - *Measured, code-scope tree.* A radius-aware index (always admit the smallest-radius keys, route the rest by direction) reached 84% of targets at 3.7% scored, against an 86.6% ceiling. When the key radius tracks depth (correlation 0.6), it reached 97–99% of the ceiling.
8. **Energy is dominated by the weight path, not by multiplication.**
   - *Derived* from published constants; nothing ran on an M1. For a converted 135M model at 2K context, table-driven ternary weights on the efficiency cores project to about 4–7 mJ per token, against about 44 mJ for fp16 llama.cpp-class serving and about 27 mJ for Q4_0.
   - *Derived.* The hyperbolic read with compact keys takes about 7% of step time.
   - The M1 measurement procedure is specified in §4.
9. **Negative results.**
   - A power-law (horocycle) positional prior extrapolated worse than exponential decay (3.264 vs 3.226 nats at 2,048 tokens).
   - A conservative Hamiltonian read never settles on a key; with damping it costs about 100 softmaxes.
   - Primes, CRT and Galois-field hashing give no advantage over simple tabulation for exact content memory (all within 0.2% of the ideal collision count; naive modulo 231× worse).
   - Two-level advertisement cells plateau at 99.6% admission.
   *All Measured.*

## 1. The route to a chat model

| Model | Size / tokens | Result (Literature, arXiv) |
|---|---|---|
| TinyStories | 1–80M | Narrow-domain fluency and consistency at 28–80M (2305.07759) |
| MobileLLM | 125M / 350M, 1T | MT-Bench 2.33 / 3.28 (2402.14905) |
| SmolLM2 | 135M (2T) / 360M (4T) | IFEval 29.9 / 41.0 (2502.02737) |
| LFM2-350M | 11T with logit distillation | IFEval 65.1 with 6 attention + 10 convolution layers (2511.23404) |
| Mamba-in-the-Llama | 8B instruct, 20B tokens | MT-Bench by attention share 50% 7.35, 25% 6.86, 0% 5.64 (2408.15237) |
| LoLCATs | 8B, 40M tokens, LoRA | MMLU 52.8 vs 66.6; 23.8 without its sliding window (2410.10254) |

Reading. Conversion literature keeps some exact attention: fully recurrent conversions lose most. This lab replaces only the attention *score*, keeping softmax attention over the cache. That is the cheapest regime in this table, and on the stand-in it cost almost nothing.

The proposed converted model (GCM-L, *Hypothesis* for quality):
- SmolLM2-135M-Instruct weights, 4-bit first. Ternary needs about 30B QAT tokens to saturate (ParetoQ, as reported by the training reviewer).
- Curvature-converted attention heads.
- A hybrid cache: a few exact layers, windowed layers with a RoPE-transported recurrent state, and a content-addressed or radius-aware index for long reads.
- Multiplier-free integer serving through table-driven kernels.

## 2. Converting dot-product attention

### 2.1 Three flat-limit constructions (Derived)

For one head of width `r`, curvature `κ = ε²`, and scaled coordinates `a = εq`, `b = εk` lifted to the unit hyperboloid `x ↦ (√(1+|x|²), x)` with geodesic distance `d`:

| Construction | Score | Flat limit |
|---|---|---|
| Key-norm curvature homotopy (architecture reviewer) | `(|b|² − d(a,b)²) / (2ε²√r)` | `⟨q,k⟩/√r − |q|²/(2√r)` |
| Intrinsic polarization (lead) | `(d(o,b)² − d(a,b)²) / (2ε²√r)`, with `o` the origin | same |
| Norm completion (mathematics reviewer) | separate completion coordinates put `q₀ = Q`, `k₀ = C`, so `z = QC − q·k` and `β(δ − arcosh z) = const + s + O(1/QC)` | the dot score `s`, as `QC → ∞` |

- All three follow from `d(a,b)² = |a − b|² + O(ε⁴)` near the origin and the polarization identity `2⟨q,k⟩ = |k|² + |q|² − |q − k|²`.
- The per-query term does not change a softmax.
- RoPE rotates spatial coordinates only, which is a hyperboloid isometry. All three therefore remain relative-position scores (*Measured*: invariance error below 10⁻⁴ in the Rust test, 8.5×10⁻¹⁴ in the mathematics reviewer's check).
- A fourth score converts well without being exact at the start: the **Gromov product** `(q|k)_o = ½(d(o,q) + d(o,k) − d(q,k))`.
  - *Derived.* In a rooted tree it is the depth of the lowest common ancestor.
  - *Measured, stand-in.* Its fitted scale explained 86–93% of the teacher's score variance, against 57–68% for the committed Lorentz read.

### 2.2 Stand-in conversion (Measured)

Teacher: a textlm attention model (3 layers, width 128, single-head RoPE attention, SwiGLU; 607,488 parameters), 1.9897 bits per byte on WikiText-2 validation (256 windows of 128 bytes).
- Zero-shot conversion fits one scale per layer by least squares on row-centred causal teacher scores.
- Distillation minimizes KL(teacher ‖ student) for 400 steps of 32 × 128 bytes.

| Score | Zero-shot | After Q/K + temperature distillation | After all-weight distillation |
|---|---:|---:|---:|
| dot (drift floor) | 1.9897 | 1.9909 | 1.9929 |
| intrinsic polarization (log ε = −6) | 1.9897 | 1.9905 | 1.9930 |
| cosine, fitted β | 2.0511 | 1.9908 | 1.9936 |
| Gromov product, fitted scale | 2.0490 | 1.9907 | 1.9951 |
| Euclidean distance | 3.9259 | 1.9929 | 1.9943 |
| committed Lorentz read `β(δ − d)` | 4.0115 | 1.9946 | 1.9946 |

Notes:
- All-weight distillation at a higher learning rate only adds drift: it is worse than Q/K-only distillation for every score, dot included.
- *Measured, drift away from the teacher as curvature grows (mean row KL at log ε = −3).*

  | Layer | Intrinsic | Key-norm |
  |---|---:|---:|
  | 0 | 0.032 | 0.024 |
  | 1 | 0.056 | 0.24 |
  | 2 | 0.086 | 0.53 |

  The intrinsic score drifts more gently on layers 1–2 and about equally on layer 0. On synthetic heads the architecture reviewer found the opposite ordering. Real heads should decide which score is the default.

### 2.3 The flat basin and curvature annealing

*Measured (mathematics reviewer, toy code-tree heads, one seed):*

| Arm, fine-tuned 1,500 steps from a dot teacher (non-root accuracy at 48 / 96 / 192 stored nodes) | Non-root accuracy |
|---|---|
| Dot teacher | 0.0 / 1.7 / 0.4% |
| Norm-completion conversion, curvature learnable | 0.0 / 1.4 / 0.4% |
| Same, completion annealed into the curved regime over 750 steps | 35 / 35 / 31% |
| Direct curved lift from step 0 (overall accuracy first falls to about 1%) | 59 / 57 / 53% |
| Hyperbolic head trained from scratch | 85.5% at 192 |

- *Derived.* An exact start sits where the curvature effect is O(κ), so its gradient is tiny. A distillation objective toward the flat teacher rewards zero curvature outright.
- The Rust tool therefore anneals: `anneal_to`/`anneal_steps` raise a floor on every head's log-scale linearly and hold it (§3).
- The curvature question must be asked with annealed arms against a matched dot control. The measure is held-out loss *and* hierarchy-dependent events. "Did learned curvature grow" is not the test.

## 3. The Rust conversion tool

`crates/uor-r4-training/src/kappa_llama.rs` and `examples/kappa-conversion.rs` (branch commits `c67e230`, `50bee85`):
- **Loading.** A Hugging Face Llama checkpoint: `config.json` plus BF16 or F32 `model.safetensors`, grouped-query attention, tied or untied output head. Biases, RoPE scaling and interleaved RoPE are refused.
- **Scores.** Every head's score is replaced by `dot`, `key_norm` or `intrinsic`, with a learnable per-head curvature and temperature. A dot run therefore carries the same non-curvature freedom and serves as the matched control.
- **Numerics.** The squared arcosh uses a four-term series (`2z − z²/3 + 4z³/45 − z⁴/35`) below `z = 0.05`, so the flat limit keeps f32 precision and finite gradients.
- **Modes:**
  - `tokenize`: the checkpoint's own byte-level BPE;
  - `probe`: largest logit difference, KL, top-1 agreement and NLL against dot over a curvature grid;
  - `train`: next-token loss, or KL to a dot teacher such as SmolLM2-360M-Instruct; optionally trains the query/key projections; curvature annealing;
  - `sample`: greedy chat-template reply.
- `probe` and `train` claim and seal their report roots.
- **Checks.** Seven focused tests: config parsing, the series switch, flat-limit agreement, RoPE relative-position invariance, gradients to curvature and query/key weights, the curvature floor, and distillation toward a flat teacher. An end-to-end run on a synthetic bf16 grouped-query checkpoint covered probe, both training objectives and annealing.
- **Not done.** No real checkpoint has been converted: the SmolLM2 weights are not reachable from the review environment (Hugging Face egress is blocked). They are on the owner's Mac.

## 4. Roadmap

| # | Where | Work | Decision it makes |
|---|---|---|---|
| M1a | owner's M1, under 1 h | `kappa-conversion mode=probe` and `mode=sample` on SmolLM2-135M-Instruct | Is the flat-limit conversion within f32 rounding of the checkpoint on the real model? |
| M1b | owner's M1, hours | Distil SmolLM2-360M-Instruct into the converted 135M. Arms: dot; intrinsic with learnable curvature; intrinsic annealed; key-norm annealed. Query/key trainable, paired data order, 2 seeds | Do curved heads match or beat the dot control on held-out KL and NLL, and do they win on hierarchy-dependent events? Also yields per-head curvature histograms |
| M1c | owner's M1, 1 h | llama.cpp f16 and Q4_0 SmolLM2-135M joules per token (idle-subtracted, two run lengths); Candle-Metal throughput | Replaces every assumed M1 constant |
| M2 | M1 days, or about 1 GPU-hour if approved | 4-bit QAT distillation of the converted student on permissively licensed chat data | Chat quality at 4 bits: masked assistant-turn NLL, instruction-following checks, multi-turn recall |
| M3 | here and M1 | Integer serving of the 4-bit converted model: table-driven weights, per-query q·k product tables, table softmax, norms and curvature; chat CLI | Measured joules per token. The claim holds at ≥2× below Q4_0 on the efficiency cores |
| M4 | here and M1 | Hybrid cache and admission at 2K–8K tokens; event-level evaluation | ≥99% of attention mass admitted at ≤2% scored on real heads? |
| M5 | M1 | Parameter memory arms: none, learned product keys, E8 or sign sub-codebooks, n-gram addressed | ≥0.02 nats, or factual-recall gain, at equal active parameters |

`scripts/kappa-m1-pilot.sh` runs M1a and M1b in one command on the owner's Mac.

## 5. Adversarial review

[pending: feasibility/engineering and science/novelty reviews]

## 6. Owner decisions

1. Allow Hugging Face (huggingface.co and its download hosts) in this environment, or keep real-checkpoint work on the M1.
2. Paid compute: about one H100-hour covers the 135M 4-bit conversion; ternary needs far more.
3. Does a converted stack satisfy "no transformer backbone at serving"? The stack is Llama-derived blocks with curvature-converted attention, table-executed ≤4-bit maps and integer serving. The alternative is to restate the rule as "no dense float matmul and no multiplier at serving".
4. One small audited SIMD crate with `unsafe` for the NEON table kernels, outside the frozen `forbid(unsafe_code)` crates? Without it the weight path stays 12–16× slower (*Measured*, x86 ratios).
5. D5: are memory layers addressed by fixed geometric codes allowed? That is sparse parameter access without a learned gate.
6. Evaluation: judge geometry by event-level metrics (scope and entity resolution, long-memory recall) alongside mean NLL?

## 7. Cost of this phase

[pending: CPU, wall and storage ledger]
