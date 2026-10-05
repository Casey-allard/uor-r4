# Literature survey: low-cost in-context recall, induction and long-range retrieval, and what applies to a quaternion-recurrence plus read-mixer stack with no multipliers in serving

**Scope and method.** For the 2025–2026 preprints below I read the abstracts through the research-paper search tool. For the well-known 2019–2025 papers I worked from known content and ids, and I re-read the abstracts of Birdie, the mechanistic SSM evaluation, Birth of a Transformer, DeltaProduct, Memory Mosaics, Just Read Twice and T-MAC. Each claim is labelled as one of:
- **[measured]**: a result the authors report, within their stated scope.
- **[proof]**: a theorem the authors state.
- **[hyp]**: my inference for UOR-R4. It is not a result.

I did not re-measure anything. The internal figures I quote come from the task brief, not from my own check of the repo: MQAR d16 0.81 / d200 0.01, lineage-key 1.000 at every distance, and 1024/1024 unseen pairings.

---

## 1. The recall problem and why fixed-state models fail at it

**Zoology / MQAR**, Arora et al., arXiv:2312.04927.
- **Mechanism:** multi-query associative recall (MQAR) is a synthetic task, shown to predict much of the perplexity gap between gated-convolution models and attention.
- **Result [measured]:** in their study, most of the quality gap between attention and gated convolutions (H3, Hyena, RWKV) on the Pile comes from associative-recall tokens. Input-dependent sparse token mixing closes it.
- **Cost:** attention-like recall needs state that grows with the number of stored pairs.
- **Relevance:** the project's `examples/mqar-bench.rs` (PR #1698) is the right instrument. It measures the deficit this literature identified as the main one.

**Based**, "Simple linear attention language models balance the recall-throughput tradeoff", arXiv:2402.18668.
- **Mechanism:** Taylor-feature linear attention plus sliding-window attention plus short convolutions.
- **Result:** [proof] a lower bound showing recurrent state size must grow with the amount of recalled information. [measured] a Pareto frontier trading recall against state size.
- **Relevance:** the r-layer's fixed quaternion state cannot be the recall store. Recall needs a growing cache read by the a-layers, which matches the architecture as it stands.

**"Repeat After Me: Transformers are Better than State Space Models at Copying"**, arXiv:2402.01032.
- [proof] A two-layer transformer can copy strings of exponential length, while a fixed-state model is limited by its state size.
- [measured] The gap appears on copying and retrieval.

**"The Impossibility Triangle of Long-Context Modeling"**, arXiv:2605.05066. Preprint; abstract read.
- [proof, as stated] No model has all three of: per-step cost independent of length, state size independent of length, and recall that grows with length. With the first two, it recalls at most O(poly(d)/log V) pairs.
- **Relevance [hyp]:** any design that "serves without a growing cache" is capped by this information bound. The exact log sieve and read-mixer caches are not optional. They are what lets the model escape the bound.

**Just Read Twice**, arXiv:2407.05483.
- [proof] Recall hardness reduces to set disjointness, so a recurrent model's memory requirement depends on the order information arrives in.
- [measured] Repeating the prompt (JRT-Prompt) gives +11.0 ± 1.3 points across 16 recurrent LMs. Non-causal prefix processing (JRT-RNN) reaches 99% of Transformer quality at 360M parameters.
- **Relevance [hyp]:** for grounded chat, re-reading the relevant memory record after the question arrives is a cheap lever.

---

## 2. Induction heads and previous-token ("lineage") mechanisms

**Induction heads**, Olsson et al., arXiv:2209.11895.
- **Mechanism:** a two-step circuit. A previous-token head writes "the token before me" into each position. An induction head then matches the current token against those written predecessors and copies the token that followed.
- [measured] Induction heads form during a sharp phase change that coincides with in-context learning ability.

**Birth of a Transformer: A Memory Viewpoint**, arXiv:2306.00802.
- [measured, plus theory] Treats the weight matrices as associative memories. Global bigrams are learned fast, while the induction head develops slowly and depends on properties of the data.
- **Relevance:** the slow, data-dependent formation fits the seed sensitivity reported for `rrarra`.

**"The Token Before the Value Is the Key: How Hybrid Architectures Organize Induction Circuits"**, arXiv:2609.15545. Preprint; abstract read.
- **Mechanism:** splits induction into three roles: Carrying (moving predecessor information), Matching (finding the source by content) and Copying (returning its value).
- [measured] In recurrent–global hybrids, Carrying concentrates in the efficient (recurrent or convolution) layers and is dominated by lag 1. Removing the convolution or masking lag 1 relocates or damages Matching, and changes recall on natural text.
- **Relevance:** this is the closest external match to PR #1701. The term k_t + j·k_{t−1} hard-wires Carrying at lag 1, so the read layer only has to learn Matching and Copying. That explains why recall becomes distance-independent.

