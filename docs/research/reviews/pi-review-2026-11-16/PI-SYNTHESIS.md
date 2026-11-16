# PI synthesis: the fastest lawful path to a 1B-equivalent model (16 Nov 2026)

Author: claude (PI role), read-only. Repo at origin/main `ef1f4a641` (worktree payload-probe-387). No builds, no GPU.

Tags. **[checked p:l]** = I read that line this session. **[survey X]** = taken from survey file X, which marks it checked; I did not re-read it. **[est]** = my arithmetic or a guess at an unmeasured quantity. **[hyp]** = a hypothesis that a phase below tests. **[mem]** = from memory, not verified.

Surveys used: A1, A1b, A1c, A2 (+A2b), B (+B2), C (+C2), architect SYNTHESIS of 15 Nov, and the gap analysis of 16 Nov. Where C and C2 disagree, I use C2.

---

## 0. The answer in one paragraph

Stop trying to *search* a billion coordinates, and *write* them. On the one panel every line shares, count tables beat every trained model: on ft/dev the query-token bigram gets 6,300 top-1 and the trained Q4 writer 149 [checked docs/labs/m1-w2-capacity-capture/README.md:13-16]. Scored coordinate search runs at about 0.5-1.1M trials per GPU-day, so touching 1e9 coordinates once takes 900-2,000 GPU-days [survey B2.3]. A GPU counting pass over D87 should take hours [est, §3.4].

So the model is a **geometric count machine**. It has four parts:
- exact n-let count rows, keyed by prime products;
- count rows keyed by the *geometric address* of the context, meaning the sign code of the SpiralCore-transported state, read by the Pick XOR+popcount read;
- conjunction rows keyed by token pairs;
- an in-session match/copy read.

A count-written integer mixer combines them, and the readout comes from the union of table hits. All of this state is WRITTEN in one or a few GPU passes. Only a core of at most 1e5 coordinates (the transport and address parameters) is SCORED, with the existing rows-K engine, and only after the written model has set the floor. The earliest test costs about 1 GPU-day and 2-3 units: write the count tables on wick, then measure ft/dev top-1 against 6,300 and 7,002, and a held-out D87 BPB learning curve.

---

## 1. A measurable definition of "1B-equivalent"

"Equivalent" is behavioural. It means a model that does on D87 what a ~1B neural LM does relative to count models, and does it inside the M1 budget. The parameter count is not the target. No transformer is run: every comparator below is a count model or a published number.

### 1.1 The quality bar Q (BPB on held-out D87 DEV bytes)

- **Q1, primary, relative:** BPB(model) <= 0.80 x BPB(C6). C6 is the project's own order-6 interpolated count model (the Phase 1 model), trained on D87 TRAIN and scored on the same DEV bytes.
- **Q2, stepping stones:** 0.95 x C6 (the core adds anything), then 0.90 x C6.
- **Q3, absolute, secondary:** <= 0.90 BPB, the repo's existing M4 bar [survey A1b: L/m4-serving.md:57-62; board :184]. This is reported but does not gate, because D87 is donor reasoning text [checked docs/integration/DECISIONS.md:291 (D87)]. Its entropy per byte is unknown and probably below web text [hyp], so an absolute bar tuned on web text can be trivially easy or impossible here.

Why 0.80 is the bar, from the yardsticks:

