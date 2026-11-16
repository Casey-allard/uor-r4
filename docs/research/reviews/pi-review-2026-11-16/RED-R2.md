# RED R2: rules and owner-intent attack on PI-SYNTHESIS (16 Nov 2026)

Reviewer: claude, red team R2 (rules + mathematics), read-only, at origin/main `ef1f4a641` (worktree payload-probe-387). No builds.
Tags: **[checked path:line]** = read this session. **[est]** = my arithmetic. **[hyp]** = untested.

**Verdict: SURVIVES-WITH-FIXES.** Writing state by counting, rather than searching 1e9 coordinates, is the right economics, and it fits the owner's "If this is not the correct track ... that is likely the track we want to build for". As written, though, the plan is a classical hashed n-gram/PPM lane placed beside the discrete trunk. Geometry is an optional add-on in it, with a hash fallback. That fails §3.1, §3.7/D85 and §3.12 in substance. The fixes below keep the same costs and the same phases.

---

## Rule texts relied on

- §3.1: "Every mechanism, training signal and fix comes from the project's geometry ... *Why:* the bet is that geometry replaces expensive dense operations." [checked CHARTER.md:26]
- §3.2: "No softmax, log-softmax, cross-entropy (implicit softmax), softplus, sigmoid, tanh, GELU or normalized exponential, anywhere: runtime, losses, gradients, sampling, teachers, surrogates, comparators, including renamed or table forms." [checked CHARTER.md:27]
- §3.3: "No dense matrix multiplication or GEMM anywhere ... including wrappers and hand loops." [checked CHARTER.md:28]
- §3.5: CPU research runs only "in a named, separately scanned Rust module. Such a program may not substitute CPU output for a GPU qualification bar" [checked CHARTER.md:30]
- §3.6: "panel scoring and milestone evaluation remain GPU research compute"; "a dense model disguised as lookup tables is excluded" [checked CHARTER.md:31]
- §3.7: "The default model stores token addresses, vector coordinates, SpiralCore carry, route choices, memory keys, memory reads and learned state in one declared integer or fixed-point representation. Training and serving call the same typed operator definitions and the same state-transition path" [checked CHARTER.md:35]
- §3.10: D41 is the sole neural exception, and "an owner release (§5.1) is required at dispatch" [checked CHARTER.md:38]
- §3.12: "one shared vector writer serves input, carry and output" [checked CHARTER.md:40]
- §4.4: "Compare against lawful non-neural baselines (count tables)." [checked CHARTER.md:47]
- §4.3: "≥ 2 seeds or ≥ 200 fresh rows ... a fresh transfer number with every development gain" [checked CHARTER.md:46]
- §5.8: "Approved sequences ... live in `PLAN.md` and bind until the owner changes them." [checked CHARTER.md:65]
- The owner's gap order, recorded with his "Yes, that base" (step 2: "conjunction memory: token-pair rows addressed by the prime product, written by the output error"; step 3: "a readout objective that answers from the pick") [checked PLAN.md:38]
- D87: "Not decided here: wick's role after the first corpus ... both need a later owner decision." [checked docs/integration/DECISIONS.md:291]
- M4 lawful likelihood. Owner Card 1: "Yes, a linear ratio (Recommended)"; "-log2 is used only as a **measurement**". The owner pinned the 0.90 bar to the `chat-v0-p2` `heldout.u16` stream (e5f400b0…) [checked docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md:3,17,47]. A zero-mass target is unscorable [checked same sheet:73].
- The M1 trunk is already a count model inside the D11 state: "Exact additive sufficient-statistic training over a public 19-table M1 trunk", using `uor_r4_discrete::trunk::{SparseCountTable, TrunkTables, TrunkTrainRow}` [checked crates/uor-r4-cpu-train/src/bin/cpu-trunk-count-driver.rs:1-19].

---

## BLOCKERS

### B1. §3.1: the predicting state is a classical hashed n-gram, and geometry is optional

**Finding.**
- S2, S4, S5 and S6 carry nearly all the predictive mass the plan expects. They are keyed by token identity:
  - S2's key is "product of token primes mod a table prime", stored with a 16-bit fingerprint;
  - S4's key is a pair product;
  - S5 is an exact suffix match;
  - S6 is a table of interpolation weights.
- Only S3 (the SpiralCore sign code plus the Pick read) and S1 are geometric.
- P2's fail branch says: "keep S3 as a hash and log it for §3.1". That ships a non-geometric default.
- §4.4 names count tables as the *control*. Under this plan the control becomes the model.