**Momentum Attention**, arXiv:2602.04902. Preprint; abstract read. The claims are broad and the venue is unknown, so weight it low.
- **Mechanism:** adds a shear q_t + γ(q_t − q_{t−1}) to queries and keys.
- **Result [measured, as stated]:** induction in a single layer, removing the usual two-layer requirement.
- **Relevance:** independent support for "put the predecessor into the key so one read layer can do induction".

**Mechanistic evaluation of Transformers and SSMs**, arXiv:2505.15105.
- [measured] Only Transformers and Based fully solve associative recall. Mamba and DeltaNet come close; H3 and Hyena fail.
- Causal interventions show Mamba performs induction through its short convolutions, not through the SSM. Remove the convolutions and the mechanism changes.
- **Relevance:** the width-4 causal convolution in the r-layer is probably what produces the short-range recall the bench saw (d16 0.81). It cannot carry a match out to d200 through a decaying recurrence.

**"Anatomy of Associative Recall in Fixed-State Recurrences"**, arXiv:2609.16183. Preprint; abstract read.
- [measured] Under matched training, the short convolution dominates recall (+0.5). The delta-rule-versus-diagonal transition advantage shrinks to +0.03 once both sides have the convolution.
- Distractor-haystack retrieval falls to chance. That failure is interference, not capacity: a distance curriculum takes the same architecture from 0.021 to 1.000.
- Training is a lock-in lottery: lock-in goes from 1/10 seeds to 7/10 with the curriculum (p = 0.02). Gating the curriculum ramp on measured accuracy locks in 6/6 at L = 512.
- **Relevance:** this directly explains why `rrarra` solved on seed 2 but not seed 1. It also says the fix for any remaining fragility lies in data and curriculum, not more architecture.

**Canon layers**, "Physics of Language Models Part 4.1", arXiv:2512.17351.
- **Mechanism:** a weighted sum of neighbouring tokens ("horizontal information flow") inserted into any sequence architecture.
- [measured, synthetic plus academic-scale] Reasoning depth roughly doubles. Canon layers lift NoPE to RoPE level and linear attention to Mamba2/GDN level.
- **Relevance:** the general version of the lineage key. Canon layers place learned neighbour mixing at several sites (before attention, inside the MLP, and so on). The project currently mixes neighbours only in the r-layer convolution and the a-layer key.

**RWKV token shift**, arXiv:2305.13048.
- **Mechanism:** each channel is a learned interpolation between x_t and x_{t−1}, feeding R, K and V. This is a one-tap lineage mix that costs almost nothing.
- RWKV-7, arXiv:2503.14456, keeps token shift and adds a generalized delta rule.

**Short convolutions in H3, Hyena, Mamba, Mamba-2 and Gated DeltaNet:** arXiv:2212.14052, 2302.10866, 2312.00752, 2405.21060, 2412.06464.
- H3's "shift SSM" exists so the next layer can compare the current token with its predecessor. That is lineage again.
- All later models keep a depthwise convolution of width about 4 ahead of the recurrence.

**Data diversity selects induction over shortcuts**, arXiv:2512.18634.
- [proof, single-layer analysis] When trigger distances are diverse, training finds an induction head. When they are concentrated, it finds a positional shortcut and fails out of distribution.
- **Relevance:** MQAR and panel data should vary key–query distance widely. A narrow distance range invites exactly the short-range-only solution the bench measured.

**Attention residual stream on values**, arXiv:2412.15113.
- [measured] Passing values directly between heads speeds up in-context-learning formation at 8M and 1B parameters.
- [hyp] The analogue here is a direct value channel from read layer 1 to read layer 2.

### Why j specifically: a check of the PR #1701 mechanism

This section is my own arithmetic, not a citation.

**[proof]** Left multiplication by the unit quaternion j is a signed permutation:

j·(a + bi + cj + dk) = −c + d·i + a·j − b·k

In served code it is register moves and sign flips, with no multiplier, so it fits D11 exactly.

**[proof]** The operator L_j is skew-symmetric and orthogonal, so ⟨q, j·q⟩ = 0 for every q. The same holds for i and k, and i·q, j·q, k·q and q are pairwise orthogonal. The predecessor channel therefore never interferes with the current token's own channel for the same embedding. A learned two-tap convolution does not have this guarantee.

