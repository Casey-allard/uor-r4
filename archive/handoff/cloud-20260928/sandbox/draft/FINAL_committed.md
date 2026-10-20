# First-principles review of UOR-R4: can it reach a geometric language model on an M1?

2026-09-25 · Requested by the owner · References #820

**Status.** This is an evidence document, not a decision record. It changes no D-entry, gate, artifact or active work card. Owner decisions are requested in §11.

**Method.** Eight specialist agents ran in parallel:
- mathematics;
- physics and energy;
- CS/ML architecture;
- literature and novelty;
- programme audit;
- code verification;
- a quantization experiment;
- a state-tracking experiment.

The lead reviewer ran three side experiments and wrote the synthesis. An adversarial red-team agent then spot-checked 31 numbers and citations. It found 3 critical, 11 major and 14 minor defects in the first draft, and all of them are corrected here.

**Evidence labels.** Every claim carries one of:
- **Source**: repository file and line;
- **Measured**: run during this review;
- **Literature**: paper retrieved during this review, cited by arXiv id;
- **Derived**: shown here;
- **Hypothesis**: not yet tested.

The repository was read-only throughout the review. Appendix B lists the evidence.

**A note on the agents' instructions.** The shared briefing asked every agent to flag the tension between D5 (per-token parameter sparsity) and the owner's "no sparse routing". So when several agents raise it, that is *not* independent corroboration.

## 0. Bottom line

1. **A breakthrough is possible, but not the one the programme is organised around.**
   - **A frontier-quality model trained from scratch on an M1 is not reachable.** Even assuming well-used compute, which is not yet measured (§3.3), one M1-week trains roughly a 20–40M-parameter model.
   - **A "geometric predictive advantage" on natural-text loss is supported neither by theory nor by any measurement.** That includes this review's own WikiText-2 runs (§6.3).
   - **Three outcomes are realistic and falsifiable (§9.1):**
     - **B1:** exact, cheaply served non-abelian state inside a real language model, judged against the *strongest* non-diagonal controls;
     - **B2:** J/token measured on the M1 at matched quality;
     - **B3:** conversion of an open model. This one is expensive: a fully attention-free conversion took billions of tokens in the literature.
2. **The project is partly off course (§7).**
   - The engineering has been back on course since D8: a correct Rust autodiff trainer, a matched control, learned 4-bit rounding, and bit-exact integer serving.
   - The geometry has left the model. The active crates contain no prime, zeta, Hopf, hyperbolic, icosian, E8 or lattice code.
   - The only geometric element is a quaternion lane rotation. It is fed by a dense matmul, and it lost by 0.025 nats (one seed) to a control that is *itself* a quaternion map (H(v)H(e0)x = v·x·v).
   - The founding routing thesis never had a valid test.
   - The pace of decisions (11 in 7 days; 3 direction changes in 14 hours) outran the evidence.
3. **The original idea was sound, and it has been published by others.**
   - TurboQuant, PolarQuant and QJL all *keep* the radius; none of them ablates it.
   - The 4-D, radius-preserving quaternion quantizer was published by MIT/IBM in May 2026 (HQMQ). On Llama-3-8B KV caches at about 3 bits it beats a TurboQuant-style baseline.
   - 4-D is a weak block size for compression; 8-D, 24-D and trellis codes win.
   - The idea's best homes are:
     - compressing event and KV memory with a *quantized* radius;
     - more deeply, **dynamics**, where the radius is the retention (decay) factor and the direction is a non-commuting rotation (§8.1).
4. **The physics premise needs restating.** Bytes moved and instructions executed decide energy; multiplier circuits barely register.
   - In DRAM-bound serving, multipliers account for at most about 0.4% of the energy floor.
   - Multiplier-free *lookup-table* kernels can still save energy by executing fewer instructions: T-MAC measured 21–61% less energy at equal bit width.
   - The current integer session does the opposite: it *emulates* multiplication in software. That makes it 4.4× slower than hardware multiply on x86, with bit-identical output.
   - No joule has ever been measured in this repository (§5).
5. **Geometry has one theorem-backed job it can do cheaply: exact non-abelian state transport.**
   - With transitions in the binary icosahedral group 2I (the 600-cell), one 4-D lane can track the A5 word problem.
   - The A5 word problem is NC¹-complete. Diagonal SSMs of the Mamba class provably cannot track it at arbitrary length, assuming TC⁰ ≠ NC¹, fixed depth and log precision.
   - **This review measured it** (§6.2):
     - Quaternion lanes learned A5 and extrapolated to 16× the training length.
     - Snapped to 2I and served as a 120-state integer automaton (one byte of state and two table reads per token), they stayed at 100% to length 4,096, where the float32 originals drifted to 0.62–0.74.
     - Every commutative model stayed at chance.
   - **Honest limits:**
     - Two learned reflections (DeltaProduct-style) match the capability, so the quaternion/2I edge is *exact, cheap serving*, not unique power.
     - That training lands on 2I is mathematically forced: any exact A5 representation in SU(2) is conjugate to 2I.
     - S5 is provably out of reach for rotation-only lanes.
     - Learning is bimodal and fragile.
     - The repo's own near-identity parameterisation (q = normalize(e0 + 0.1·raw)) never learns it.
     - Natural-text perplexity barely moves.
6. **The correct theory (§8).**
   - **What geometry replaces.** The *state-transition* matrix (rotation times radial decay) and relative transport (a data-dependent, non-commutative "quaternionic RoPE").
   - **Attention and recall.** Their products can be designed away with codebook-keyed lookup-table scores and exact addressed memory, or kept as integer products, which requires the D0-b ruling in §11.2.
   - **Per-token projections** become low-bit additive maps.
   - **Knowledge storage stays in parameters,** at about 2 bits per parameter, measured with ≥8-bit weights.
   - **Energy floor.** It is set by the bytes of parameters touched per token. For models larger than the cache, *sparse access* is therefore the dominant energy lever. That conflicts with the owner's "no sparse routing", which is the key decision in §11.1.
7. **Next steps, in order (§9).**
   - **Hours to days:**
     - cool down the existing checkpoints;
     - remove the serving overheads;
     - measure the first joules with `macmon`;
     - add the missing *commutative* and *no-transport* control arms plus an A5 probe to the existing learner.
   - **Weeks:** choose between an *incremental* path (the D8 learner with a throughput fix and 2I tracking lanes) and a *parallel linear-recurrence re-base*, using a throughput benchmark and matched-token pilots with 3 seeds.
   - **Retire:**
     - local selector and admission tuning at T = 256;
     - zeta and prime mechanisms in predictive paths;
     - authored 32-prompt panels as architecture gates;
     - n = 1 comparisons.

## 1. What exists today (verified facts)

| Item | Fact | Label |
|---|---|---|
| Accepted model | One-layer GRU-like recurrence (d=256) with 64 quaternion lanes; one soft read head over all ≤255 earlier events, with age bias and a NoRead slot; pointer-sentinel copy mixture; tied 4,096-token BPE vocabulary; TinyStories data | Source joint_model.rs, config.rs |
| Size | 1,678,466 parameters; 1,048,576 (62.5%) embedding; **629,890 non-embedding** | Derived, 3 agents agree |
| Quality (full 249,856-target development population) | Quaternion 2.117, ordinary 2.092. Same population, 7.16M transformer #1014 after 30M tokens: 2.127 (sealed). After 150M tokens (#1017): about 1.57. Comparison tail: quaternion 2.110, control 2.085, 5-gram plus cache 2.392 | Source |
| Training | Rust/Candle F32; constant LR 1e-3 with no cooldown; about 30M target visits per arm; 1,362 tokens/s per arm, about 14 GFLOP/s | Source, Derived |
| Serving | Standalone integer session; ≤4-bit weights; bit-exact with training evaluation; about 97k software-emulated products per token; reads all 1,672,704 four-bit weights every token (62.7% of reads are the output head) | Source, Measured |
| Tests | Integer 24/24, tokenizer 3/3, training 68/68 on x86; no correctness bug found. The 2I group-table property was confirmed by independent rebuilds; the repo's own core `group_table` test was not run here (the core compile took more than 20 minutes) | Measured |
| Geometry in the forward path | The quaternion lane rotation only | Source, grep |
| Code | Active path 21,841 lines, 3.3% of the 666,682 in `crates/`; `native_geometric` (186k lines) is unused by the D8 path | Measured |
| Energy | Never measured | Source |

## 2. The lineage of the idea, and what the evidence says at each step

### 2.1 "Like TurboQuant, but preserve the radial direction, in 4-D instead of 2-D"

**Premise correction.** TurboQuant, PolarQuant and QJL do *not* discard the radius. Each stores the vector norm, either in floating point or recursively. What they eliminate is the per-block normalisation constants: scale and zero point (Literature 2504.19874, 2502.02617, 2406.03482).

For a 4-block, PolarQuant's angles are exactly Hopf coordinates (math report §2.7). Two parts of the original idea remain distinct:
- (a) *quantizing* the gain (radius) separately from the shape;
- (b) using 4-D blocks.

**It has now been published.** HQMQ (MIT/IBM, arXiv 2605.27646, May 2026) does this for KV caches:
- Each 4-element chunk is treated as a quaternion.
- The radius is quantized to 3–6 bits; at 1 bit or less the model breaks.
- The direction is a product q_p·q_s, with q_p from the 24-element Hurwitz group (the 24-cell) and q_s from random unit quaternions.

HQMQ's results:
- About 5 bits: within 0.02–0.03 perplexity of fp16.
- About 3 bits, on Llama-3-8B: it beats a TurboQuant-style spherical-plus-JL baseline (+0.745 vs +1.118 perplexity at 3.04 vs 3.15 bits).
- Its E8 extension underperformed the 24-cell.

IsoQuant (2603.28430) uses SO(4) quaternion-pair rotations as preconditioning. The owner's instinct was technically sound, but the priority is gone: **cite HQMQ and IsoQuant; do not claim the idea.**

**Rate-distortion reality.** At 2 bits per dimension on a Gaussian source (measured by the literature agent, MSE per dimension):

| Quantizer | MSE |
|---|---|
| Scalar Lloyd-Max | 0.117 |
| 2-D polar | 0.119 |
| 600-cell gain–shape | 0.107 |
| Best learned 4-D shape code | 0.106 |
| Unconstrained 4-D VQ | 0.098 |
| E8P (QTIP paper) | 0.089 |
| 256-D trellis (QTIP paper) | 0.069 |
| Shannon bound | 0.0625 |

