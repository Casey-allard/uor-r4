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