Interference from other tokens, ⟨k_s, j·k_{t−1}⟩ with s ≠ t−1, is not removed. It only has random-like statistics. That fits a key that generalized to 1024/1024 unseen pairings, though it does not prove why.

---

## 3. Delta-rule and test-time-regression memory

**DeltaNet** (arXiv:2406.06484), **Gated DeltaNet** (arXiv:2412.06464), **DeltaProduct** (arXiv:2502.10297).
- **Mechanism:** the delta rule S ← S(I − β k kᵀ) + β v kᵀ, a "write with erase" that performs one online regression step per token.
- [measured] Better associative recall than additive linear attention. Hybrids with a few attention layers beat transformer baselines at 1.3B parameters and 100B tokens.
- DeltaProduct uses products of n_h Householder reflections per token. [measured] It improves state tracking and length extrapolation.

**Test-time regression**, arXiv:2501.12352.
- [proof-style framework] Linear attention, SSMs, DeltaNet and softmax attention are all special cases of regression performed at test time. It also shows why linear attention misses correlations between tokens.

**Complex KDA**, arXiv:2609.24797, and "The Automaton Underneath", arXiv:2609.18966. Both preprints.
- [proof] Every orthogonal diagonal-plus-rank-1 matrix is a CKDA transition. One CKDA layer tracks any finite subgroup of SO(3).
- [measured] An additive input path b_t "parasitically" stops a Householder recurrence from learning exact automata. Removing it gives exact generalization at 16× the training length.

**Relevance [hyp].** A unit-quaternion rotation of R⁴ is a product of two Householder reflections. The r-layer is therefore in the same family as DeltaProduct with n_h = 2.
- Concretely: if the r-layer adds token input on top of the transport, 2609.18966 predicts that state tracking will not length-generalize. That is testable with a single ablation.
- A delta-style write with erase into a small per-layer key/value state is a candidate for in-layer recall at medium range. Per the Based bound, it cannot replace the read cache.
- Serving would need β and k kᵀ in fixed point. With ternary or table quantization this can be multiplier-free, but it remains unmeasured.

**Titans**, arXiv:2501.00663.
- **Mechanism:** a neural long-term memory updated at test time by surprise, meaning the gradient of a recall loss, with momentum and forgetting.
- [measured] Context beyond 2M tokens and needle-in-haystack wins.
- **Cost:** a gradient step at inference time, which conflicts with D11 unless the update rule is discretized.

---

## 4. Explicit retrieval over a growing cache: sublinear, block-structured and exact

**Memorizing Transformers**, arXiv:2203.08913.
- **Mechanism:** approximate k-nearest-neighbour lookup into a non-differentiable cache of past keys and values.
- [measured] Perplexity keeps improving as the memory grows to 262K tokens.
- **Relevance:** the a-layer cache plus the exact log sieve are this design, with exact rather than approximate kNN.

**Landmark Attention**, arXiv:2305.16300.
- **Mechanism:** a learned landmark token per block. Attention first selects blocks through their landmarks, then attends inside the chosen blocks.
- [measured] Retrieval extends to 32K tokens and beyond.

**Native Sparse Attention (NSA)**, arXiv:2502.11089, and **MoBA**, arXiv:2502.13189.
- **Mechanism:** each query picks the top-k blocks via compressed block summaries; NSA adds a sliding-window branch.
- [measured] Matches or beats full attention, with large speed-ups at 64K.

**Relevance [hyp]** for landmarks, NSA and MoBA: block selection by geometric or prime address (selecting a page, then reading inside it) is explicitly allowed by AGENTS.md. The per-block summary could be a quaternion sum, giving O(√T) or O(log T) reads at serving time.

**Log-linear attention**, arXiv:2506.04761, and **adaptive decay**, arXiv:2605.06946.
- **Mechanism:** a Fenwick-tree hierarchy gives O(log T) states per query at O(T log T) compute.
- [measured] Better recall than fixed-state models. Input-dependent decay per level helps most at long range.
- **Relevance [hyp]:** a Fenwick tree over quaternion state sums is integer-friendly, and it is a principled middle ground between the r-layer's O(1) state and the a-layer's O(T) reads.

**Reformer / LSH attention**, arXiv:2001.04451.
- **Mechanism:** angular LSH buckets the keys so attention is computed within buckets.
- **Cost:** O(T log T).
- **Relevance:** random-rotation LSH on quaternion keys is a sign-bit hash, which costs only XOR and popcount and fits the frozen TLA/R4G1 kernel. Caution: AGENTS.md is explicit that "a prime/hash identity is not a semantic distance". LSH preserves angles approximately; prime identity does not.