**Why it is classical, not prime routing.** The owner's prime-router direction (ADR-0003: primes as data, gcd admission, semiprime experts, ordered n-lets; pipeline-map.md:297-305 [checked]) is project geometry. The proposal removes its substance in two ways.
1. **Reducing mod a table prime destroys the arithmetic.** gcd admission and backoff by division stop working, so the key is just a hash.
2. **A product of token primes is order-invariant** (a mathematical defect). "the cat sat" and "sat the cat" share a key, and repeated tokens collide by multiplicity. ADR-0003 recovers order through the semiprime chain gcd(e_t, e_{t+1}) = p_t. A bare n-let product does not recover it.

**Fix: make the written state geometric in substance.**
- **S2 as a true prime route.** Give each (token, lag) pair its own prime q(t, j); about 24.6k primes, the largest about 2.8e5, so about 18 bits each [est]. The order-k key is K_k = Π_{j≤k} q(t_{-j}, j), exact in u128 for k ≤ 6 [est]. Properties:
  - the key is order-preserving;
  - backoff is exact division, K_{k-1} = K_k / q(t_{-k}, k);
  - shared-suffix admission is gcd(K, K').
  - Hash only for bucket placement, and verify the exact K in the row, not a 16-bit fingerprint.
- **S3 addressed by the transported orbit state.** Snap each 8-coordinate block of the S1 state to its nearest E8 lattice point with the Conway–Sloane decoder. That decoder uses integer rounding and a parity fix, O(8) per block, and is lawful.
  - The address is a product code of the per-block root indices: the existing 240×240 = 57,600 E8 product-key layout [survey A1c].
  - A miss probes the 240 root neighbours of one block: a bounded geometric neighbourhood. No nearest index is needed, which removes the contradiction with §5 item 6 (see M3).
- **Output through geometry (§3.12).** Values stay exact token ids (§3.12 keeps identity apart from address). At least one expert scores candidates by the Pick read between the predicted state code and each candidate's S0 E8 code, so the shared vector writer serves the output.
- **Pre-register the P2 fail branch.** A loss to the hash control keeps a geometric address on the default path: try another one (Hopf sector, 600-cell cell, golden-shell index). It logs a §3.1 proposal in the owner's queue and never adopts the hash as the default (§5.1).

### B2. §3.7/D85 and §3.12: two lanes again (tables beside the trunk, no shared operator)

**Finding.**
- The S2, S4 and S6 tables are new hash tables with their own key functions, read by a separate probe path.
- S1 (the trunk transport) touches only S3, and S2, S4 and S5 never pass through the trunk state.
- The prior review diagnosed exactly this failure: two lanes with no shared operator.
- The plan's GPU sort-reduce kernel (the write operator) and its CPU hash probe (the read operator) are not "the same typed operator definitions". They need a declared common key-and-read definition with bit-identical tests (§3.7; native-space SKILL.md items 1-3).

**Fix.**
- Build S2 and S4 **inside the existing D11 trunk state**, as further `SparseCountTable`s in `TrunkTables` read by `discrete_d11`. The M1-closing 7,002 model is already that: count tables in one integer state [checked count-driver.rs:1-19].
- The GPU counting kernel implements the `TrunkTrainRow` → `SparseCountTable` update behind one typed key definition (`pack_u32_key` or its prime-route successor). It must be bit-identical to the CPU driver and to `discrete_d11` serving on raw table words.
- Write the five native-space ORDER items before P1:
  - the state schema covering S0-S6;
  - the shared operator paths;
  - the replay oracle;
  - the update rule ("increment, then error-write", in place of a gradient);
  - the lossless format.
- Without these five items, P1 is an inventory unit, not a training run (SKILL.md "Before writing model code").

---

## MAJORS

### M1. Owner intent: "1B-equivalent" as defined moves the goalposts

**Finding.**
- Q1 = 0.80 × C6, and C6 is S2 alone: the plan's own first phase, smoothed by a recipe the author picks. So the bar is an internal ablation, and a weakly smoothed C6 makes it easy.
- 5.3e8 count cells is about 0.5B, and a count is not a parameter-equivalent. A pruned table of 5e8 n-grams from 1.87e9 tokens holds information on the order of the corpus itself (memorization), and generalizes only through backoff [est].
- The owner's words: "a 1B model is at least sort of useful"; "real intelligence" [checked DECISIONS.md:291].
- The 1BW ratio comes from shuffled single sentences. That understates the neural advantage on long documents, and D87 traces are long and self-repeating.
- Q3 demotes the owner's own 0.90 BPB bar to "secondary, does not gate". That is a bar change, so it needs a decision card (§5.1). The plan also says the absolute bar was "tuned on web text", but the owner pinned it to the `chat-v0-p2` held-out stream [checked m4 sheet:47].

**Fix: an honest, testable definition. "1B-equivalent" requires all of the following.**
- (a) **Budget:** ≥ 1e9 learned integer cells, each written or refined by training, in ≤ 4 GB resident with a lossless artifact. Report D5-a/D5-b (§3.9).
- (b) **Absolute quality:** ≤ 0.90 BPB on the owner-pinned `chat-v0-p2` held-out stream, through PRLL. Also report it on a sealed D87 DEV split.
- (c) **Relative quality:** ≤ 0.80 × a *fixed external recipe* count model on the same bytes. Use modified Kneser–Ney order 5 with the standard discounts, or PPMd at a stated order, reproduced to a published fixture. This is the §4.4 lawful baseline, not the plan's own C6.
- (d) **Replies:** R1-R3, plus a *fresh* held-out-phrasing panel (§4.3 transfer), each above the no-memory control.
- (e) **Speed:** ≥ 1,000 tok/s on the M1.
- **Naming:** until (b) and (d) pass, call the result "count-class, 1B-cell", never "1B-equivalent".
- **Neural numbers:** use the published ones (1BW, nncp) only as background, never as a gate. §3.10 bans neural comparators, and a calibration-by-proxy is close to one.

### M2. Owner intent: little geometric learning is left on wick

**Finding.**
- Counting is maximum-likelihood training in the statistical sense. Even so, the geometry (S1) learns nothing unless P5 passes its gate, so the transport stays frozen at its default.
- The only error-driven learning is S4's refinement in 1-3 passes. Memory: "Geometry carries learning" (owner, 12 Oct).
- **Fix:** make error-written refinement the training operator over **all** written rows (S2, S3, S4). Counts initialize the rows, then the output error is written into every addressed row, as the owner's step 2 says ("written by the output error") and as lexlean's working part does [memory note]. Run it for several D87 epochs on wick through the existing BVG-K/rows-K block step (`run_block_transitions`), so the trunk and the rows train on one path.
- The scored S1 core starts in P3, not after it, with a re-seeded random-transport control.

### M3. S3's miss path needs the nearest index that §5 item 6 forbids

**Finding.**
- "On a miss, the Pick XOR+popcount read takes the nearest filled code" over 2^24 cells per scale is nearest-neighbour search in Hamming space.
- Doing it without an index means a scan, and the plan bans building the index. So the plan contradicts itself.
- The 64-128-bit code must also be cut to 24 bits, which is truncation or hashing.
- **Fix:** E8 lattice decoding plus the 240-neighbour probe (B1). Alternatively, the existing CUDA inverted file (13/13 PTX, parity at V ≤ 4,096 [survey A1c]); it is an existing engine.

### M4. Process: the plan replaces an owner-approved order and an owner direction without a card

**Finding.**
- PLAN.md:38-39 records the gap order and "start the 1b training run on wick ... proceed with it".
- The plan drops step 3 (the readout), turns step 6 (the 1B run) into a counting pass, and says "Replace it".
- The owner's 16 Nov words invite a better track, but §5.1 and §5.8 still require a decision card approved after the top session's own decision review. That card is logged in the owner's queue with its review record, and the PLAN row is updated the same day.
- **Fix:** one decision card ("written-state track supersedes steps 3, 5 and 6 of PLAN.md:38"), reviewed against the whole project, logged, and then go ahead. Keep step 2's "written by the output error" literally.

### M5. P6 is a new donor generation and needs an owner release

**Finding.**
- "Extend the donor corpus on wick and recount" is a D41-family donor pass. §3.10 says "an owner release (§5.1) is required at dispatch", and D87 leaves "wick's role after the first corpus" to a later owner decision.
- **Fix:** P6 becomes an owner-queue proposal with its sheet staged, and the lab goes on with other work. It is not a plan step the lab runs.

### M6. Existing and retired engines dropped against "prioritize existing engines ... even if they are retired or failed"

These A1c revival candidates are dropped, or deferred behind the P2 gate:
- **Starling ~7-neighbour hard read.** Its retirement cause, the softmax surrogate, is gone, and §3.1 requires "~7 nearest neighbours" for flock reads.
- **E8 orbit/product-key memory.** It is deferred "as a trained table", yet its slot layout is cited as S3's reuse. That is inconsistent: make it S3 (B1).
- **VSA SplitMix64 binding.** Measured −0.014 to −0.023 BPB. The plan uses SplitMix only as the *control*. Run VSA as a candidate address alongside it.
- **Icosian-mantissa weights.** At about 2.5 B per weight they suit the ≤ 4 GB budget for the S6 and scored-core weights.
- **The pooled-rank objective, re-registered as a hinge** (437/512). It is not named. It should be the S4 refinement objective.
- **The whole-path source** (compiler + exact store + E8 index + emitter). It is unblocked by a hard pick and is unmentioned.
- **The skip-mix table-injection lane** (+28.45‰). It is the existing precedent for S6 and is unmentioned.
- **The CUDA inverted file.** Unmentioned.
- **The object store (D19).** Only "if mmap is not enough".
- **The Hamilton gate** (14.2k tok/s integer). Unmentioned.
- **Kept:** the rank-weight read (as S6), #989, the W2 tables, the Pick read and rows-K.

**Fix:** give each one a named slot or a P2/P3 arm. Mandatory are Starling (as the miss read), E8 product-key (as S3), hinge (as S4) and VSA (as an address arm).

**Assumption 2 also ignores internal evidence that geometry carries information** [survey A1c]:
- quaternion transport beat identity by 0.071-0.077 nats;
- VSA gained;
- E8 orbit lost to a learned product key by only 0.0231 BPB.

### M7. The BPB instrument as specified cannot score count models

**Finding.**
- PRLL marks a zero-mass target unscorable [checked m4 sheet:73]. A union-of-hits candidate set gives zero mass to every target outside it.
- P0 also builds a new instrument when the owner-approved PRLL module (`crates/uor-r4-integer/src/lawful_likelihood.rs`) already exists.
- **Fix:** reuse PRLL. Add a lawful floor expert: integer uniform mass over V, plus a unigram. Compute the normalizer in closed form, Σ_t score(t) = Σ_e w_e · L + floor·V, so no V-wide loop is needed.
- Drop P0's "decision review whether a count ratio is lawful". Owner Card 1 already approved a linear ratio [checked m4 sheet:3,17].

---

## MINORS

### m1. §3.2 is clear, with stated exclusions

- A count ratio, linear interpolation, KN/Witten-Bell discounting and PPM escape are linear ratios, not normalized exponentials.
- Division normalization is not banned: D81 allows integer divide in serving, and PRLL's ratio is owner-approved.
- **Exclude explicitly:**
  - logistic, stretch or logit mixing (stretch is the inverse of the sigmoid, a renamed form);
  - SSE/APM stages;
  - exponentiated-gradient, Hedge or multiplicative-weights updates for S6 (normalized exponentials);
  - maximum-entropy or log-linear smoothing.
- The instrument may compute −log2 of the integer mass ratio in the offline scorer only (Card 2).

### m2. §3.3: the mixer is not a dense product, if it stays small

- S6 is a per-candidate sum over E ≈ 10 per-expert scalar weights: interpolation, not GEMM.
- Forbid growing it into a learned vocab×feature or candidate×hidden weight table. That would be a dense mat-vec in table form (the lexlean output layer, memory note) and "a dense model disguised as lookup tables" (§3.6).

### m3. §3.5/D84: the CPU oracle is allowed only as a finite fixture

- `cpu-trunk-count-driver` is a named D84 module ("D84 CPU development research only" [checked :3]). Using it as the parity oracle on fixed fixtures is a finite kernel test or host check (§4.5).
- It may not count full D87 to produce or qualify tables.
- All Q and R scoring runs on wick's GPU (§3.6: "panel scoring and milestone evaluation remain GPU research compute"). The M1 measures only tok/s and the replay parity.

### m4. §4.5: P1, P2, P3 and P5 are GPU training or evaluation

- Each needs a one-page sheet and a non-author FIT. The plan never mentions sheets.

### m5. Statistics

- Count models are deterministic, so "2 seeds" means nothing for them. Use ≥ 200 fresh rows and a fresh panel number (§4.3).
- 6,420 = G + 2·sqrt(npq) is an unpaired band. Two models on the same 14,287 items need the *paired* band, about 2·sqrt(discordant pairs), the method M1's band 82 used.
- ft/dev has been the development panel for six weeks. Every gain needs Fresh V2 or held-out phrasing.

### m6. Wording against §4.1

- "Kill the count base" should read "close this configuration". A negative closes a configuration, never a mechanism.

### m7. Replay coverage

- P4's replay on 3 prompts is thin. Compare raw words for every carried field over a fixture of at least 64 prompts, including misses and backoffs.

### m8. Control design in P2

- A hash of the last k tokens and a sign code of the whole prefix carry different information.
- Add a **random-transport** control: a re-seeded, non-geometric, path-dependent integer map of the same prefix at the same code width and fill rate. It isolates *geometry* from *any path mixing*.
