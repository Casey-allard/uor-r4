# Geometric core architecture — content-addressed, multiplier-free

**Superseding September 19 review:** see [takeover assessment](takeover-review-2026-09-19.md) and [current execution handoff](deepseek-next-step-2026-09-19.md). This is a preserved dated record, not the current task queue. #1284 and #1290 are merged; D0-b is owner-adopted. Static residue scores are not recurrent-model ceilings; universal ternary-recurrence/training impossibility and unqualified harmonic/compute claims are corrected in the review. Preserve original measurements at their exact scope.

Owner: Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · Status: **draft, pending owner review**

Authority: [`DECISIONS.md`](DECISIONS.md) (D0-b is the serving contract). Sequence and acceptance stay
with [`project-track.md`](project-track.md); live results with [`current-state.md`](current-state.md).
This file adds design reasoning and the scaling projection only.

Evidence for everything marked *measured*: [`native_geometric_lowbit_attention_2026-09-19.txt`](../evidence/native_geometric_lowbit_attention_2026-09-19.txt)
and [`native_geometric_lowbit_chat_2026-09-19.txt`](../evidence/native_geometric_lowbit_chat_2026-09-19.txt).

---

## 0. The question this answers

The dense low-bit recurrence (`learner/lowbit_core.rs`) now trains — backward pass, straight-through
estimator, BPTT and Adam are implemented and the learning test passes — but the trained model copies
its input and does not answer, and free generation collapses to one repeating fragment. The question
is whether that is an optimiser problem, a data problem, or an architecture problem.

**It is an architecture problem, and it is now measured rather than argued.**

## 1. Why the dense ternary recurrence fails (two independent causes)

The core is `h_t = relu(W_x[:,t] + W_h·h_{t-1})`, with `W_h` ternary.

**(a) Magnitude.** For a ternary matrix with non-zero fraction `p`, the spectral norm is ≈ `2√(p·d)`
(Bai–Yin), i.e. ≈16 at `d = 96`. The recurrence therefore expands by roughly `√d/2` per step. Measured
peak `|h|` from a random initialisation:

| `dim` | 8 steps | 16 steps | 32 steps |
|---:|---:|---:|---:|
| 32 | 1.96e3 | 3.66e6 | 2.13e9 |
| 64 | 5.24e4 | 2.07e9 | 2.15e9 |
| 128 | 3.42e5 | 2.14e9 | 2.15e9 |

`i32` saturates at 2.147e9. **The integer state saturates between 16 and 32 steps at every width.**
That is exactly the observed ~24-token workable window in the chat run, and exactly why a 40-token
configuration scored 18.47 — worse than the uniform `ln 259 = 5.56`.

**(b) Burial.** Every past input is summed into the *same* channels, so one item to remember is buried
under `T·|x|` of accumulated distractor mass. Measured on the delayed-recall task (retrieve a token
seen `D` steps earlier):

| `dim` | D=2 | D=4 | D=8 |
|---:|---:|---:|---:|
| 64 | 1.00 | 0.20 | 0.00 |
| 128 | 1.00 | 1.00 | 0.00 |
| 256 | 1.00 | 1.00 | 0.00 |

Capacity buys delay 4 and never delay 8. This is a representational limit, not a training-budget limit:
the item's contribution is `O(1)` against `O(√T)` of accumulated distractors in the same channels.

**(c) The obvious fix is falsified.** Right-shifting the recurrent term (`h = relu(W_x + (W_h·h) >> k)`)
makes the recurrence contractive but also *destroys the pathway*:

| decay `k` | D=1 | D=2 |
|---:|---:|---:|
| 0 | **1.00** | **1.00** |
| 2 | 0.16 | 0.00 |
| 3 | 0.00 | 0.00 |

There is a structural reason this cannot be repaired: a ternary matrix cannot be near-orthogonal
(entries are `±1`, so `‖W_h‖₂ ≥ 1` and is ≈`√d` for any dense random draw), so the only `k` that
stabilises the recurrence is one that also erases memory. **Stability and memory are not jointly
available from a dense ternary mixing matrix with a scalar decay.** That kills the whole family, not
just one setting.

## 2. What the serving contract actually permits

D0-b allows integer add/subtract/shift/bitwise/popcount/compare, table and array reads, bounded index
arithmetic, exact fixed point, weights at most 4 bits, and **no multiplier instruction in the kernel**.
It explicitly permits bounded low-bit linear maps, in a form that executes as adds/subtracts/shifts/
table reads.

Two consequences a language model can be built on:

1. **A ternary operand times anything is a conditional add.** If `k_i ∈ {-1,0,1}`, then `k_i · v_j` is
   `+v_j`, `-v_j` or nothing. Outer products, and the contractions that read them, are therefore
   multiplier-free *by construction* rather than by compiler grace.