**Product-key memory**, Lample et al., arXiv:1907.05242.
- **Mechanism:** the key is split into two halves, each matched against √N sub-keys, then a top-k over the Cartesian product. This addresses N slots for O(√N) work.
- **Relevance [hyp]:** a natural fit for "semiprime experts" and least-divisor addressing, where a composite address factors into two learned sub-codebooks.

**Infini-gram**, arXiv:2401.17377, and **kNN-LM**, arXiv:1911.00172.
- Infini-gram uses suffix arrays for exact ∞-gram counts over trillions of tokens. [measured] It complements neural LMs and lowers perplexity when interpolated.
- kNN-LM interpolates nearest-neighbour next-token distributions. [measured] Large perplexity gains.
- **Relevance:** the exact log sieve is effectively a session-scoped suffix or n-gram store, and these papers support using it as a learned interpolation component, not only a grounding filter.
- [hyp] Keying the sieve by lineage tuples (token, predecessor, pre-predecessor) is the exact-memory analogue of the PR #1701 key. This unites the network-side and memory-side mechanisms under one canonical identity.

**Hierarchical and multiscale memory.** Infini-attention (arXiv:2404.07143) adds compressive memory beside local attention. Memory Mosaics (arXiv:2405.06394) use networks of associative memories, with keys built from leaky averages of the past and values from the next token ("predictive disentanglement"). [measured] Memory Mosaics match transformers on medium-scale language modelling. They are a non-transformer precedent for "key = lineage, value = successor".

---

## 5. Data mix and training procedure: whether recurrent hybrids learn recall at all

- **Birdie**, arXiv:2411.01030 [measured]: reinforcement-learning-tuned mixtures of objectives (copying, infilling, deshuffling) plus bidirectional prefix processing greatly improve phone-book lookup and long-paragraph QA in SSMs, without changing the architecture.
- **2609.16183**: the distance curriculum and accuracy-gated ramp turn a lock-in lottery into reliable lock-in (see Section 2).
- **2512.18634**: a diverse distance distribution selects induction over positional shortcuts (see Section 2).
- **Revisiting AR**, arXiv:2508.19029 [measured]: recurrent models are far more sensitive to learning rate than transformers on associative recall, which can corrupt reported comparisons. Recurrent and attention models also benefit differently from scaling width versus depth.
- **Relevance:** the project's 29M learning-rate sensitivity (the lr5e-4 best in memory notes) and seed fragility are known phenomena in this literature. They are not evidence against the architecture.

---

## 6. Geometric, quaternion, Clifford and hyperbolic models

- **GATr**, arXiv:2305.18415: E(3)-equivariant transformer over projective geometric algebra G(3,0,1). [measured] Strong on physics and robotics tasks. Its attention is a dot product of multivectors and still uses float multipliers.
- **Quaternion RNN/LSTM**, arXiv:1806.04418 [measured]: up to 3.3× fewer parameters on speech. Quaternion NLP networks (Tay et al. 2019, arXiv:1906.04393) report about 75% parameter reduction.
- **QuatRo / CARE**, arXiv:2511.11665: quaternion and Clifford rotary embeddings. The owner excludes RoPE.
- [hyp] The fixed-j lineage trick is not RoPE. It encodes a structural relation (predecessor) with a constant operator, not absolute or relative position.
- **Versor**, arXiv:2602.10195: conformal-geometric-algebra rotor sequence model. Its claims are broad (200× fewer parameters, wins on WikiText), so treat it as unverified.
- **Hyperbolic attention networks**, arXiv:1805.09786 [measured]: gains on hierarchical and relational tasks. The project's Lorentz read score is in this family. I know of no evidence that hyperbolic scoring helps exact recall specifically.

**Relevance:** no geometric-algebra sequence model in the literature I found reports solving long-range MQAR through its geometry alone. The working mechanisms everywhere are lineage features plus a content-matching cache. The project's geometry is best used to supply cheap, orthogonal, exactly invertible lineage operators, which is what fixed j does. Its role is not to be a semantic metric.

---

## 7. Integer-only and LUT serving

