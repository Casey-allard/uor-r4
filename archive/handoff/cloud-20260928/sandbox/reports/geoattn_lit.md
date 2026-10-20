# Prior art for the UOR-R4 geometric-attention ideas

This is a literature report for the first-principles review, retrieved on 2026-09-26 via alphaXiv, Firecrawl and Parallel search.

- **Sources.** Numbers are quoted from the retrieved paper text.
- **Labels.** **UNVERIFIED** means I could not retrieve the source. *Analysis* means my own reasoning, not a paper's claim.
- **Hardware.** No retrieved timing or energy figure was measured on an Apple M1. Every figure comes from an A100, H100, L20 or RTX 4090 GPU (sometimes with CPU offload) or from simulated hardware.

## Headline findings

1. **Almost every component idea is already published. The owner's specific combinations were not found.** The missing combinations are:
   - icosian/H4 routing;
   - "entangled" prime/CRT tokens;
   - OSPF-style advertisement inside an LM memory;
   - cost-driven switching between geometries.
2. **What works in LLMs: cheap codes admit candidates, then exact softmax ranks them.** Examples are HashAttention, MagicPIG, Quest, RetrievalAttention and NSA.
   - Using binary scores *as* the attention weights makes quality collapse.
   - Learned codes beat fixed ones: 32 learned bits outperform random LSH at more than 1000 bits.
3. **Coarse block summaries fail at exact recall.** On ∞-Bench KV retrieval, Quest scores 0.0 and InfLLM 0.5–5.0, against 17.5–30.5 for full attention. Exact addressed memory therefore stays necessary.
4. **Non-Euclidean geometry changes quality only a little and costs more compute.** This covers hyperbolic, mixed-curvature, quaternion and geometric-algebra models.
   - In pretraining and machine translation (MT), quality moves by −4.3 to +1.0 points.
   - With LoRA fine-tuning, the gain reaches +7.5.
   - Compute is 1.4–1.8× higher, and no paper shows an energy saving.
5. **The strongest language-modelling evidence is for test-time-mutable memory** (delta rule, TTT, Titans). These models match or beat Transformer++ perplexity. They trail on recall unless a few exact-attention layers are kept.

---

## 1. "Store tokens in a different geometry and recall them quickly"

