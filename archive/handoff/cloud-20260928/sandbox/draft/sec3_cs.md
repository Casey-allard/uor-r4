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