- **BitNet b1.58**, arXiv:2402.17764 [measured]: ternary weights match FP16 LLaMA perplexity from about 3B parameters, with matmul reduced to additions.
- **T-MAC**, arXiv:2407.00088 [measured]: bit-wise table lookup replaces multiply-accumulate. BitNet-3B runs at 30 tok/s on one core and 71 tok/s on eight cores of an M2-Ultra, using 70% less energy than llama.cpp.
- Follow-ups: Vec-LUT, arXiv:2512.06443, a vector LUT for parallel tokens with up to 4.2×; and T-MAN, arXiv:2511.11248.
- **Relevance:** this is a measured, M-series-scale existence proof for D11-style serving of the dense parts.
- [hyp] What remains unsolved is the read score, an activation-by-activation product. Options:
  - quantize keys and queries to ternary or 4-bit and compute the score by LUT (D0-b allows weights of 4 bits or less, but here both operands vary at runtime, so it needs product tables);
  - use XOR/popcount sign-LSH scores;
  - use exact-match lineage keys, where an integer equality or hash probe replaces the score entirely for the copy path.

---

## 8. Five ideas most relevant to UOR-R4

**1. Adopt canonical lineage as a first-class, multiplier-free operator, and extend it to n-lets.** [hyp, grounded in 2609.15545, 2505.15105, 2512.17351, 2602.04902 and RWKV token shift]
- Keep k_t + j·k_{t−1} (PR #1701, option #1704). It is a signed permutation and adds no multiplies.
- Test the orthogonal n-let key k_t + j·k_{t−1} + k·k_{t−2}, plus i on the query or value side. Identity, i, j and k give four mutually orthogonal channels per quaternion ([proof] above). That maps directly onto the project's "ordered n-lets" mechanism.
- Bench it on multi-token-key MQAR and on Associative Treecall-style hierarchical keys (2505.15105).
- Also test lineage on values (v_t + j·v_{t+1} is not causal, so instead read the successor via a pointer offset), and Canon-style lineage before the a-layers in every block, not only in the key.

**2. Make recall lock-in deterministic through data, not seeds.** [measured externally: 2609.16183, 2411.01030, 2512.18634, 2508.19029]
- Train with a distance-diverse curriculum gated on accuracy (ramp key–query distance only after the current band locks in).
- Mix in recall-heavy objectives (copying, phone-book, infilling) and use a learning-rate sweep for each layer pattern.
- Report the seed lock-in rate (for example x/10), not single-seed outcomes.
- This should turn the `rrarra` seed-1 failure into a measured probability and test whether lineage is still needed once the curriculum is in place. Prediction: lineage raises the lock-in rate and the curriculum raises it further.

**3. Split Carry, Match and Copy explicitly, and connect the network's matching to the exact log sieve.** [hyp; precedents: kNN-LM 1911.00172, Infini-gram 2401.17377, Memory Mosaics 2405.06394]
- Index the session sieve by the same canonical lineage tuple the network uses, so exact-memory hits and learned read hits share one identity.
- Let a learned gate interpolate between them, as kNN-LM does.
- Add a diagnostic: with the sieve off, measure Matching quality separately from Copy quality using the probes in 2609.15545. Without that split, the "3/109 with the sieve off" result cannot be attributed to either.

**4. Read sublinearly through a hierarchy of geometric pages instead of a flat O(T) scan.** [hyp; precedents: log-linear 2506.04761 and 2605.06946, landmark 2305.16300, NSA 2502.11089, MoBA 2502.13189, PKM 1907.05242]
- Build Fenwick-level or block summaries as integer quaternion sums.
- Select pages by top-k over the summaries, using a product-key split (which maps onto semiprime and least-divisor addressing), then do exact lineage matching inside the chosen pages.
- This is the route to long context on an M1 that stays within D11 and never claims a hash is a semantic metric.

**5. Make the r-layer a delta/Householder memory and remove additive input contamination.** [hyp; grounded in 2502.10297, 2609.24797, 2609.18966, 2412.06464]
- Treat the quaternion transport as a two-reflection (DeltaProduct n_h = 2) transition.
- Ablate the additive input term to test the state-tracking prediction of 2609.18966.
- Optionally add a delta-rule write with erase on a small key/value state for medium-range recall.
- For serving, quantize β and the keys to ternary or 4-bit and run the update through T-MAC-style tables (2407.00088), measuring tokens/s and energy on the M1 under D11.

**Single highest-value next measurement [hyp].** Run idea 2 (multi-seed lock-in rate under a distance curriculum) against idea 1 (lineage n-lets), crossed, on `examples/mqar-bench.rs`. Then carry the winner into the 232-request panel with the sieve both off and on. That separates "the architecture now has induction" from "the data and seed happened to find it".