2. **A power-of-two scale is a shift.** So a per-row or per-channel magnitude costs nothing.

What the contract does **not** permit is anything that needs `exp` (softmax attention), a true divide
(per-token normalisation by a data-dependent scalar), or a genuine variable-by-variable product.

## 3. The proposed core: content-addressed linear attention

Linear attention ([Katharopoulos et al.][ka]; RWKV [arXiv:2305.13048][rwkv]; HGRN2 [arXiv:2404.07904][hgrn2];
MatMul-free LM [arXiv:2406.02528][mmlm]), with every operand made ternary and every scale a power of
two:

```text
S_t[i][j] = (S_{t-1}[i][j] >> decay) + k_i(t) · v_j(t)        (dk × dv matrix state)
y_j(t)    = Σ_i q_i(t) · S_t[i][j]                             (matched-filter read)
logits    = W_o · relu(y(t))
```

* `S_ij = k_i · v_j` is a conditional add or subtract — no multiplier.
* `q_i · S_ij` likewise. `relu` is a `max`. The final projection is the existing ternary linear map.
* `W_o` and `V` carry per-row power-of-two scales, so magnitudes are shifts.
* The decay is a right shift. It is optional: with `decay = 0` the state grows **linearly** in `T`
  (measured peak `|S| < 2^20` over 512 steps), not geometrically, so it does not saturate.

**Why this fixes both causes.** The read is a *matched filter*: `y = Σ_s (q·k_s) v_s`. A stored pair
whose key aligns with the query contributes `Θ(dk)`; `N` distractors contribute `Θ(√(N·dk))` of
noise. The signal-to-noise ratio is therefore ≈ `√(dk/N)` — *capacity buys horizon*, whereas in the
dense core it did not. And because the read is a filtered sum rather than a running sum into shared
channels, an item is never buried by mass, only outvoted by luck.

**Why it is a genuine change of kind, not more of the same.** The dense core stores a *state*; this
stores *pairs*. Pairs are what copying, induction and associative recall need, and they are exactly
what the project's "exact addressed memory" language has been pointing at.

## 4. Measured results

Induction task: `[x, y, filler*delay, x] → y`, i.e. predict what followed the last occurrence of the
current token. Held out, deterministic seeds, one layer.

| `dk = dv` | lr | delay | accuracy |
|---:|---:|---:|---:|
| 64 | 0.05 | 4 | 1.00 |
| 64 | 0.05 | **16** | **1.00** |
| 128 | 0.005 | 16 | 1.00 |
| 256 | 0.005 | 16 | 1.00 |
| 512 | 0.005 | 16 | 1.00 |

For comparison, the dense recurrence scored **0.00 at delay 8 at every width tested**, and the chat
run's response-only accuracy was 0/470. **The content-addressed core retrieves across a distance the
dense core cannot represent at all, and it does so at 100% while scaling in width.**

**The negative result that matters more: training is fragile.** At `dk = 128, lr = 0.05` the run
collapses to the uniform predictor (loss exactly `ln 8`). At `dk = 64`, delay 32 collapses at settings
where delay 16 succeeds. Larger batch, longer delay and higher lr all change the regime. A architecture
that finds the solution only sometimes cannot be scaled, so **this is the top open item**:

* suspected first cause — the readout is un-normalised, and its magnitude grows with the number of
  stored pairs and with `dk`, so the loss surface steepens as capacity rises;
* suspected second cause — the straight-through estimator on three separate ternary tables at random
  initialisation gives a poorly conditioned start.

Both are addressable inside the contract: a **power-of-two readout normalisation** (shift the read by
the leading bit of the accumulated key mass — a bit scan plus a shift, no divide) is the natural fix,
and query/key tying or a Delta-rule update are the natural second. These are predictions, not results.

## 5. Where the project's own mathematics fits

This is not a generic linear-attention port. Each project mechanism has a concrete slot, and the slots
that are empty are named honestly.

| Project mechanism | Role here | Status |
|---|---|---|
| Prime addresses / ordered n-lets | key and query tables are *addressed* per token — a selected row read, which is the memory the architecture actually uses | **used** |
| Exact `Z[φ]` / H4 roots as a codebook | the 120 canonical H4 roots are a ready-made structure for *key codes*, so that `k_i` is a signed root coordinate rather than a random ternary bit | **proposed** — not yet wired |
| Paired-H4 / icosian `E8` | the optimal 8-dimensional lattice quantiser; the natural quantiser for the *value* and *output* tables at more than ternary precision (≤4 bits is permitted), and for the state `S` itself | **proposed** (Stages 6 in the transition plan) |
| Zeta-zero phases | a fixed, structured schedule for the decay exponents `k_i` — a multi-timescale memory rather than one decay per layer | **proposed** — currently a single scalar decay, which §1(c) shows is the weak form |
| R4/S3 transport, Hopf observation | a candidate for the *output* mixing, where a genuine rotation of the readout is cheap in `Z[φ]` | **proposed**, unmeasured |
| Chirality / polarity | a sign structure on keys that is orthogonal to magnitude | **proposed**, unmeasured |

