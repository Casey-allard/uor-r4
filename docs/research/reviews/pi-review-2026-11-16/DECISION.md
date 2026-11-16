# DECISION: the written-state geometric count model (PI, final, 16 Nov 2026)

Author: claude (PI role), read-only. origin/main `ef1f4a641` (worktree payload-probe-387). No builds, no GPU. Supersedes PI-SYNTHESIS.md wherever they differ. It answers RED-R1.md and RED-R2.md, which both returned SURVIVES-WITH-FIXES.

Tags:
- **[checked p:l]**: I read that line this session.
- **[red Rn]**: a red team's checked claim that I did not re-read.
- **[est]**: my arithmetic or a guess.
- **[hyp]**: a hypothesis that a phase tests.
- **[mem]**: from memory, not verified.

This is a proposal for the top session's decision review (§5.1). It changes nothing by itself. It needs the decision card in P0.

---

## 0. The decision in one paragraph

Keep "write, don't search". The project cannot score 1e9 coordinates (900-2,000 GPU-days, survey B2.3), but it can count and error-write 1e9 integer cells on wick in hours per pass.

The written state lives **inside the D11 trunk state**. It extends `TrunkTables`, which is already a count model keyed by the trunk's geometric read codes, and that model closed M1 at 7,002 [checked crates/uor-r4-discrete/src/trunk.rs:612-628; docs/integration/DECISIONS.md:287].

- **Keys:**
  - n-let rows use **positional-prime route keys**. Their canonical meaning is an ordered prime product. They are stored as their exact factor encoding: the packed ordered ids.
  - State rows are keyed by **E8-snapped transport codes**.
  - The in-session copy is the existing **Pick** read.
- **Conjunction rows are pointer-valued**, and the output error writes them.
- **The mixer** is a lawful bucketed linear interpolation fitted by integer EM, then Brier error-writing.
- **The bar is honest.** It requires two things:
  - the owner's absolute 0.90 BPB;
  - 0.80 x a sealed strong lawful count reference R\* (modified-KN), with PPMd as an outside anchor.

The model is called **"count-class, 1B-cell"** until both bars and the reply panels pass.

---

## 1. Disposition of every blocker and major

