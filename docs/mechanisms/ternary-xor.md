# Ternary and XOR/Hamming arithmetic

**Status: PARKED.** As proposed, a dense layer of −1/0/+1 weights removes the multiplier but not the sum over every weight, which the owner's no-dense-matrix-multiplication rule still counts against it. It was never trained in. Its two small pieces, bit counting and three-valued switches, already run inside the current model ([`discrete-learning`](./discrete-learning.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

A normal network stores each weight as a decimal number, and using it costs a multiplication. Here each weight is only −1, 0 or +1, so using it means add, subtract or skip. Two bit patterns are compared by counting how many bits differ, which is called the Hamming distance. The hope was cheap arithmetic with no multiplier. The plan to replace the big grids of multiplications this way has not been adopted, because every weight is still touched for every word and the owner's no-dense-matrix-multiplication rule counts that against it. The bit counting and the three-way switches are used in the current model.

## What it replaces in a transformer

The arithmetic inside the big grids of multiplications (the matrix multiplications): the projections that turn each word's vector into queries, values and layer outputs.

![Three panels. Left: a row of 16 ternary weights stored as two bit strips, one marking +1 and one marking -1, and an activation bit strip. An AND mask lights the surviving bits, and the two bit counts are subtracted. Middle: two 8-coordinate E8 root vectors drawn as bar rows with differing coordinates highlighted, where a tie in L1 displacement is broken by the Hamming count. Right: a dense grid with every weight cell lit beside a routed grid with only a few cells lit.](../figures/mechanisms/ternary-xor.svg)

*The operators are AND and subtract, not XOR (left). Inside the current model, bit counting breaks ties between routes (middle). The multiplier goes away but the dense grid stays (right).*

## How it works

**Storage (Definition).** A ternary weight vector $w\in\{-1,0,+1\}^n$ is stored in two bit planes. Plane $P$ is set where $w_i=+1$, plane $N$ where $w_i=-1$, and $P\wedge N=0$. The repository's low-bit code uses two bits per weight (`CODE_POS=1`, `CODE_NEG=2`), which is the same thing.

**The dot product is exact (proof).** For a binary activation vector $x\in\{0,1\}^n$,
$$\sum_i w_i x_i \;=\; \operatorname{popcount}(P\wedge x)-\operatorname{popcount}(N\wedge x).$$
Each position contributes $+1$ if and only if $P_i=x_i=1$, and $-1$ if and only if $N_i=x_i=1$. A zero weight is skipped for free. For multi-bit activations $x=\sum_b 2^b x^{(b)}$, repeat per bit plane and combine with shifts and adds. A power-of-two row scale is a shift. A 2026-10-12 re-check agreed in 5,000 random trials (**measurement**, [owner-ruling record](../../archive/labs/owner-ruling-and-storage-2026-10-12/README.md)).

**XOR is not the ternary dot product (proof).** XOR gives the Hamming distance $d(a,b)=\operatorname{popcount}(a\oplus b)=|a|+|b|-2|a\wedge b|$. Over the two-element field GF(2), $-1$ and $+1$ are the same element, so "ternary XOR matmul" is false as stated. What is true is the AND-and-subtract identity above.