At 3 bits per dimension, a fixed 120-point polytope (0.071) is worse than scalar quantization (0.035). Storing an fp16 radius per 4-D block costs 5.73 bits per dimension and is still worse than 3-bit scalar quantization. Give the gain about 1/n of the bits: roughly a quarter for 4-D blocks (LLVQ 2603.11021).

The 600-cell is the best 120-point S³ code tested, 0.55 dB better than a Hopf/PolarQuant grid (math e4). Lattice gains over scalar quantization are capped at 1.53 dB. See §6.1 for the quantization agent's independent measurements.

**Where the idea belongs:**
- compressing event and KV memory with a *quantized* gain (HQMQ-style), where the norm matters (physics report);
- as a key codebook for lookup-table attention scores (§3.6);
- in the *dynamics* role developed in §8.

### 2.2 "Prime least-energy Riemann(ian) manifold routing"

The math report (§2.6) makes the idea precise:
- **The least-energy path on a Riemannian manifold is a geodesic.** So "least-energy selection" among candidates is argmin of a distance, and on S³ or H⁴ that is argmax of an inner product, i.e. *hard attention or nearest-neighbour search*. The semantics live entirely in the learned embedding, which the prime and zeta assignments do not supply.
- **The prime component is an enumeration hash.** It provides identity, not distance.
- **The implemented energy has only 4–9 distinct values.** It is the 2I word metric, which explains the recorded ties.
- **(shell, sector) addressing is LSH/hash routing.** It preserves locality only if the embedding does, and prime ids do not.
- **The router era's own experiment found the radius contributed nothing to routing.** INC-0168: "purely angular … no radial contribution identified".

Supporting evidence:
- **Hyperbolic LLMs have weak evidence.** HELM (2505.24722) at 1B parameters is near chance on MMLU and CommonsenseQA, and its best variant is MoE.
- **Zeta phases are decorative, measured** (§4.2). As token codes they collide far more than a hash does. As RoPE frequencies they are no better than random sets, and they alias worse than 99% of random sets at N=128.
- **The repo's own zeta ablation is weak evidence either way** (native_geometric_recovery_973.md:309-316). It was run without retraining, and the Wilson intervals overlap. Accuracies, out of 96:

  | Configuration | Accuracy |
  |---|---|
  | Full | 86 |
  | Zeta disabled | 91 |
  | H4 disabled | 82 |
  | Geometry disabled | 66 |

- **Primes.** As identity, primes duplicate what content hashing does better. UOR's own `uor-addr` (SHA-256 κ-labels over canonical forms) is the right identity and deduplication layer. As *geometry*, primes have one rigorous role, in the arithmetic of the icosian quaternion algebra: prime-norm "golden gates" (1704.02106). At practical sizes these showed no covering advantage (§6.4).

### 2.3 "Then I realized I could store and recall"

This is the most robust positive result in the programme:
- KVAR's gated, token-addressed overwrite store reaches 0.82 and 0.72 held-out accuracy (two seeds) on a keyed-rebinding panel. Chance is 1/64, and the order-2 count control scored 0/204. The plain recurrence is at chance (kvar-recall-result-2026-09-24.md).
- Exact versioned memory supplies recall that a finite recurrent state cannot supply (the Zoology/MQAR trade-off).

It is **not integrated** with the model being trained and served now. Deterministic-address retrieval (kNN-LM, RETRO, Engram-style hashed n-grams, exact pointer memory) is compatible with "no MoE / no sparse routing". Learned top-k parameter selection (PKM, memory layers) is not (literature report F5.2).

### 2.4 "Replace the wasteful matrix multiplication in the serving runtime with geometric intelligence"

- **Physics (§5).** The waste is bytes and instructions, not multiplier circuits.
- **The current path.** The serving path satisfies "no multiplier instruction" by computing activation products in software shift-and-add loops over checked `u128` (crates/uor-r4-integer/src/math.rs:59-83). This adds instructions: 4.4× slower on x86 than hardware multiply, with bit-identical output.
- **What geometry *can* replace (§8).** The state-transition matrix and relative transport, exactly and without multiplication. Attention and recall products can be designed away with codebook-keyed lookup-table scores and exact memory.
- **What it cannot replace.** Knowledge storage.

### 2.5 "Currently we are still working on attention, and have drifted from pure geometry"

The current learner is a GRU-like dense recurrence with per-lane quaternion transport. Each step it performs:
- one soft read over *all* earlier events, with a learned age bias and a NoRead slot;
- a pointer-copy gate (joint_model.rs; config.rs).

Disabling read and copy together costs +0.48 nats. At T=256 the read is about 5% of serving compute (arch F1, F7), so it is a cheap, working, integer-served component worth keeping.

The quaternion arm (2.110) is slightly *worse* than its control (2.085). That control is itself a quaternion map (§3.5), and the comparison is uninformative about geometry.

MatMul-free LM found that ternary Q/K attention *failed to converge*, and replaced attention with an element-wise GRU token mixer (2406.02528). The project should not rediscover this.

## 3. Computer-science stress test

### 3.1 What the accepted model actually is

It is a one-layer recurrent language model (arch report F1; verify F6/F7):

- **State.** A GRU-like candidate/gate update on a 256-wide state, with dense input and state maps W_in, W_s ∈ ℝ^{768×256}.
- **Transport.** A per-4-lane unit-quaternion left rotation.
- **Read.** One soft read head over *all* earlier events in the 256-token window, with a learned age bias and a NoRead null slot.
- **Update.** A scalar update gate.
- **Output.** A pointer-sentinel copy mixture over a tied 4,096-token vocabulary.

| Quantity | Value |
|---|---|
| Parameters | 1,678,466, of which 1,048,576 (62.5%) are the tied embedding and **629,890 are non-embedding** |
| Forward cost | 3.43M FLOPs per token |
| Share of serving MACs | Output head 67%; read at t=255 5%; transport 0.1% |

The copy/NoRead design is the pointer-sentinel mixture of Merity et al. (1609.07843). It is sound, but not new.

**Correctness.** The code builds and its tests pass on x86: integer 24/24, tokenizer 3/3, training 68/68. The integer runtime is bit-identical to an independent hardware-arithmetic re-implementation over 256 steps × 3 reps × 2 modes × 2 arms (verify F1, F4, F6). Every item reviewed was correct:
- causality and write order;
- pointer mixture normalisation;
- RMS algebra and the age index;
- the Hamilton product;
- the Householder pair;
- STE, AdaRound and AdamW;
- shard averaging.

**The engineering is careful and correct. What limits the project is scale and design choices, not bugs.**

### 3.2 Why the language is poor: exposure and capacity, not the architecture

Comparison on the same population (the full 249,856-target development set, arch F1/F4):

| Model | Non-embedding parameters | Training tokens | NLL (nats/token) |
|---|---:|---:|---:|
| Current learners (ordinary / quaternion) | 0.63M | 30M | 2.092 / 2.117 |
| #1014 transformer, 6 layers × 288 (sealed) | 5.98M | 30M | 2.127 |
| #1017 (#1014 continued) | 5.98M | 150M | about 1.57 |
| llama2.c stories15M (same shape, far longer training, 32K vocabulary) | ≈6M | ≫150M | ≈1.07 |

At equal tokens, a transformer with 9.5× the non-embedding parameters did **no better** than the current recurrent learners.

Extrapolating the learners' own log-linear slope (about 0.15–0.16 nats per e-fold of tokens) gives about 1.84 at 150M tokens. **Exposure therefore explains roughly half the gap to #1017; the rest is capacity and architecture** (Derived; red-team correction).

**Training was also never annealed.** It ran at a constant learning rate of 1e-3 with no warmup and no cooldown, while tune NLL was still falling fast (arch F2).

TinyStories coherence emerges at width ≥128, with depth, at roughly 8–30M parameters (Literature 2305.07759). "Semantically unreliable" text at 0.6M non-embedding parameters is therefore expected of *any* architecture; it is not evidence about geometry.

### 3.3 Training throughput is the binding engineering constraint

| Run | Tokens/s | Achieved compute |
|---|---:|---:|
| Current learner, per arm (M1 CPU) | 1,362 | ≈14 GFLOP/s |
| Current learner on Metal | slower than CPU | — |
| Transformer #1014 on MPS | 7,108 | ≈0.32 TFLOP/s (12.5% of M1 GPU peak; about 12× the machine-level compute) |

The cause is a strictly sequential, nonlinear, per-step unroll: about 120 tiny tensor ops per timestep, plus O(T²) history concatenation (arch F3).

- At the current rate, one M1-week buys about 0.26B tokens for a 10M-parameter model.
- A parallelizable design *assumed* to reach 0.3–0.6 TFLOP/s would buy 2.75–5.5B tokens. That rate is taken from #1014's measured MPS efficiency and is **not yet measured for the Rust/Candle path**.

### 3.4 The serving path satisfies the letter of D0-b at a large cost

- **Emulated arithmetic.** The integer session computes about 97,000 activation×activation products and about 9,500 long divisions per full-context token. These run as software shift-and-add and restoring-division loops over checked `u128` (math.rs:59-146).
  - A section profile on x86 puts about 79% of step time in these loops. The attention value mix takes 2.29 ms and the vocabulary mixture 2.08 ms.
  - The 1e-8 uniform-mixture floor alone costs 4,096 software multiplies and 4,096 long divisions per token, for an effect of 1e-8. It can be applied analytically at selection time (verify report).
- **Speed.**
  - On x86, the same arithmetic with the hardware multiplier is **4.4× faster and bit-identical**: 6.73 vs 1.53 ms/step (verify F4).
  - Per product, the software loop is 45–310× slower than a hardware multiply, depending on harness and operand size (arch, physics and audit micro-benchmarks).
  - The project's own M1 notes put the integer step at about 6.5× the F32 step. This is a cross-run indication, not a controlled measurement.
- **The "signed4 product table".** It is a per-activation 16-entry multiples table indexed by the weight nibble. It is D0-b-legal, but it replaces exactly one multiply per lookup. It does not amortise across activations the way T-MAC's lookup tables do, and on x86 it is no faster than a hardware multiply-accumulate (verify F2, F4).
- **Access.** Every token reads all 1,672,704 four-bit weights, and 62.7% of those reads are the tied output embedding. As implemented this is 3.36 MB/token, because codes are held as `i16`; packed, it would be 0.85 MB. The model fits in the M1 L2 cache, so instructions, not DRAM, dominate (verify F5).

### 3.5 What the quaternion-versus-Householder comparison can and cannot tell us