| ID | Finding | Ruling | One-line reason |
|---|---|---|---|
| R1-B1 | The C6 denominator is weak and self-set | **ACCEPT** | R\* is sealed before model work: interpolated modified-KN with full type counts, no pruning, N tuned on DEV-tune. PPMd is the outside anchor (§2). |
| R1-B2 | DEV leakage through sibling answers and near-duplicates | **ACCEPT** | Split by prompt id, dedupe, publish a 13-gram overlap audit. No BPB ratio counts as evidence without it (P0). |
| R1-B3 | Squarefree prime products lose order and repetition | **ACCEPT** | Verified: the repo's products are squarefree bitsets [checked crates/uor-r4-training/src/sieve_frame_compiler.rs:264-272]. n-let keys become positional primes; the S4 pair key stays order-free by design. |
| R1-B4 | The P1 gates cannot fail | **ACCEPT** | P1 now measures a DEV-fit EM ceiling U_count against 0.90 x R\*, plus a pruning-cost gate and a bit-identity gate. Each can fail (§4). |
| R1-M1 | Memory is 7-24 GB, not 3.5 GB | **ACCEPT** | Elias-Fano sorted tries, rank-quantized counts, entropy pruning to fit, with the pruning cost measured against R\* (§3.5). |
| R1-M2 | S3 contradicts itself: every cell is full, or a nearest index is needed | **ACCEPT** | S3 is keyed by E8 product codes (57,600 per scale) with an exact back-off. Misses use the 240-root neighbourhood and a Starling ~7-neighbour read, so no unbounded search is needed. The controls are fair at equal bytes. |
| R1-M3 | The mixer optimizes top-1, and the lawful evidence says 0.90-0.95 | **ACCEPT** | The mixer becomes bucketed Jelinek-Mercer with integer EM, plus PPM exclusion/inheritance and SEE as count ratios, trained on bits. The plan now says plainly that the last ~10% needs generalization. |
| R1-M4 | Replies are prompt spans, so token-valued counts cannot emit them | **ACCEPT** | S4 becomes pointer-valued and copies through Pick. A shuffled-role control is added, domain-tagged ft TRAIN goes into TRAIN, and there is a lawful stop and repetition rule. |
| R1-M5 | PRLL already exists, so the law review is redundant | **ACCEPT-MODIFIED** | Reuse PRLL. But a uniform floor fed to `masses_from_scores` is cancelled by its min-shift [checked crates/uor-r4-integer/src/lawful_likelihood.rs:3-4]. So score the mixed integer counts, all ≥ 1, through the existing no-shift `mle_masses_from_counts` [checked :113-117]. |
| R1-M6 | P5's objective and gate do not match | **ACCEPT** | P5 trains and gates on the integer Brier score of the mixed mass; BPB is reported beside it. |
| R2-B1 | §3.1: the plan is a classical hash n-gram and geometry is optional | **ACCEPT-MODIFIED** | Positional-prime keys, E8-snapped state codes, Pick copy and a geometric output expert are on the default path, with no hash default. The canonical key meaning is the prime product, stored losslessly as packed ordered ids (a bijection by unique factorization). This follows the repo's practice of storing products as factor bitsets. |
| R2-B2 | D85: two lanes, so build inside TrunkTables | **ACCEPT-MODIFIED** | It lives in `TrunkTables` as new typed table families under one key operator. But `SparseCountTable` is a `BTreeMap<u32, BTreeMap<u16,u32>>` sized for 1,049,856 rows [checked trunk.rs:510-513, :612]. It cannot hold 1e9 entries or keys past order 2. So it stays the oracle form. A frozen compact form is the artifact. "Bit-identical table words" means the canonical sorted (key, token, count) triple stream. |
| R2-M1 | The honest bar | **ACCEPT-MODIFIED** | Merged with R1-B1: the absolute 0.90 BPB (owner) on the pinned stream, plus 0.80 x R\* on sealed D87 DEV-test, ≥ 1e9 cells, ≤ 4 GB resident, and the count-class label (§2). |
| R2-M2 | Little learning happens on wick | **ACCEPT-MODIFIED** | Counts initialize the rows, then the integer Brier error is written into the S3, S4 and S6 rows for 2-3 epochs on wick. S2 error cells are a measured arm, because they double S2's bytes. The scored core starts in P5 with a random-transport control. |
| R2-M3 | S3's miss path needs a forbidden nearest index | **ACCEPT** | Same fix as R1-M2: the E8 decoder plus the 240 root neighbours. |
| R2-M4 | A decision card is needed for the gap order | **ACCEPT** | The card is in P0. Step 2 ("written by the output error") is kept literally. Step 3 (readout from the pick) is kept as the Pick expert and the S4 pointer, not dropped. |
| R2-M5 | P6 donor data needs an owner release | **ACCEPT** | P6 is an owner-queue proposal only (§7). |
| R2-M6 | Revival engines were dropped | **ACCEPT-MODIFIED** | Starling, E8 product key, hinge and VSA get named slots. Icosian, the whole-path source and the inverted file are deferred, each with a reason (§5). |
| R2-M7 | PRLL zero-mass and the floor | **ACCEPT-MODIFIED** | As R1-M5: a full-support unigram plus a uniform floor expert in every bucket, scored with the no-shift ratio. The offline scorer sums over V = 4,096, and serving needs no normalizer. |

Minors:
- **Accepted:** R2 m1-m8 and R1 m1-m5.
- **Ruled:** R2-m1 bans SSE/APM, while R1-M3 allows a linear-domain APM. Stretch/logit APM is **banned**. A linear-domain piecewise table is **off by default** and goes to the owner queue as a §3.2 interpretation (§7). The default mixer never needs it.

