# Geometric attention: the owner's mechanisms, made precise and tested

2026-09-26 · Requested by the owner · References #820

**Status.** A design and evidence note, not a decision record. It answers the owner's objection to the [low-energy plan](low-energy-plan-2026-09-25.md). That plan kept a few conventional attention layers; the owner wants attention itself redesigned from geometric mechanisms. Those mechanisms are: tokens stored in another geometry; routing over a mutable Riemannian manifold with a "Hamiltonian heatmap"; switching to the cheapest geometry; geodesics; triangulation; Hamming distances acting like OSPF, with locations advertising their data; E8, icosian and quaternionic spaces; primes, CRT and Galois fields; and a recursion that compares token angles in each embedded space.

Labels: **Measured** (run for this note; scratch code in the review sandbox), **Derived**, **Literature**, **Hypothesis**.

## 0. The answer

The owner's picture is closer to the mathematics of attention than the low-energy plan allowed. The measurements below say which parts pay and where.

1. **Attention already is a "Hamiltonian heatmap".** Softmax weights are a Boltzmann distribution over memory slots, with energy −⟨q,k⟩ and temperature 1/β.
   - On normalized vectors the weight exp(β cos θ) is the von Mises–Fisher kernel. For small angles it is a heat kernel in the geodesic distance θ.
   - So the redesign question is not whether to use energy and geometry. It is **which manifold, which energy, how many candidates and how many bits per comparison.**
2. **Geodesics cost one inner product, and they pay on hierarchies.** Ranking in spherical, hyperbolic or Euclidean geometry is one (Euclidean or Lorentzian) inner product followed by a monotone map (§2).
   - **Measured on a tree retrieval task:** with 8 dimensions, hyperbolic (arcosh) keys found the query's deepest stored ancestor 98.5% of the time among 192 stored nodes. Dot product managed 54.1% at 8 dimensions and 68.2% at 16.
   - On flat recall and on real text, hyperbolic matched dot product (§3, §8).
