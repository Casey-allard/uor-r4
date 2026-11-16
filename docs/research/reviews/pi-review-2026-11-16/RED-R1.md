# RED TEAM R1: attack on PI-SYNTHESIS.md (16 Nov 2026)

Reviewer: claude (red team R1). Read-only, no builds, no GPU. Repo at origin/main `ef1f4a641a0df1883ccec7844c43e25dd087786c`.
Tags: **[checked p:l]** read this session; **[C2 #n]** C2's verified list; **[est]** my arithmetic; **[mem]** from memory, not verified.

**Verdict: SURVIVES-WITH-FIXES.** "Write, don't search" is right, and B2.3's arithmetic backs it. But as written, the bar can be gamed with a weak denominator, or it cannot be reached with a strong one. The n-gram key as the repo defines it is unordered. The memory budget does not fit. And the P1 gates cannot fail. Fix the six items below before P1 runs.

---

## BLOCKERS

### B1. The bar's denominator, C6, is weak and set by the project itself, so Q1 can be gamed.
- **Weak context.** D87 is about 1.0096e9 donor tokens [D87 row], which come to 1.87e9 V4096 targets [checked claude-406/ORDER.md:17]. That makes a V4096 token about 2 bytes [est]. So an order-6 context is about 12 bytes, or 2-3 words. Compare the 1BW KN 5-gram, which sees about 4 *words* of context with a 793k vocab [C2 #6]. C6 is much closer to a word trigram than to the yardstick model.
- **Free knobs that inflate C6's bits:**
  - smoothing that is not specified ("interpolated");
  - storing only the top-k successors per key, which drops the totals and the N1/N2/N3+ type counts that modified-KN and escape estimation need;
  - count >= 2 pruning.
  Each one makes the denominator worse, and so makes 0.80x easier to reach.
- **Why the ratio means little here.** The 1BW and enwik9 ratios [C2 #6, #7] were measured against *strong* count models. 0.80 of a weak C6 is not 1B-class. A plain modified-KN at order 8-12 may get most of that 20% on its own [est].
- **FIX:**
  1. Replace C6 with **R\***, the strongest lawful count reference: interpolated modified-KN, orders 1..N, with N picked on a DEV-tune slice (expect 8-12 at 2 B/token), full type counts, and no pruning. Build it on wick (59 GB RAM), from suffix-array counts or a disk sort.
  2. Seal R\* before any model work.
  3. Add an outside lawful anchor: PPMd at high order, primed on TRAIN, with only DEV's marginal size counted [mem: PPMd is counts + SEE + exclusion, no logistic].
  4. Restate Q1 as BPB <= 0.80 x R\*, with absolute BPB reported next to it.
  5. Rename the claim to what can be measured: "≥20% fewer bits than the sealed strong lawful count reference". §3.10 forbids running the neural comparator that the "1B" label implies [checked CHARTER.md:39].

### B2. Data leakage will make any long-order count model look 1B-class.
- D87 is donor reasoning plus final answers [checked DECISIONS.md:291]. D86 allowed up to 4 alternative answers per prompt [checked DECISIONS.md:290]. A random byte split puts sibling answers and near-duplicate templates on both sides. Long-order counts and the S5 match then *memorize* DEV.
- **FIX:**
  - split DEV by prompt id, keeping all alternatives of a prompt on one side;
  - dedupe near-duplicates;
  - publish a 13-gram TRAIN/DEV overlap audit with the split. Without it, no BPB ratio is evidence.

### B3. The S2 key, "product of token primes", is an unordered set in this repo.
- The repo's prime products are *squarefree* and stored as prime-factor bitsets [checked crates/uor-r4-training/src/sieve_frame_compiler.rs:264; prose-panel-gate.rs:105].
- A product key loses both order and repetition. For example, "a b c" collides with "c b a", and "the x the" collides with "x the".
- That is correct for S4 conjunctions, which are order-free by design. For n-grams it is wrong. Taking the product mod a table prime adds hash collisions on top.
- **FIX:** key S2 by the ordered packed ids: 12 bits x n, so order 5 is 60 bits in a u64, and orders 6+ use a u128 or (u64, fingerprint). If prime geometry is wanted, use positional primes (a distinct prime per (token, slot)). Add a test that "a b" and "b a" give different keys.

### B4. The P1 kill tests cannot fail, so the top risk (assumption 1) is first tested in P3.
- **P1(a), ft/dev >= 6,420.** A count model with bigram backoff is >= the bigram almost by construction on the templated ft panel. The M1 tables already reached 7,002 there [D83, checked DECISIONS.md:287]. The σ is also wrong for a paired comparison: the band is 2·sqrt(b+c), as W2 used (F ≈ 135, so the threshold would be ≈ 6,435) [checked docs/labs/m1-w2-capacity-capture/README.md:27-30].
- **P1(b), C6 >= 15% fewer bits than C2, and the curve still falling.** A 6-token model beats a token bigram by far more than 15% on any text, and count curves always fall with more data [est]. Neither part can fail.
- **FIX: P1 kill = the linear-mixing ceiling.**
  1. On DEV, fit the bucketed linear mix weights *on DEV itself* by integer EM, over every expert P1 has (all orders and S5). This is an upper bound on what any bucketed linear mixer of those experts can do.
  2. If that bound U > 0.90 x R\*, then "counts + linear mix reach 0.80" is dead in P1. Redirect the plan to a generalizing component.
  3. This costs the same GPU-day as P1 and makes P1 a real test.

---

## MAJORS

### M1. Memory does not fit 4 GB, even on the PI's own count.
- **PI's count.** 3e8 x 8 B at a 0.75 load factor = 3.2 GB, plus S3 0.8, S4 0.13 and 0.1 = **4.2 GB > 4 GB** [est].
- **Realistic count.** N = 1.87e9 targets.
  - Assume distinct grams are 0.6-0.9 N for order 6, 0.5-0.8 N for order 5, 0.35-0.6 N for order 4 and 0.15-0.35 N for order 3, with 50-85% singletons.
  - Then the count >= 2 survivors for orders 3-6 come to **0.6-1.5e9 entries**, not 3e8 [est].
  - Per-context headers (total, N1, N2, N3+) and load factor bring the cost to 12-16 B/entry, so **7-24 GB**.
- **FIX:**
  - store the tables as sorted context tries with Elias-Fano coding (tongrams-class, about 1.5-3 B/gram [mem]) and rank-quantized integer counts;
  - prune by measured BPB cost (entropy pruning) to fit 4 GB, and report that cost against R\*;
  - put orders above 5 on an mmap'd suffix array (about 7 B/token, so about 13 GB on disk [C2 #2 rate, est]). It is read only on long matches.
- P4's 4 GB-resident gate will otherwise fail.

### M2. The S3 design contradicts itself.
- With 2^24 cells per scale and 1.87e9 writes, the average is about 111 writes per cell [est]. So every cell is filled, "Pick on a miss" never fires, and S3 is just a 24-bit hash of the state.
- If S3 is instead keyed by the full 64-128-bit code, misses need a nearest-neighbour search over about 1e9 codes. That is the sublinear index §5.6 forbids.
- **FIX:** choose one design and size it. Make the P2 controls fair:
  - the hash of the last k tokens at the *best* k in 1..12;
  - the **same transport with randomized group elements**, which keeps the prefix dependence and removes the geometry;
  - equal bytes for every arm.
- Otherwise a win for S3 may only mean "a longer context than the hash had".

### M3. The mixer optimizes the wrong objective, and the lawful evidence predicts 0.90-0.95, not 0.80.
- **Wrong objective.** Weights "written as hit counts" optimize top-1, not bits.
- **What lawful count machinery achieves:**
  - On 1BW, the best 2013 combination came to about 10% fewer bits than KN-5 [C2 #6].
  - Goodman's linear mixes of count models (KN, cache, skipping, clustering) give about 1 bit/word over a Katz trigram, and only a few % over KN-5 [mem].
  - PPMd (counts + SEE + exclusion) is 1.472 on enwik9, and paq8px with logistic mixing is 0.998 [C2 #7]. Whether ppmonstr is lawful is unknown [mem].
  - All of these are online, single-file numbers, not held-out ones. Held-out per-document text gets less benefit from adaptation.
- **FIX:** use the lawful tools in order:
  1. bucketed deleted interpolation (Jelinek-Mercer), with weights by integer EM on a held-out TRAIN slice (a ratio of linear sums; no exp);
  2. PPM exclusion plus inheritance, and SEE from integer escape counts;
  3. a *linear-domain* piecewise-linear SSE (APM over quantized p in the linear domain, not stretch);
  4. optionally, binary decomposition of V4096 into 12 binary decisions with Brier-trained linear mixing per bit.
  Budget to 0.90-0.95 x R\* [est], and say plainly that the last 10% needs generalization, not more tables.

### M4. Replies: counts cannot emit the panel values; this is a copy-binding problem.
- The M2-fresh256 values are 7-18-token spans *in the prompt*. The model's first token was neither value in 1,024/1,024 rows [checked docs/research/notebook/cpu-training.md:27]. Forced values were 200-250/256 [:28].
- So S4's token-valued "reply-value counts" cannot produce a novel span. S5's suffix match fails whenever the question rephrases the statement ("my" -> "your").
- **FIX:**
  - make S4 **pointer-valued**: key = the conjunction of question tokens, value = which role or span to copy, counted from TRAIN, after which S5 continues the span;
  - add a shuffled-role control next to the no-memory control;
  - mix ft/M-World reply data into TRAIN with a domain tag, or D87's reasoning-trace style will dominate short replies;
  - define a lawful stop and repetition rule for greedy count decoding, which loops [mem].

### M5. The P0 law review is redundant; use PRLL as it exists.
- The lawful BPB scorer PRLL is merged, and owner Cards 1 and 2 already rule that a linear ratio plus a −log2 measurement is lawful [checked docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md:8-18].
- **FIX:**
  - feed PRLL integer scores over all 4,096 tokens: a full-support order-0 expert in every bucket, so the hit union never gives a target zero mass;
  - scale the scores to at least 2^24, so that PRLL's phi+1 floor does not act as hidden add-one smoothing;
  - compute BPB on the DEV UTF-8 bytes.
- This retires half of assumption 3.

### M6. P5's objective and gate do not match.
- The objective is the rank-margin hinge on top-1; the gate is DEV BPB. BPB can stay flat while the hinge improves.
- **FIX:** gate P5 on the same quantity it trains, or train it on a Brier score over the mixed integer probabilities (lawful), and keep BPB as the reported number.

---

## MINORS

- **m1. GPU pass: the time claim holds; the data path does not.**
  - Sorting 1.5e10 keys at 0.5-2e9 keys/s takes 8-30 s, not 2-8 min [est].
  - The u16 token stream (3.7 GB) fits *on* the GPU. Generate keys per (order, hash-range partition) on the device, sort-reduce, and prune to count >= 2 *before* copy-out. Unpruned output is about 3-5e9 distinct grams x 16 B = 48-80 GB, which is more than wick's 59 GB host RAM.
  - Order 6 needs 72-bit keys, which doubles the sort bytes.
  - The CPU oracle cannot hold the full D87 tables, so run parity on a 1-10% slice.
  - The S3 transport is a causal scan: use an associative scan or parallelism per window.
  - D85: break top-k ties by token id, and make the key function one typed operator shared by CUDA and serving, with a differential test. Counting then replays exactly.
  - Wall time is set by tokenizing, I/O and the trie build, not by the GPU.
- **m2. Speed: 1,000 tok/s is plausible only if the state is resident.**
  - About 20-30 DRAM misses per token, with M1's memory-level parallelism, take a few µs [est].
  - But mmap page faults to SSD (about 50-100 µs each [mem]) at about 20 per token would give about 500-1,000 tok/s.
  - S4 pair enumeration is O(L²) per user turn, not "about 4"; cache it once per turn.
  - Measure with a cold cache and with the full-size tables.
- **m3. R3 (ft/dev 7,002) is a weak 1B criterion.** It is a templated panel that counts on ft TRAIN may pass trivially. Keep it as a floor, not as part of the definition.
- **m4. A 4096² dense bigram table could be read under §3.6 as "a dense model disguised as lookup tables"** [checked CHARTER.md:33]. Store it sparse.
- **m5. The 0.795 yardstick compares paq8px to ppmonstr.** For a lawful pure-count base, the comparable ratio is paq8px against PPMd, 0.678 [C2 #7]. So 0.80 is the *easier* reading.

---

## What the PI missed (existing engines)

- **The Rust suffix array** from Lee et al. 2021's dedup tool, which infini-gram builds on [mem].
  - It gives exact counts at unbounded order, with no hashing, collisions or pruning: the right counting engine and oracle on wick, and R\*'s source.
  - It is about 13 GB for D87 [est].
- **KenLM `lmplz`** builds a modified-KN model on about 1B tokens in CPU-hours [mem]. It is C++, so it is reference and diagnostic only (§3.11), but it is the fastest route to R\*.
- **tongrams (Elias-Fano tries)** for M1.
- **PPMd** as a lawful outside anchor [mem].

---

## Ranked fix list (do these before P1)

1. Ordered n-gram keys (B3).
2. Prompt-level split and overlap audit (B2).
3. Seal R\* as the denominator (B1).
4. Make the P1 kill the DEV-fit EM ceiling U against 0.90 x R\* (B4).
5. Compact storage, with the pruning cost measured (M1).
6. PRLL with a full-support expert (M5).