| Yardstick | Count / PPM | Stronger model | Ratio of bits |
|---|---|---|---|
| One Billion Word | KN 5-gram, ppl 67.6 -> log2 = **6.08 bits/word** [C2 #6, verified] | 1B-class neural LMs, ppl about 22-30 [mem] -> 4.45-4.91 bits/word | **0.73-0.81** [est] |
| enwik9 (online) | ppmonstr 1.256, PPMd 1.472 [C2 #7, verified] | paq8px 0.998 (counts + match model + mixing) | **0.795** vs ppmonstr, 0.678 vs PPMd [est arithmetic on verified numbers] |
| enwik9 (online) | ppmonstr 1.256 | nncp 0.853 (a transformer), cmix-neif 0.715 | 0.679, 0.569 |

- On 1BW, a 1B-class neural model spends about 0.73-0.81 of the KN 5-gram's bits.
- On enwik9, a count-plus-match context mixer (paq8px) spends 0.795 of a strong PPM's bits.
- So **0.80 x a strong count model is the "1B-class" gap, and there is verified evidence that count + match + mixing machines can close it.** The caveat is that paq8px's mixing is logistic (banned) and the setting is online, not held-out (assumption 1, §6).
- For scale (est): 6.08 bits/word at about 5.5 bytes/word [mem] is about 1.1 BPB for KN on news text. That is consistent with B2.5.

BPB needs a probability. The count rows give an exact rational, count / total, mixed **linearly** with integer weights. That is not an exponential and not a squashing function [hyp that this is lawful under CHARTER §3.2; checked CHARTER.md:27]. The evaluator takes log2 of that rational in a reference/diagnostic path. The repo's lawful-likelihood sheet docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md has a rank branch and a Q24 branch [checked docs/research/notebook/float-training.md:59]. **Do not use the rank branch:** its floor is 0.7188, and it cannot reach 0.90 once more than 1.892% of targets fall outside the top 8 [checked same line]. Phase 0 fixes the instrument: use the Q24 branch, or an integer arithmetic-coding length on the count rational.

### 1.2 Reply panels (each with a no-memory control, per L-010)

- **R1:** on the M2 fresh panel, at least **52/128** complete correct replies. This is the existing bar; the current result is 0/128 [survey A1b: L/m2-exact-memory.md:44-45].
- **R2:** in free generation on the CPU-pilot reply panel, at least **200/256** complete. That is the forced-value ceiling measured there (200-250/256, against 0/256 free) [survey A1c: docs/research/notebook/cpu-training.md:28]. Passing R2 means value choice is no longer the block.
- **R3:** ft/dev next-token top-1 at least **7,002/14,287**, the best model ever measured on it (the M1 19-table trunk) [survey A1b: L/m1-discrete-attention.md:68-71]. The floor is the bigram's 6,300.
- Every R gain must exceed the same model with conjunction rows and the in-session match read turned off.

### 1.3 Budget B on the Apple M1 (8 cores, 16 GB)

- State: at most **4 GB resident**, and at most 6 GB on disk with mmap.
- Decode: at least **1,000 tok/s** at 2k context on one performance core, reported first (owner metric). This is 50x the minimum of 20 tok/s in survey B, and 1/11 of the D11 model's measured 11,026 tok/s [survey B2.2: antigravity-110 REPORT:28-30].
- D85: train and serve share one typed operator for every address, read and token choice; replay of addresses, reads and token ids is bit-identical on wick (CUDA) and the M1 (CPU); the artifact is the integer state itself.

**1B-equivalent := Q1 and R1-R3 and B, all at once.**

---

## 2. Verdict on the gap order (Pick -> conjunction -> readout -> windows -> contract -> corpus trainer -> 1B run)

**Replace it.** Keep its parts, but reorder them around writing instead of scoring.

1. Its last step, a scored 1B run, is infeasible. At 0.5-1.1M trials per GPU-day, 1e9 coordinates take 900-2,000 GPU-days [survey B2.3]. And the trainer still refuses more than 64 rows [survey B2.1: training.rs:307].
2. Its first five steps build operators for a learner (line C's scored search) that has no trained-quality number on language [survey A1b Q2]. The only learners with a measured win are counts:
   - bigram 6,300 against writer 149 [checked m1-w2-capacity-capture/README.md:13-16];
   - pair experts, 137 -> 170 of 239;
   - pair rows, 431 -> 527 of 2,218 [survey A1, board :68-69];
   - lexlean's working parts, which are n-gram routing plus errors written into addressed rows (memory note, lexlean-r4-findings).
3. What survives, re-ordered:
   - **Pick: done** (#4045). It becomes the address read over written rows.
   - **Conjunction memory: moves up.** It is also a written count table.
   - **Readout: dropped as a separate gap.** The candidate set is the union of table hits (at most a few hundred tokens), so there is no 4,096-wide scan and no nearest index [est].
   - **Windows (carry): reduced.** Counting needs no carry across windows. Only the transport used for geometric addresses must run the same way in the counting pass and in serving, and that is a contract item.
   - **Contract: kept, and small.** The schema is the tables plus the transport parameters.
   - **Corpus trainer (408): deferred** to Phase 5, for a core of at most 1e5 coordinates.
   - **1B run: replaced** by the full-D87 counting pass, which takes hours.

---

## 3. Recommended architecture and training path

### 3.1 The state (one native integer state, D85)

| Part | What it holds | WRITTEN or SCORED | Reused engine (path) |
|---|---|---|---|
| S0 tokens | V4096 BPE; each token gets a prime and an E8 code | fixed | uor-r4-tokenizer [survey A1c §c]; prime-zeta codebook (board :150) [survey A1] |
| S1 transport | SpiralCore/FBS E8 steps with causal prefix holonomy, giving a 64-128-bit sign code per position at 2-3 scales | fixed geometry + at most **1e5 SCORED** params (Phase 5) | uor-r4-discrete discrete.cu (line C) [survey gap analysis]; rows-K `score_rows_k` rowsk_device.rs:56 and `run_block_transitions` training.rs:3310 [survey B2.4] |
| S2 n-let rows | orders 1-6; key = product of token primes mod a table prime; value = (token u16, count u16), top-k per key, pruned at count >= 2 | **WRITTEN**, one counting pass | the logic of `cpu-trunk-count-driver` (crates/uor-r4-cpu-train/src/bin/cpu-trunk-count-driver.rs, "exact additive sufficient-statistic training") [checked file exists; survey B2.4] as the CPU oracle; the archived source-free n-gram table #989 (22.3% held-out top-1) [survey A1c]; the T/G tables of W2 |
| S3 geometric-address rows | key = the S1 sign code; value = next-token counts; on a miss, the Pick XOR+popcount read takes the nearest filled code (latest wins ties) | **WRITTEN**, one pass per S1 setting | `pick_device` pick_read_cuda.rs:56; `DiscreteReadMode::Pick` discrete_d11.rs:298/:801; native-vector read.rs [survey B2.4]; address cells A8 (board :151); E8 orbit/product-key slot layout (57,600 per layer, k = 8) [survey A1c] |
| S4 conjunction rows | key = prime product of two user-turn tokens; value = reply-value counts, then error-written refinement | **WRITTEN**: counts in 1 pass, error writes in 1-3 passes | pair experts (claude-367) and pair rows (claude-370) [survey A1 board :68-69]; exact-memory oracle 80/80 (board :179) |
| S5 session match | the session's own token history: the longest exact earlier-suffix match predicts its successor (induction) | no training (session carry) | Relation read (board :154); turn memory [survey A1] |
| S6 mixer | integer weight per (bucket = order hit, match length, address distance, agreement) x expert | **WRITTEN** as hit counts on a held-out TRAIN slice | the softmax-free rank-weight read, archived; its reason for retirement is gone [survey A1c] |

Prediction: score(t) = Σ_e w[bucket, e] · c_e(t) · L / total_e, in integer arithmetic. P(t) is score(t) / Σ score, used for BPB only. Greedy decode takes the argmax over the hit union. There is no exponential, no squashing and no matmul: the work is lookups, integer multiply-add over at most a few hundred candidates, and popcount [hyp for §3.2/§3.3 compliance; integer multiply has been allowed since D80/D81, board :158 per survey A1].

The owner's mechanism is kept whole (token -> transport -> XOR+popcount hard read -> conjunction memory -> readout) [survey gap analysis, "intended mechanism"]. What changes is only how the rows learn.

### 3.2 State size on the M1 (est; Phase 1 measures the distinct n-gram counts)

| Part | Entries | Bytes/entry | Size |
|---|---|---|---|
| S2, orders 1-2 | <= 4,096^2 = 16.8M possible; about 8M distinct [est] | 4 | 0.03 GB |
| S2, orders 3-6, pruned at count >= 2 | 3e8 [est; pruning is tuned to fit] | 8 (16-bit fingerprint, token, count, pad) | 2.4 GB |
| S3 | 3 scales x 2^24 cells x top-4 = 2.0e8 | 4 | 0.8 GB |
| S4 | <= 4,096^2/2 = 8.4M pairs x top-4 = 3.4e7 | 4 | 0.13 GB |
| S0+S1+S6 | the D11 artifact is 86.5 MB [survey B2.2] | n/a | 0.1 GB |
| **Total** | **about 5.3e8 integer cells** | | **about 3.5 GB** (inside B's 4 GB) |

About 0.5e9 written cells is the "1B-scale" state: the same order as a 1B model's weights, at 4-8 bytes each, but filled by counting.

### 3.3 Serving speed (est)

- Per token there are about 15 hashed random reads (6 orders, 3 scales, about 4 pairs, the match read and the mixer). At about 100-150 ns each on M1 DRAM, unpipelined, that is about 2 µs.
- Add one transport step, which is bounded by D11's whole step at about 90 µs (11,026 tok/s, checked by survey B2.2), and a merge over at most 200 candidates (about 2 µs).
- That gives about 5-10k tok/s on one thread. **Expected: 5-10k tok/s. Bar: 1,000.** The S1 transport dominates the cost, so with S3 turned off the model runs faster than 10k tok/s [est].

### 3.4 GPU-days per pass on wick (est)

- **Keys:** 1.87e9 targets [survey B2.1, from claude-406 ORDER:17] x (5 orders + 3 scales + pairs) is about 1.5e10 keys.
- **Method:** sort-reduce on 64-bit keys, sharded by key prefix to fit 24 GB. Host RAM (59 GB) holds a run of about 22 GB (1.87e9 x 12 B).
- **Compute:** at 0.5-2e9 keys/s [mem, CUB radix sort class], about 2-8 minutes. Moving the u16 stream (3.7 GB) over PCIe is about 0.2 s per shard pass.
- **Total with overheads: at most 2 GPU-hours per full pass. With a 10x margin, it is still under 1 GPU-day.**
- **Determinism:** counts are commutative integer sums, so the result does not depend on atomic order. D85 replay is exact by construction [est].
- **Transport for S3:** element-wise integer work per position, about 1e3 ops x 1.87e9 = 1.9e12 ops, so minutes to 2 h on a 4090 [est; Phase 2 measures it].
- **Comparison:** the scored path needs 0.8 GPU-days for 457k trials [survey B2.3], and 900-2,000 GPU-days to touch 1e9 coordinates. **Writing is at least 10^3 times cheaper per learned cell** [est].
- **Scored core (Phase 5):** with 1e5 coordinates and 457k trials per D87 pass of 0.8 days, that is about 4.6 trials per coordinate per pass [est from survey B2.3]. That is adequate for a tuning core and inadequate for anything bigger.

---

## 4. Phased plan

The units are PRs. Every phase ends on a number, a panel and a control.

| # | Deliverable | Kill / continue (number, panel, control) | Starts from | Units | wick? |
|---|---|---|---|---|---|
| **P0** | A BPB instrument (the Q24 or arithmetic-code length on an integer rational; not the rank branch); a sealed D87 DEV byte split; the bytes/target of D87 at V4096; reconcile 1.87B against 2.1B targets [survey B2.1 vs gap analysis item 7] | Passes if the instrument reproduces a hand-computed unigram BPB exactly on a fixture. A decision review records whether a count-ratio probability is lawful under §3.2 | m4-lawful-likelihood-bpb sheet; claude-403 ingest; uor-r4-tokenizer | 1-2 | no |
| **P1** (**the core-risk test**) | A GPU sort-reduce counting kernel (nvcc PTX, its own module) with CPU-oracle parity; S2 orders 1-6 plus the S5 match read plus a fixed-weight mixer; written on ft TRAIN and on D87 TRAIN at 1%, 10% and 100% | (a) ft/dev top-1: **continue if >= 6,420** (G 6,300 + 2σ; σ ≈ sqrt(14,287·0.44·0.56) ≈ 59) [est]; strong if >= 7,002. (b) D87 DEV BPB: **continue if C6 has >= 15% fewer bits than C2 (the bigram)** [est threshold] **and the curve is still falling from 10% to 100% of the data**. Kill the count base if C6 does not beat C2 by 15% | cpu-trunk-count-driver (oracle); #989 n-gram table; W2 T/G tables; build.rs PTX pattern (memory note, no-nvrtc) | 2-3 | yes, at most 1 GPU-day |
| **P2** | S3 geometric-address rows: line C transport sign codes plus Pick read on a miss | **Continue if S3 gives >= 2% fewer D87 DEV bits** than both (i) P1 alone and (ii) **a random-hash control** (a SplitMix hash of the same last k tokens, same table bytes). If it loses to the random hash, geometry adds nothing beyond hashing: keep S3 as a hash and log it for §3.1 | discrete.cu transport; pick_read_cuda.rs:56; address cells A8 | 2 | yes, hours |
| **P3** | S4 conjunction rows plus the error-written refinement; S6 mixer written from hit counts; reply panels | R1 >= 52/128, R2 >= 200/256, R3 >= 7,002, each above the **no-memory control** (S4 and S5 off). Bits: continue toward Q2 (0.90 x C6). Below 0.95 x C6 after the mixer, Phase 5 or more data becomes the lever, not more tables | pair rows claude-370; pair experts claude-367; archived rank-weight read | 2-3 | yes, for writing |
| **P4** | The integer artifact (S0-S6) served in uor-chat on the M1; a CUDA-to-CPU replay fixture | State <= 4 GB resident; **>= 1,000 tok/s** at 2k context (D11 at 11k is the reference); addresses, reads and token ids bit-identical between wick and the M1 on 3 fixed prompts | discrete_d11 serving `DiscreteReadMode::Pick`; uor-chat; object store D19 only if mmap is not enough | 2 | parity fixture only |
| **P5** | A scored core of at most 1e5 S1 coordinates through rows-K at 4,096 rows (the 408 CLI); the objective is the rank-margin hinge (board :156) on the mixed score, scored on a TRAIN slice | **Continue only if** DEV BPB falls by >= 1% per GPU-day over the first 2 GPU-days, against a re-seeded random-coordinate control of equal trials. Otherwise freeze S1 at its geometric default | rows-K rowsk_device.rs:56; training.rs:3310 | 2-3 | yes, 2-6 GPU-days |
| **P6** | More data, if the P1 curve is still falling at 100%: extend the donor corpus on wick (the owner's "lowkey" direction, D87 row [checked DECISIONS.md:291]) and recount | BPB keeps falling per doubling of data | the D87 generation instrument | 1-2 | yes, days of donor time |

Total to a served verdict: **about 11-15 units, under 2 GPU-days for P1-P4, and 2-6 more for P5** [est].

---

## 5. What NOT to do (stop or defer)

1. **Stop** planning any scored run that trains more than 1e6 coordinates. That covers the "1B run" and claude-408 used as a corpus trainer. **Defer** the 408 CLI to P5 [survey B2.3].
2. **Stop** new float-trainer arms of line A, and do not close line A's D85 export gap. Line A stays a reference only (D85, [checked DECISIONS.md:289]).
3. **Do not** take line B's Q31 trunk as the base. Its toy receipt is an echo/garble at vocab 384 [survey A2 addendum]. The Pick read already exists in line C.
4. **Do not** port lexlean v10, v11 or holo v13 (banned forms), and do not count the v8 "2B" reply store as a model [survey A2b].
5. **Do not** distill weights, hidden states or logits ((c) and (d) stay banned), and keep the embedding import (b) deferred [checked DECISIONS.md:290-291].
6. **Do not** build the 4,096-wide readout or a sublinear nearest index (gap 4). The candidate union replaces both.
7. **Defer** spherical harmonics with l >= 2, icosian spin weights, the hyperbolic Starling read, Hopf sector routing and E8 address memory as a trained table, until P2 says whether geometry beats a hash.
8. **Do not** use the rank-branch BPB for the 0.90 bar (§1.1), or reply panels without the no-memory control (L-010).
9. **Do not** add harness features for this plan. A plan row is enough (memory note: research first).

---

## 6. The three assumptions most likely to be wrong

1. **A linear integer mix of count experts closes about 20% of the bits on held-out D87.**
   - **Why it may fail:** the verified 0.795 ratio (paq8px against ppmonstr) uses logistic mixing, which is banned, and online adaptation. Held-out text with linear mixing may stop at 0.90-0.95 [hyp].
   - **Tests:** P1 gives the learning curve and the C6/C2 ratio; P3 gives the mixer's gain; P5 says whether the scored core adds bits per GPU-day; P6 says whether more data does.
   - **If it fails:** the served model is still a lawful count-class LM, near KN quality, fast and small. "1B-equivalent" then needs more donor data (P6), not more search.
2. **The geometry carries information beyond a hash (S3 beats the random-hash control).**
   - **Why it may fail:** C found no outside support for E8 orbits or prefix holonomy as token geometry [survey C §(c)].
   - **Test:** P2, directly.
   - **If it fails:** the project's claim that "geometry carries learning" is in trouble. The model keeps working, but its geometry is only addressing, and §3.1 becomes an owner question for the queue (it is never a hold).
3. **Count-ratio probabilities, linear mixing and log2 in the evaluator are lawful under §3.2, and the reply block is solvable by memory.**
   - **Why it may fail:** §3.2 bans renamed forms "anywhere ... comparators" [checked CHARTER.md:27]. A strict reading could treat a sum-normalised mix as a renamed normalisation. Separately, D87's reasoning-trace style may not transfer to the templated ft and M2 panels [hyp].
   - **Tests:** P0's decision review settles the law. P3's R1/R2, with the no-memory control, settles transfer. If the law fails, fall back to top-1 and rank metrics only, and leave BPB as diagnostic code.

**Top risk:** assumption 1. Everything rests on written counts reaching 0.80 of a strong count model's bits without logistic mixing. P1 and P3 measure it for under 2 GPU-days, before any scored training is spent.
