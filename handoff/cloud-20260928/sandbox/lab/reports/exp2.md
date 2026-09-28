# exp2 — "Locations advertise their data": can a geometric index admit ≥99% of the right keys while scoring 1–3% of 4K–16K candidates?

Experimentalist report, 2026-09-26. Labels: **M** Measured (commands in §2), **D** Derived, **L** Literature (retrieved this session), **H** Hypothesis. One seed throughout; the repository was not touched.

## 1. Verdict

1. **Yes for flat associative recall, but only if cells are organized by content, not storage order (M).**
   - At 16,384 stored keys, a 256-cell index admitted **≥99.8%** of targets while scoring **1.2%** of keys exactly. That cost **452–456 comparisons per query, advertisements included (2.8% of N)**.
   - This held both for a fixed codebook, learned once and assigned at insertion, and for per-store k-means.
   - Dot and hyperbolic keys tie.
   - Storage-order chunks (the cycle-1 design) never exceeded 55%, even with 10% of keys scored.
   - With trained advertisements (cycle 1's routing loss) the chunks reached at most 90%. The chunk advertisements alone cost 12.5% of N.
2. **The capacity law, measured directly (D+M).** A storage-order advertisement sums unrelated keys, so its cosine with a member is 1/√c: measured 0.251 at c = 16 and 0.127 at c = 64. Content cells reach 0.44–0.65, and they tighten as N grows.
3. **The routing rule matters as much as the geometry (M).**
   - A dot-product advertisement must also publish its squared norm. Ranking cells by ⟨q,c⟩ found the right cell 19.5% of the time at one probe; ⟨q,c⟩ − ‖c‖²/2 found it 98.2% of the time.
   - The Lorentzian score already carries that term through its time coordinate, so **hyperbolic routing worked unmodified**.
4. **The 1–3% result needs queries that are near-copies of their keys (M).**
   - At query–key cosine ≈ 0.89, ≥99% admission cost 4.8–6.8% of N. At ≈ 0.71 it cost 11.8–16.3%.
   - Real LLM attention has such a gap: RetrievalAttention measured that indexes built on keys must scan 30–50% (**L**, arXiv 2409.10516).
   - **A three-stage cascade restores it:** content cells admit 25%, then 256-bit per-key sign codes (XOR + popcount) keep 1%, which is scored exactly. At cosine ≈ 0.71 this admitted 99.8% (dot) and 100% (hyperbolic) of targets at 16K.
5. **Hierarchies need a radius-aware index, and the owner's radius + direction quantizer works as a router (M).**
   - Always admit a "core" of the 0.5% smallest-radius keys; route the rest by direction cells.
   - On this repository's code-scope tree with 1,024 stored scopes, this admitted 84.2% at 3.7% scored, against an 86.6% ceiling (the exact top 3%). Plain Lorentz k-means admitted 61.9%, a sign-code scan 65.3%, storage-order chunks 10.5%.
   - When training made the radius track depth (correlation 0.61–0.64 on a synthetic tree), the core + direction index reached **99% of the ceiling at 1K and 97% at 4K**. Plain k-means reached 42% and 79% of it.
6. **On hierarchies, geometry decides before the index does (M).** With identical training, dot keys ranked the target first 11.4% of the time at 1,024 stored scopes (0% of non-root queries); hyperbolic keys did so 61.2% of the time. No index recovers a target the geometry never ranks first. **≥99% admission is not reachable on hierarchies yet**, because the full scan itself is only 21–85% at 1K–4K.
7. **Hamming codes route poorly as advertisements (M).** Trained 64-bit codes agree with their key's code in about 92% of bits, and majority-vote cell advertisements needed 11–16% of N. A flat scan over the codes admitted 100% at 1%, at the cost of N cheap comparisons. Sign codes are reliable for near-duplicates and unreliable for ranking centroids.
8. **Source order carries hierarchy (M).** Storing scopes in source-text order raised storage-order chunk admission from 10.5% to 42.0% at 3% scored. That is still half of the content index.
9. **Unmeasured:** hierarchical admission at 16K. My learned 87K-node synthetic tree failed at full scan (≈0%).

## 2. Setup

- **Code** (in `$S/lab/exp/exp2/`): `geoattn_exp2.py` (the lead's `geoattn.py` plus parameter saving and tree options), `index_eval.py` (evaluator), `make_tables.py` and `tree_table.py` (tables), `diag_*.py`. Raw results are in `runs/ev_*.json` with their full arguments.
- **Commands** (all with `PYTHONPATH=$S/leadlib:$S/pylib`, one thread and `taskset`):
  - training, e.g. `python3 geoattn_exp2.py --task mqar --kind dot --dk 64 --K 32768 --D 256 --steps 1500 --eval_Ds 256,512 --no_route_eval --save_params runs/mq_dot.npz --out runs/mq_dot.json`;
  - evaluation, e.g. `python3 index_eval.py --params runs/mq_dot.npz --Ns 1024,4096,16384 --stores 2,2,2 --nq 512 --noise 0,0.5,1,1.5 --kmeans 16,64,256 --out runs/ev_mq_dot.json`.
- **Encoders.**
  - **Flat (MQAR):** dk = 64, 32,768 key tokens, trained at 256 stored pairs.
    - Dot and hyperbolic: 1,500 steps; Hamming: 600; the routing-loss run ("trained advertisements"): 800.
    - All reach 100% at 1K candidates.
    - Queries are near-copies of their keys: cos(q, k_target) = 0.995 (dot) and 0.999996 (hyperbolic); the Hamming code cosine is 0.84 (≈92% of bits agree).
  - **Code tree:** 6,284 scopes, dk = 16, trained at 48 or 512 stored scopes.
  - **Synthetic 4-ary tree of depth 8** (87,381 nodes): d = 64, dk = 16, trained at 256 stored nodes with half of the training queries given one stored ancestor.
- **Stores.**
  - 2 stores × 512 queries per N (1 store at 16K for trees).
  - Tree queries are unstored leaves; the target is the deepest stored ancestor.
  - The value-keyed positions of the MQAR layout were dropped: they never outranked a target in any full scan (0.000).
- **Admission.**
  - A query compares itself with every advertisement and admits whole cells in rank order until at least f·N items are in; those items are scored exactly.
  - **Comparisons/query = advertisements compared + items scored.**
  - "Recall" means the target was admitted; "final" means it won the exact score among the admitted items.
- **Families.**
  - Storage-order chunks (c = 8, 16 or 64; mean, majority-vote or Lorentzian bundle).
  - Content cells:
    - k-means in each key's own geometry: Euclidean; Lorentzian k-means with Lorentzian centroids; k-majority for Hamming codes;
    - a fixed codebook learned from every possible key and assigned at insertion;
    - two-level cells (coarse clusters of the fine centroids).
  - LSH sign buckets.
  - Radius-aware indexes: a core plus k-means, or a core plus direction cells.
  - A flat Hamming scan: a sign code for every key.
  - Exact top-f: the ceiling for any admission rule.
- **Query noise** stands in for the query–key gap of real attention: q′ = q + σ‖q‖/√dk·g, with g standard Gaussian, so cos(q′, q) ≈ 0.89 at σ = 0.5 and 0.71 at σ = 1.

## 3. Findings

### F1. Flat recall: cheapest configuration reaching ≥99% admission, near-copy queries (M)

Entries are comparisons per query (% of N); "max" means 99% was never reached with ≤10% of keys scored. A flat Hamming scan reached 100% at 1% scored in every row, plus N cheap code comparisons.

| keys | N | storage-order chunks | LSH buckets | fixed codebook | k-means cells | two-level cells | core + direction |
|---|---:|---|---|---|---|---|---|
| dot | 1,024 | max 55.1% | 177 (17.3%) | 103 (10.0%) | 94 (9.2%) | max 98.3% | — |
| dot | 4,096 | max 55.4% | 392 (9.6%) | 209 (5.1%) | 191 (4.7%) | max 98.9% | — |
| dot | 16,384 | max 55.1% | 803 (4.9%) | **452 (2.8%)** | **456 (2.8%)** | 856 (5.2%) | — |
| dot, trained advertisements | 16,384 | max 90.4% | — | **451 (2.8%)** | **453 (2.8%)** | — | — |
| hyperbolic | 4,096 | max 49.3% | 154 (3.8%) | 129 (3.2%) | 189 (4.6%) | 232 (5.7%) | 131 (3.2%) |
| hyperbolic | 16,384 | max 47.3% | 482 (2.9%) | **453 (2.8%)** | **454 (2.8%)** | 811 (5.0%) | 456 (2.8%) |
| Hamming (64 bits) | 16,384 | max 49.9% | max 97.9% | 1,853 (11.3%) | 2,672 (16.3%) | max 93.7% | — |

Reading:
- **Trained storage-order advertisements** (routing loss, 8-key chunks) at 16K admitted 54% at 1% scored, 70% at 3% and 89% at 10%. Their 2,048 chunk advertisements alone are 12.5% of N.
- **Two-level cells plateaued at 99.6%:** each coarse advertisement summarises ~512 keys, so the capacity law returns.

### F2. The routing rule (M; `diag_two_level.py`: dot keys, 16K, 1,024 cells)

| ranking of advertisements | right cell at 1 probe | at 4 probes | at 8 probes |
|---|---:|---:|---:|
| ⟨q,c⟩ (MIPS, the model's own score) | 19.5% | 55.7% | 78.1% |
| ⟨q,c⟩ − ‖c‖²/2 (L2: the advertisement publishes its norm) | **98.2%** | 100% | 100% |
| cosine | 93.0% | 99.6% | 99.8% |

- **D.** ⟨q,c⟩ is a cell's mean score. It favours small, tight cells, whose centroids have large norms, whatever the query is.
- **D.** −⟨q,c⟩_L = q₀c₀ − q·c contains the norm through c₀ = √(1+‖c‖²). This is why Lorentzian routing needed no correction.
- **L.** The Routing Transformer normalises queries and keys onto the unit sphere before its online k-means for the same reason (arXiv 2003.05997).
- **M.** Routing to centroids through 256-bit sign codes instead cost 1–7 points at 1–3% scored. Codes estimate the small angle to a near-duplicate key well, and the ~50–60° angles to centroids poorly.

### F3. Query–key misalignment breaks 1–3%, and a cascade repairs it (M)

Admission at ≈2% of keys scored, 16K keys (`make_tables.py noise`):

| index | dot σ=0 | dot σ=0.5 | dot σ=1 | hyp σ=0 | hyp σ=0.5 | hyp σ=1 |
|---|---:|---:|---:|---:|---:|---:|
| full-scan accuracy | 100 | 98.9 | 82.4 | 100 | 100 | 94.6 |
| storage-order chunks | 24.9 | 21.8 | 17.9 | 20.2 | 18.0 | 14.7 |
| LSH buckets | 99.4 | 79.4 | 40.6 | 100 | 77.1 | 41.9 |
| k-means cells | 100 | 99.9 | 92.8 | 100 | 100 | 95.0 |
| core + direction | — | — | — | 100 | 100 | 96.4 |
| flat Hamming scan (+N code comparisons) | 100 | 100 | 100 | 100 | 100 | 100 |

Cascade at σ = 1, 16K (`ev_cascade_*.json`): 256 cells admit 25% of keys, codes keep the top 1%, and those are scored exactly.

| keys | code | admission @1% scored | final top-1 (full scan) | cost per query |
|---|---|---:|---:|---|
| dot | 256 bits | **99.8%** | 84.2% (83.2%) | 256 adverts + 4,128 code + 164 exact |
| dot | 64 bits | 89.6% | 79.1% | same |
| hyperbolic | 256 bits | **100%** | 94.3% (94.1%) | same |
| flat 256-bit scan | — | 99.8–100% | — | 16,384 code + 164 exact |

**Bytes read per query and head (D)**, with admission at 1% scored (M, dot keys, 16K, dk = 64, fp16). Value reads are the same for every method.

| method | bytes read | vs full scan | admission, σ=0 | admission, σ=1 |
|---|---:|---:|---:|---:|
| full scan | 2 MiB | 1× | 100 | 100 |
| 256-bit flat code scan | 533 KiB | 3.8× less | 100 | 99.8 |
| cascade (256-bit codes) | 181 KiB | 11× less | 100 | 99.8 |
| 64-bit flat code scan | 149 KiB | 14× less | 100 | 85.2 |
| content index alone (256 cells) | 56 KiB | 36× less | 100 | 65.8 |

### F4. Hierarchies: radius-aware hyperbolic routing (M)

Admission at ≈3% scored (actual fraction in brackets). Core + direction also shows its final top-1. "Ceiling" is admission of the exact top 3%.

| keys (training size), storage | N | full scan (non-root) | core + direction | core + k-means | k-means, no core | sign-code scan | chunks | ceiling |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| code tree: hyp (512), random | 1,024 | 85.0 (81.7) | **84.2 / 83.1** (3.7%) | 82.8 | 61.9 | 65.3 | 10.5 | 86.6 |
| code tree: hyp (512), source order | 1,024 | 85.4 (82.3) | **84.1 / 82.8** (3.8%) | 80.4 | 63.2 | 67.5 | 42.0 | 87.1 |
| code tree: hyp (512), + 1,024 leaf distractors | 2,048 | 83.2 (81.0) | **84.7 / 81.5** (3.4%) | 84.3 | 69.9 | 71.1 | 7.9 | 86.7 |
| code tree: hyp (512), + 2,007 leaf distractors | 4,096 | 75.6 (75.6) | 75.5 / 75.0 (3.2%) | 75.8 | 75.9 | 76.8 | 11.1 | 76.5 |
| code tree: hyp (48), random | 1,024 | 61.2 (52.7) | 79.6 / 60.2 (3.7%) | 78.7 | 57.8 | 65.7 | 8.4 | 85.8 |
| code tree: dot (48), random | 1,024 | 11.4 (0.0) | — | — | 15.5 | 14.6 | 13.6 | 15.1 |
| synthetic: hyp, radius–depth corr. 0.61 | 1,024 | 63.3 (17.0) | **90.0 / 63.5** (3.8%) | 89.6 | 38.2 | 10.5 | 5.2 | 90.9 |
| synthetic: hyp, corr. 0.64 | 4,096 | 20.8 (11.8) | **60.4 / 20.8** (3.7%) | 61.0 | 49.3 | 31.6 | 3.5 | 62.1 |
| synthetic: dot | 4,096 | 20.0 (1.8) | — | 43.2 | 43.9 | 34.5 | 15.1 | 43.0 |

Reading:
- **The core does the work.** It adds 11–52 points over plain k-means, and more the better the radius tracks depth.
  - The learned code-tree keys have radius–depth correlation 0.32, and core + direction retains 97% of the ceiling at 1,024 scopes.
  - The synthetic keys have 0.61–0.64 and retain 97–99%.
  - Plain Lorentz k-means misses shallow ancestors: a deep query is far from a shallow cell's centroid.
- **Sign codes discard the radius** and scan poorly on hierarchies: 10–77%, against 100% on flat recall. This agrees with cycle 2.
- **Training scale matters.** Keys trained with 48 stored scopes kept 52.7% of non-root queries at 1,024 stored (full scan); training with 512 kept 81.7%.
- **A negative.** With uniformly sampled stores on the 87K-node tree, 92% of training targets were the root, and the hyperbolic model learned only the always-root rule (2.6–4.6% non-root; `st8u_hyp.json`). Ancestor-biased sampling partly fixed this: 36% vs 16% for dot on the biased stores.
- At 16K neither model can do the task (full scan ≈ 0%), so admission there is meaningless.

## 4. Proposals

**P1 — A content-addressed memory index per read head (fixed codebook / IVF).**
- **Design.** Assign each key to one of 256 cells at insertion. A cell advertises its centroid plus its squared norm (dot keys) or a Lorentzian centroid (hyperbolic keys). A query probes cells until ~1–2% of keys are in, then scores those exactly.
- **Gain.** ≥99% admission at ~3% of comparisons for copy/recall heads at 16K, and 36× fewer bytes than a full fp16 scan (F1, F3).
- **Cost.** A 256 × dk table per head: 8 KiB at 4 bits and dk = 64.
- **Cheapest falsifier.** Dump (q, k) pairs from the Rust D8 read head on the code validation set and run `index_eval.py`. If admission of the exact top-1 is below 99% at 5% scored, the head is out of distribution (OOD); use P2.

**P2 — Three-stage admission: cells (25%), then 256-bit per-key codes (1%), then exact.**
- **Gain.** Robust to the query–key gap: 99.8–100% at cosine ≈ 0.71 (F3). The middle stage is XOR + popcount, which needs no multiplier and fits D0-b.
- **Cost.** 32 bytes per key, plus code comparisons against ~25% of keys.
- **Cheapest falsifier.** The same dump. The gate is ≥99% admission of the exact top-1, and of the attention mass, at ≤2% scored exactly.

**P3 — Radius-aware routing as the default for Lorentz heads (core + direction cells), with a training pressure that makes radius track depth.**
- **Gain.** It retains 97–99% of the exact-top-f ceiling on hierarchies, where Lorentz k-means retains 42–79% (F4).
- **Cost.** One sort by radius per store.
- **Cheapest falsifier.** On real LM keys, core + direction must beat Lorentz k-means at equal comparisons. Cycle 3 found only a weak radius–depth correlation from next-token training; if that correlation is near zero, the core adds nothing.

**P4 — Routable queries (H).** Add a loss that pulls each query toward the centroid of its target's content cell. This is cycle 1's trained advertisements, moved from chunks to cells, and would shrink P2's first stage.
- **Cheapest falsifier.** MQAR with noisy queries; the gate is ≥99% at 3% scored at cosine ≈ 0.71.

## 5. Recommended next experiments (ranked; each ≤15 min on one core here)

1. **Real attention geometry.** Export q/k from saved cycle-3 D8 checkpoints (dot and Lorentz) on the code validation set. Run `index_eval.py` with every family plus the cascade. This decides between P1 and P2, and measures the real radius–depth link that P3 needs.
2. **Integer routing.** Use 4-bit centroids and int8 queries, and measure the admission loss of P1 and P2 at 16K.
3. **Codebook drift.** Learn the codebook from one set of documents and assign another's keys. The fixed codebook here came from the same key table.
4. **A learnable hierarchy at 16K.** Take a 3-ary tree of depth 9 (9,841 internal nodes) with ancestor-biased sampling and 3,000 steps. This is needed before claiming anything about hierarchical admission at 16K.
5. **Three seeds** for the headline numbers: F1 at 16K and F4 at 1,024.

## 6. Open questions for the owner

1. Does routing by inner products against a small per-head centroid table (4-bit, adds only) count as the "geometric routing" that D0-b allows? Or must routing be XOR + popcount only? The answer makes P1 (fewest bytes) or P2 (no multiplier) the default.
2. Indexing works only if training shapes the geometry for it: queries near their keys' cells, and radius tracking depth. Is an auxiliary index-aware loss acceptable in the D8 learner?

**Resources (M).**
- One-thread process time: training 5,518 s (10 runs, plus ~11 min in 3 stopped runs), evaluation ≈ 1,700 s. Under load 23–28, CPU use is ≈ 1.2 h (estimated from per-step cost).
- Disk: 121 MB in `$S/lab/exp/exp2/`.