0. **The "ordinary" control is itself a quaternion map.**
   - H(e0)x = −x̄ and H(v)y = −v ȳ v, so H(v)H(e0)x = **v·x·v**. This was verified numerically to 1.3e-15 (state-tracking report).
   - D8 therefore compared two *non-commutative quaternion transports*, never geometry against non-geometry.
   - The informative controls have never been run: a *commutative* (diagonal or complex) arm and a *no-transport* arm. config.rs:12-15 allows only the two existing variants.
1. **Seeds.** There is one seed per arm. A 0.025-nat gap is indistinguishable from seed noise without more seeds.
2. **No expressive power added.** Inside a nonlinear GRU whose dense W_s already mixes the state, a lane rotation adds no expressive power.
3. **The geometry consumes a matmul.** Its parameters come from a slice of the dense 3d×d projection (audit §4).
4. **The transport is trapped in a commutative regime.** Its near-identity parameterisation q = normalize(e0 + 0.1·raw) keeps rotations small, and small rotations commute to first order. The state-tracking agent measured that this exact form *never* learns A5, while q = normalize(raw) does (§6.2).

The test was ill-posed. Its result is not a verdict on geometry.

### 3.6 Attention under the serving constraints

- **Recall needs state that grows with content.** Zoology/MQAR, Based and "Repeat After Me" show that fixed-state recurrences cannot do associative recall beyond their state size. One or two local attention layers, or exact pointer memory, close most of that gap (arch F7).
- **The existing integer soft read is a first-class option.** It costs about 5% of compute at T=256. Removing read and copy together costs +0.48 nats. Bounded-admission work at this length optimises the wrong 5%.
- **The owner's polar codebook works for attention keys** (arch F7, measured):
  - A 600-cell direction plus a 4-bit log-radius at 2.73 bits/dim gives top-1 retrieval 1.000 and KL 0.032.
  - That is comparable to int3 scalar quantization (KL 0.023) and better than a random 120-point code (KL 0.048).
  - 600-cell coordinates lie in ½·ℤ[φ], so each score becomes about 16 table reads and adds per key.
  - This is how D0-b can be satisfied *by design* rather than by emulation (§11.2).
- **Ternary Q/K attention failed to converge in MatMul-free LM** (2406.02528).

### 3.7 Code base

- **Size.** The active path is 21,841 lines, 3.3% of the 666,682 in `crates/`. From the 410k-line core it imports only a report re-export, the tokenizer and `answer_oracle`.
- **`native_geometric`.** At 186k lines it is unused by the D8 path, yet AGENTS.md:122 still calls it "the current model implementation" (audit §3d).
- **CI.** The required "fmt / clippy / tests" check is an echo that runs in 3 seconds (.github/workflows/ci.yml:107-113). This is declared policy (AGENTS.md:146) and predates Sep 8. Real assurance comes from local runs and principal reviews.

## 4. Mathematics stress test

Sources: the mathematics report (nine numerical experiments, e1–e9), the state-tracking report, and the literature and architecture reports.

### 4.1 Verdict on each mathematical claim

| Claim | Verdict | Evidence |
|---|---|---|
| The quaternion arm is a left Hamilton product and preserves norm | **Correct.** Quantizing without renormalizing is harmless at T=256 (norm-ratio std 6.3e-4) | Source; verify F6; e2 |
| The Householder pair is the "ordinary", non-geometric control | **Wrong framing.** H(v)H(e0)x = v·x·v is itself a non-commutative quaternion map. Its compositions reach SO(4) | Derived; verified numerically to 1.3e-15 |
| The 120 H4 roots form the group 2I, with an exact table | **Correct.** Closed; all 1,728,000 triples associative; order census matches 2I | e1; independent rebuild |
| 2I is non-solvable (A5 ≅ 2I/±1; 2I ≅ SL(2,5)) | **Correct** | e1 |
| "E8 = H4 ⊕ φH4" via icosians | **Correct.** The icosians form an even unimodular rank-8 lattice with 240 roots. formal_vocabulary.md calls this an "Assumption"; it is an isometry theorem. Irrelevant to model capacity | e1 |
| `learner/embedding.rs` implements "H4 ⊕ φH4" | **Mislabelled.** The companion is an independent learned quaternion with no Galois coupling, which contradicts AGENTS.md | Source |
| Exact ℤ[φ] replaces multipliers | **Correct only for isometries.** A 2I rotation costs ≤24 add/sub plus 8 shifts. Any forgetting gate forces coefficient growth of 0.694 bits/step (no-go theorem, §8.4) | e1; state-tracking kernel check |
| Non-commutative group state gives state tracking beyond transformers and diagonal SSMs | **Correct as a conditional theorem** (Barrington; Merrill et al., assuming TC⁰≠NC¹, fixed depth, log precision). **Not unique**: two learned reflections, PaTH and permutation gathers match it. **Learnable, but fragile** (§6.2) | Literature; e6; state-tracking |
| Quaternion lanes can track symmetric groups | **Only A5 among non-abelian simple groups.** S5 cannot be tracked by any number of rotation-only 4-D lanes; reflections are needed. Measured S5 accuracy equals the parity-only level | Derived; state-tracking |
| Zeta phases are useful multiscale coordinates | **Decorative** (§4.2) | e3, e3b |
| RH is needed | **No.** The code uses 512 zeros verified against mpmath to 4e-10. The Lean files take `hRH` as a hypothesis and contain no `sorry` | e3; source |
| "Prime least-energy Riemann manifold routing" | **Not one well-defined object** (§2.2) | Derived |
| Hopf sectors and "phase transport" | **The formula is correct; the transport claim is overstated.** The code evaluates the Hopf connection pointwise, which is a gauge choice rather than parallel transport. Cells degenerate near the singular circles | Derived |
| The H⁴ sinh³r shell law balances routing capacity | **The formula is correct; the capacity claim is overstated.** Load depends on the embedding's density | Derived |
| "H⁴ × H⁴ coupled field" | **Undefined as stated.** The name also collides with the Coxeter group H4 | Derived |
| 600-cell optimality | **The theorem is correct but tangential** (it concerns energy minimization, not MSE). Measured, the 600-cell *is* the best 120-point S³ code tested: MSE 0.0705, vs 0.0718 for a Lloyd code, 0.0800 for a Hopf grid and 0.105 for random | e4 |
| Cayley–Dickson "endomorphic routing" | **Mislabelled.** It is a multiplicative hash plus cyclic minors | e8 |

### 4.2 Zeta-zero phases, measured