The honest summary: the architecture currently uses the project's **addressed-select** idea and the
low-bit operator contract. The geometric content of the tables is not yet exploited. That is the
cheapest place to look for a mechanism the literature does not have.

## 6. Scaling: what would actually produce coherent chat

This is the part where the honest answer is uncomfortable, so it is stated with its assumptions.

**Parameter count.** The tables cost `vocab·(2·dk + 2·dv)` ternary weights, i.e. `vocab·(2dk+2dv)/4`
bytes at 2 bits per weight. At `vocab = 32k`, `dk = dv = 1024`: ≈33 MB per layer, ≈400 MB for 12
layers. This fits the target machine. Serving is add/shift-bound: ≈`2·dk·dv` adds per token per layer,
≈25M adds/token at 12 layers — plausible at hundreds of tokens/s on M1 *if* the kernel is
memory-friendlier than it is now.

**Training is the wall, and it is a first-principles wall, not a budget detail.** The recurrence is
sequential and the state is `dk × dv` per layer, so BPTT cost per token is `O(dk·dv·L)` with
`L` layers, and it does not parallelise over sequence length the way a transformer's does. A coherent
chat model of this family needs on the order of `10^9` parameters and `10^10`–`10^11` tokens. On one
M1 at the measured rate of this code (tens of thousands of token-steps per second at `dk = dv = 64`),
that is on the order of `10^5`–`10^6` hours. **Therefore: coherent chat cannot be trained on this
machine, by this architecture or any other, within the project's compute rules.** The project's premise
— frontier capability *served* on a consumer laptop with no multiplier — remains coherent. Its implicit
premise — that such a model can also be *trained* on that laptop — does not survive this arithmetic.

There are exactly three honest options, and this is a decision for the owner:

1. **Narrow the target.** Train a strong, small, *domain-scoped* model (a tool, a memory, a
   constrained assistant) where `10^7`–`10^8` parameters and `10^8`–`10^9` tokens suffice. This is
   reachable locally and would be a real product.
2. **Separate training from serving.** Train elsewhere (rented or institutional compute) and serve
   locally under D0-b. This is arguably the actual research contribution: *a serving contract no
   current model satisfies*. It requires the owner to authorise compute outside this machine.
3. **Find a sample-efficiency result.** This is the only path that keeps the original premise, and it
   is genuine open research, not an engineering task.

## 7. Falsifiable predictions for the next stage

1. Power-of-two readout normalisation removes the collapse: at `dk = 256`, `lr = 0.05`, delay 16, the
   run stops falling to uniform and the accuracy becomes insensitive to batch size. **Falsified if the
   regime map is unchanged.**
2. With normalisation, the induction horizon grows with `dk` at fixed training budget (the `√(dk/N)`
   prediction), reaching delay ≥ 64 at `dk = 256`.
3. A phi-structured key codebook beats a random ternary one at equal `dk` on delay ≥ 32, because it
   makes `q·k` a *graded* similarity rather than a coin flip.
4. A per-channel (`k_i`) decay schedule beats a single scalar decay at delay ≥ 64, for the reason
   §1(c) gives: it is the only way to hold both fast and slow timescales in one state.
5. A single layer is not enough for the instruction corpus: response accuracy stays near zero until
   something composes two operations. If a two-layer version does not move response-only accuracy off
   zero, the *task*, not the depth, is the problem.

## 8. Recommendation

Do §7.1 and §7.2 next and nothing else. Normalisation is a small, contract-compliant change, it
targets the only measured blocker to scaling, and its failure or success is unambiguous. Then re-run
the chat/instruction corpus on the content-addressed core and read the raw output before any further
architectural work.

Do **not** scale `dk`, add layers, or change the tokenizer until the regime map under normalisation is
flat. §4 shows that the current architecture can reach 100% at `dk = 512`; it also shows that it
sometimes does not reach 100% at `dk = 64`. Adding parameters to an unstable optimiser produces larger
failures, not capability.

**Owner decisions needed** (§6): which of the three training-compute options is intended, because that
determines whether the goal is a narrow local model or a locally-served large one.

[ka]: https://arxiv.org/abs/2006.16236
[rwkv]: https://arxiv.org/abs/2305.13048
[hgrn2]: https://arxiv.org/abs/2404.07904
[mmlm]: https://arxiv.org/abs/2406.02528