**What it does not remove (proof, by counting).** A layer with $D$ inputs and $W$ outputs still reads all $W\times D$ weights for every token. The multiplier disappears; the dense sum stays. A contraction applied only to a routed subset would not have this cost ([hypothesis](#what-has-never-been-tested)).

**Where it runs today (Definition).** In the discrete model the same ingredients serve as routing parts, in [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu):
- A coordinate Hamming count breaks ties when two candidate ports have equal L1 displacement for a transported query.
- A ternary gate parameter switches flock levels on or off, with candidates visited in the order 0, −1, +1.

**Geometry.** On its own this is Boolean hypercube arithmetic, not a lattice. The E8 lattice is the (8,4,4) Hamming code lifted by Construction A, but Hamming distance on E8 labels is not the E8 lattice distance (decision D32).

**Rule check.**
- *D11 (integer serving):* compatible in form, since add, subtract, skip and AND with popcount need no multiplier. Not shown to be fast: the one multiplier-free engine measured ran 68.0 tokens/s against 127.5 on the D10 engine (multiplies allowed) ([record](../../archive/labs/d11-serving-2026-10-10/README.md)).
- *D23 (no softmax):* nothing to scan; scoring is a hard count.
- *D25/D30 (no dense matrix multiplication):* the dense ternary sum is still a dense sum, so it would not pass as a matmul substitute. The tie-break and trit gates inside routing are not matmuls.
- *D32/D37:* D32 names ternary and Hamming computation as a preferred substrate. D37 allows float training offline, but a ternary master would still need a new pre-registered sheet and an export gap ([decisions](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)).

## What has been tried

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| Sep | Hamming refinement primitive: query, exact read, state update, authored policy, 240-bit code, no training | Passes five finite causal cases (1–4 hops, up to 256 occurrences scored per hop). Not a language model | positive, primitive only | [record](../../archive/native_geometric_hamming_refinement_973.md) |
| Sep | One seeded 4-input-table circuit (3,076 gates, 6,152 parameter bytes), two Hamming hops, not fitted | Trajectory check passes. Language: 0/24 complete answers, 1/460 response symbols, mean cross-entropy 10.998 | inconclusive | [record](../../archive/native_geometric_hamming_policy_973.md) |
| 12 Sep | Learned Boolean query-update tables for a second dependent read | Ablation controls pass; final independent qualification not run; authored finite task | positive, small | [record](../../archive/native_geometric_dependent_attention_973.md) |
| 19 Sep | Dense ternary recurrence $h_t=\mathrm{relu}(W_x x_t+W_h h_{t-1})$, integer state | State overflows the 32-bit limit by 32 steps at widths 32, 64 and 128. Recall over 4 steps works at width ≥128; over 8 steps it scores 0.00 at widths 64, 128 and 256 | negative | [record](../../archive/integration/geometric-core-architecture-2026-09-19.md) |
| 20 Sep | Ternary projection of the frozen output head (PR #1308) | Bits per target on 17,342 targets: empirical head 7.1708, plain ternary 7.4162, activation-aware ternary 7.1909, float head 6.8741. Neither ternary form beats the empirical head. The complete search was NOT_RUN | negative | [record](../../archive/integration/geometric-query-review-2026-09-20.md) |
| 22 Sep | Learned seven-slot ternary realizer | 31/31 authored cases; review qualified the comparator and meaning claims | inconclusive | [record](../../archive/integration/state-conditioned-lexical-result-2026-09-22.md) |
| 23 Sep | Ternary readout refit on a frozen state | The integer kernel reproduces the float fit. Gain about 0.13 bits against a 0.30-bit bar; the first-draft conclusion was retracted | positive, bar missed | [record](../../archive/integration/ordinary-lexical-readout-refit-result-2026-09-23.md) |
| 26 Sep | Hamming sign-bit keys against float keys, toy recall | 64-bit codes recall as well as float at 1,024 candidates with 8 bytes per key instead of 128. As final scores they cost 0.014 bits per byte, so they should admit candidates and exact scores should rank | positive | [record](../../archive/integration/geometric-attention-2026-09-26.md) |
| 27 Sep, 1 Oct | Signed 4-bit packed coefficients, a 4-bit integer gate and an integer q/k projection (not ternary, same multiplier-free family) | 8/8 baseline/candidate pairs match with 87% fewer retained coefficient bytes; 0 disagreements across 4 saved models; 128/128 answers on both held seeds | positive | [packed](../../archive/integration/packed-integer-comparison-2026-09-27.md), [gate](../../archive/integration/geometric-attention-integer-latch-2026-10-01.md), [q/k](../../archive/integration/geometric-attention-integer-qk-2026-10-01.md) |
| 9 Oct | Advisory review of the owner's two-page XOR/Hamming note | Boolean identities valid; proves no semantic distance, routing or learning gain. No compute | inconclusive | [review](../../archive/labs/nemesis-xor-review-2026-10-09/README.md) |
| 10 Oct | Multiplier-free integer engine against the float path, three 29M-parameter artifacts, 64 held-out windows | Bit-identical outputs; 68.0 against 127.5 tokens/s. Served quality 1.20138 bits per byte against a target of 0.90 or less, so the gap is the model | negative | [record](../../archive/labs/d11-serving-2026-10-10/README.md) |
| 12 Oct | Independent re-check of the popcount identity; owner's no-matmul ruling | Identity exact; assessed as a quantisation lever that drops the multiplier, not the dense contraction. No model run | inconclusive | [record](../../archive/labs/owner-ruling-and-storage-2026-10-12/README.md) |
| 13 Oct | Discrete model with ternary gate proposals and coordinate Hamming in flock selection | 17/17 controls pass at source `76198c3b5`; gate-cost witness `[448, 256, 64]`; applied updates 0 | positive, controls only | [record](../../archive/labs/discrete-flock-writer-integration-2026-10-13/README.md) |

**Reading these honestly.** The failures are failures of configurations: an unnormalised, ungated recurrence (the takeover review withdrew the broader reading); a quantiser applied after training to a float copy with a reduced search; one frozen-state refit; one engine on one laptop. None tests ternary weights trained from step 0 in a normalised or routed structure. The positive rows are authored finite tasks or toy recall, not language quality. The 4-bit rows use the same multiplier-free family but not the ternary alphabet.

## What has never been tested

- **Any bit-sliced or popcount contraction.** No kernel and no code exists in the tree, and the training crate has no ternary-weight code.
- **A ternary weight trained in from step 0.** Every use of the ternary linear layer is a post-hoc snap of a float master.
- **The actual XOR arm,** with weight planes plus bit-sliced activations, separated from the weight-only arm.
- **A ternary alphabet of true group elements,** such as the three cube roots of unity, instead of a truncated scalar.
- **A matched comparison against the icosian-mantissa weights** ([`spin-weights`](./spin-weights.md)) at equal steps, width and weight bytes.
- **Tokens per second** of any ternary contraction on the shared CUDA harness.
- **A corpus run that isolates the ternary gate moves.** Only finite construction controls exist, with zero updates applied.

## A fair test needs

*From the earlier brief, [archive/mechanisms/ternary-xor-arithmetic-2026-10-20.md](../../archive/mechanisms/ternary-xor-arithmetic-2026-10-20.md).*

- **Mechanism trained in from step 0**, not quantised post hoc. Every existing `TernaryLinear` use is a
  post-hoc snap of a float master; that arm is a reading, never a verdict.
- **A matched control at equal steps, width and weight bytes per token.** For this project the honest control
  is `quaternion_weights=all` / icosian-mantissa, not an FP16 dense model — that is what already sits at
  parity and is already multiplier-free on the weight side.
- **≥ 2 seeds with paired per-block analysis, or ≥ 200 fresh rows** (D22). With existing noise of
  0.0012–0.0027 BPB, the bar is **≥ 0.01 BPB or 2× measured noise, whichever is larger**, fixed before
  compute, plus a stated tokens/s bar on the same harness.
- **Both operands declared, two arms:** (i) weight planes only, activations multi-bit; (ii) weight planes
  **plus** bitsliced activations — the actual XOR arm. This separates the popcount cost from the ternary cost.
- **A qualified GPU kernel for the contraction** before any language fit: exactness against `forward_i32`,
  device execution proven, host fallback refused.
- **Fresh transfer for any KEEP** (D22), and a declared way to show the dense contraction left the executed
  path, including the training gradient (see open questions).
- **Uninterpretable if:** the run trains on CPU (D24); arms differ in dtype, width, steps or tokeniser; the
  weight planes are rebuilt per token rather than per artifact; the popcount path is a forward-time wrapper
  while the trained graph keeps its dense contraction; or the comparison is a configuration D25 excludes.

## What would reopen it

1. A GPU kernel for the bit-sliced or table-lookup ternary contraction exists and matches `TernaryLinear::forward_i32` for binary and 8-bit activations, with host fallback refused (decision D24: no model compute on the CPU).
2. A pre-registered rung trains it in from step 0, against the matched icosian-mantissa arm at equal steps, width and weight bytes, with at least 2 seeds or 200 fresh rows. The bar is the larger of 0.01 bits per byte or twice the measured noise, set before compute.
3. Tokens per second on the same CUDA harness goes up, not only multiplications or bytes touched. The integer engine lost 68.0 against 127.5, and product-key memory lost the same way, so this is the bar to beat.
4. The contraction is applied where it is not a dense $W\times D$ sum (a routed or selected subset), or the owner rules it an accepted, labelled non-geometric substitution, before any run.
5. Or the alphabet is made geometric: three values that are actual group elements, such as the three cube roots of unity, instead of a truncated scalar quantiser.

Open questions: which operand is binary (today's activations are integer, so the XOR arm needs bit-plane activations); whether $\{-1,0,+1\}$ fits the exact $\mathbb Z[\varphi]$ objects; and, if ternary wins on cost and ties on quality, whether that is an accepted non-geometric substitution (an owner decision). These are **hypotheses**, not results.

## Also known as

Ternary weights; XOR matmul; bit-sliced ternary contraction; Hamming or popcount distance; `TernaryLinear` and low-bit; multiplier-free weights; modulo-256 substrate; "ternary alternative with xor" (owner prompt, 12 October 2026).

## Code

- [`learner/lowbit.rs`](../../crates/uor-r4-core/src/native_geometric/learner/lowbit.rs): ternary linear layer, two bits per weight; add, subtract or skip with a power-of-two scale. Used only for post-hoc export of float copies.
- [`learner/lowbit_core.rs`](../../crates/uor-r4-core/src/native_geometric/learner/lowbit_core.rs): the dense ternary recurrence of the 19 September negative.
- [`learner/lowbit_attention.rs`](../../crates/uor-r4-core/src/native_geometric/learner/lowbit_attention.rs): low-bit attention variant.
- [`learner/head_projection.rs`](../../crates/uor-r4-core/src/native_geometric/learner/head_projection.rs): ternary search of the output head.
- [`hamming_refinement/metric.rs`](../../crates/uor-r4-core/src/native_geometric/hamming_refinement/metric.rs) and [`hamming_policy/policy.rs`](../../crates/uor-r4-core/src/native_geometric/hamming_policy/policy.rs): XOR, popcount and integer add over a 240-bit code.
- [`vsa/hypervector.rs`](../../crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs): XOR binding of codes, a code operation rather than a weighted sum.
- [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): the active use, the coordinate Hamming tie-break and the ternary gate parameter.

Audit source: `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/ternary-xor.json`.

## Related mechanisms

[spin-weights](./spin-weights.md): the competing multiplier-free weight representation, level with dense in its own bars. [discrete-learning](./discrete-learning.md): runs the trit gates and Hamming tie-breaks. [flock-read](./flock-read.md): selection uses frame and payload Hamming and ternary level gates. [spiralcore-fbs](./spiralcore-fbs.md): the base-3 binder tree these gates sit on. [vsa](./vsa.md): XOR binding of codes is the legitimate XOR use. [e8-orbit-memory](./e8-orbit-memory.md): E8 as a lifted Hamming code.
