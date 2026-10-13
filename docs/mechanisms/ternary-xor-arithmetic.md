# Ternary weights and XOR: a multiplier-free contraction, not a matmul replacement

**Milestone:** M4 #2032 (served cost); owner-flagged on M1 #2029. **Lab:** unassigned. **Owner prompt, 12 October:**
*"no matmul at all anywhere, same with softmax - there is a ternary alternative with xor that may work."*
[D25](../integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction) records this as an
owner-suggested candidate, not an adopted mechanism. This brief picks no lab and no rung.

## The idea

Two claims are bundled in one sentence, and they are not equally true.

**(a) Ternary weights remove the multiplier — true, and elementary.** With `w ∈ {−1,0,+1}`, `w·x` is `+x`,
`−x`, or nothing. The inner loop needs no multiplier, only a conditional add/subtract/skip. This is the
oldest result in the line: BinaryConnect already says binary weights replace "many multiply-accumulate
operations by simple accumulations, as multipliers are the most space and power-hungry components of the
digital implementation of neural networks" ([1511.00363](https://arxiv.org/abs/1511.00363)). Ternary adds a
skip and costs 2 bits per weight instead of 1. **What it does not do: the contraction stays a dense W×D
sum.** Every weight is still touched every token. It removes a multiply, not the matmul.