R1 "missed engines":
- **Accepted:** the Rust suffix array (from Lee et al.'s dedup tool [mem]). It serves as R\*'s exact counter and as the overlap-audit tool.
- **Accepted:** the Elias-Fano trie algorithm, re-implemented in Rust.
- **Accepted:** PPMd, as a Rust port.
- **Rejected as a source of the sealed number:** KenLM (C++, §3.11). It is allowed only as a diagnostic cross-check on a 1% fixture.

---

## 2. The final definition of "1B-equivalent"

The label is **"count-class, 1B-cell"** until bars A, R and B all pass at once. Only then is it "1B-equivalent (lawful-count reading)". The label never implies a neural comparison, because §3.10 bans them [checked CHARTER.md:38].

**A. Absolute quality (the owner's bar, unchanged).** The model scores ≤ **0.90 BPB** on the owner-pinned `chat-v0-p2` held-out stream, through PRLL. The bar is the owner's (ROADMAP.md:17, via the m4 sheet §5 [checked docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md:1-20, §5]; the stream pin is [red R2, sheet:47]). The same number is also reported on sealed D87 DEV-test.

**Q. Relative quality.** BPB(model) ≤ **0.80 x BPB(R\*)** on sealed D87 DEV-test bytes.
- **R\***: interpolated modified Kneser-Ney with the standard three discounts per order and full N1/N2/N3+ type counts. It is unpruned, orders 1..N, with N chosen on DEV-tune (8-12 expected at about 2 B/token [red R1]). It is built in Rust from exact suffix-array counts on wick, and its hash and BPB are sealed in P0 before any model work.
- **Anchor:** PPMd (var.H-class: counts + SEE + exclusion, no logistic), a Rust port validated byte-for-byte against the reference coder on a public fixture, then primed on TRAIN and scored on DEV-test. It is reported and does not gate.
- **Stepping stones:** 0.95 x R\* and 0.90 x R\* are recorded checkpoints, not bars.
- **Honesty note:** the lawful literature predicts 0.90-0.95 for counts plus linear mixing [red R1-M3, est]. The gap from 0.90 to 0.80 must come from the geometric components (S3, the P5 core). If it does not, the model keeps the count-class label.

**R. Replies.** Each one exceeds the **no-memory control** (S4 and S5 off) and a **shuffled-role control**:
- **R1:** ≥ 52/128 on the M2 fresh panel (the existing bar; 0/128 now [survey A1b]).
- **R2:** ≥ 200/256 in free generation on the CPU-pilot reply panel (the forced-value ceiling [red R1-M4: cpu-training.md:27-28]).
- **R4:** a fresh held-out-phrasing panel of ≥ 200 rows (§4.3 transfer), above both controls by the paired band.
- **R3 floor:** ft/dev ≥ 7,002/14,287 (the D83 model [checked DECISIONS.md:287]). It is a floor, not part of the definition (R1-m3).

**B. Budget and speed.**
- State: ≥ **1.0e9 integer cells**, each written or refined by training, ≤ **4 GB resident**, with a lossless artifact. Report D5-a and D5-b (§3.9).
- Speed: ≥ **1,000 tok/s** warm at 2k context on one M1 performance core, reported first. Cold-cache tok/s is reported beside it.
- D85: replay is bit-identical between wick (CUDA) and the M1 (CPU) on ≥ 64 prompts.

**Statistics.**
- Count models are deterministic, so they have no seeds. Every gain needs a fresh panel or the sealed DEV-test (§4.3).
- BPB comparisons use a paired prompt-block bootstrap, and its 95% interval must exclude 0.
- Reply comparisons use the paired band, about 2·sqrt(discordant).

**What "1B" counts.** It counts the written and refined cells, ~1.2e9 [est §3.5]. They are not neural parameters, and the label says so.

---

## 3. The final architecture

### 3.1 The state: one D85 integer state, extending the D11 trunk

| Part | What it holds | How it learns | Home |
|---|---|---|---|
| S0 tokens | V4096 ids; one prime per (token, slot); one E8 code per token | fixed | tokenizer; prime-zeta codebook [survey A1] |
| S1 transport | the D11 discrete trunk: Hamilton gate, read and rec codes per position | fixed geometry; at most 1e5 scored params in P5 | `TrunkTables` is already built for `DiscreteHamiltonGateV1` [checked trunk.rs:612] |
| S2 n-let rows | orders 1..N under positional-prime route keys; value = (token, count), plus context type counts | **counted**; Brier error cells as a P3 arm | new family in `TrunkTables` |
| S3 state rows | key = (E8-snapped transport code, optionally the last token); value = (token, count, i8 error cell) | **counted, then error-written** | extends the existing read/rec/query tables [checked trunk.rs:614-628] |
| S4 conjunction rows | key = the squarefree prime product of two user-turn tokens (order-free, correct here); value = **pointer** (role or span offset) and count, plus an error cell | counted from TRAIN; error-written with the hinge (pointer choice) | new family in `TrunkTables` |
| S5 session copy | the **Pick** read: the latest earlier position whose (p-1, p) block codes are nearest by XOR+popcount, with its gated successor | no training (session carry) | `DiscreteReadMode::Pick` [checked crates/uor-r4-integer/src/discrete_d11.rs:286-298]; `pick_device` [checked crates/uor-r4-discrete/src/pick_read_cuda.rs:56] |
| S6 mixer | integer weights w[bucket, expert], with ≤ 4,096 buckets x ≤ 12 experts, so ≤ 49k weights | integer EM, then Brier error-write | the skip-mix table-injection precedent [survey A1c] |

### 3.2 Keys

**One typed key operator, `PrimeRouteKey`, shared by the CUDA writer, the CPU oracle and the `discrete_d11` loader.**

*Meaning.* Each (token t, slot j) pair gets its own prime q(t, j), and K_k = Π_{j≤k} q(t_{-j}, j).
- Order is preserved.
- Back-off is exact division, K_{k-1} = K_k / q(t_{-k}, k).
- Shared-suffix admission is gcd(K, K').

*Storage.* By unique factorization, K_k corresponds one-to-one with the ordered tuple (t_{-1}..t_{-k}). The stored word is therefore the packed ordered ids, 12 bits x k: a u64 up to k = 5 and a u128 up to k = 10.
- Division becomes dropping the last slot, and gcd becomes slot-wise equality.
- The key is exact and has no hash. Hashing is allowed only for bucket placement, and the full key is checked in the row.
- The repo already stores prime products as their factor encoding [checked sieve_frame_compiler.rs:264-272].

*Prime sizes [est].* There are 4,096 x 12 = 49,152 (token, slot) primes; the largest is ~6.0e5 (20 bits).

*Required tests:*
- `"a b"` and `"b a"` give different keys;
- `"the x the"` and `"x the"` give different keys;
- factor(K) equals unpack(word) on a fixture.

**How E8 and the transport enter: four places, all on the default path.**
1. **S3 address.** Each 8-coordinate block of the trunk read state is snapped to its nearest E8 lattice point by the Conway-Sloane decoder (integer rounding plus a parity fix, O(8)).
   - The root indices form the existing 240 x 240 = 57,600 product-key layout per scale [survey A1c].
   - Back-off: (code, last token) → code alone.
   - A miss is filled by a **Starling hard read over the ~7 nearest of the 240 root neighbours** (§3.1 flock rule [checked CHARTER.md:26]). The neighbourhood is bounded, so no index is needed.
2. **S5 Pick.** The XOR+popcount copy read over transported block codes (#4045).
3. **Output expert (§3.12).** A Pick score between the predicted S3 code and each hit candidate's S0 E8 code, so the shared vector writer serves the output. Identity stays exact token ids (§3.12 [checked CHARTER.md:40]).
4. **The P5 core.** It tunes S1's transport parameters, so the addresses themselves learn.

### 3.3 Where it lives

- `crates/uor-r4-discrete/src/trunk.rs` gains:
  - the `PrimeRouteKey` operator;
  - S2, S3-extended and S4 table families on `TrunkTables`;
  - a frozen compact form with a lossless round-trip to the `SparseCountTable` oracle form on fixtures.
- The GPU writer (nvcc PTX in its own module, no NVRTC) emits the canonical sorted (key, token, count) triple stream.
- The D84 CPU driver `cpu-trunk-count-driver` [red R2: count-driver.rs:1-19] is the finite parity oracle, on a 1-10% slice only (R2-m3).
- `discrete_d11` loads the frozen form. "Bit-identical" means the SHA-256 of the canonical triple stream is equal across the GPU writer, the CPU oracle (on the slice) and the serving loader.

### 3.4 The lawful mixer (§3.2/§3.3)

score(t) = Σ_e w[b, e] · c_e(t) · L / tot_e + w_u · uni(t) + f

- b is the bucket: (longest order hit, Pick match length, E8 distance, expert agreement).
- f ≥ 1 is a uniform floor.
- Every term is an integer multiply-add. There are ≤ ~300 hit candidates per token, plus the closed-form floor.

Rules:
1. **Weights.** Weights are initialized by integer EM (deleted interpolation, Jelinek-Mercer) on a held-out TRAIN slice. The update is w_e ← Σ w_e p_e / Σ_e' w_e' p_e', a ratio of linear sums.
2. **Escapes.** PPM exclusion and inheritance, and SEE, use integer escape counts only.
3. **Refinement.** The integer **Brier** error, (1[t = target] − P(t)) scaled to Q16, is written into the addressed S3, S4 and S6 cells. That is LMS on a linear mixture, with no exponential.
4. **Banned:**
   - logistic, stretch or logit mixing;
   - stretch-domain SSE/APM;
   - Hedge, exponentiated-gradient or multiplicative weights;
   - maximum-entropy or log-linear smoothing;
   - any vocab x feature or candidate x hidden weight table (§3.3, §3.6; R2-m2).
5. **Scoring BPB.** The integer score vector (every entry ≥ f ≥ 1) is scored with PRLL's `mle_masses_from_counts`, an exact ratio with no shift [checked lawful_likelihood.rs:113-117]. Then `bits_q12` and `bits_per_byte` [checked :151, :195].
   - Do **not** use `masses_from_scores` for count models: its min-shift [checked :3-4] cancels the floor.
   - `-log2` stays in the offline scorer (Card 1 [checked m4 sheet:1-20]).

### 3.5 Memory, with the compression plan [est; P1 measures the gram counts]

| Part | Entries | B/entry | Size |
|---|---|---|---|
| S2, orders 1-2 | ~8e6 (stored sparse, not a dense 4096², R1-m4) | 4 | 0.03 GB |
| S2, orders 3..N, entropy-pruned to fit | 0.8e9 (from 0.6-1.5e9 surviving count ≥ 2 [red R1-M1]) | ~3.2 (Elias-Fano trie ~2.2 + 1 B rank-quantized count) | 2.56 GB |
| S3, 2 scales | 2 x 1e8 | 3.5 | 0.70 GB |
| S3 error cells (i8) | 2e8 | 1 | 0.20 GB |
| S4 pointers + error cells | 8.4e6 pairs x top-2 = 1.7e7 | 4 | 0.07 GB |
| S0, S1, S6 (the D11 artifact is 86.5 MB [survey B2.2]) | n/a | n/a | 0.10 GB |
| **Total** | **~1.22e9 cells** | | **~3.66 GB resident** (≤ 4 GB) |

Compression and fit:
- Prune by measured BPB cost (entropy pruning) until it fits. P1 reports the pruning cost, which is the deployable model's BPB against unpruned R\*.
- If 4 GB forces more than a 3% cost, order > 6 moves to an optional mmap'd suffix array of ~13 GB on disk (1.87e9 x ~5 B + the 3.7 GB stream), read only on long matches. Cold tok/s is reported for it.

Wick-side storage:
- R\* unpruned is ~3-5e9 distinct grams x 16 B = 48-80 GB, more than wick's 59 GB host RAM [red R1-m1].
- So it is written as sorted runs on wick's disk and streamed. The GPU writer prunes to count ≥ 2 on the device before copy-out.

### 3.6 Serving tok/s on the M1 [est]

| Step | Cost |
|---|---|
| S2: N ≈ 8 orders x ~4 Elias-Fano misses x ~100 ns | 3.2 µs |
| S3: 2 scales x ~3 misses + a 7-neighbour read only on a miss | 1 µs |
| S5 Pick: 2k positions x one 64-bit XOR+popcount | ~1-2 µs |
| S4: cached once per user turn (R1-m2) | ~0 per token |
| Merge over ≤ 300 candidates; greedy decode needs no normalizer | ~2 µs |
| S1 trunk step: D11's whole step is 1/11,026 s | ≤ 91 µs [checked DECISIONS.md:287] |
| **Total** | **~100 µs/token, about 10k tok/s warm** |

- Cold mmap: ~20 page faults x 50-100 µs gives 500-1,000 tok/s on first touch, so the bar is set on the warm, resident state.
- With S3 off, the model runs faster than 10k tok/s.

### 3.7 GPU passes on wick [est]

- **Keys:** 1.87e9 targets [red R1: claude-406 ORDER:17] x ~11 key families = 2e10 keys, 128 bits each.
- **Method:** generate the keys on the device from the resident 3.7 GB u16 stream, then radix sort-reduce per hash-range partition.
- **Throughput:** at ~0.5-1e9 128-bit keys/s, that is 20-40 s of sort per family. The wall time is I/O plus trie build, ≤ 2 h per pass.
- **Error-write epoch:** about one scoring pass of mixer arithmetic, ~1-3 h.
- **Total:** P1-P3 is ≤ 2 GPU-days in all; P5 is 2-6 GPU-days.
- **Determinism:** counts are commutative integer sums; top-k ties break by token id; error-writes apply in canonical key order. Replay is exact.

---

## 4. The phased plan

Every GPU training or evaluation phase needs a one-page sheet and a non-author FIT (§4.5). Each "close" closes a configuration, never the mechanism (§4.1).

### P0: instruments, seals and contracts (no wick except R\*'s counting run)

- **Deliverables:**
  1. **The decision card**: "the written-state track supersedes steps 3-6 of PLAN.md:38-39". Step 2 is kept literally and step 3 is kept as the Pick/pointer readout [checked PLAN.md:38-39]. It carries the top session's whole-project review and is logged in the owner's queue, and the PLAN row is updated the same day.
  2. **The native-space ORDER items** (SKILL.md, D85 [checked DECISIONS.md:289]):
     - the state schema S0-S6;
     - the shared operator paths (`PrimeRouteKey`, the counting kernel, error-write, the read, PRLL scoring);
     - the replay oracle;
     - the update rule ("count, then integer Brier error-write");
     - the lossless format (the canonical triple stream plus the frozen form).
  3. **The D87 DEV split and leakage audit.** D87 stays read-only: the split is a derived manifest, never a moved shard.
     - TRAIN, DEV-tune and DEV-test are split by prompt id, with all D86/D87 alternatives on one side [checked DECISIONS.md:290-291].
     - Near-duplicates are removed.
     - A 13-gram TRAIN/DEV overlap audit is published, using the suffix-array tool.
     - The 1.87e9 vs 2.1e9 target counts are reconciled, and bytes/target is measured.
  4. **R\* sealed** (Rust modified-KN from suffix-array counts on wick), and **PPMd** as a Rust port validated against the reference coder on a public fixture. Both get BPB on DEV-test and a sealed hash.
  5. **The PRLL floor**: the full-support unigram and uniform-floor expert, scored through `mle_masses_from_counts`.
- **Gate (can fail):**
  - (a) PRLL reproduces a hand-computed unigram BPB exactly on a fixture.
  - (b) PPMd's byte length equals the reference coder on the fixture.
  - (c) R\*'s order-2 BPB equals an independent count computed in the CPU oracle on a 1% slice.
  - (d) The overlap audit reports 13-gram TRAIN/DEV overlap ≤ 1% of DEV positions, or the split is redone.
  - **Power:** (a) to (c) are exact equalities; (d) is a census.
- **Starts from:** `lawful_likelihood.rs`; the m4 sheet; claude-403/406 ingest; the tokenizer; Lee et al.'s Rust suffix array [mem].
- **Size:** 4-5 PRs.
- **Wick:** yes, for R\* counting only (hours; it needs a sheet because it is an evaluation).
- **Prerequisites:** none.

### P1: the count base inside TrunkTables (the first core-risk test)

- **Deliverables:**
  - `PrimeRouteKey`;
  - the GPU counting kernel;
  - the S2 families and the frozen form;
  - the S5 Pick expert, the floor expert, and EM-fitted S6 weights;
  - each written on D87 TRAIN at 1%, 10% and 100%.
- **Gates (each can fail):**
  - **(a) D85 identity.** The GPU canonical triple-stream SHA equals the CPU oracle's on the 1% slice and equals the `discrete_d11` loader's read-back.
  - **(b) Pruning cost.** The deployable ≤ 4 GB model scores ≤ 1.03 x R\* on DEV-test. Paired prompt-block bootstrap; DEV-test ~1.9e7 targets, ~1e4+ prompts. A 0.5% BPB difference is detectable at > 99% power [est: per-token sd ~3 bits gives SE ~7e-4 bits per token before blocking].
  - **(c) Count ceiling.** U_count is the bucketed linear mix fit by EM **on DEV-test itself**: an upper bound for any such mixer of P1's experts (R1-B4).
    - If U_count ≤ 0.85 x R\*, the count path can carry most of Q.
    - If U_count > 0.90 x R\* (expected [hyp]), counts plus linear mixing cannot reach 0.80. P2 and P5 become the critical path, and the record says so. This is a branch, not a kill.
  - **(d) Floor.** ft/dev ≥ 7,002 using the paired band (≈ 2·sqrt(b+c)).
- **Control:** R\* (sealed). The P1 model without S5 isolates Pick's gain.
- **Starts from:** `trunk.rs` `TrunkTables`, `SparseCountTable` and `pack_u32_key` [checked trunk.rs:481-628]; `cpu-trunk-count-driver` (oracle); the build.rs PTX pattern; the #989 n-gram table [survey A1c].
- **Size:** 3 PRs.
- **Wick:** yes, ≤ 1 GPU-day.
- **Prerequisites:** P0 items 1-5.

### P2: geometric state rows (S3) and the geometric output expert

- **Deliverables:**
  - the E8 snap of the trunk read state;
  - the S3 product-key rows with exact back-off;
  - the Starling ~7-neighbour miss read;
  - the output Pick expert;
  - claude-408's CPU Pick applier and its test gaps (§6).
- **Gate (can fail):** S3 plus the output expert gives ≥ 1% fewer DEV-test bits than **the best** of three controls at **equal table bytes**. The paired prompt bootstrap 95% interval must exclude 0. The controls are:
  - (i) a token-context key at the best k in 1..12;
  - (ii) **the same transport with re-seeded random group elements**, which keeps the path dependence and removes the geometry;
  - (iii) SplitMix of the same prefix.
  - **Power:** as P1(b), so a 1% effect is detectable at > 99% [est].
  - **Also reported:** the oracle ceiling U_geo against U_count.
- **On a fail:** close this configuration and run the pre-registered next address arms, in order:
  1. VSA SplitMix binding as a *geometric* binding;
  2. the Hopf sector;
  3. the 600-cell cell;
  4. the golden-shell index.
  If all lose, a §3.1 proposal goes to the owner's queue. **A hash never becomes the default.**
- **Starts from:** discrete.cu transport; `pick_read_cuda.rs:56`; `DiscreteReadMode::Pick`; the E8 product-key layout and the Starling read (archived) [survey A1c]; VSA [survey A1c].
- **Size:** 3 PRs.
- **Wick:** yes, hours.
- **Prerequisites:** P1(a) and P1(b).

### P3: conjunction pointers, error-written learning, replies

- **Deliverables:**
  - pointer-valued S4, counted from TRAIN, with domain-tagged ft/M-World TRAIN mixed in (never ft/dev);
  - the hinge on pointer choice (437/512 revival);
  - integer Brier error-write over S3, S4 and S6 for 2-3 wick epochs (S2 error cells as an arm);
  - a lawful stop and repetition rule;
  - the reply panels.
- **Gate (can fail):**
  - R1 ≥ 52/128, R2 ≥ 200/256 and R4 above **both** the no-memory and the shuffled-role controls, by the paired band.
  - **Power:** at 52/128 against a control near 0, power is ~1. R4 has ≥ 200 fresh rows.
  - Error-written rows must give ≥ 1% fewer DEV-test bits than count-only rows (bootstrap as before), with Brier falling every epoch.
  - The record states Q's standing against 0.95/0.90/0.80 x R\*.
- **Starts from:** pair rows claude-370 and pair experts claude-367 [survey A1]; the exact-memory oracle (80/80); Pick #4045; the hinge objective [survey A1c].
- **Size:** 3 PRs.
- **Wick:** yes, ≤ 1 GPU-day.
- **Prerequisites:** P1, plus P2 or its pre-registered next arm.

### P4: the served artifact on the M1

- **Deliverables:** the frozen S0-S6 artifact in `discrete_d11` and uor-chat, plus a CUDA-to-CPU replay fixture.
- **Gate (can fail):**
  - resident ≤ 4 GB;
  - ≥ 1,000 tok/s warm at 2k context, with cold reported;
  - raw-word equality of every carried field (addresses, reads, Pick choices, pointer choices, token ids) on ≥ 64 prompts, including misses and back-offs (R2-m7);
  - bar A scored on the pinned `chat-v0-p2` stream on wick's GPU (§3.6: scoring is GPU research compute).
- **Starts from:** `discrete_d11`, uor-chat, and the D19 object store only if mmap fails.
- **Size:** 2 PRs.
- **Wick:** only for the parity fixture and the bar A scoring.
- **Prerequisites:** P3.

### P5: the scored geometric core

- **Deliverables:**
  - ≤ 1e5 S1 transport coordinates through rows-K at 4,096 rows (`score_rows_k` [checked crates/uor-r4-discrete/src/rowsk_device.rs:56]);
  - `run_block_transitions` [checked crates/uor-r4-discrete/src/training.rs:3310];
  - the Driver's 1..64-row cap lifted [checked training.rs:307,312];
  - the objective is **the integer Brier score of the mixed mass** (R1-M6).
- **Gate (can fail):** DEV-test Brier falls ≥ 1% per GPU-day over the first 2 GPU-days, against **a re-seeded random-coordinate control with equal trials**, and DEV-test BPB does not rise. Otherwise S1 is frozen at its geometric default and the configuration is closed.
- **Starts from:** the claude-407/408 rows-K CLI.
- **Size:** 2-3 PRs.
- **Wick:** yes, 2-6 GPU-days.
- **Prerequisites:** P2 and P4.

### P6: more data (owner queue only, §7)

The lab does not run it.

**Total:** ~17-19 PRs; ≤ 2 GPU-days for P0-P3 plus 2-6 for P5 [est].

---

## 5. Engines reused, revived or deferred

| Engine | Status | Use |
|---|---|---|
| `TrunkTables` / `SparseCountTable` / `pack_u32_key` (M1 trunk) | existing | the home of all written state, and the oracle form (P1) |
| `cpu-trunk-count-driver` (D84) | existing | finite parity oracle on 1-10% slices (P1) |
| Pick read: CUDA, `discrete_d11` and the 408 inventory | existing | S5 copy, output expert, pointer copy (P1-P3) |
| rows-K + `run_block_transitions` | existing | the P5 scored core |
| PRLL `lawful_likelihood.rs` | existing | every BPB number (P0 on) |
| Hamilton gate (`DiscreteHamiltonGateV1`) | existing | S1, already inside the trunk |
| pair rows 370 / pair experts 367 / exact-memory oracle | existing | S4 (P3) |
| #989 source-free n-gram table, W2 T/G tables | archived | S2 fixtures and a cross-check (P1) |
| E8 orbit / product-key layout | **revived** | S3 address (P2) |
| Starling ~7-neighbour hard read | **revived** (its softmax surrogate cause is gone) | S3 miss read (P2) |
| Pooled-rank hinge (437/512) | **revived** | S4 pointer objective (P3) |
| VSA SplitMix binding | **revived** | P2 address arm 1 |
| Rank-weight read, skip-mix injection | **revived** | S6 mixer precedent (P1/P3) |
| Lee et al. Rust suffix array | external, Rust | R\* counts, overlap audit, optional long-match store |
| Elias-Fano trie (tongrams algorithm) | re-implemented in Rust | the frozen form |
| PPMd | Rust port | outside anchor (P0) |
| D19 object store | existing | P4 fallback only if mmap fails |
| Icosian-mantissa weights | **deferred** | ≤ 49k mixer weights need no compression; revisit if the P5 core grows |
| Whole-path source (compiler + store + E8 index + emitter) | **deferred** | a second serving path against D85's one path; revisit after P4 |
| CUDA inverted file | **deferred** | E8 decoding with bounded neighbours removes the need; fallback if the P2 miss read is too slow |
| Hopf sector, 600-cell, golden shell | **deferred to P2's fail branch** | pre-registered arms 2-4 |
| Spherical harmonics l ≥ 2, hyperbolic Starling | **deferred** | no slot until P2 shows geometry carries bits |
| Line A float trainer, line B Q31 trunk, lexlean v10/v11, holo v13 | **not used** | reference only, or banned forms (unchanged from PI §5) |

---

## 6. What stops or is deferred, and claude-408's open items

**Gap order** [checked PLAN.md:38]:
- **(1) Pick:** done (#4045). It is now S5, the output expert and the pointer copy.
- **(2) Conjunction memory:** kept and moved to P3. The prime-product pair key is kept, and "written by the output error" is kept literally as the Brier error-write.
- **(3) Readout from the pick:** kept, as the Pick expert plus the S4 pointer through the hit-union mixer. A 4,096-wide scan is not built.
- **(4) Windows / carry:** reduced to the transport-determinism item in the P0 contract.
- **(5) Contract:** this is P0 item 2.
- **(6) Corpus trainer:** deferred to P5, for ≤ 1e5 coordinates.
- **(7) "1B training run on wick"** [checked PLAN.md:39]: it becomes the P1-P3 counting and error-write runs, and a scored 1e9-coordinate run stops. This change is what the P0 decision card covers.

**Stopped:**
- any scored run over 1e6 coordinates;
- line A arms;
- the 4,096-wide readout and the sublinear nearest index;
- any hash as a default address.

**claude-408 Still open** [checked docs/loop/tasks/one-model-end-to-end/claude-408/REPORT.md:161-175]:
1. **The CPU applier for Pick:** P2, PR 1. It is needed for the D85 replay and the P4 fixture.
2. **The test gaps:** these go into P2, PR 1:
   - the Pick-specific out-of-orbit refusal;
   - the non-identity `candidate_to_frame`;
   - the serving loader admitting any frame below 64 (restrict it to the 17 step frames in both CUDA and serving);
   - the 19-table trunk with Pick (P1/P2 now writes one, so add the test);
   - the Pick trace in `evaluate_prefix_traced`;
   - the planted wrong-pick fault;
   - the inventory's gate levels above 0;
   - the `from_parts` vs `from_bytes` mismatch.

   Deferred:
   - the CUDA `pick_prefix` recompute: CUDA is not the serving path, so this is a speed item if P2's wall time needs it;
   - the stale instrument `Cargo.lock`: left as found, as 408 did.
3. **Conjunction memory:** P3.

---

## 7. Owner queue (proposals; nothing waits on them)

1. **P6, more donor data.** A D41-family donor pass extending D87, and/or a public pretraining corpus, starting only if the P1 learning curve from 10% to 100% is still falling by ≥ 2% per doubling.
   - §3.10 requires a release at dispatch [checked CHARTER.md:38], and D87 leaves wick's post-corpus role and a public corpus to the owner [checked DECISIONS.md:291].
   - The sheet is staged, and the lab goes on with P2-P5.
2. **A §3.2 interpretation.** Is a linear-domain, count-ratio piecewise calibration table (APM without stretch) lawful? Default: off. The plan does not depend on it.
3. **The meaning of the label.** The goal is restated as "1B-equivalent (lawful-count reading)" = bars A, Q, R and B (§2), with "count-class, 1B-cell" until then. The owner may widen or narrow it, and the lab proceeds on §2.
4. **Conditional.** If every P2 geometric address arm loses to its controls, a §3.1 proposal on what "geometry carries learning" means for addressing. The lab continues with the best geometric arm on the default path meanwhile.

Wick for student training is the owner's own 16 Nov direction ("train what is left for training on wick's gpu the 4090", the goal statement), so P1-P5 need no new release. The caps of §3.5 and §3.6 still apply.

---

## 8. The three riskiest assumptions

1. **Lawful counts plus linear mixing plus geometry close Q (0.80 x R\*).** The lawful evidence supports about 0.90-0.95 for counts alone [red R1-M3, est], so the last ~10% rests on S3, S5 and the P5 core.
   - **Measured by:** P1(c) (the U_count ceiling), P2 (U_geo and the controls), P3 (the deployable mix), P5 (bits per GPU-day).
   - **If it fails:** the result stands as a lawful count-class model, and P6 goes to the owner queue.
2. **Geometric addresses carry bits beyond fair controls at equal bytes.**
   - **For it:** the M1 model is already a count model keyed by trunk read and rec codes, and it beat the bigram by 702 on ft/dev [checked trunk.rs:614-628; DECISIONS.md:287; W2 README via PI]. But that panel is templated.
   - **Measured by:** P2, against the randomized-group control.
3. **Pointer-valued conjunctions plus Pick solve the copy-binding block** (0/128 now) and transfer from D87's reasoning-trace style to short replies.
   - **Measured by:** P3, against the no-memory and shuffled-role controls.

Memory and speed (≤ 4 GB, ≥ 1,000 tok/s) are next in line, and P1(b) and P4 measure them.

**Top risk:** assumption 1, specifically its geometric tail. P1(c) tells within one GPU-day whether counts alone stop above 0.90 x R\*. If they do, the 1B label depends on P2 and P5 showing that geometry adds bits at equal bytes.