3. **The cost of attention is candidates × bits, and geometry cuts both.**
   - **Angles by Hamming distance** (XOR plus popcount; the sign-disagreement rate estimates θ/π). 64-bit learned codes recalled as well as float attention at 1,024 candidates. Used as the *final* attention scores in a language model, they cost 0.014 bits/byte, so they should **admit** candidates and exact scores should **rank** them. The published systems do the same (§10).
   - **Routing by advertisement** (the owner's "locations advertise their data"). Untrained advertisements routed poorly. *Trained* advertisements admitted the right key 95–98% of the time while only 160 of 1,024 keys were scored.
4. **E8, Hamming codes and Galois fields are one object.** E8 is the [8,4,4] Hamming code lifted to a lattice (verified, §4). E8 is the better code per bit for recall and routing, but as snapped final scores in a language model it trained worse than sign bits.
5. **The radius carries information.** This is the owner's original instinct, measured in three places:
   - angle-only scoring lost 21% of recall at 8 dimensions where magnitude-bearing scores held 100%;
   - binary codes, which discard magnitude, failed on the tree;
   - in hyperbolic space, the radius *is* the depth in the hierarchy.
6. **Primes, CRT and semiprimes give exact algebra, not similarity.** A product of primes is a bitset in a costlier encoding (gcd corresponds to AND), and CRT residues destroy nearness. They belong in identity, deduplication and addressing (§7).
7. **A single superposed ("entangled") state cannot replace attention.** Its recall capacity is set by its width (§6).

## 1. What an attention step costs, and where geometry acts

Take one head of width 64 at 4,096 tokens of context, per generated token (Derived):

| Design | Bytes read | Score operations | Value multiply–adds |
|---|---:|---:|---:|
| Dot-product softmax, fp16 keys and values | 1,048,576 | 262,144 multiply–adds | 262,144 |
| 64-bit Hamming keys, int8 values, all positions | 294,912 | 8,192 word XOR+popcount | 262,144 |
| 64-bit Hamming keys, top-32 values | 34,816 | 8,192 | 2,048 |
| Routed: 256-bit bundles per 16-token chunk, keep 4 chunks | 12,800 | 2,176 | 4,096 |
| Two-level routed (bundles of bundles) | 6,144 | 512 | 4,096 |

At long context, the value reads dominate. Routing is the lever that removes them; Hamming scoring removes the multiplies from the comparisons.

## 2. Geometries, geodesics and their ranking primitive

| Geometry | Distance | What ranking computes | When it helps |
|---|---|---|---|
| Sphere Sⁿ | θ = arccos⟨q̂,k̂⟩ | ⟨q̂,k̂⟩ (or a Hamming estimate of θ) | Directional similarity: the default for tokens |
| Hyperbolic Hⁿ (hyperboloid) | arcosh(−⟨q,k⟩_L), with ⟨a,b⟩_L = −a₀b₀ + a·b | One Lorentzian inner product | Hierarchies. Trees embed in H² with arbitrarily small distortion (Sarkar), but need growing dimension in Euclidean space. Measured in §4b: 98.5% against 54.1% for dot product at 8 dimensions |
| Euclidean | ‖q−k‖² = ‖q‖² + ‖k‖² − 2⟨q,k⟩ | ⟨q,k⟩ plus norms | Flat, magnitude-bearing data |
| Product S × H × E, learned curvature | Sum of the factors | One inner product per factor | Mixed structure. This is "use the cheapest geometry for the current problem", made precise: each head or factor takes the geometry whose distortion is lowest for its data |

Two further points:
- **A head's QK map is a learned metric.** ⟨W_q x, W_k y⟩ = xᵀ(W_qᵀW_k)y, so each head defines a (generally indefinite) bilinear form G = W_qᵀW_k on the embedding space. The owner's "mutable manifold" is this form. It changes with the layer and, with fast-weight or test-time updates, with the context.
- **Hamiltonian versus dissipative dynamics.** A conservative (Hamiltonian, norm-preserving) memory cannot forget. The review's no-go result (§4 of the review) says every forgetting lane must round and dissipate. Retrieval can be Boltzmann-weighted; the state dynamics must be dissipative.

## 3. Angles by Hamming distance

**Theorem** (Goemans–Williamson; Charikar's SimHash). For a random hyperplane r, P[sign⟨r,x⟩ ≠ sign⟨r,y⟩] = θ(x,y)/π. With m bits, Hamming/m estimates θ/π with standard deviation √(θ(π−θ))/(π√m).
- **Measured check:** θ/π = 0.3255, and the disagreement rate over 200,000 hyperplanes was 0.3258.

**Measured: associative recall.** The task is MQAR-like: D key–value pairs, then 32 queries, each asking for the value paired with a key.
- **Model:** a minimal retrieval head: q = W_q E[token]; k = W_k E[previous token]; the readout is tied to the value embeddings.
- **Training:** 64 pairs.
- **Test:** up to 512 pairs, that is 1,024 candidate positions.

| Scoring (64 dims or ≈64 bits per key) | Recall, 128 pairs | 256 pairs | 512 pairs (1,024 candidates) | Hard top-1 at 512 pairs |
|---|---:|---:|---:|---:|
| Dot-product softmax (float) | 100.0% | 100.0% | 100.0% | 100.0% |
| Spherical geodesic (cosine, float) | 100.0% | 100.0% | 100.0% | 100.0% |
| Hyperbolic geodesic (arcosh, float) | 100.0% | 100.0% | 100.0% | 100.0% |
| Hamming: 64 sign bits (XOR+popcount) | 100.0% | 100.0% | 100.0% | 100.0% |
| E8 block codes (8 blocks of 240 roots ≈ 63 bits) | 100.0% | 100.0% | 100.0% | 100.0% |
| Superposed binding memory, 64-wide state (no attention) | 0.9% | 0.7% | 0.4% | — |

All five scoring geometries recalled every query at 1,024 candidates. The Hamming codes did it with 8 bytes per key instead of 128 (fp16), with XOR plus popcount in place of 64 multiply–adds, and even with hard top-1 selection and no softmax at all. The single-vector binding memory, which has no attention, stayed at chance.

**Precision per bit.** The same task at 512 pairs, with narrower codes (recall with softmax / with hard top-1):

| Width per key | Dot (float) | Cosine (float) | Hyperbolic (float) | Hamming (bits) | E8 blocks (≈bits) |
|---:|---:|---:|---:|---:|---:|
| 8 | 100.0% / 100.0% | 78.6% / 78.5% | 100.0% / 100.0% | 21.9% / 13.1% | 25.8% / 16.7% |
| 16 | 100.0% / 100.0% | 100.0% / 100.0% | 100.0% / 100.0% | 100.0% / 100.0% | 100.0% / 100.0% |
| 32 | 100.0% / 100.0% | 100.0% / 100.0% | 100.0% / 100.0% | 100.0% / 100.0% | 100.0% / 100.0% |

- **Every geometry reaches 100% once the code has enough distinct values** for the keys: 16 bits or dimensions suffice for 512 keys.
- **At 8 bits the binary codes hit their ceiling.** There are 256 sign patterns and 240 E8 roots, fewer than 512 keys. E8 does slightly better than sign bits (25.8% vs 21.9%), as the better spherical code should.
- **Angle-only scoring lost 21% at 8 float dimensions.** Dot product and the hyperbolic distance keep the magnitude, and both held 100%. The radius carries information at low width, which is the owner's original point in a new place.
- **In bits:** a 64-dimension fp16 key is 1,024 bits; 16–64 learned sign bits did the same job here.
- **Caution from the literature (§10):** binary scores used as the *final* attention weights cost quality in language models. §8 measures this directly.

## 4. E8 is a Hamming code lifted to a lattice

- **Construction A.** Take the [8,4,4] extended Hamming code C over GF(2), and let Λ = {x ∈ ℤ⁸ : x mod 2 ∈ C}.
- **Measured.** C has 16 codewords, of weights 0 (×1), 4 (×14) and 8 (×1). Λ has exactly 240 vectors of squared norm 4. Scaled by 1/√2, their inner-product histogram, {±2: 240, ±1: 13,440, 0: 30,240}, is identical to that of the standard E8 root system. (Λ/√2 is E8; this is a classical theorem.)
- **Consequences.**
  - Decoding E8 is decoding a Hamming code plus a parity step.
  - The 240 roots form the best 240-point code on S⁷.
  - Inner products between roots take only the values {0, ±1, ±2}, so block scores are table reads.
  - The owner's "Hamming", "Galois field" and "E8 icosian" ideas are one algebraic object. Sign bits are its 1-bit-per-dimension shadow.

## 4b. Hierarchy: where the arcosh geodesic earns its place

**Task.**
- **Tree:** 4-ary, depth 5, 1,365 nodes.
- **Stored:** each sequence stores D internal nodes, always including the root, with random values.
- **Queries:** each query is a leaf. The target is the value stored at the leaf's *deepest stored ancestor*, so the scores must rank ancestors by depth and above non-ancestors.
- **Model and training:** the retrieval head of §3, trained with 48 stored nodes and tested with 48, 96 and 192.

**Measured** (recall at 48 · 96 · 192 stored nodes; one seed):

| Width per key | Dot (float) | Cosine (float) | Hyperbolic (arcosh, float) | Hamming (bits) | E8 blocks |
|---:|---:|---:|---:|---:|---:|
| 2 | 55.4% · 27.1% · 5.5% | 41.5% · 22.2% · 4.9% | 64.3% · 38.4% · 11.3% | — | — |
| 4 | 75.4% · 54.2% · 24.9% | 57.0% · 45.4% · 23.6% | 94.2% · 85.7% · 65.9% | — | — |
| 8 | 92.7% · 80.3% · 54.1% | 63.1% · 59.3% · 54.6% | 99.6% · 99.4% · 98.5% | 52.7% · 26.5% · 3.8% | 32.0% · 15.9% · 2.4% |
| 16 | 96.4% · 87.8% · 68.2% | 74.5% · 70.8% · 67.5% | 100.0% · 100.0% · 100.0% | 43.0% · 22.7% · 3.5% | 44.0% · 24.2% · 5.3% |

**Correction ([cycle 2](hyperbolic-cycle2-2026-09-26.md)).** The root is always stored, so always answering with the root's value is right 52.7%, 26.3% and 3.6% of the time at 48, 96 and 192 stored nodes. Rows at or near those numbers learned only that rule: 8- and 16-bit Hamming, E8, and dot product or cosine at 2 dimensions. Measured on the non-root queries at 8 dimensions:
- hyperbolic: 99.2%, 99.2% and 98.5%;
- dot product: 87.7%, 77.1% and 53.5%.

The headline below holds.

**Reading.**
- **Hyperbolic scoring wins decisively at low width.**
  - With 8 dimensions it recalled 98.5% at 192 stored nodes, where dot product had 54.1%.
  - Dot product with 16 dimensions reached only 68.2%.
  - Hyperbolic keys at half the width beat flat keys.
- **This is the classical theorem at work.** Trees embed in hyperbolic space with small distortion but need many more dimensions in flat space.
- **Binary codes fail on hierarchy** (3.5–5.3% at 192 nodes, even with 16 bits). Sign codes discard magnitude. In a hyperbolic embedding, magnitude (distance from the origin) encodes depth: the radius is the level of the hierarchy.
- **Where there is no hierarchy, hyperbolic matched the others** (flat recall in §3, text in §8). A hyperbolic head therefore costs nothing on flat data and helps a great deal on hierarchical data. That is the precise form of "use the cheapest geometry for the current problem".
- **Energy.** Fewer dimensions per key at equal recall means fewer bytes read. Ranking uses one Lorentzian inner product; the arcosh only shapes the weights and can be a lookup table.

## 5. Routing: locations advertise their data

**Mechanism.**
1. Every chunk of c positions publishes a bundle. For Hamming codes this is the bitwise majority of its members, the bundling operation of vector-symbolic architectures.
2. A query compares its code with the bundles and keeps the best r chunks.
3. It compares exactly only inside those chunks.
4. Recursion: bundles of bundles give O(log L) routing.

This is the owner's OSPF picture: link-state advertisements are the bundles, and the route computation is a nearest-bundle search.

**Capacity law** (Derived, then Measured). Take c random ±1 codes and their majority bundle (c odd). A member correlates with the bundle at P(the other c−1 sum to zero) ≈ √(2/(πc)).
- **Measured:** 0.271, 0.194 and 0.138 at c = 9, 17 and 33, against 0.266, 0.194 and 0.139 predicted.
- **Consequence.** A query that exactly matches a member beats a non-member bundle by z ≈ √(2/(πc))·√m standard deviations. The correct chunk must also beat the maximum of about L/c null bundles. So advertisement width m must grow roughly as c·log(L/c).

**Measured** (recall at 512 pairs, 1,024 candidates; advertisements trained with a routing loss where marked; training used 16-token chunks):

| Advertisement (codes, bundle) | 16-token chunks, keep 1 (80 comparisons) | keep 4 (128) | 8-token chunks, keep 4 (160) | two-level: super-chunks of 8, keep 2, then 4 chunks (88) |
|---|---:|---:|---:|---:|
| Dot, 64 dims, mean bundle | 10.9% | 29.3% | 33.2% | 22.5% |
| Dot, 256 dims, mean bundle | 13.1% | 34.2% | 39.6% | 25.5% |
| Dot, 1,024 dims, mean bundle | 17.7% | 39.6% | 50.3% | 28.8% |
| Hamming, 64 bits, majority bundle | 13.9% | 33.6% | 41.8% | 24.2% |
| Hamming, 256 bits, majority bundle | 27.5% | 55.1% | 69.9% | 36.6% |
| Hamming, 1,024 bits, majority bundle | 44.2% | 71.5% | 88.6% | 45.9% |
| E8, 64 dims, mean bundle | 17.9% | 39.6% | 48.1% | 29.0% |
| E8, 256 dims, mean bundle | 24.4% | 50.5% | 65.7% | 34.8% |
| E8, 1,024 dims, mean bundle | 35.2% | 63.2% | 80.9% | 41.9% |
| Hyperbolic, 64 dims, Lorentzian-centroid bundle | 10.8% | 27.2% | 31.8% | 20.2% |
| Dot, 64 dims, **advertisements trained** | 67.2% | 87.2% | 98.2% | 54.0% |
| Hyperbolic, 64 dims, **advertisements trained** | 57.0% | 82.3% | 97.5% | 50.0% |
| E8, 64 dims, **advertisements trained** | 27.1% | 53.3% | 69.6% | 37.2% |
| Hamming, 64 bits, **advertisements trained** | 23.9% | 48.2% | 62.3% | 30.6% |
| Hamming, 256 bits, **advertisements trained** | 55.5% | 81.7% | 94.7% | 50.7% |

**Reading.**
- **Untrained advertisements route poorly.** This matches the literature: block summaries fail at exact recall, and Quest scores 0 on ∞-Bench KV retrieval.
- **Among untrained bundles, binary majority votes beat mean vectors at every width.** At 1,024 wide, 8-token chunks, keep 4: 88.6% against 50.3%. The vote stops a few large keys from dominating the advertisement.
- **Width helps, as the capacity law predicts.** Majority bundles went from 41.8% to 69.9% to 88.6% at 64, 256 and 1,024 bits.
- **Training the advertisements helps most.**
  - Trained 64-dimension float keys reached 98.2% while scoring 160 of 1,024 keys, 6.4× fewer, and 87.2% at 128.
  - Trained 256-bit Hamming codes reached 94.7% and 81.7%.
  - At equal bits, trained E8 beat trained sign bits (69.6% vs 62.3%).
  - Trained hyperbolic advertisements (Lorentzian centroids) matched trained dot product (97.5% vs 98.2%).
- **Per bit, binary is the efficient form.** A 64-dimension fp16 bundle is 1,024 bits. Advertisements are stored once per chunk, so the design can pair short binary codes per key with a richer trained advertisement per chunk, amortised over the chunk: a 64-dimension fp16 advertisement per 8 tokens is 16 bytes per token.
- **Recursion costs accuracy at this width.** Two-level routing kept 50–54% at 88 comparisons, because the upper bundles cover 128 positions and the capacity law bites harder. The upper level needs wider codes.
- **No routed setting reached 100%.** Routing is admission. The design keeps exact scoring of the admitted keys and the exact addressed memory for verbatim recall (§9–§10).

## 6. Superposed ("entangled") associations

- **The mechanism.** Store all pairs in one vector, h = Σⱼ bind(kⱼ, vⱼ), with element-wise binding, and read by unbinding with the query. This is the holographic or VSA form of "entangled" pairs, and the limiting case of any fixed-size recurrent state.
- **Capacity.** The crosstalk from D−1 other pairs grows as √(D/m).

**Measured** (recall; chance is 0.2%):

| State width | Recall, 64 pairs | 128 pairs | 256 pairs | 512 pairs |
|---:|---:|---:|---:|---:|
| 64 | 1.9% | 0.9% | 0.7% | 0.4% |
| 256 | 12.7% | 4.9% | 2.2% | 1.2% |
| 1024 | 68.3% | 31.4% | 12.5% | 4.7% |
| 4,096 | stopped at step 250 of 1,000 for time (training loss 0.47 nats at 64 pairs) | | | |

**Reading.** A single superposed state cannot replace attention for recall; its capacity is set by its width. Binding belongs *inside* routing, as the bundles of §5, and in fixed-size state lanes that are not asked to recall arbitrary pairs.

## 7. Primes, CRT, semiprimes and Galois fields

- **Prime signatures.** Encode a set S as N_S = ∏_{a∈S} p_a.
  - Membership is N_S mod p_a = 0; intersection is gcd(N_S, N_T); union is lcm.
  - This is exactly a bitset (bit a ↔ prime p_a) under AND and OR.
  - The bitset costs 1 bit per attribute. The prime product costs Σ log₂ p_a bits and big-integer arithmetic.
  - The efficient form of "semiprime associations" is AND plus popcount, which is the Hamming machinery above.
  - Factoring a semiprime is hard by design. With known candidate primes it reduces to the bit test.
- **CRT / residue number systems.** They give exact, carry-free, digit-parallel integer arithmetic; small residues allow multiplication by lookup table.
  - They are useful for exact accumulation in custom hardware.
  - On a CPU that has a multiplier they save nothing, and comparisons (ranking, softmax) need conversion back.
  - Residues of nearby integers are unrelated, so CRT carries no notion of similarity.
- **Galois fields.** GF(2^k) arithmetic is XOR-based and carry-less. It is the home of Hamming codes, and so of E8 through Construction A. It serves codes, hashing and exact addressing.
- **Where they belong.** Identity, deduplication and exact addresses: the UOR κ-labels and the store-and-recall memory. They also give lattice decoding its error-correcting structure. They do not belong in similarity scoring.

## 8. Real text: geometric attention inside a recurrent language model

**Measured.** The setup is the review's §6.3: WikiText-2 bytes, 3 layers, width 128, and 4.1M training bytes in 128-byte windows. The first two layers are diagonal recurrences. The third is either a third diagonal recurrence or a 4-head attention layer with no position encoding, scored in each geometry. One seed.

| Layers 1–2 → layer 3 | Parameters | Bits/byte, 512-byte windows | At the 128-byte training length |
|---|---:|---:|---:|
| diagonal → diagonal (no attention), seed 0 | 558,720 | 1.869 | — |
| diagonal → diagonal (no attention), seed 1 | 558,720 | 1.873 | 1.900 |
| diagonal → dot-product softmax, no positions | 574,977 | 1.902 | 1.922 |
| diagonal → spherical (cosine) | 574,977 | 1.906 | 1.924 |
| diagonal → hyperbolic geodesic (arcosh) | 574,977 | 1.899 | 1.920 |
| diagonal → Hamming sign codes | 574,977 | 1.916 | 1.932 |
| diagonal → E8 block codes | 574,977 | 1.940 | 1.960 |

**Reading.**
- **Hyperbolic geodesic attention was the best attention variant** (1.899). Dot product (1.902) and cosine (1.906) are within noise of it.
- **Binary codes as the final scores cost 0.014 bits/byte (Hamming) and 0.038 (E8).** The design therefore uses codes to admit candidates and exact scores to rank them (§9–§10).
- **No attention layer beat a third recurrent layer** (1.869–1.873) at this size, with 128-byte training windows. Attention is for recall at range, which this short-window byte model barely needs; §3–§5 test recall directly.

## 9. The design: geometric routed attention

Per layer, per token. The three roles of §10 are kept apart.
1. **Codes (admission).** Ternary maps produce a query code and a key code.
   - Spherical sign codes are the default; E8 block codes are the finer option.
   - A hyperbolic head handles hierarchies. Geometry is chosen per head (§2).
   - Hyperbolic keys need their radius, because sign codes lose the hierarchy (§4b). Store them as a quantized radius plus a direction code. This is the owner's original "keep the radius" quantizer, used where the radius carries meaning (Hypothesis: the cost at 8–16 bits is not yet measured).
2. **Advertisements.** Every c tokens, a chunk publishes a majority bundle, *trained to be routable* (§5).
3. **Route.** Compare the query with the advertisements, recursively (super-chunks, then chunks), keep r chunks, and admit the top k keys inside them by XOR plus popcount.
4. **Rank exactly.** Score only the k admitted keys with int8 inner products; with k = 32–64 this is a few thousand multiply–adds. Weight them with a lookup-table exponential and mix their int8 values.
5. **Recall verbatim** through the exact addressed memory (store-and-recall), which also drafts tokens for speculative decoding.
6. **Mutable state.** Fixed-size recurrent lanes carry continuous context. Their best-evidenced form is the delta rule (§10), which is the owner's "mutable manifold". It should be measured against the diagonal lanes of the low-energy plan.
7. **Recurse.** Each layer repeats the comparison in its own learned space.

Cost (§1): about 12–35 KB read per head per token at 4k context, against 1 MB for fp16 dot-product attention, *if* admission keeps the right keys.

## 10. Prior art

The literature agent retrieved the closest prior work for each idea (report in the review sandbox; figures are quoted from the retrieved papers).

| Owner idea | Closest prior work | Status | What the evidence says |
|---|---|---|---|
| Tokens stored as cheap geometric codes, recalled fast | HashAttention 2412.14468; HAD 2502.01770; QJL 2406.03482; MagicPIG 2410.16179; Reformer 2001.04451 | Published | Works when codes *admit* candidates and exact softmax ranks them: HashAttention loses 0.78 LongBench points at 16× sparsity on Llama-3.1-8B. Using binary scores as the *final* attention weights hurts: binarising the attention matrix drops GLUE to 57.67 in HAD's ablation. Learned codes beat random ones: 32 learned bits outperform random-projection LSH at more than 1,000 bits |
| Hamming "OSPF": locations advertise their data | Kademlia (XOR routing); semantic hashing (Salakhutdinov & Hinton); InfLLM 2402.04617; Quest 2406.10774; RetrievalAttention 2409.10516 | Partly | Query-aware graphs over the KV cache scan 1–3% of keys at small loss (∞-Bench 48.9 vs 50.4). Block summaries fail *exact* recall: on ∞-Bench KV retrieval Quest scores 0.0 and InfLLM 0.5–5.0, against 17.5–30.5 for full attention. An advertisement format trained for routing inside an LM memory was not found |
| Triangulation instead of exact lookup | Nyströmformer 2102.03902; relative representations 2209.15430; Landmark Attention 2305.16300; Vivaldi | Published | Works for encoders, cross-model alignment and admission bounds. No causal-LM evidence |
| Fractal (multi-scale) associations | Squeezed Attention 2411.09688; HiP 2406.09827; NSA 2502.11089 | Partly | Hierarchical routing works at 8–27B. NSA, trained natively with compressed, selected and sliding branches, beats full attention on its benchmarks |
| Geodesics instead of dot products | Hyperbolic Attention Networks 1805.09786 (arcosh scores); HELM 2505.24722; HypLoRA 2410.04010 | Published | Small quality changes (+0.4 to +1.0 BLEU, shrinking with scale) at 1.4–1.8× compute. No energy saving reported |
| Switch to the cheapest geometry | HELM mixture-of-curvature; Gu et al. (ICLR 2019) product manifolds | Partly | Mixtures are chosen for accuracy. Cost-driven switching per query was not found |
| Mutable manifold with a "Hamiltonian heatmap" | Hopfield Networks is All You Need 2008.02217; DeltaNet 2406.06484; Gated DeltaNet 2412.06464; TTT 2407.04620; Titans 2501.00663 | Partly | Test-time-mutable memories are the best-evidenced direction for language: DeltaNet 1.3B reaches perplexity 16.87 vs 16.85 for Transformer++; Gated DeltaNet 16.42 vs 18.53. They still trail on recall unless a few exact-attention layers are kept (SWDE 49.5 → 71.0 with 2 attention layers) |
| Primes, CRT, Galois fields, "entangled" tokens | Residue HDC 2311.04872; RNS accelerators; Hrrformer 2305.19534; prime labelling of trees (ICDE 2004) | Novel as a combination; components published | No language evidence. Prime labels encode ancestry by divisibility; RNS gives carry-free arithmetic but its nonlinearities cannot stay in residues |
| E8 / icosian / quaternionic spaces | E8-lattice LSH (Jégou et al. 2008); QuIP# 2402.04396; NestQuant 2502.09720; GATr 2305.18415 | Partly | E8 works as a quantization and hashing codebook, including for LLM weights and KV caches. **E8 or icosian codes as an attention router were not found** |
| Recursive angle comparison across embedded spaces | Relative representations; Routing Transformer 2003.05997; HiP | Partly | Components only |

**The lesson from the prior art**, which the measurements above agree with: keep three roles separate.
1. Cheap geometric codes **admit** candidates.
2. Exact scores **rank** the few that are admitted.
3. Exact addressed memory handles **verbatim recall**.

## 11. Next gates

Each gate is small and can change the design.

1. **Scale the admission test.** Train advertisements at 256–1,024 bits with the routing loss and measure admission recall at 16k–64k candidates, with 1–3% of keys scored. *Gate:* at least 99% of the true keys admitted.
   - This is the published bar: RetrievalAttention scans 1–3% of keys; HashAttention loses under 1.2 points at 16× sparsity.
2. **Put geometric routed attention in the Rust hybrid.** Measure held-out loss against a dense attention layer at equal parameters, on TinyStories and a code corpus, three seeds. *Gate:* within 0.02 nats.
3. **Measure joules** on the M1 at 4k and 32k context against fp16 dot-product attention, using Phase 0 of the low-energy plan. *Gate:* at least 5× fewer J/token for the attention layer at 32k.
4. **Take the hierarchy result to real data.** §4b is one seed on a synthetic tree. Repeat it with three seeds on real hierarchies: a noun taxonomy, nested code scopes and document sections. Then test the quantized radius-plus-direction code for hyperbolic keys. *Gate:* a hyperbolic head earns its place in the model only if it wins at equal bits on real hierarchical data.
5. **Mutable-manifold lanes.** Delta-rule lanes against diagonal lanes in the hybrid at matched state bytes. The literature (§10) says the delta rule is the strongest recurrent form for language.

Stop rule: if gate 1 or gate 2 fails at three seeds, keep plain int8 attention layers with compressed caches, as in the low-energy plan, and keep geometry in the codebooks.
