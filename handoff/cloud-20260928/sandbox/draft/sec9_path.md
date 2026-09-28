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