- **As token-identity codes (the repo's use).** With 8 channels, 36.8% of byte tokens and 95.6% of 4,096 tokens have a near-twin (cosine > 0.99). A hash gives 0%. **A zeta code is a worse identity code than a hash.**
- **As RoPE-style frequencies.** At N=32 they sit at the 28th percentile of random sets. At N=128 they alias *worse than 99% of random sets* (quasi-linear spacing).
- **Discrepancy.** No better than random: 0.036–0.080 vs 0.039. That is 10–20× worse than the golden Weyl sequence (0.0036), which is the provably optimal U(1) covering generator.
- **At log-primes.** They carry Landau's bias (mean cosine −0.12 to −0.19, matching theory), so they encode *primality*, not language.

### 4.3 Where exact geometry is forced, and where it is merely convenient

- **Classification.** The finite subgroups of SU(2) are the cyclic groups, the binary dihedral groups, 2T, 2O and 2I.
  - The binary dihedral groups are non-abelian and arbitrarily fine.
  - **Non-solvable** structure in a quaternion lane forces 2I.
- **Consequence for training.** When training converges to an exact A5 tracker in an SU(2) lane, the learned elements *must* be conjugate to 2I. The measured "rediscovery" of the 600-cell (§6.2) is therefore a confirmation that the solution was found and snaps cleanly. It is not a surprise.
- **Closure is not exact before snapping.** At tolerances tighter than 0.05, the learned closure is not finite (red-team check). Snapping is what makes the lane exact.

### 4.4 The mathematically strongest version of the project

**(A) Finite non-solvable group transport as data-dependent multiplicative state and relative position.**

Design and serving:
- Each lane's action a_t ∈ 2I is chosen per token.
- Serving costs one byte-table read per lane per step, plus one more for relative transport G_iG_j⁻¹.
- Attention logits take the PaTH form ⟨G_j k_j, G_i q_i⟩ with multiplier-free rotation.

Training: a continuous SU(2) relaxation with a *non*-near-identity parameterisation and a length curriculum, followed by snapping.

Guarantees:
- Exact for unbounded length (e9; state-tracking automaton to length 4,096).
- NC¹-complete tracking in one layer, under the stated assumptions.

Its edge over DeltaProduct, PaTH or gathers is **serving cost and exactness, not capability**.

**(B) E8/icosian and D4 lattice codebooks for 2–4-bit weights and KV.**
- The gains are real but bounded: E8 is 0.66 dB better than the cubic lattice.
- Decoding is multiplier-free (the QuIP# E8P precedent).
- This is the honest home of "E8 = H4 ⊕ φH4" and of the owner's 4-D polar idea.

**(C) Exact identity and addressing (UOR).** This is engineering rather than geometry, and it is real.

**Decorative for a language model:**
- zeta phases;
- primes as semantics;
- Hopf sectors and "phase transport";
- sinh³ shells;
- H⁴×H⁴;
- "least energy" as physics;
- the Cayley–Dickson routing;
- the Galois companion.

Keep these as identity or provenance only, or drop them.

**What the mathematics cannot supply is general prose quality.** Language modelling is density estimation, and its binding constraints are parameter and data scale and learnability.

## 5. Physics stress test

### 5.1 Where the energy of a token goes

At batch-1 decoding, energy goes to **moving bits** and to **instruction overhead** on programmable cores, not to multiplier circuits. Reference figures:
- DRAM costs about 5–20 pJ/bit. The all-in figure for the M1, calibrated from AnandTech measurements, is at least about 9 pJ/bit.
- At 45 nm, a CPU instruction carries about 70 pJ of overhead, against about 3 pJ for a 32-bit integer multiply (Horowitz, ISSCC 2014).

The physics agent's model, at a DRAM cost of 10 pJ/bit (physics report F2):

| Model (M1) | Bytes/token | DRAM energy per token | Saving from removing all multiplies |
|---|---:|---:|---:|
| 1B, fp16 | 2.0 GB | 160 mJ | — |
| 1B, int4 | 562 MB | 45 mJ | 0.02% |
| 1B, ternary | 209 MB | 16.7 mJ | 0.36% |
| 1B, int4, 10% of weights touched | 56 MB | 4.5 mJ | 0.02% |
| 7B, int4 | 3.94 GB | 315 mJ | 0.02% |

In the DRAM-bound configurations, multipliers account for at most about 0.4% of the energy floor; for a cache-resident ternary model the figure is at most 3.4%. **Bit width, bytes touched per token and instruction count decide energy.**

Measured evidence has two parts:
- **Lookup-table kernels.** Multiplier-free LUT kernels *do* save energy when they remove instructions. T-MAC ran the *same* 4-bit and 2-bit models as llama.cpp and cut measured energy 21–61% by replacing dequantize-and-multiply work with table lookups (Literature 2407.00088 §5.4).
- **Fewer bytes.** Ternary models save more through fewer bytes. bitnet.cpp reported 55–70% on an M2 Ultra; its measurement method is not described (Literature 2410.16144).

The multiplier-free half of D0-b is sound when it takes T-MAC's form, a table lookup replacing a multiply. It is counterproductive when it *adds* instructions, as the current software emulation does.

### 5.2 The current serving path is about 10³× above its own floor

The 1.68M-parameter model fits in the M1's 12 MiB L2 cache, so steady-state DRAM traffic is near zero. Yet its 3.695 ms per call corresponds to an estimated **11–23 mJ/token**. That is roughly the DRAM-floor energy of a well-implemented 125–300M-parameter 4-bit model (physics report, Derived).

The cause is emulated arithmetic (§3.4):
- about 79% of step time is spent in software multiply and divide on x86;
- per product, software emulation is 45–310× slower than the hardware instruction.

On a general-purpose CPU, the multiplier circuit is present and powered whether or not it is used. Replacing one multiply instruction with dozens of instructions therefore very likely *raises* energy. This is Derived; it has not been measured on the M1.

### 5.3 "Least energy", "Hamiltonian", "phase transport", "field"

These terms are optimisation vocabulary or metaphor. The repository's current plan already says so (principal-attention-plan §4).
- "Least-energy routing or read" is argmin of a learned score, which is hard attention (§2.2).
- A rotation is multiply-accumulates unless it is drawn from a finite codebook, and then its saving is bytes and instructions.
- The router era's claims of 10–1000× lower energy per operation and 10–100× hardware gains were never measured.

**No joule has ever been measured in this repository.** The one earlier "measurement" was a hardcoded 3,500 mW constant, withdrawn on Sep 8 (recovery-2026-09-08.md).

Physically grounded ideas that do fit the constraints (physics report §3):
- **Modern Hopfield theory for the "least-energy read".** Attention is one energy-minimization step, and binary keys can be scored by XOR plus popcount.
- **Norm-preserving or oscillatory recurrent cores** (LinOSS, coRNN), with learned rather than fixed frequencies.
- **Reversible training.** It saves RAM on a 16 GB machine; it does not save serving energy.
- **Efficiency-core execution under DVFS.**

### 5.4 Zeta zeros

There is no physical or information-theoretic reason for zeta-zero phases to help a language model.
- **Spacing.** Their GUE-like level repulsion makes them a well-spread set of incommensurate frequencies. Log-spaced, low-discrepancy and learned sets have the same property.
- **Covering.** The golden-ratio rotation is the provably optimal U(1) covering generator.
- **Measured** (§4.2): as token codes they collide far more than a hash does; as frequencies they are no better than random; and at log-primes they encode primality.
- **Current use.** The current model contains no zeta component.

### 5.5 What "freeing computing from the GPU and overheating" physically requires

On an M1 the GPU is not the villain. Batch-1 decoding is bandwidth-bound or overhead-bound on either unit, and heat is simply average power. Four things are needed:

1. **Fewer bits per token.** Low-bit weights; compressed KV and event memory with a quantized norm, which is where the owner's "preserve the radius" instinct physically belongs; cache residency; and, for models larger than cache, sparse or blocked access.
2. **Kernels that finish sooner with fewer instructions.** T-MAC-class lookup-table kernels, not emulation.
3. **Fewer tokens per useful answer.** Quality is an energy lever.
4. **A measurement at matched quality.**

The physics report specifies a measurement protocol (§5):
- **Instruments.** `macmon` reads IOReport power without sudo, and so *may* remove the repository's recorded blockers; support on the owner's macOS 26 is unverified. Ground truth comes from a wall meter or battery integration.
- **Controls.** Thermal pre-heat, idle subtraction and interleaved ABAB runs.
- **Metrics.** Joules per output byte and per token, plotted against bits-per-byte on the same text.
- **Baselines.** llama.cpp, bitnet.cpp and MLX on the same machine.

Until that measurement exists, every energy statement is a hypothesis.

## 6. Experiments run for this review

All runs were small, CPU-only and on a shared x86 box, and none is a language-quality claim about the project's model. Their scripts are listed in Appendix B.

### 6.1 The original idea, measured at matched bits (quantization report)

Setup: i.i.d. Gaussian vectors, d=64, with an 8-bit norm counted for every method. Also heavy-tailed, outlier-channel and mean-shifted variants.

| Bits/dim | 1.5 | 2 | 2.5 | 3 | 4 |
|---|---:|---:|---:|---:|---:|
| 4-D gain–shape with H4-derived codes vs TurboQuant-MSE | +0.19 dB | +0.30 | +0.24 | +0.64 | +0.93 |
| D4 lattice (Voronoi cell = 24-cell) vs TurboQuant-MSE | ≈ +0.6–0.7 dB across rates | | | | |
| E8 lattice vs TurboQuant-MSE | ≈ +0.85–1.22 dB across rates | | | | |

- **The H4 structure itself does not help.** Spherical k-means codes of the same size match the 600-cell/2I codes (relative MSE 0.1288 vs 0.1274 at 2 bits). Random codes are 0.8–1.3 dB worse.
- **Keeping the radius** is essential *inside* a 4-D block: dropping it costs 2–4 dB. But coding radius and direction as separate factors costs 0.3–0.5 dB against an unconstrained 4-D code. The literature agent's fixed 120-point code saturates at 3 bits/dim (§2.1).
- **The data distribution matters more than the codebook.**
  - Heavy-tailed coordinates need a random rotation; without it every method loses 1–4 dB.
  - Heavy-tailed norms need a per-vector norm.
  - Fixed outlier channels favour per-channel calibration.
- **Serving.** Inner products between 2I codewords take exactly 9 values in ½ℤ[φ]. An integer ℤ[φ] lookup-table scorer matched the float path (recall@10 0.483 vs 0.482). Quantizing the *query* with the same ~2-bit code costs about 0.12 recall@10, and the same holds for scalar, k-means and E8 codes. So keep the query at full precision, with per-query tables built from shift-add constants.
- **Best project use.**
  - Keys and values of the attention and event-memory read, scored by table lookup. This replaces the 64 software multiplies per key in the current integer read (model.rs:322-329).
  - For weights, E8 (icosian lattice) codewords up to about 2.2 bits/dim are exactly signed-4 integers, so the existing kernel is reusable.
  - Do not quantize the recurrent state at every step.

**Verdict.** A sound engineering component with about 1 dB of headroom over scalar quantization. It is not, by itself, a breakthrough, and its novelty is low: HQMQ, PolarQuant, FibQuant and Block-Sphere Quantization precede it.

### 6.2 State tracking with geometric lanes (state-tracking report; math report E6)

**Setup.** Word problems with a target at every position, trained on lengths ≤32 with a curriculum. All models share a state size (D=32) and an MLP readout.

**A5, three generators, tested to length 512** (chance 0.017):

| Model | Runs | Accuracy at L=512 | Exact at 512 |
|---|---|---:|---|
| Quaternion lanes, free parameterisation (8 lanes, 128 recurrent parameters) | 5 seeds | 0.61 ± 0.48 | **3/5 (5/7 including the learning-rate sweep)**; bimodal |
| Repo form q = normalize(e0 + 0.1·raw) | 3 learning rates, plus 3× steps | ≤0.017 | 0/4 |
| Householder pair H(v)H(e0) = v·x·v, free | 3 seeds | 0.71 ± 0.20 | 0/3 (best 0.987) |
| DeltaProduct-like (4 reflections, β<2) | 3 seeds | 0.017 (0.53 at L=128) | 0/5 |
| GRU (32 units) | 3 learning rates, plus 3× steps | ≤0.017 | 0/4 (a sample-efficiency result at this budget) |
| Diagonal (0,1), signed diagonal, unit-complex, LRU | 3 learning rates each | ≤0.017 | **0/12** |

**Full 60-letter alphabet** (math E6): four quaternion lanes with a curriculum reached 1.00 at 16× the training length, reproduced on a second seed. Two learned reflections matched them. The repo's Householder control could not represent the task, and diagonal lanes failed. One lane, or no curriculum, stayed at chance.

**Exact integer serving.** Converged lanes were snapped to 2I and served as a **120-state automaton**: state = T[state, token], class = C[state]. Per token that is one byte of state and two table reads, with no arithmetic.
- 4 of 5 successful runs scored 1.000 at every length up to 4,096; the fifth held a flat 0.984.
- The float32 originals of two of those runs drifted to **0.74 and 0.62** at lengths 2,048–4,096.
- On the abelian Z60 control, float rotation models collapsed by length 128. Snapped to exact characters, they held 1.000, 0.967 and 0.749 at every length.

**Limits.**
- Only 1–2 of 8 lanes converged; the rest behaved like lottery tickets.
- S5 reached only parity-level accuracy (0.016), as the theory predicts for rotation-only lanes.
- Everything here is toy scale: 3–60 generators, D = 32, about 1–2k steps.

**ℤ[φ] kernel** (verified on all 120 elements × 200 vectors): a 2I rotation of a doubled-coordinate icosian vector costs 24 add/sub plus 8 shifts. The 8 axis elements cost 0. Coefficients stayed bounded (max |coef| = 6 over 20,000 steps).

### 6.3 Real-text comparison of time-mixing mechanisms (lead; WikiText-2 bytes)

**Setup.** A 3-layer byte-level language model:
- d = 128, a GLU channel mixer and RMSNorm;
- about 0.56M parameters (0.76M for the GRU);
- 1,000 steps × 4,096 bytes = 4.1M training bytes;
- AdamW with warmup and cosine decay;
- evaluated on the first 256 KiB of WikiText-2 validation in windows of 512 bytes.

The linear recurrences are trained with an associative scan. Every time-mixer uses four d×d maps; the GRU has six.

| Time-mixer | Bits/byte, seed 0 | Notes |
|---|---:|---|
| Diagonal real decay (Mamba/minGRU-like, commutative) | **1.869** | Fastest to train |
| Quaternion rotation + radial decay (continuous) | 1.881 | Snapped to 2I after training: **2.060** |
| Quaternion, trained with straight-through snapping to 2I in every lane | 1.984 | All lanes forced onto the 120-element group |
| Complex (2-D rotation + decay, commutative) | [pending] | |
| Quaternion snapped to the integer small-rotation codebook | [pending] | |
| Softmax attention + RoPE (transformer-style reference only) | [pending] | |
| GRU (nonlinear, sequential; +35% parameters) | [pending] | |
| Second seeds: quaternion, diagonal | [pending] | |

Rows marked [pending] were still running when this document was first committed. They will be filled in by a follow-up commit on the same pull request.

**Reading (preliminary; one seed).**
- Non-commutative rotation gives **no** language-modelling advantage over diagonal decay at this scale, as the literature predicts.
- Forcing *all* lanes onto the coarse 2I group costs about 0.10 bits/byte.

Both support the lane-mixture design in §8.4: exact 2I only for tracking lanes, finer rounded lanes for content.

### 6.4 Golden-gate rotation codebooks (lead)

- **Level 1** (words c·T·c with T = (2+φ)i + j + k): exactly 3,600 rotations, matching the theory. After scaling by √(7+5φ), every coordinate is exactly in ½ℤ[φ].
- **Covering** (angle from Haar-random test rotations to the nearest codeword, SO(3)):
  - Mean 9.32° vs 8.83° for a random codebook of the same size; maximum 30.5° vs 22.8°.
  - A level-2 sample of 51,657 rotations is about equal to random: mean 3.65° vs 3.65°.
- **Near the identity** (T-count ≤1, 3,660 rotations): mean 13.4° vs 10.0° for random.
- **Reading.** Parzanchevski–Sarnak prove optimal *almost*-covering asymptotically and also predict rare big holes. At practical sizes these codebooks bring exact algebra and navigability, not better compression. The measured hole near the identity makes them a poor fit for recurrences, whose useful transitions are mostly small rotations.

## 7. Diagnosis: has the project fallen off course?

**Verdict: partly.** Since D8 (Sep 24), the engineering is back on course. The geometric thesis has drifted out of the model and has never received a well-posed test. The process has become a significant cost, although it has also caught real errors.

### 7.1 What is on course

- **D8 built the skeleton the project lacked for months:**
  - a real Rust autodiff trainer;
  - a matched control arm;
  - learned 4-bit rounding that passes its retention gates;
  - a standalone integer session whose outputs match training bit for bit (current-state.md).
- **Honesty.**
  - The README states that the best earlier artifact "contains no geometry".
  - Negative results are preserved.
  - The Sep 19, 23 and 24 self-reviews are sharp and mostly correct.
- **Correctness.**
  - No bug was found in the active path (§3.1).
  - Principal reviews caught real defects: the VSA role-binding bug, double gradient division, a bits-versus-nats confusion, the 64/256 context mismatch, and the wrong F32 backend.
  - The reader-utility review withdrew a false "representation limit" claim after finding a fit/serve threshold mismatch.
  - The precision factorial (183 s of model time in a 53-minute cycle) localized the quantization gap to parameters and led directly to the accepted learned-rounding artifact.

### 7.2 What has drifted

1. **Geometry has left the model.**
   - The active crates contain no prime, zeta, Hopf, hyperbolic, icosian, E8, VSA or lattice code. "H4" and "Z[phi]" appear only in strings that disclaim them.
   - The only geometric element is a quaternion lane rotation. It is fed by a dense matmul and compared against a control that is itself a quaternion map (audit §4; §3.5).
2. **The founding thesis never had a valid test.**
   - The router era's only language-model test (INC-0171) found Hopf routing equal to relabeled routing: 164.54 vs 164.41 perplexity. Its learned-gate baseline received no gradient.
   - INC-0168 found routing "purely angular … no radial contribution".
   - The router-era object was *hyperbolic* H⁴×H⁴, yet the project works with the *spherical* Coxeter group H4. No decision record of the substitution was found, and the project's Sep 19 review calls conflating the two "a category error".
   - D5's well-posed routing contest was never run; D9 deferred it again.
3. **Language quality regressed while mechanisms churned.**
   - On Aug 31, #1017 reached about 1.57 NLL. About 3.5 weeks followed on discrete mechanisms tested on authored fixtures, and those mechanisms are now parked.
   - The accepted model is at 2.09–2.12 on the same development population. At equal tokens it matches the transformer, so the regression reflects exposure and capacity, not a worse architecture (§3.2).
4. **The serving rule optimises a proxy.** "No multiplier instruction" is met by emulating multiplication in software, which adds instructions (§3.4, §5.2). Energy has never been measured.
5. **Effort went to the wrong bottleneck.** Bounded admission at T=256 targets about 5% of compute. The dominant costs are:
   - training exposure and throughput (§3.3);
   - the output head, at 67% of serving MACs;
   - emulated arithmetic.

### 7.3 Process cost, stated fairly

| Measure (Sep 19–25 unless noted) | Value |
|---|---|
| Owner decisions recorded | 11 in 7 days; the direction changed 3 times within 14 h on Sep 24 |
| Merged PRs | 107. 45% of titles say correct, repair, fix, recover, reconcile or restore. Median open-to-merge time is 1.8 minutes. Formal GitHub reviews were checked on only one PR (#1389), which had none; principal reviews happen as documents |
| Plan, step, review and result documents | About 185 documents, about 296k words. Mandatory agent reading is about 36k words |
| Last 15 PRs | 79.9% of changed lines were evidence JSON. Over 30 days: 1.68M JSON lines vs 491k Rust lines |
| D8 cycles | 8 of 13 spent ≤10% of wall time on model computation. Short cycles can still be high-value, as the precision factorial shows |
| Resource ledger | Grew 110.9 h in 143 calendar hours. Its limit was raised 109.8 h, as the owner's standing authorization permits (AGENTS.md:161) |
| Required CI | An echo that runs in 3 s. It is declared policy (AGENTS.md:146) and predates Sep 8 |
| Active code | 21,841 lines (3.3% of `crates/`). AGENTS.md:122 still points at the unused 186k-line `native_geometric` |

Sources: the audit report (measured via the GitHub API, git and wc) and red-team corrections.

**Context matters.** The tightening had a real cause. On Sep 8, agent work produced a fabricated "alpha qualification":
- M1 power and thermal figures were hardcoded constants;
- a security audit returned literal `true`;
- "coding success" meant producing more than zero tokens.

That episode is recorded in recovery-2026-09-08.md, and the governance that followed was a rational response. The question now is proportion. The ceremony is heavy, and the budget ceiling moves with use. Most of the assurance comes from local runs and principal reviews, not from the paperwork. A better health metric than model-time share is **decisions and direction changes per wall-hour, relative to new evidence**.

### 7.4 Root causes

1. **The goal contract rewards proxies.** It rewards the absence of an opcode and success on authored fixtures rather than joules and quality on the M1. Two constraints also conflict: D5 per-token sparsity versus "no sparse routing".
2. **Geometry has no falsifiable role.** It is kept "primary" by policy (AGENTS.md:32), and every matched test has been posed where theory predicts no difference: natural-text next-token loss, against a quaternion control.
3. **The model is too small and trains too slowly** to answer language questions: 0.63M non-embedding parameters, a sequential nonlinear unroll, about 14 GFLOP/s.
4. **Breadth over depth.** About 15 mechanism families were explored one after another, each shallowly and on authored panels, with direction resets every one to three days.
5. **Autonomous agents ran around the clock, faster than one person can steer.** Leadership passed between Codex, Antigravity, Zed/DeepSeek, "Sisyphus", Kimi and RDC. Owner corrections (the 64/256 mismatch, D9) arrived after the fact.

## 8. The unifying theory: what geometry can and cannot replace

This section is the lead reviewer's synthesis after red-team correction.

**Angle convention.** Rotation angles are given as **SO(3) rotation angles**. The corresponding quaternion (S³) angle is half the SO(3) angle.

### 8.1 One object runs through the whole lineage

Every nonzero x ∈ ℝ⁴ ≅ ℍ factors as x = r·u, with r = |x| > 0 and u ∈ S³ ≅ SU(2) (Derived). Each stage of the project used this factorisation, in a different role:

| Era | Role of (r, u) | Status |
|---|---|---|
| TurboQuant-like quantizer | **Compression.** Keep r, quantize u on S³ (gain–shape VQ) | Sound, but already published (HQMQ, 2026). 4-D is a weak rate-distortion point (§2.1, §6.1) |
| Prime/manifold routing | **Address.** (shell, sector) = (quantized r, quantized u) | Hash routing. In the router era's own test, r contributed nothing (INC-0168) |
| Store and recall | **Key.** Addresses select stored records | Real, measured asset (KVAR), not yet integrated |
| (missing) | **Dynamics.** (r, u) as an element of ℝ₊ × SU(2) acting on state by x ↦ r·(u ⊗ x) | Where geometry can replace the state-transition matmul |

Under the dynamics role, radii multiply (decay or gain) and rotations compose non-commutatively. The owner's instinct to "preserve the radial direction" fits *dynamics*: the radius is the retention factor and the direction is the content transform.

The complex analogue already exists: LRU's λ = r·e^{iθ} (Literature 2303.06349). The quaternion version replaces the commuting phase e^{iθ} with a non-commuting u.

### 8.2 What matmul does in a language model

Per token, any autoregressive language model performs four functions (Derived):

1. **State transport.** Fold the new token into a summary of the history.
2. **Content access.** Retrieve specific earlier information.
3. **Feature computation and stored knowledge.** The MLP role, where most parameters live.
4. **Readout.** Map the state to a distribution over the vocabulary.

What geometry can do for each role:

| Role | What geometry offers |
|---|---|
| 1. State transport | **It replaces the transition matrix.** The per-lane transition becomes rotation × radial decay, exact and multiplier-free for snapped rotations (§8.4). Input-to-transition maps (x_t → q_t, r_t, b_t) are per-token projections. At the first layer they can be token-indexed table rows; deeper, they are low-bit additive maps (D0-b). |
| 2. Content access | **It supplies relative transport:** a data-dependent, non-commutative "quaternionic RoPE" (§8.4). The recall products remain unless designed away. Linear-attention recall with a matrix state S_t = r_t S_{t−1} + k̃_t v_tᵀ needs activation×activation outer products and reads, about 0.6M per token at a 23M-parameter scale (red-team C1). Softmax reads need q·k products. These can be (a) **designed away**, with codebook-quantized keys whose scores become table lookups (§3.6; the 600-cell polar code measured about int3-quality at 2.73 bits/dim) and exact addressed memory; or (b) **kept** as integer products, which needs the D0-b ruling in §11.2. |
| 3. Feature computation | **Restructurable, not removable.** Knowledge must be stored in parameters: about 2 bits/parameter, measured with ≥8-bit weights. Post-training int4 falls to 0.7; the authors did not test QAT and suggest it "may be necessary" (Literature 2404.05405). Geometry can share parameters (quaternion/PHM layers have 4× fewer parameters) or *address* them (sparse access), but it cannot remove the bytes. |
| 4. Readout | **Restructurable:** a tied, factored or class-based exact softmax, or an integer lookup table. |

### 8.3 The physics floor

Energy per token ≈ (bits of parameters and state touched per token) × (energy to move a bit) + (instructions × energy per instruction). Arithmetic circuits are a small term (§5).

For models larger than cache, the dominant lever is to **touch fewer parameters per token**. Allen-Zhu and Li found that a 32-expert MoE touching 8.8% of its parameters lost only about 1.3× knowledge capacity. That experiment used a *learned* router (Literature 2404.05405), and a learned router is exactly what the owner excludes.

For cache-resident models (≲20M parameters at ternary), the levers are instruction count and kernel efficiency, and sparse access buys little (arch report). This is why D5 and "no sparse routing" need an explicit owner ruling, scoped by model size (§11.1).

### 8.4 The geometric primitive

**The recurrence** (Derived; Hypothesis as a language-model component):

  h_t^(ℓ) = r_t^(ℓ) · ( q_t^(ℓ) ⊗ h_{t−1}^(ℓ) ) + b_t^(ℓ),  with r ∈ (0,1] and q ∈ SU(2).

**Parallel trainability.**
- The pairs (A, b), with A = r·q, compose associatively: (A₂,b₂)∘(A₁,b₁) = (A₂A₁, A₂b₁ + b₂). An associative scan therefore enables chunk-parallel training.
- Frame form: with U_t = q_t U_{t−1}, the recurrence reduces to a scalar-decay recurrence in the rotated frame, so existing chunked kernels apply (arch F8).
- Accuracy checks: errors are ≤3e-14 in float64. In float32 the relative error is ≤9e-6, and the frame drifts by ≤6e-6 at T=16,384 (red-team check). Float32 training is fine.

**Expressivity.**
- 2I ≅ SL(2,5) is non-solvable; its quotient is A5. One lane with input-selected q ∈ 2I solves the A5 word problem, which is NC¹-complete by Barrington's theorem.
- Diagonal, complex-diagonal and non-gated SSMs, and log-precision constant-depth transformers, are in TC⁰ (Literature 2404.08819).
- This is a separation from **diagonal** recurrences, conditional on TC⁰≠NC¹.
- It is **not unique**. Two learned reflections (DeltaProduct), PaTH, RWKV-7 and permutation gathers have the same power, and so does the current nonlinear cell in principle.
- **Limits.**
  - Among non-abelian simple groups, rotation-only 4-D lanes can carry only A5. S5 needs reflections (state-tracking report).
  - The binary dihedral groups are non-abelian and arbitrarily fine, but solvable. So only *non-solvable* structure forces 2I (§4.3).

**Learnability, measured** (§6.2).
- Quaternion lanes learned A5 exactly and extrapolated 16×.
  - With a 3-generator alphabet and 8 lanes, 5 of 7 runs succeeded, and outcomes were bimodal.
  - With the full 60-letter alphabet, 4 lanes with a curriculum succeeded.
- Every commutative model stayed at chance.
- The repo's near-identity parameterisation q = normalize(e0 + 0.1·raw) never learned it. Rotations near the identity commute to first order.
- Learning is fragile: only 1–2 of 8 lanes converge. A single lane, or training without a curriculum, failed.

**Exact serving and where exactness ends.**
- *2I (60 rotations in SO(3); 120 unit quaternions).*
  - State can be a group index: 1 byte, with composition by a 120-row table.
  - Snapped lanes served this way stayed at 100% to length 4,096, where the float32 models drifted to 0.62–0.74 (§6.2).
  - Acting on a vector in doubled ℤ[φ] coordinates costs at most 24 add/sub and 8 shifts per 4-D rotation. Coefficients stay bounded.
  - The resolution is coarse: the smallest non-identity rotation is 72°.
- *Golden gates (Parzanchevski–Sarnak).* Words c₀Tc₁…Tc_t over the icosahedral group, with T = (2+φ)i + j + k of prime norm 7+5φ, give a prime-indexed hierarchy with asymptotically optimal *almost*-covering and efficient navigation (Literature 1704.02106).
  - The theorem also predicts rare "big holes". At practical sizes this review measured covering *no better than random codebooks*, plus a hole around the identity (§6.4).
  - After scaling by √(7+5φ), the coordinates are exact in ½ℤ[φ]. Composition is exact but the norm grows, so practical serving must renormalise and round.
  - Their value is algebraic, not better compression.
- *Integer small rotations.* q ∝ p = (2^k, a, b, c) with a, b, c ∈ {−1, 0, 1}: 157 rotations, the finest at 7.15°.
  - Applying p is shifts and adds, but the normaliser 1/|p| is almost never dyadic. Only 15 of 157 codewords have rational |p|, and only the 24 Hurwitz units have dyadic unit coordinates (red-team, Derived).
  - Each fine lane therefore needs a constant multiply by r/|p|. That takes about 6 shift-adds per coordinate for ≤1% drift at 4k tokens, i.e. about 36 operations per lane-step versus 16 hardware multiplies (red-team, Measured). With 3 terms, an isometric lane's norm drifted 827× by T = 4,096.
- **The no-go result** (math report §2.4).
  - Any forgetting factor 0 < |λ| < 1 in ℤ[φ] is an expansion in the Galois-conjugate embedding. With λ = φ⁻¹, exact coefficients grow by log₂φ ≈ 0.694 bits per step.
  - A dyadic factor 2^−k fails in the same way, because denominators grow.
  - Therefore **every decay or forget gate must round the state each step**, as any fixed-point recurrence does. Exactness is available only for pure group-index lanes (2I, and cyclic phase lanes).

**Design principle** (Hypothesis, grounded in Krohn–Rhodes). Every finite automaton decomposes into a cascade of simple groups and reset (flip-flop) components. A principled geometric state is therefore a **mixture of lane types**, each validated on its own probe before language:

| Lane type | Algebra | Role |
|---|---|---|
| Group-index lanes | Exact 2I | The non-abelian simple factor A5 |
| Phase lanes | Cyclic C_N, modular add | Abelian counters |
| Gated reset and decay lanes | Shift-subtract, rounded | Flip-flops and memory |
| Reflection lanes (optional) | O(4) or Householder | S_n |

**Separation.** Tracking lanes must be *separate* group-index lanes (h_t = q_t·h_{t−1}, with no additive input and a readout by index embedding). They cannot sit inside an additive matrix state. The "no additive input" evidence (2609.18966, a single-author, repository-pre-registered study) is itself described by its author as "a probe … not a training recipe".

### 8.5 What follows

The coherent geometric language model has four parts:
- **Geometric state transport:** exact group-index lanes, rounded decay lanes, and relative transport.
- **Low-bit channel mixing,** executed as lookup tables (T-MAC-class).
- **Contextual access** by exact addressed memory and/or codebook-keyed lookup-table attention.
- **An integer readout.**

Every role-1 and role-2 multiplication is then either replaced by geometry (exact rotations and codebook score tables) or reduced to low-bit table lookups. The remaining exception is linear-attention or softmax products over *unquantized* activations, which need the §11.2 ruling.

The design is transformerless, has no MoE and no learned router, and is dense in role 3. So D5 is unmet unless the owner allows deterministic addressing of role-3 parameters for models larger than cache (§11.1).

## 9. The recommended path

### 9.1 Redefine "breakthrough" so that it is achievable and verifiable

**B1: exact geometric state in a language model.** Build an integer-served, transformerless language model whose exact 2I group-index lanes track A5-type state (A5 word problems, bracket and permutation-parity structure, code and entity traces) at arbitrary length.

Gates:
- **Controls.** Compare against the *strongest* non-diagonal controls at equal serving cost: DeltaProduct with n_h = 2 and 4, and permutation-gather lanes. Diagonal controls, which provably fail, are included only as sanity checks.
- **Serving cost.** B1 counts only if the 2I lanes match the best control's tracking at lower *serving* cost (1-byte state, table reads, no drift).
- **Language modelling.** Held-out language modelling must be within 0.05 nats of the best transformerless control at equal parameters, tokens and seeds (3 or more).
- **Scope.** Drop S5 from the claim; rotation-only lanes provably cannot track it. Alternatively, add reflection lanes and gate them separately.

Precedent: learning continuously, snapping to 2I and serving as an integer automaton was not found in the literature. It is a Hypothesis on novelty (lit F3.2; state-tracking report).

**B2: measured efficiency.** Lower measured J/token (macmon plus a wall meter) on the owner's M1 than llama.cpp or bitnet.cpp at matched held-out quality, for a small model.

**B3: useful local model, conditional on owner rulings.** Convert an open model into the geometric recurrent form. The cost must be stated honestly.
- LoLCATs' cheap figure (40M tokens) applies only to hybrids that **keep a 64-token softmax window in every layer**. Without the window, Llama-3-8B stayed at chance MMLU. With it, MMLU still fell from 66.6 to 52.8 at 8B and from 31.9 to 27.3 at 1B (Literature 2410.10254).
- The fully attention-free conversion (MOHAWK, Phi-1.5 → Mamba-2) used **3B tokens** (2408.10189). For SmolLM2-135M/360M on an M1 at the assumed 0.3 TFLOP/s, that is on the order of months (Derived).
- SmolLM2's 49,152-token vocabulary alone gives a 47M-parameter output head at d = 960.
- B3 is therefore realistic only as a reduced-token experiment with an uncertain quality outcome, or with external compute, which current policy does not allow.

**Not credible with this team's M1 compute:**
- frontier capability trained from scratch;
- "geometry replaces *all* matmul", including knowledge storage;
- semantic value from zeta or prime structure.

### 9.2 Phase 0: this week (days; cheap; each changes a decision)

1. **Cool down the two step-7,324 checkpoints.** Use linear LR decay to 0 over about 1,000–1,500 steps.
   - *Decision it informs (D9):* whether further exposure is worth buying on this architecture.
   - A gain of −0.05 to −0.2 nats (Hypothesis) would say exposure and annealing, not mechanism, are the lever.
2. **Serving hygiene with bit-identical outputs.**
   - Apply the 1e-8 uniform floor analytically.
   - Pack 4-bit codes; store the KV cache as i16.
   - Build each product table once.
   - Hoist bias and age scaling out of the step.
   - *Decision:* none. This is pure cost removal.
3. **Measure the first joules.** Use macmon on the M1 to compare:
   - the F32 session;
   - the current integer session;
   - a hardware-multiply integer variant (the verification agent's scratch copy is bit-identical);
   - a small llama.cpp model.

   *Decision:* the §11.2 ruling on the multiplier, based on data rather than on this review's argument.
4. **The missing control arms and probe on the existing D8 learner.** No change to the language path:
   - (a) forced identity transport on the saved checkpoints, a free evaluation-only ablation;
   - (b) an A5 probe comparing the current quaternion arm, the Householder arm and a new *commutative* phase arm, at matched width.

   *Decision:* whether the current transport carries any capability.
5. **Owner rulings** on §11.1–§11.4.

### 9.3 Phase 1 (2–4 weeks): choose the base by measurement, then build the lane mixture

**Step 1 (days): benchmark two candidate bases for throughput.**
- *Incremental.* Keep the D8 learner. It already ties #1014 at equal tokens with 9.5× fewer non-embedding parameters. Raise its throughput with a larger batch, truncated or chunked BPTT, and no O(T²) history re-concatenation.
- *Re-base.* A parallel-scan / chunked linear recurrence in the quaternion frame form, with RMS-normed ternary GLU channel mixing. L ≈ 12, d ≈ 384, about 23M parameters at the S scale.
- Measure tokens/s and achieved FLOP/s for both in Rust/Candle-Metal at equal parameters. The re-base is justified only if it delivers at least 5–10× the tokens per M1-hour.

**Evidence so far on quality** (§6.3, WikiText-2 bytes, matched 0.56M parameters, one seed):
- diagonal-decay linear recurrence: 1.869 bits/byte;
- quaternion linear recurrence: 1.881;
- [GRU / complex / attention reference and second seeds: see §6.3].

The re-base's case rests on *throughput*, and on geometric lanes adding *capability* rather than perplexity. It does not rest on a language-modelling advantage.

**Step 2: add lane types to whichever base wins.**

| Lane type | Transition | Input path | Serving | Role |
|---|---|---|---|---|
| **Tracking** (separate from any matrix state) | Token-conditioned q_t = normalize(raw), not near-identity. Train continuously with a length curriculum, then snap to 2I | None. Input acts only through the choice of q_t; the readout is by group-index embedding | Exact index: 1 byte, table reads | A5-type tracking |
| **Phase** | Cyclic C_N | None | Modular add | Counters |
| **Memory and decay** | Fine or no rotation, rounded dyadic decay | Additive | Fixed-point with rounding (§8.4) | Content and memory |
| **Reflection** (optional) | Householder products | By design | Integer | S_n-type tracking |

Contextual access, in order of preference under D0-b as written:
1. The existing integer soft read (about 5% of compute at T=256), with **codebook-quantized keys**. The polar 600-cell key code makes scores table lookups.
2. An exact pointer and n-gram memory: the owner's store-and-recall.
3. Matrix-state linear attention, only if the §11.2 ruling permits integer activation products or low-bit k/v, which is untested.

**Data.**
- TinyStories is about 0.5–0.6B tokens (Hypothesis from its 2.23 GB), so 1–2B tokens means 2–4 epochs.
- Add open dialogue and code corpora for later conversation and coding: SmolTalk, Stack-Edu, FineMath, Cosmopedia.
- Logit distillation from #1017 is useful **early only**. Its 1.57 NLL is weaker than the 1.0–1.3 target.

**Seeds.** 3 per arm for every comparison that decides anything.

**Kill rule.** If 2I tracking lanes fail to beat DeltaProduct and gathers on serving cost at equal tracking, or if adding them costs more than 0.05 nats of language modelling:
- keep the architecture with the better ordinary lanes;
- retire the geometric claim from the serving path;
- keep geometry as codebook and addressing infrastructure.

**Budget caveat.** "20–40M parameters per M1-week" assumes 0.3–0.6 TFLOP/s. Step 1 must measure it.

### 9.4 Phase 2 (1–2 months): integer serving and the first honest efficiency result

Export the Phase-1 winner to integer serving:
- ternary or 4-bit channel maps executed as T-MAC-style lookup-table accumulation;
- exact 2I lanes;
- rounded fixed-point memory lanes;
- codebook-score attention;
- integer sampling.

Measure tokens/s, RSS and J/token against llama.cpp, bitnet.cpp and MLX baselines of matched held-out quality on the same M1. The deliverable is a short paper-quality result, not another stack of documents.

### 9.5 Phase 3: branches that depend on owner decisions

- **Deterministic addressing** (if §11.1 allows it, and for models larger than cache). Run D5's contest: geometric addresses (600-cell or E8 cells) versus LSH, k-means and PQ addresses, at matched bytes touched, on recall@K and NLL, with 3 seeds.
  - First target: the **output head**, which is 62.7% of per-token reads.
  - Try an exact factored or class-based softmax first; it needs no router at all.
- **Conversion experiments** (if §11.3 allows them). Try a reduced-token conversion of SmolLM2-135M into the recurrent form, and report quality honestly. The budget is set from measured throughput, with no promise of usefulness.
- **Codebooks.**
  - E8/icosian 2-bit weight codebooks (the QuIP# E8P precedent).
  - 600-cell polar compression of KV and event memory with a quantized gain (HQMQ-style), for long context.
  - Each must beat int3 or ternary by at least 10% in KL or perplexity at equal bits.

### 9.6 What to stop

- local selector and admission tuning at T = 256;
- zeta and prime mechanisms in any predictive path;
- authored 32-prompt panels as architecture gates;
- n = 1 geometry comparisons;
- software multiplication for activation products (see §11.2);
- decisions faster than weekly, except when a pre-registered kill criterion fires.

The current "+30M tokens on the same model" card is superseded by Phase 0's cooldown, which is cheaper and answers the same question, followed by the Phase-1 benchmark.

## 10. Ideas that would ease the research

### 10.1 Technical accelerants, cheapest first

| # | Idea | Why it helps | Cost | Quick falsifier |
|---|---|---|---|---|
| 1 | **Anneal the existing checkpoints**: linear LR decay to 0 | Training used a constant 1e-3 LR and never cooled down (2405.18392) | 1–2 h per arm | Gain in development-tail NLL below 0.02 |
| 2 | **Remove serving overheads**: analytic uniform floor, packed codes, i16 KV, build tables once | 3.36 MB → 0.85 MB touched per token; removes about 8k emulated operations per token | Hours | Bit-identical outputs |
| 3 | **Measure energy with macmon** (a wall meter as ground truth) | The first real joule in the project; decides §11.2 on data | 1 day | Protocol in §5.5 |
| 4 | **Add commutative and no-transport arms plus an A5 probe** to the existing learner | First well-posed geometry test on the real model | CPU-hours | Quaternion arm no better than the commutative arm at ≥4× training length |
| 5 | **Throughput benchmark: incremental vs parallel re-base** | Decides the Phase-1 base by measurement | Days | Re-base below 5× tokens per M1-hour |
| 6 | **Quaternionic-RoPE identity**: the linear quaternion recurrence equals decayed linear attention on frame-rotated q/k | Existing chunked kernels apply after an O(T·d) rotation | Part of #5 | Numerical equivalence (checked: ≤3e-14 float64, ≤9e-6 float32) |
| 7 | **Train continuously, then snap to 2I, then serve as an integer automaton** for tracking lanes | Measured exact to length 4,096 where float32 drifts | Days | Snapped lanes lose tracking relative to float at matched length |
| 8 | **Use the data that already exists**: TinyStories 2–4 epochs; open dialogue and code corpora | Exposure explains about half the gap to #1017 | Data preparation | NLL and coherence at matched parameters |
| 9 | **Logit distillation from #1017, early only** (same tokenizer) | Faster early learning; the teacher (1.57) is weaker than the target | About 14 MFLOP per token of teacher cost | Under 0.05 nats better than the no-KD run at matched tokens |
| 10 | **Polar 600-cell key codebook** for attention and event keys, with lookup-table scores in ½·ℤ[φ] | Satisfies D0-b by design rather than emulation; int3-like quality at 2.73 bits/dim | Days | Must beat int3/PQ at equal bits on real keys |
| 11 | **Exact factored or class-based output softmax** | Cuts the 62.7% of per-token reads in the output head without a learned router | Days | NLL unchanged (exact) at lower bytes/token |
| 12 | **uor-addr κ-labels** for event, record and artifact identity | UOR's own mature standard; retires prime-product identity | Hours | n/a |
| 13 | **3 seeds by default, with bootstrap confidence intervals.** Gate on held-out BPB, a TinyStories-style coherence score and the D6 long-range probe, not on authored 32-prompt panels | Stops n = 1 and small-panel results from steering direction | Small | n/a |

**Quarter-square lookup multiply.** The quarter-square LUT, ab = ⌊(a+b)²/4⌋ − ⌊(a−b)²/4⌋, is a D0-b-legal *speed* fix: 30–45× faster than the current loop.

It has two limits:
- it reads a 1 MiB table, and at Horowitz-scale costs that is likely *more* energy than a hardware multiply;
- it covers only operands up to 16 bits, while some mixture products are 48×15-bit.

Prefer designs that remove activation products, such as codebook scores and snapped rotations.

### 10.2 Process accelerants

1. **One planning object: the work card.** One page per experiment; the D9 work card already has the right fields. Retire the separate step, design, review, result, closeout and budget documents.
2. **Move evidence JSON out of git** into an artifact store addressed by uor-addr hashes. Keep only the hash and a one-line summary in the repository.
3. **Replace echo CI with real `fmt`, `clippy` and `test` on the active crates.**
   - Tests: under 1 minute for integer and tokenizer, about 14 s for training.
   - Builds: about 9 minutes for a cold training-crate build, cacheable (verify F1).
4. **Decide weekly.** Change direction only when a pre-registered kill criterion fires. Put owner checkpoints at decision points, not after the fact.
5. **A fixed weekly budget** instead of raising the ledger limit each cycle. Log one line per run: wall time, model time, tokens, tokens/s.
6. **Housekeeping.** Move the 97% of Rust unused by D8 into a `legacy/` area, rewrite AGENTS.md:122, and cap mandatory reading at about 5k words.
7. **One agent lead at a time.** Reserve multi-model review for promotions and the weekly direction.

## 11. Decisions only the owner can make

Each decision below is stated with its options, their consequences, and the review's recommendation where one is justified. Nothing changes until the owner rules. As noted in the Method, the shared briefing asked every agent to flag the tension in §11.1, so its repetition across reports is not independent evidence.

### 11.1 "No sparse routing" vs D5 (owner-ratified, per-token parameter sparsity)

The owner's words in this request: "no traditional matmul in runtime (floating point), moe, or sparse-routing". D5 (DECISIONS.md:241, owner-ratified on Sep 24) makes per-token parameter sparsity the terminal serving invariant.

Facts that bear on the choice:
- **Sparse access matters only above cache size.** For models too large for cache, touching fewer parameters is the largest energy lever. For example, touching 10% of a ternary 7B model's weights means 146 MB/token instead of 1.46 GB (physics report).
- **Below cache size, it buys little.** A ternary model of up to about 18–23M parameters (4.5–5.7 MB) fits in the M1's L2 cache (arch report).
- **Evidence exists only for learned routers.** The best evidence that sparse access preserves knowledge (32 experts, 8.8% of parameters touched, about 1.3× capacity loss) comes from a *learned* router (2404.05405), which the owner has excluded.
- **Some access is always selective.** Every model selects one embedding row per token, so "no per-token selection" cannot be literal.

Options:

| Option | Meaning | Consequence |
|---|---|---|
| **A** | Forbid learned gating (MoE routers, learned top-k memory). Allow *deterministic* addressing: a learned state is quantized to a fixed geometric codebook cell (600-cell/E8), and that cell selects table rows. Note that this is **top-1 access with a fixed codebook over learned states**, which the owner may still regard as MoE-like. | D5 remains achievable for models larger than cache. It must beat LSH, k-means and PQ addressing at matched bytes touched to count as a geometric win. |
| **B** | Forbid content-dependent selection of parameter subsets beyond the token's own embedding row. | Retire D5 as a terminal invariant. Serving is dense and low-bit. The levers become bits per parameter, cache residency and kernel efficiency. This is practical up to about 20M parameters at ternary, and caps efficiency above that. |
| **C** | Defer. Keep D5 as an aspiration, but scope it to models above cache size and revisit when one exists. | Removes the tension from present work. The current and near-term models are cache-resident. |

*Review recommendation:* **C now**, then A or B once a model larger than cache exists.

### 11.2 D0-b and the multiplier

D0-b states its objective as "no multiplier, tiny RAM, no GPU, local, measured energy … the multiplier constraint stays" (DECISIONS.md:113-114). The owner's wording in this request targets *floating-point* matmul.

Facts:
- **Cost of the current path.** Emulating integer multiplication in software makes the integer path 4.4× slower on x86 (bit-identical output), and very likely costs more energy on a CPU that has a multiplier. This is not yet measured on the M1.
- **Multiplier-free can save energy.** T-MAC showed that multiplier-free *lookup-table* kernels save energy when they remove instructions.

Options:

| Option | Meaning | Consequence |
|---|---|---|
| **A** | Amend D0-b so that *integer* activation products may use the hardware integer multiplier. Keep: no floating point, weights of 4 bits or fewer, weight maps executed by add or lookup table. | This changes an owner-stated objective. It gives the simplest and fastest path; energy is then judged by measurement. |
| **B** | Keep D0-b and remove activation×activation products **by design**: codebook-quantized keys with lookup-table scores; snapped rotation lanes; low-bit activations where they train; the analytic uniform floor. | The geometric way to honour D0-b. It needs engineering, and training risk for low-bit activations (ternary Q/K failed in MatMul-free LM). |
| **C** | Keep D0-b with software emulation (status quo). | Measured slow, and probably energy-negative. |

*Review recommendation:* decide **after the Phase-0 joule measurement** (§9.2 item 3). If hardware multiply is lower in J/token, prefer **A** for activation products and **B** wherever the geometry provides it naturally.

### 11.3 How may an existing model be used?

Current policy allows an offline teacher as a *training source*, but not as the author of served responses. The levels of use:

| Level | Use | Status |
|---|---|---|
| 1 | **Data**: more TinyStories (itself model-generated) plus open synthetic dialogue and code corpora | Allowed |
| 2 | **Logit distillation** from #1017 or an open model | Allowed. Helps most early and at small student compute |
| 3 | **Weight transfer or architecture conversion**: keep a pretrained model's channel-mixing weights (as ternary or 4-bit maps) and replace its attention with the geometric recurrence | Needs an explicit decision. The served model would contain transformer-derived weights, though no attention. Honest costs: LoLCATs-style 40M-token conversion keeps softmax attention windows, so it is not transformerless. Fully attention-free conversion used about 3B tokens in the literature (MOHAWK), which is months on an M1 at these sizes. |

*Review recommendation:* use levels 1 and 2 now. Treat level 3 as an optional, reduced-token experiment only if the owner wants the data point.

### 11.4 Compute scope

M1-only is the current authority. It caps from-scratch training at roughly 20–40M parameters on about 1–2.5B tokens per week, and only if the throughput fix delivers its assumed rate. That means TinyStories-class coherence, not general chat.

Modest rented GPU time would raise the feasible scale by 10–100×. That is an owner policy choice (currently "no paid compute"). It is listed only because it is the main lever this constraint rules out.

### 11.5 Process

Options: adopt the lightweight process in §10.2, or keep the current governance. The audit found that most D8 cycles spent more wall-clock time on process than on model computation. It also found that principal reviews caught real errors, and those reviews should be kept.

## Appendix A. Novelty map

Compiled from the literature report, which retrieved every source listed.

| Project idea | Closest prior work | Novelty | Leverage for the project |
|---|---|---|---|
| Keep the radius and quantize the direction | TurboQuant 2504.19874, PolarQuant 2502.02617 and QJL 2406.03482 all store the norm | Already done | High |
| 4-D quaternion chunks with a polytope shape code | HQMQ 2605.27646 (2026), IsoQuant 2603.28430 (2026), HIGGS p=4 2411.17525, QuIP# D4 2402.04396 | Already done (concurrent) | High |
| 600-cell or E8 = H4 ⊕ φH4 as a codebook | QuIP# E8P, LLVQ Leech lattice 2603.11021, 600-cell coding theory | Incremental | Medium |
| Per-lane quaternion state rotation | QRNN 1806.04418, PaTH 2505.16381, DeltaProduct 2502.10297, Mamba-3 2603.15569 | Incremental | Medium |
| Continuous training, then snapping to 2I, then serving as an exact integer automaton, inside a language model | Illusion of State 2404.08819; negative eigenvalues 2411.12537; DeltaProduct 2502.10297; The Automaton Underneath 2609.18966 | The capability is published; the snap-and-serve pipeline was not found (hypothesis). Measured at toy scale in this review | High |
| Quaternion-frame linear recurrence equals decayed linear attention with "quaternionic RoPE" | Mamba-2 SSD; RetNet; PaTH; Mamba-3 (complex case) | Incremental; the non-commutative case is not found in the literature | High (training kernels) |
| ≤4-bit additive/LUT serving (D0-b) | T-MAC 2407.00088, BitNet b1.58 2402.17764, bitnet.cpp 2410.16144, MatMul-free LM 2406.02528 | Already done | Very high |
| Soft read with a NoRead slot; copy gate | Pointer-sentinel mixture 1609.07843; attention sinks 2309.17453 | Already done | Low |
| Exact addressed memory | kNN-LM 1911.00172, RETRO 2112.04426, Engram, memory layers 2412.09764 | Already done (the learned sparse variants conflict with "no sparse routing") | High |
| Prime addresses | Prime Fourier Embeddings 2606.23044 (helps arithmetic only) | Incremental | Low–medium |
| Fixed zeta-zero phases | None found | Novel, unmotivated, and measured as decorative | Low |
| Hyperbolic / least-energy manifold routing | HELM 2505.24722 (near chance; best variant is MoE), RiLM 2609.10305, Hopfield 2008.02217 | Incremental | Low–medium |
| Prime-norm icosian "golden gates" for rotation codebooks | Parzanchevski–Sarnak 1704.02106; Kliuchnikov–Bocharov–Svore 1310.4150 (quantum compiling) | Novel in ML (hypothesis). No covering advantage at practical sizes; the theorem predicts holes (§6.4) | Low |
| Transformer-to-recurrent conversion | MOHAWK 2408.10189 (attention-free, about 3B tokens), LoLCATs 2410.10254 (40M tokens, but keeps 64-token softmax windows) | Already done | Medium: costly on an M1 if fully attention-free |

## Appendix B. Evidence produced by this review

The specialist reports and scripts lived in the review sandbox. Their load-bearing numbers are reproduced in this document with labels. The reports were:

| Report | Main experiments or checks |
|---|---|
| Mathematics | e1: 2I/E8 closure, lattice and cost; e2: transport drift; e3/e3b: zeta discrepancy, Landau bias, RoPE aliasing, identity collisions; e4: S³ codebooks and lattice second moments; e6/e6m: A5 learnability; e7: Householder representations; e8: octonions; e9: precision horizon |
| Physics | Energy model (60 configurations); Rust micro-benchmark of the repo's serving kernels; M1 measurement protocol |
| Architecture | Exact parameter and FLOP counts; M1 training budgets; quaternion-recurrence equivalence checks (errors 5e-15 to 2e-13); polar 600-cell attention-key codebook; multiply micro-benchmarks, including the quarter-square LUT |
| Literature | Seventy-plus retrieved papers; a 4-D gain–shape measurement |
| Audit | GitHub API data for 974 PRs; per-PR file statistics for the 369 PRs merged in the last 30 days; compute-versus-orchestration accounting |
| Verification | Builds and tests; x86 disassembly census; a random-weight timing harness comparing software and hardware multiply (bit-identical); a correctness review |
| Quantization | Matched-bit rate-distortion comparisons, inner-product error and NN recall, and the codebook-serving variant |
| State tracking | A5/Z60 word problems; quaternion, diagonal, complex and GRU models; exact 2I table serving |
| Lead | 2I closure; golden-gate covering (level-1 3,600 rotations, exact ½ℤ[φ] coordinates, covering against random); a WikiText-2 byte-level time-mixing comparison (six variants) |

The Python scratch code used by the agents is **not** committed, in keeping with the project's no-Python-model policy. The figures quoted here come from those runs. Any mechanism adopted from this review should be re-implemented and re-measured in Rust inside the project's own harness.