- **HashAttention** (2412.14468, ICML'25)
  - **Mechanism:** learned MLPs map q and k to 32-bit codes. The score is −popcount(q⊕k), and exact softmax runs over the top-k.
  - **Measured:** Llama-3.1-8B at 16× sparsity loses 0.78 points on LongBench and 1.13 on RULER@16K. Task fine-tuning allows up to 32×.
  - **Cost:**
    - 32 bits per token per head, about 1.6% of an fp16 128-d key (my arithmetic).
    - On an A100, attention latency is up to 4.3× lower and throughput 3.12× higher at 32K.
    - The mapping MLP dominates below about 8K context.
  - **Limits:**
    - Random-projection LSH fails to match it even with more than 1000 bits.
    - Codes trained at ≤64K do not transfer to 128K.
- **HAD** (2502.01770)
  - **Mechanism:** Q and K are sign-binarised by distillation; softmax runs over the top 30.
  - **Measured** (HAD vs full softmax):
    - GLUE: 80.81 vs 82.59.
    - DeiT-B: 79.24 vs 81.74.
    - DeiT-T: 66.59 vs 72.01.
  - **Cost:** a simulated content-addressable memory (CAM) uses 79% less area and 87% less power.
  - **Ablation:** binarising the attention matrix itself collapses GLUE to 57.67.
  - **Limit:** encoder-only. The authors call decoder LLMs "very sensitive to small loss increases".
- **EcoFormer** (2209.09004)
  - **Mechanism:** 16-bit kernelised hash codes give linear attention in which multiplications become additions.
  - **Measured:** ImageNet 70.44 vs 70.77; LRA Retrieval 78.67 vs 79.62.
  - **Cost:** estimated on-chip energy is 73–94.5% lower. The estimate uses 45 nm figures (an FP16 add costs 0.4 pJ, a multiply 1.1 pJ).
  - **Limits:** a severe drop on short sequences; no language-modelling results.
- **BiBERT** (2203.06390)
  - **Mechanism:** a fully binary BERT.
  - **Measured:** GLUE 63.2 vs 83.9. Binarising attention causes the largest single loss.
  - **Cost:** 56.3× fewer FLOPs.
- **QJL** (2406.03482)
  - **Mechanism:** sign bits of a Johnson–Lindenstrauss (JL) projection of each key, with the query kept at full precision. The estimator is unbiased.
  - **Measured:** at 3 bits per number, the KV cache shrinks more than 5×. LongBench scores stay within about ±2.6 points of the baseline.
- **Reformer** (2001.04451)
  - **Mechanism:** angular LSH with sort-and-chunk; O(L log L).
  - **Measured:** a model trained with full attention scores 52.5% on the duplication task with one hash round and 94.8% with eight. Training with LSH reaches about 100%.
- **MagicPIG** (2410.16179)
  - **Mechanism:** SimHash tables on the CPU (8–11 bits × 75–300 tables) with importance *sampling* instead of top-k. Keys must be centred; without centring, needle-in-a-haystack (NIAH) accuracy is about 0.
  - **Measured** at about 2% of attention compute (RULER, 64K context):
    - Llama-3.1-8B: 86.1 falls to 83.6–84.8.
    - 70B: 89.2 falls to 86.7–88.8.
    - It beats exact top-k on aggregation tasks by up to 8 points.
  - **Cost:** 54 ms per token at 96K on an RTX 4090, with 14–28 GB of hash tables.
- **Routing Transformer** (2003.05997)
  - **Mechanism:** online spherical k-means routing; O(n^1.5·d).
  - **Measured:**
    - WikiText-103 perplexity 15.8 vs 18.3 for Transformer-XL.
    - On CIFAR-10, random routing is worse than local attention.

**Verdict: already published.**
- **Long-context retrieval:** it works when learned or centred codes only *admit* candidates. It fails when the code does the final ranking.
- **Language modelling:** it is competitive only when routing keeps softmax inside each bucket. No Hamming-only attention LM was found.

## 2. "Hamming distances acting like OSPF … locations advertise their data, or data carries its location and history"

- **Kademlia** (IPTPS 2002)
  - **Mechanism:** 160-bit IDs, XOR distance and one k-bucket per distance range. This is the canonical XOR-metric router.
  - The paper's O(log n) lookup claim is not in my retrieved excerpt.
- **Semantic hashing** (Salakhutdinov & Hinton, IJAR 2009)
  - **Mechanism:** learned 20-bit document addresses. Recall enumerates the Hamming ball around the query address.
  - **Measured** on 402K documents:
    - a shortlist of about 2,500 documents takes 0.5 ms (plus 10 ms of TF-IDF re-ranking), vs about 500 ms for LSH;
    - filtering through the shortlist *raised* TF-IDF accuracy.
  - **Caveat** (authors): "the converse is not necessarily true", i.e. similar content need not get similar addresses.
- **Attention approximates Sparse Distributed Memory** (2111.05498)
  - Softmax approximates Sparse Distributed Memory (SDM) Hamming-circle intersections.
  - No language-modelling benchmark.
- **HNSW** (1603.09320)
  - **Mechanism:** a hierarchical proximity graph. Search is O(log N) for low intrinsic dimension and degrades in high dimension.
  - **Cost:** 60–450 bytes per object.
- **RetrievalAttention** (2409.10516)
  - **Finding:** queries and keys are out-of-distribution. An IVF index must scan 30–50% of keys to reach 0.95 recall.
  - **Mechanism:** a query-aware graph built from the prefill queries, which scans 1–3% of keys.
  - **Measured** on Llama-3-8B (vs full attention):
    - ∞-Bench: 48.9 vs 50.4.
    - RULER: 84.70 vs 86.54.
    - KV retrieval: 9.0 vs 17.5.
  - **Cost:** 0.188 s per token at 128K on an RTX 4090.
- **InfLLM** (2402.04617)
  - **Mechanism:** 128-token blocks, each "advertised" by 4 representative tokens.
  - **Measured:**
    - Passkey retrieval is 100% at 1,024K tokens.
    - In RetrievalAttention's re-evaluation: RULER 43.74 vs 86.54, and KV retrieval 0.5–5.0.
- **Landmark Attention** (2305.16300)
  - **Mechanism:** a trained landmark token per 50-token block gates retrieval.
  - **Measured:** PG-19 perplexity 14.72 vs 14.55 for Transformer-XL; passkeys retrieved beyond 32K.
- **Quest** (2406.10774)
  - **Mechanism:** each 16-token page stores min/max key bounds, which give an upper-bound score.
  - **Measured:**
    - Passkey at 100K: 96%.
    - LongBench is near-lossless.
    - KV retrieval scores 0.0.
  - **Cost:** the attention kernel is 7.03× faster, but end-to-end speedup is only 1.74–2.23×.

**Verdict: partly published.**
- **Published pieces:** XOR routing, learned binary addresses, block "advertisements" and graph routing over the KV cache.
- **Not found:**
  - link-state route tables inside an LM;
  - tokens that carry their own location or history as the retrieval key.
- **Long-context retrieval:** it works while scanning 1–3% of keys, provided the index is query-aware.

## 3. "Geometric triangulation instead of exact lookup"

- **Nyströmformer** (2102.03902)
  - **Mechanism:** 64 segment-mean landmarks; O(n).
  - **Measured:** SST-2 91.4 vs 90.0; LRA 58.95 vs 58.77.
  - **Cost:** 12.7× faster at 8192 tokens.
  - **Limit:** encoder-only. *Analysis:* the segment means pool future tokens, so the method is not causal.
- **Relative representations** (2209.15430)
  - **Mechanism:** each point is encoded by its cosines to a set of anchors.
  - **Measured** on zero-shot model stitching (relative vs absolute): TREC 75.89 vs 21.49. In-domain accuracy costs a little: 90.06 vs 91.54.
- **Vivaldi** (SIGCOMM 2004)
  - **Mechanism:** network coordinates learned by spring relaxation.
  - **Measured:** 11% median error on 1740 hosts; the Internet violates the triangle inequality.
- Quest's min/max bound (section 2) is triangulation used only to admit pages.
- **UNVERIFIED:** P-GNN anchors, Landmark MDS, GNP.

**Verdict: already published.**
- **Where it works:** encoders, cross-model alignment and page-admission bounds.
- **Language modelling:** no causal-LM evidence.
- **Retrieval:** *(analysis)* coordinate error around 10% is tolerable for admission. Where approximation reaches final recall, exact KV retrieval fails (Quest 0.0).

## 4. "Fractal associations"

My searches found only hierarchical, multi-scale routing. I ran no "fractal attention" search, so that part is **UNVERIFIED**.

- **Squeezed Attention** (2411.09688)
  - **Mechanism:** offline k-means and a 2-level centroid lookup; O(c′ log L + k).
  - **Measured:** LongBench drops ≤0.11 points at 3.1× KV reduction.
  - **Limit:** it needs centroids for about 5% of tokens; at 1% the score falls to 19.55 from 43.07.
- **HiP** (2406.09827)
  - **Mechanism:** tree search over block representatives, relying on attention locality.
  - **Measured:** Llama-3.1-8B keeps 96% of its LongBench performance.
  - **Cost:** decode attention is 16.5× faster at 32K.
- **NSA** (2502.11089)
  - **Mechanism:** three gated branches trained natively (compressed tokens, selected blocks and a sliding window).
  - **Measured** at 27B MoE (3B active):
    - general benchmarks 0.456 vs 0.443 for full attention;
    - LongBench 0.469 vs 0.437;
    - perfect needle retrieval at 64K.
  - The authors report that clustering-based selection is hard to train.

**Verdict: partly published** (multi-scale hierarchy).
- **Long-context retrieval:** it works at 8B–27B, given attention locality and enough representatives.
- **Language modelling:** NSA beats full attention when trained natively.

## 5. "Geodesics instead of dot products"

- **Hyperbolic Attention Networks** (1805.09786)
  - **Mechanism:** the score is α = f(−β·arccosh(−⟨q,k⟩_M) − c). This is already "arcosh" scoring.
  - **Measured** on WMT14 En-De BLEU: 17.3 → 18.0 for the tiny model and 27.1 → 27.5 for the base model. Gains shrink with scale.
- **HELM** (2505.24722)
  - **Mechanism:** a fully hyperbolic LLM with mixture-of-curvature experts.
  - **Measured:** at 1B, a 5-benchmark average of 24.9 vs 23.9 for DeepSeekV3-1B, near the 25% chance level.
  - **Cost:** 1.43–1.55× runtime and 1.5–1.8× training time.
- **HypLoRA** (2410.04010)
  - **Mechanism:** hyperbolic LoRA fine-tuning. Token embeddings measure as strongly hyperbolic.
  - **Measured** on arithmetic (HypLoRA vs LoRA): Qwen2.5-7B 88.3 vs 80.8.
  - **Limit:** results are sensitive to curvature.
- **Transformer Dissection** (1908.11775)
  - A distance-based RBF kernel matches the exponential dot-product kernel: WikiText-103 perplexity 24.13 vs 24.10.
- **GATr** (2305.18415)
  - The authors report attention "that use[s] the geometric product rather than the dot product" gave "a worse performance in practice".
- *Analysis:* in a single constant-curvature space, the geodesic is a monotone function of an ambient bilinear form (arccos, 2·arccos|q·k| on S3, or arccosh). So the geodesic top-k is the same as the dot-product top-k. It still computes that product and adds a transcendental function.

**Verdict: already published.**
- **Language modelling:** gains are small (+0.4–1.0 points) and shrink with scale, at 1.4–1.8× cost.
- **Efficiency:** no evidence of savings.

## 6. "Switch between geometries and use the cheapest for the current problem"

- **HELM mixture-of-curvature experts:** varying curvature beats constant curvature (24.1 vs 23.7). The motive is accuracy, not cost.
- **Gu et al.** (ICLR 2019, OpenReview HJxeWnCcF7): product manifolds with learned curvature cut graph distortion by 32.55%.
- **McNeela et al.** (2401.15478), negative evidence: mixed curvature loses to a Laplacian baseline out of distribution.
- **Closest cost-aware switch:** NSA's learned per-token gate.
- **Router cost:** HashAttention's router costs more than it saves below about 8K context.

**Verdict: partly published** (mixtures chosen for accuracy).
- **Not found:** switching per query to the *cheapest* geometry, and no evidence either way on whether it works.

## 7. "Route over a mutable Riemannian manifold with a Hamiltonian heatmap"

- **Hopfield Networks is All You Need** (2008.02217)
  - **Mechanism:** the energy is E = −lse(β, Xᵀξ) + ½ξᵀξ. One update step is exactly softmax attention.
  - **Measured:** exponential storage capacity.
- **Energy Transformer** (2302.07253)
  - **Mechanism:** recurrent descent on an energy function.
  - **Cost:** about 2× the attention FLOPs.
  - **Limit:** no language results.
- **Sinkformers** (2110.11773): in the mean-field limit, doubly stochastic attention follows the heat equation. Accuracy numbers are **UNVERIFIED**.
- **Fast weight programmers** (2102.11174)
  - **Mechanism:** linear attention is an outer-product memory; the paper adds a delta rule.
  - **Measured:** WikiText-103 perplexity 29.4, where plain linear attention exceeds 260.
- **DeltaNet** (2406.06484)
  - **Measured** at 1.3B parameters on 100B tokens:
    - perplexity 16.87 vs 16.85 for Transformer++;
    - recall (SWDE) 49.5 vs 66.6, rising to 71.0 with 2 attention layers.
- **Gated DeltaNet** (2412.06464): perplexity 16.42 vs 18.53 for Transformer++.
- **TTT** (2407.04620): perplexity keeps falling out to 32K context, where Mamba plateaus.
- **Titans** (2501.00663)
  - **Mechanism:** surprise-driven memory updates with momentum and forgetting.
  - **Measured** at 760M parameters: perplexity 18.61 vs 25.21 for Transformer++.

**Verdict: partly published.**
- **Energy reading:** a reformulation of attention with no efficiency gain.
- **Mutable memory:** test-time-mutable memory is the best-evidenced idea in this report for language modelling. Recall still needs hybrids with exact attention.
- **Not found:** a learned Riemannian metric updated at inference and used for routing.
- **UNVERIFIED:** I did not search for Hamiltonian or symplectic networks.

## 8. "arcosh quantum tokens held as entangled semi-primes (modular factoring, CRT, Galois fields)"

- **Binary spatter codes** (Kanerva 2009; content seen only via PMC9869149): XOR binding with Hamming similarity.
- **HRR** (Plate 1995) and **TPR** (Smolensky 1990): bibliographic records only.
- **Hrrformer** (2305.19534)
  - **Mechanism:** keys and values are bound via FFT and unbound with q.
  - **Measured:** LRA 60.83 vs 54.39 for a Transformer.
  - **Limit:** classification only. It needs a softmax clean-up, because raw unbinding gives chance accuracy.
- **TP-Transformer** (1910.06611): role binding lifts Math dataset interpolation to 81.92 from 79.54.
- **Residue HDC** (2311.04872)
  - **Mechanism:** phasor residues give carry-free CRT arithmetic, decoded by a resonator network.
  - **Limit:** no language results.
- **Mirage** (2311.17323)
  - **Mechanism:** residue number system (RNS) arithmetic.
  - **Measured:** trains DNNs at FP32-level accuracy.
  - **Cost:** 23.8× faster in photonic simulation.
  - **Limit:** nonlinearities cannot stay in RNS.
- **Prime labelling** (Wu, Lee & Hsu, ICDE 2004)
  - **Mechanism:** a node's label is its parent's label times its own prime. Ancestry is tested by divisibility.
  - **Cost:** labels reach 28–35 bits at depth 7–9.
- **UNVERIFIED:** QNLP or "quantum" tokens, RSA accumulators, CryptoNets, and neural-network hardware using Galois-field arithmetic.

**Verdict: novel as a combination, as far as found.** Every component is published.
- **Language modelling:** no evidence.
- *Analysis:* CRT and prime products give exact identity, membership and carry-free arithmetic. They impose no similarity between different tokens.

## 9. "E8 / icosian / spherical / quaternionic spaces"

- **Jégou et al.** (ICASSP 2008, "Query-Adaptative LSH"): E8 lattice hashing of 8-d subvectors. E8 decoding takes 104 operations vs 3595 flops for the Leech lattice.
- **Andoni & Indyk** (FOCS 2006): practical variants use Leech-lattice decoders.
- **Construction A:** the [8,4,4] Hamming code yields E8 (Conway & Sloane, seen via secondary sources).
- **QuIP#** (2402.04396)
  - **Mechanism:** a Hadamard rotation plus the E8P codebook.
  - **Measured:** Llama-2-70B at 2 bits reaches perplexity 3.91 vs 3.12 at FP16.
- **NestQuant** (2502.09720)
  - **Mechanism:** nested Voronoi codes on the E8 (Gosset) lattice.
  - **Measured:** Llama-3-8B with 4-bit weights, activations and KV cache reaches perplexity 6.63 vs 6.14 at FP16.
- **GATr** (2305.18415): an E(3)-equivariant transformer in projective geometric algebra G(3,0,1). No language results.
- **Quaternion Transformer** (1906.04393)
  - **Mechanism:** Hamilton-product weight sharing, with 75% fewer parameters.
  - **Measured:** En-Ro 18.5 vs 22.8 BLEU.
- **Icosian/H4 attention or hashing:** **not found**.

**Verdict: partly published.**
- **E8:** demonstrated as a quantisation and LSH codebook, including for LLM weights and KV caches.
- **Routing:** no LM found uses E8 as its attention router.
- **Icosian/H4 routing:** novel as far as found, with no evidence.

## 10. "A point of recursion that compares token associative angles at each joint embedded space"

Components found:
- anchor-angle codes that transfer between independently trained spaces (relative representations, section 3);
- angular bucketing (Reformer and Routing Transformer, section 1);
- recursive descent through a hierarchy (HiP and HNSW, sections 2 and 4).

**Verdict: partly published** (components only).
- **Not found:** a recursion that compares angles across stacked embedding spaces.
- **Language modelling:** no evidence.

---

## Cross-cutting implications for UOR-R4 (analysis)

1. **Keep three roles separate.** The literature's failures come from merging them.
   - A cheap geometric or bit index handles **admission**.
   - Exact scoring handles **ranking**.
   - Exact addressed memory handles **verbatim recall**.
2. **Train codes on the query distribution.** Fixed prime, zeta or hash codes on their own should be expected to behave like random LSH.
3. **Report end-to-end gains, not kernel gains.**
4. **Treat M1 energy as unmeasured.**

## Summary table

| Owner idea | Closest prior work (ids) | Status | Evidence it works | Biggest open risk |
|---|---|---|---|---|
| Store tokens in another geometry and recall them fast | 2412.14468, 2502.01770, 2209.09004, 2406.03482, 2410.16179, 2001.04451 | Published | Yes for long-context admission: −0.8/−1.1 points at 16× (8B); −0.4 to −2.5 RULER-64K at ~2% compute | Binary scores used as final weights collapse; fixed codes need >30× the bits |
| Hamming "OSPF" routing (advertise/carry location) | Kademlia 2002, semantic hashing 2009, 2111.05498, 1603.09320, 2409.10516, 2402.04617, 2406.10774 | Partly | Yes: 1–3% of keys scanned, −1.5 ∞-Bench (8B) | q/k distribution mismatch; block advertisements fail exact KV recall (0.0–5.0 vs 17.5–30.5) |
| Triangulation instead of exact lookup | 2102.03902, 2209.15430, Vivaldi 2004, 2305.16300 | Published | Encoders and admission only | No causal-LM evidence; ~10% coordinate error cannot replace exact recall |
| Fractal associations | 2411.09688, 2406.09827, 2502.11089, 1603.09320 | Partly (hierarchies); fractal search UNVERIFIED | Yes: HiP keeps 96% of LongBench; NSA beats full attention at 27B | Needs attention locality and representatives for ~5% of tokens |
| Geodesics instead of dot products | 1805.09786, 2505.24722, 2410.04010, 1908.11775 | Published | Small: +0.4–1.0 (MT/pretraining), +2.3–7.5 (LoRA) | 1.4–1.8× cost; ranking reduces to a bilinear form (analysis) |
| Switch to the cheapest geometry | 2505.24722, Gu 2019, 2401.15478, 2502.11089 (gate) | Partly; cost-driven switching not found | None for cost-driven switching | Router overhead exceeds savings; out-of-distribution overfit |
| Mutable manifold with Hamiltonian heatmap | 2008.02217, 2302.07253, 2110.11773, 2102.11174, 2406.06484, 2412.06464, 2407.04620, 2501.00663 | Partly | Strongest LM evidence: delta-rule/TTT models match or beat Transformer++ perplexity at 0.76–1.3B | Recall gap without attention hybrids; energy formulations add ≥2× FLOPs |
| arcosh semi-prime / CRT / Galois tokens | 2311.04872, 2311.17323, 2305.19534, 1910.06611, Wu 2004, 1805.09786 | Novel combination; components published | None for LM | Identity is not similarity; label growth; nonlinearities outside RNS |
| E8 / icosian / spherical / quaternionic | Jégou 2008, FOCS 2006, 2402.04396, 2502.09720, 2305.18415, 1906.04393 | Partly; icosian not found | Yes for weight and KV compression; none for routing | No router evidence; quaternion MT loses up to 4.3 BLEU |
| Recursive angle comparison across joint spaces | 2209.15430, 2003.05997, 2001.04451, 2406.09827 | Partly | Model stitching yes (75.89 vs 21.49); LM none | Untested in causal LMs |

## UNVERIFIED / not retrieved

- **Not retrieved at all:**
  - P-GNN (1906.04817), Landmark MDS, GNP
  - CryptoNets, RSA accumulators, neural-network hardware using Galois-field arithmetic
  - Res-DNN and RNSnet internals
  - QNLP "quantum" tokens and semi-prime resonator factorisation
  - Hamiltonian and symplectic networks
  - "Fractal attention"
- **Partly retrieved:**
  - Sinkformer accuracy numbers
  - Kademlia's O(log n) statement
  - Conway & Sloane and Kanerva 2009, whose primary texts I saw only through secondary sources
  - Jégou et al.'s table values