**(b) XOR supplies the arithmetic — only at the bitplane level, and only if one operand is binary.** XOR is
addition in GF(2) ([GF(2)](https://en.wikipedia.org/wiki/GF(2)): "the addition is the same as that of the
logical XOR operation"; vector addition "is the bitwise operation called XOR"). It is *not* addition of
ternary values. Over GF(2) the only coefficients are {0,1}: `−1` and `+1` are the same element, and `0` is
not a third value with a group law. The same source states the limit: bit-strings over GF(2) can be given a
field multiplication, "but the multiplication operation cannot be a bitwise operation". "Ternary weights
make the dot product an XOR" is false as stated. What *is* true is narrower and useful.

**The ternary dot product against a binary operand is two popcounts over disjoint weight bitplanes.** Encode
each weight in two planes, `P` (set where `w=+1`) and `N` (set where `w=−1`), with `P & N = 0`. For binary
`x`: `Σᵢ wᵢxᵢ = popcount(P & X) − popcount(N & X)`. Verified here on 2,000 random ternary vectors with binary
activations (`n ∈ {7,16,64,255}`): exact in every trial. The zero case needs no plane — it is the complement
of `P | N`, so the skip is free. Note the operators: the mask is **AND**, the sign correction is a
**subtract**, and XOR is not among them. XOR does appear as the Hamming form `popcount(a ⊕ b)`, but that is a
**distance, not a dot product**: recovering the signed sum needs the `|a| + |b| − 2|a ∧ b|` correction, so an
"XOR dot product" always carries an AND term. The project already uses the XOR form exactly where a distance
is wanted — [flock-rank-reads.md](flock-rank-reads.md),
[hamming_refinement/metric.rs](../../crates/uor-r4-core/src/native_geometric/hamming_refinement/metric.rs).

**The project's own ternary packing is already in that form.** [lowbit.rs](../../crates/uor-r4-core/src/native_geometric/learner/lowbit.rs)
stores `CODE_ZERO=0, CODE_POS=1, CODE_NEG=2` two bits per weight: the high bit *is* `N`, the low bit *is*
`P`, and `11` is unused. The disjointness the identity needs is a structural invariant of the existing
format, not a new encoding choice; extracting the planes is an AND against `0x55…`/`0xAA…`.

**With multi-bit activations it still works, at a price.** Bitslice each activation bit-plane, run the
identity per plane, reduce with shifts and adds. The bit-sliced operand is `D` bits wide, so one output row
costs `2·⌈D/64⌉` 64-bit AND+popcount operations per activation bit-plane: for the stack's `4096→576` head,
**128 word ops per row at 1-bit activations and 1,024 at 8-bit, against 4,096 conditional adds for the naive
loop**. GPUs have AND/XOR/popcount on the integer pipe, so this is not exotic; it also gives the head's
integer Q-format activations a natural planar layout.

**Where XOR cannot go.** (i) XOR is not even "the" bitwise multiply for binary weights: encode ±1 as bits
and the product is **XNOR**-plus-popcount; encode {0,1} and it is **AND** — "instead of the xnor operation we
use the and operation to simulate 1-bit multiplication"
([PrecisionBatching](https://ar5iv.labs.arxiv.org/html/2003.00822) §3.4). Which window operation appears is
a property of the encoding, not of the arithmetic. (ii) A ternary zero is a *skip*, and "operand absent" is
not a two-input Boolean function of (activation bit, weight bit) — no gate expresses it. Every real
implementation therefore adds a second bit plus either a skip or a lookup: bitnet.cpp encodes
`{−1→00, 0→01, 1→10}` and then "transforms the weights back to their original values and performs the
vanilla GEMV operations", replacing the multiply with a 3²=9-entry table (TL1) or 3³=27 (TL2). (iii) Beyond
bitsliced planes, multi-bit addition needs carry propagation — that is integer addition; XOR is only the
carry-free half of an adder, and Booth/carry-save recoding solves the same problem by *avoiding* the
multiplier, with no XOR in it. (iv) XOR does not compose with the project's geometry: it is commutative and
associative, quaternion and octonion multiplication are neither (`q ⊗ u` changes sign under operand swap),
and the icosian `Z[φ]` ring's content is exactly that sign, gluing and orientation. Binary labels can index
part of a multiplication table; they cannot supply the law. (v) XOR is legitimate on *codes* — VSA/HDC
binding (`vsa/hypervector.rs`: `A ⊕ B ⊕ B = A`) is real, implemented and multiplier-free — but it binds
equal-length codes and cannot form a weighted sum. **For "XOR matmul" to hold, the weights must degrade to
binary {0,1} (or sign bits, dropping the {−1,0,+1} alphabet) and the activations must be binary or bitsliced.
That is XNOR-Net's regime, not a ternary one.**

## What it replaces and why that matters

D25 bans dense matrix multiplication in training and serving. The [geometry audit](../integration/geometry-audit-2026-10-11/README.md)
prices the current 29M stack at **30.27 M MACs/token**, of which quaternion transport — the only geometric
operation in the forward — is **0.05 %**, and the dense projections, MLP and head are ~97 %. There is a real
target here, and ternary touches exactly it. But three things must be said plainly.

1. **It does not answer D25.** That rule is about the algebraic structure of the executed path (D25:
   "replacing a library call with a handwritten dense matrix contraction is not a geometric replacement").
   A ternary contraction is still a W×D dense contraction: dense weight access, every weight touched, O(W·D)
   work per token. It removes the *multiplier instruction*, which is a D11 concern. The
   [operator plan](../labs/operator-phase-2026-10-12/README.md) already names this boundary for OP2: "the
   existing `quaternion_weights=all` expresses Hamilton blocks through dense GEMM."
2. **Operation substitution has already measured as a loss here.** The multiplier-free D11 integer engine ran
   **68.0 tok/s against the float path's 127.5** on the same artifact and windows at identical NLL
   ([d11-serving-2026-10-10](../labs/d11-serving-2026-10-10/README.md), cited by the audit's B5). Weight
   bytes, not multipliers, are what this laptop is short of. That loss is the baseline any ternary claim must
   beat.
3. **The project already has a better-justified multiplier-free weight representation.** Residual icosian
   pairs with a 2-bit mantissa norm reach 1.5148 BPB against dense's 1.5136 at equal parameters — about
   19–20 bits per quaternion, multiplier-free on the weight side
   ([icosian-mantissa](../labs/icosian-mantissa-2026-10-11/README.md)). A ternary weight is ~1.58 bits and is
   *not* in `Z[φ]`: it is a 3-point scalar quantiser that discards the exact coefficients the project treats
   as primary.

**Which lever is it?** A quantisation lever in the shape of an arithmetic one, and it collides with OP2
rather than complementing it. OP2 replaces those dense projections with *native Hamilton structure* — fewer
operations by structure. Ternary replaces the same projections with the *same* operation count and cheaper
instructions. They are substitutes for one budget line, not layers.

## Where it lives in code

The substrate exists; the bitsliced form does not.

- **Ternary packing and the multiplier-free map (exists).** [lowbit.rs](../../crates/uor-r4-core/src/native_geometric/learner/lowbit.rs)
  — `TernaryLinear::quantize` (2 bits/weight, disjoint POS/NEG codes), `forward_i32` (add/subtract/skip;
  power-of-two row scale, so the scale is a shift; `#![forbid(unsafe_code)]`), with exactness, scale and wrap
  tests. Callers: `bin/kvar-recall.rs` (`qmat_t`, `qdot_t`), `bin/readout-diagnostic.rs`,
  `bin/prior-frozen-evaluate.rs`, `bin/head-projection.rs`.
- **The XOR pieces (exist, separate).** `vsa/hypervector.rs`, `vsa/context_engine.rs` (XOR binding),
  `relational_attention/circuit.rs` (`XOR` gate kind), `hamming_refinement/metric.rs` ("XOR, popcount and
  integer addition only"), `hamming_policy/policy.rs`.
- **No bitsliced ternary contraction anywhere.** No `uor-r4-training` code mentions ternary; the trained
  path's contractions are `Tensor::matmul` throughout `crates/uor-r4-training/src/`, and `TernaryLinear`'s
  forward is a scalar `Vec` host loop used by offline binaries. Export surfaces are float (`rmatmul`,
  `qmat`), never bitplanes. **Do not read `TernaryLinear`'s existence as an implemented XOR path: there is no
  popcount contraction in the tree and no GPU kernel for one.**
- `scripts/check_no_softmax.py` ratchets the executable tree; no analogous interlock exists for matmul, so a
  ternary rung needs a declared way to show the dense contraction actually left the executed path.

## Sources and their claim classes

The evidence behind this brief is tabulated with an explicit scope label per line - **[THEORY]** (operation count or model estimate, no runtime measurement), **[KERNEL]** (custom kernel benchmark) or **[E2E]** (end-to-end wall-clock) - in [ternary-xor-evidence-factsheet.md](ternary-xor-evidence-factsheet.md). Read that table before quoting any speedup from this line: several widely repeated numbers are operation counts or estimates, not measurements, and no paper in this line calls a ternary-weight network XOR-based.

## What has been tried, honestly

External, each URL fetched and its measured scope stated:

| Work | Claim it supports | Measured scope |
|---|---|---|
| BitNet b1.58 ([2402.17764](https://arxiv.org/abs/2402.17764)) | Ternary `W̃ = RoundClip(W/γ,−1,1)`, absmean `γ`, 8-bit activations; `log₂3`; "the matrix multiplication of BitNet only involves integer addition" | Quality parity with FP16 LLaMA **only from 3B up** (3B 9.91 vs 10.04; 700M 12.87 vs 12.33 *worse*). The 2.71× latency (1.87 vs 5.07 ms/token) and 3.55× memory at 3B are on a **2-bit GPU kernel**, not ternary/bitwise arithmetic; 71.4× energy is a modelled 7 nm estimate. **No bitwise/XOR claim anywhere, and QAT keeps a dense FP16 master matmul in the training graph** |
| BitNet b1.58 2B4T ([2504.12285](https://arxiv.org/abs/2504.12285)) | Native 1-bit LLM, 2B/4T tokens, comparable to full-precision peers of similar size | One model, one scale; eval average 54.19 vs Qwen2.5-1.5B's 55.23. CPU latency (29 ms TPOT) names no CPU, so it is not a hardware-relative speedup; the GPU path packs then **unpacks before the matmul** ("pack-store-load-unpack-compute") |
| bitnet.cpp ([2410.16144](https://arxiv.org/html/2410.16144v2)) | The strongest hard number here: real end-to-end wins from **lookup tables plus integer adds** | Apple M2 Ultra **1.37×→5.07×**, Intel i7-13700H **2.37×→6.17×** tokens/s, 6.46× at 7B on 2 threads; 55.4–70.0 % (x86), 71.9–82.2 % (ARM) energy/token. I2_S unpacks to "vanilla multiply-then-addition GEMV"; TL1 is a 3²=9-entry int16 LUT. **No ternary compute is bitwise** |
| T-MAC ([2407.00088](https://arxiv.org/abs/2407.00088)) | Bit-wise **table lookup** replaces mpGEMM: `A×W = Σᵢ 2ⁱ(A×Wᵢ)`, linear in weight bit-width | 30 tok/s 1-core, 71 tok/s 8-core for BitNet-b1.58-3B on M2-Ultra (11 tok/s on a Pi 5); 4× throughput and 70 % energy vs llama.cpp. Authors state the kernel is **bandwidth-bound** |
| BinaryConnect ([1511.00363](https://arxiv.org/abs/1511.00363)), BNN ([1602.02830](https://arxiv.org/abs/1602.02830)) | Binary weights replace many MACs with accumulations; binary weights *and* activations replace most arithmetic with bitwise ops | MNIST/CIFAR-10/SVHN near SOTA; the multiplier argument is analytical, not an end-to-end speedup. BNN's own binary kernel ran one MNIST model 7× faster than an **unoptimized** kernel |
| XNOR-Net ([1603.05279](https://arxiv.org/abs/1603.05279)) | Binary filters **and** inputs → convolution by XNOR + popcount; 58× faster conv ops, 32× memory | ImageNet; binary-weight AlexNet 2.9 % below full-precision top-1. The 58× is an **operation count**; weights-only binary is ~2× by add/subtract. **Binary, not ternary — this is where the XOR argument actually lives** |
| Ternary Weight Networks ([1605.04711](https://arxiv.org/abs/1605.04711)), TTQ ([1612.01064](https://arxiv.org/abs/1612.01064)) | `{−1,0,+1}` is more expressive than binary; 2-bit storage; TTQ learns the values | TWN: ResNet18B 65.3 vs 67.6 FPWN, VOC mAP₅₀ 76.8 vs 86.7, and its own wording "the zero terms need not be accumulated" — the saving is **zero-skipping**, with no kernel or timing. TTQ calls its models "sparse binary weight networks, which can potentially be accelerated with custom circuit" — a proposal, never measured |
| PrecisionBatching ([2003.00822](https://ar5iv.labs.arxiv.org/html/2003.00822)) | The honest precedent for **bitserial** arithmetic: 1-bit tensor decomposition, AND + popcount, exact reconstruction | **>8× end-to-end wall-clock on a Tesla T4** vs Cutlass, <1 % error vs FP32, 1.5–2× over 8-bit at equal error. Decode costs named: fixed-point multiply-and-cast, bitwise transpose ("negligible overhead"), memory ~32/n, compute ×k |
| Integer-only ternary training ([2412.04787](https://arxiv.org/abs/2412.04787)) | Ternary-only training works via stochastic rounding, with no shadow weights | **8-bit is what matches BitNet b1.58** — pure ternary training carries a measured cost |

Also: 1.58-bit QAT is SOTA for small models "when doubling hidden layer sizes"
([2407.09527](https://arxiv.org/abs/2407.09527)) — capacity paid back in width
([reconnaissance](../research/external-literature-reconnaissance-2026-10.md) §C.2). **No work was found that
calls a ternary-weight network "XOR-based" and means a multiplication replacement**: XOR/XNOR branding
belongs to binary ±1, and where ternary is measured the gain is attributed to storage and table lookup.

In this repo: `TernaryLinear` states and tests the multiplier-free property (`forward_i32` equals
`forward_reference` on integer inputs), but every use is an **export/quantisation** step on a float master
(`TernaryLinear::quantize(&master, …)`), never trained in from step 0 — and the one measured multiplier-free
engine ran ~1.9× slower.

## What success looks like

- The bitsliced contraction is exact against `forward_i32` on random ternary matrices with both binary and
  8-bit activations, on GPU, no host fallback (D24).
- A pre-registered rung trains the mechanism in from step 0 at the project's recipe scale, with paired
  per-block analysis like the icosian line (noise 0.0012–0.0027 BPB).
- Quality within 2× noise of the matched arm, **weight bytes per token materially down**, and **tokens/s up
  on the same CUDA harness at the same shape and dtype** — the property the D11 engine lost.
- A compute-ledger row beside it per the audit's §6(1): MACs/token, weights touched, popcounts/token, bytes
  moved, wall-time at fixed quality, artifact size.

## What failure looks like

- Planar activations cost more quality than the instruction saving buys — quality outside the bar, or no
  wall-clock gain — **and** falling back to weight-side ternary only (activations multi-bit, contraction back
  to conditional adds) does not recover it. Then the mechanism is a quantiser with a popcount tax, and the
  icosian-mantissa arm already dominates it.
- The gain exists only in MACs and bytes touched, not in tokens/s — exactly the product-key memory result
  (−90 % MACs at the replaced layer, 1.65× *slower*; [kimi-direction](../labs/kimi-direction-2026-10-12/README.md) §3)
  and the D11 engine result.
- A result produced with a float master holding the ternary weights during training, or with one arm's
  activations float and the other's planar. That compares quantisers, not arithmetic.

## A fair test needs

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

## Open design questions

1. **Which operand is binary?** Weight-side ternary alone gives no XOR — the contraction is conditional adds.
   The XOR arm needs planar activations, and today's activations are integer Q formats. Is a planar layout
   trainable from step 0, or does it cost a conversion every layer?
2. **Is `{0,±1}` compatible with exactness?** The project's exact objects live in `Z[φ]`. `±1 ∈ Z[φ]`, but a
   per-coordinate ternary snap discards the φ-coefficient and the glue/orientation. Could the alphabet be a
   *sub-ring* choice whose three values are actual group elements (e.g. the Eisenstein cube roots) rather than
   a truncation? That is the version of the owner's idea that would be geometric.
3. **Can the sign/orientation law replace the multiplier?** For the icosian case the multiplier-free mechanism
   already exists (Cayley-table reads plus adds, [spin-native-weights.md](spin-native-weights.md)). Is a
   ternary alphabet anything other than a coarser version of that table?
4. **Does the reduction dominate?** Per row the identity yields one integer per bit-plane, and the inter-plane
   reduction is shift-and-add over `⌈log₂D⌉` accumulator bits. At which D does that stop beating a
   conditional-add loop in wall-clock on the actual GPU?
5. **Geometry side.** If ternary wins on cost and ties on quality, is that a KEEP for a *non-geometric*
   arithmetic substitution (D25's label) or a demotion of the exact-coefficient family? The owner withdrew the
   rectified-square G1 gate on exactly this ground, so this is an owner decision, and it comes before any run.
