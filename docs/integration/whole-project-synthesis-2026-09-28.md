# Whole-project synthesis and the track decision

2026-09-28 · Director (Claude, Lab 1) · Owner request: "synthesize all results so we can pick the correct track … create the novel missing mechanisms" · References #820, #973, #962, #963, #964

**Status.**
- The track (§2) is **owner-approved**: "Approve, start with D2", 2026-09-28.
- Its measurements (§4) are pre-registered here, and **D0 is decided**: the stack is the core.
- An external Codex audit was verified and integrated on 2026-09-28 (§7). It corrects the scope of three claims, corrects D3's cost accounting, and adds D6, an evaluation-only information audit. It also adds D7, a geometric address channel.
- **Owner decisions, 2026-09-28 15:27 UTC:** D6's outcomes are evidence, not a veto; and the geometric read/address operator (**G**, formerly D7) is on the main path.
- **Decided so far:** D0 (the stack), D1 (keep the quaternion transport), and D2 (frozen gate FAIL on the margin; the store works, the learned read does not generalise).

**Labels:**
- **Measured:** a project record, cited.
- **Derived:** arithmetic on records.
- **Literature:** arXiv IDs, graded in the review packet.
- **Hypothesis:** untested.

**Sources.** Five read-only research passes built this synthesis:
- the native lineage;
- geometry by role;
- the stack, serving and efficiency;
- dialogue and addressing;
- the external literature, with IDs verified.

Their ledgers cite about 150 records under `docs/integration/` and `docs/evidence/`, plus the unmerged lab branches.

---

## 0. The answer in one page

**Five findings from the whole record.**

1. **No model in this project has yet been given the conditions to learn useful language.**
   - **Capacity:**
     - The retained native model has 1.68M parameters, only 0.63M of them outside the embedding.
     - The dialogue model has 5.4M; the stack has 7.15M.
     - TinyStories coherence emerges at about 8–30M parameters (Literature 2305.07759).
   - **Conditioning:**
     - In 74.6% of the dialogue model's supervised positions, the whole preceding prompt lay outside the 256-token window.
     - Only 11.45% of training responses fit with their prefix (*Measured*, `dialogue-context-audit-2026-09-27.md`).
   - **Data:** the stack was trained on code, and the dialogue line on 6.3M response targets.
   - **The same failure everywhere:**
     - The best dialogue model's response loss, 3.068, never beat an order-5 count model at 2.466 (`dialogue-artifact-replay-2026-09-27.md`).
     - Instructions score 0/7 in every arm.
     - Prose is 0/5 at every native checkpoint (`language-continuation-result-2026-09-26.md`).
   - *Hypothesis:* that these limits are the **cause** of the failures is a working hypothesis. The facts above are measured; the causal reading is not. D0 (capacity-matched) and D5 test it (§7).
2. **Geometry used as a drop-in *score or mixer* ties or loses in every parameterisation tested, once the model is deeper or larger.**
   - Lorentz reads: +0.021 and −0.021 inside the stack.
   - The finite 2I read score was harmful: complete answers fell from 27 to 17.
   - Rotation lost to diagonal decay by 0.009 bits/byte.
   - The one clean positive, removing the stack's rotation costing +0.059, is confounded, from a single seed.
   - The records give reasons (`geometry ledger`; *Measured*/*Derived*):
     - depth absorbs the geometry;
     - several contests compared geometry against a relabelled copy of itself;
     - finite coding *can* erase information.
   - **Scope** (§7):
     - These negatives retire the parameterisations tested, not geometric reads as a family.
     - The 2I read changed three things at once: it dropped each lane's magnitude, snapped to 120 roots, and replaced the dot with a learned MLP.
     - Its norm-controlled arm never ran, so its cause is **unresolved**. D6 settles it without a new fit.
3. **Geometry used as *exact discrete structure* wins wherever the task has that structure.**
   - Finite-group lanes track A5 exactly to length 4,096 in 6 of 6 seeds, each compiling to an exhaustively verified 60-state automaton. A matched transformer scores 0.008–0.027 (B1 §6, §8).
   - Exact identity is load-bearing: the KVAR overwrite store scores 0.83 and 0.75, against 4 of 204 for the count baseline (`kvar-hard-successor-result-2026-09-24.md`).
   - Authored versioned stores answer updated relations at 60/60 and 474/474 (`docs/native_geometric_current_version_read_973.md`, `docs/native_geometric_historical_version_973.md`).
   - In every win, an ordinary component given the same structure catches up. The value is **exactness**, and geometry is one way to get it.
4. **What fails is the *learned interface* to exact structure, and read ranking.**
   - A1–A4 admitted the right source but selected it at most 2 times in 24.
   - The oracle re-rank took a distractor class from 0/5 to 5/5, so ranking is sufficient (#1440).
   - Whenever a score rather than version order decides, the stale record wins (`scoped-memory-review`).
   - Context lanes on swap stories rotated at almost every token and learned nothing (Stage C).
5. **The hard artifact's losses come from the weight representation. Its energy cost comes from instruction count.**
   - At width 576, rounding flips 15–23% of greedy decisions, while the interface and the integer arithmetic add fewer than 10 flips (`dialogue-child-native-observation`, `precision-factorial-result`).
   - Serving emulates about 97k runtime-by-runtime products and 9.5k divisions in software per token (review §3.4). That is why it costs about 4.3× the J/token of float (ROADMAP §4.3).
   - Lab 3's width-576 path now runs at 3.2 ms/step and 23 MB RSS, with exact parity (#1450, #1452).

**The track.** A small recurrent core that learns language under the right conditions, given *architectural* exact memory for what small models cannot learn, and served through a geometry-coded hard artifact. Geometry moves out of the scores, where the evidence retires it, into the places where it gives exactness or multiplier-free structure:
- exact identity addressing;
- exact automata;
- lattice/Hadamard weight coding;
- exact ℤ[φ] fixed transforms.

**Four mechanisms the record says are missing** (§3):

| | Mechanism | What it fixes |
|---|---|---|
| M1 | **Architectural exact relational memory (AERM)** | Updated relations |
| M2 | **Conditioned, anti-echo response training** | Responsiveness |
| M3 | **Geometry-coded hard artifact** | Fidelity and energy |
| M4 | **Event-gated exact state** | Explicit state, later and optional |

**Decisive first measurements** (§4), each small and each able to change the plan:
- **D0 (decided 05:50 UTC):** the cycle-4 reading. The stack scored 1.998113 against its transformer control's 2.011149 at full exposure, so the stack family is the core.
- **D1:** transport attribution at matched width.
- **D2:** the AERM probe against a dense-read core at equal parameters.
- **D3:** the T2 index contest. As coded it is a compression-fidelity screen; its index decision moves to a gain-controlled follow-up with real inspected cost (§7).
- **D4:** geometry-coded quantization of the existing dialogue child against #1433's legal-code result.
- **D6 (new, from the audit):** an evaluation-only audit of what the 2I read representation discards. No fit.
- **G (formerly D7; on the main path since 15:27 UTC, owner):** the geometric read/address operator. D6 and D3 shape it rather than gate it.

**What stops** (§5):
- geometry as a drop-in score or mixer;
- exposure-only continuation;
- lanes in the language path;
- per-product-table kernels;
- single-seed authored panels as capability evidence.

---

## 1. Evidence synthesis

### 1.1 Why nothing talks yet: capacity, conditioning, exposure

| Model | Parameters | Training | Best language evidence | Source |
|---|---|---|---|---|
| Retained native (joint recurrent) | 1,678,466 (62.5% embedding) | 64.2M target visits over 15,672 steps, TinyStories-#1017 | Tail NLL 1.975–1.996, against #1017 at 1.574 and 5-gram+cache at 2.392. Prose 0/5 | language-continuation-result |
| R1d / dialogue child (width 576) | ≈5.4M | 6.3M response-supervised positions, chat-v0 | Response NLL 3.068 vs count 2.466. Factual 0/8, instructions 0/7, middle turns unresponsive in 10/10 | dialogue-artifact-replay; dialogue-prefix-paired-result |
| Cycle-4 stack `rrarra` | 7,153,860 | Code (lab BPE), 29,999,104 visits | **Final, 7,324 updates: 1.998113 vs its transformer control 2.011149** (512 development windows, 131,072 targets), 789.3 vs 788.1 tok/s. Continuations repetitive. Code-loss parity selects an architecture; it does not qualify dialogue (§7) | cycle 4 §6–8; #1437 |

**Measured:**
- The read and copy path is load-bearing: removing it costs 0.48–0.52 nats and drops answers to 0/32.
- The learning slope is about 0.15 nats per e-fold of exposure, yet prose stayed 0/5 from step 8,348 to 15,672.

**Literature and Derived:**
- One M1-week trains roughly 20–40M parameters (review §0, §9).
- TinyStories coherence needs about 8–30M parameters and at least 2 layers for instructions (2305.07759).
- The smallest instruction models with published scores are 135–360M parameters (SmolLM2-135M-Instruct: IFEval 29.9).
- No peer-reviewed open-domain multi-turn chat model exists at 5–50M parameters.

**Reading.**
- The product milestone is reachable at M1 scale only as a narrow-domain conversational agent.
- It must be trained on dialogue with the prompt inside the window, at 10–30M parameters.
- It needs help for exactly the things small models cannot learn, such as exact overwrite of facts.

### 1.2 Memory and retrieval: exact stores work; learned interfaces and ranking fail

**Exact stores** (*Measured*):

| Store | Result |
|---|---|
| KVAR gate + token-addressed overwrite | 0.8284 and 0.7451 fresh accuracy, vs count 4/204 |
| Authored revision memory | 60/60, fresh 28/28 |
| Initial-version requests | 474/474 |
| Scoped (entity, relation) store | 16/16, 11/11, 11/11. Controls: NoRead 0/38, Unscoped 33/38 |

**Where it fails** (*Measured*):
- **Learned admission and selection:** A1–A4 selected the source at most 2 times in 24, and answered 0 of 12.
- **A learned sparse read gate** collapsed to recency: 319 vs 323.
- **Bounded admission** lost answers: 17/17 against 27/25 with full access.
- **Ranking:**
  - Swapping the read mass between the entity and the distractor gave 5/5 (#1440).
  - Scores beat version order and the stale record wins (ParseScoreAuthority 19/38).
  - One authored write-side link error defeated a correct store: the third "now" fact was stored with previous=0.
- **The missing piece**, named by `addressed-lexical-bridge` and `native-sparse-read`: a *learned mapping from language to the address*.
- **KVAR's own caveat:** it trained only with its soft, unscaled read-gate surrogate. Direct hard training reached 0.196.

**Literature:**
- Tool use as a learned text API appears only at about 775M parameters (Toolformer 2302.04761). At 1–10M parameters memory must be **architectural**: fixed addresses plus a learned gate.
- Fixed or hashed addressing beats learned routing (Hash Layers 2106.04426). Hashed n-gram memory with prime-sized multi-head tables and a context gate gave large gains at 27B (Engram 2601.07372, grade C).
- External edit memories beat weight edits on updated facts (MQuAKE 2305.14795, SERAC 2206.06520).
- Rerankers dominate retrieval quality (1901.04085).

### 1.3 Geometry by role

| Role | Verdict | Key numbers | Why, per the records |
|---|---|---|---|
| Continuous score (Lorentz, H4, 2I, curvature) | Ties or loses at depth, **in the parameterisations tested** | Stack ±0.021; transfer +0.046; 2I read HARM (27→17 answers); curvature stays at t≈0.02 | Absorbed by depth; the small-scale gain was mostly read temperature and copy distance, not hierarchy. **The 2I read's cause is unresolved**: it dropped per-lane magnitude, snapped to 120 roots and swapped in a learned MLP together, and its norm control never ran (D6, §7) |
| Mixer or transport (quaternion) | Tie or loss; one confounded win | Diagonal 1.869 vs quaternion 1.880 BPB; D8 Q vs O was quaternion against quaternion; `norot` +0.059 (1 seed, MLP-confounded) | Complex or rotating gates lose to real gating on text (Literature 2312.00752) |
| Exact finite state (lanes → automata) | **Win** on A5; text cost unstable | Exact to 4,096 in 6/6 seeds; transformer 0.008–0.027; ΔNLL up to +0.069; natural-language tracking not learned | NC¹ tracking needs non-diagonal transitions (2404.08819). Language-model gains concentrate in code, math and state tracking (2411.12537, 2502.10297) |
| Hash, address, identity (prime, CRT, zeta, Hopf) | Identity yes, similarity no | Hashing lands within 0.2% of the ideal collision count; zeta-off 91/96 beats full 86/96; Hopf routing loses to a random gate | A prime product is a bitset; CRT destroys nearness |
| Codebook or quantization (E8, D4, 600-cell) | Generic lattice gain; H4 ties k-means on synthetic data | E8 +0.85–1.22 dB over scalar on synthetic data; 600-cell ≈ k-means | Lattice shaping is real and capped at 1.53 dB. **Exact add-only decode** is the geometric asset. QuIP#'s E8P: 2-bit MSE 0.089 vs scalar 0.118 (2402.04396). **T2's geometric-vs-learned contrast is confounded**: its H4/E8 arms decode to unit roots, k-means to raw centroids (§7) |
| Exact arithmetic (ℤ[φ], icosians) | Instrument | A 2I rotation is 24 add/sub + 8 shifts; any decay forces rounding | Exact only for isometries |

**Across roles.** Geometry wins where the task has the matching structure: non-solvable groups for lanes, explicit hierarchies for hyperbolic keys. On generic next-token loss it ties. The project's founding premise survives in a narrower form: **exact addressed memory and exact finite structure**, served without multipliers.

### 1.4 Serving and efficiency

**Fidelity is a weight-representation problem** (*Measured*):

| Model | Parameter step | Interface and integer steps |
|---|---|---|
| Native | +0.037 to +0.050 nats | +0.00003 |
| Dialogue child | 909 of 3,914 greedy decisions flip | 3 and 4 flips |
| Stack (per-32 group scales) | 5–7% flips, +0.012 nats | — |

- The native format's per-row power-of-two scales are much coarser than the stack's group scales (*Hypothesis*: that granularity explains much of the gap).
- Learned rounding, GPTQ and QAT each recover 40–55% of the gap.

**Energy is an instruction problem** (*Measured*/*Literature*):
- Per-product tables replace one cheap multiply (0.07–3.7 pJ) with a cache read (about 5 pJ) (Horowitz; 1506.02626).
- The winning lookup-table kernels do three things (T-MAC 2407.00088; −20% to −61% energy measured on M2 Ultra):
  - keep the table in registers;
  - replace 2–4 or more products per lookup;
  - store weights at 2 bits or fewer.
- A cache-resident 1–20M model needs its own measurement.

**Serving now** (Lab 3, #1450/#1452, *Measured* by the lab):
- Width-576 dialogue runs at 3.16 ms/step, 304 tok/s, 23.2 MB RSS and 1.82 MB touched per token, with exact 58-turn parity.
- The upgraded call-graph auditor finds 0 forbidden instructions, re-checked with the completed patterns (#1451).
- A capability API and a WASM build exist.

### 1.5 Process lessons

**What wasted weeks:**
- Mechanisms were tested as drop-in components on 32-row authored panels with one seed. "Admission versus ranking" was confused.
- Instrument defects:
  - a 1024× unit error;
  - fit/serve thresholds that did not match;
  - a 64-token training horizon against a 256-token evaluation;
  - dialogue windows that dropped the prompt.
- 11 owner decisions in 7 days, and 3 direction changes within 14 hours on 09-24 (review §7.3).

**What worked:**
- pre-registered gates with kill rules;
- same-binary anchors (the B1 anchor was bitwise exact);
- sealed report roots;
- independent review.

---

## 2. The track decision

### 2.1 Options

| Option | For | Against | Verdict |
|---|---|---|---|
| A. Scale the core only | Capacity is the first-order gap | A 10–30M model will not learn exact fact updates or new instructions by itself (§1.1, §1.2) | Necessary, not sufficient |
| **B. Core + architectural exact memory + geometry-coded artifact** | Offloads exactly what small models cannot learn onto structures the project has *proven* work; puts geometry where it has a structural reason to win | Needs a learned write/read gate, where KVAR needed a surrogate; needs narrow-domain data | **Chosen** |
| C. More geometric components | Founding premise | The fairly tested ones tie or lose (§1.3) | Stop |
| D. Converted transformer | Quality now | Excluded by D11 | — |

### 2.2 The architecture

```
tokens ─► [recurrent core: `rrarra` stack (if D0 passes) or native learner, 10–30M params]
            │  in-window reads (the stack's `a` layers) for copy/recall within the window
            │  recurrent state carries context across window boundaries
            ├─► [M1 AERM: exact relational memory, integer]
            │      key  = multi-head prime-sized hash of (entity span, relation), xor/shift only
            │      write gate (learned) → overwrite with version++, previous link kept
            │      read gate (learned) → current-version value → residual + copy distribution
            └─► head → reply (M2 anti-echo training; learned end of turn)
served as [M3 geometry-coded bundle: Hadamard incoherence + lattice/grouped 2–4-bit codes,
           LUT accumulation, product tables, audited D11 kernels]
```

### 2.3 How the track meets the approved milestone (ROADMAP §8)

| Milestone category | How the track answers it |
|---|---|
| Responsive multi-turn | M2: the prompt stays in the window, anti-echo training, a learned end of turn; the recurrent state carries older turns |
| Updated relation | M1: version order, not score, decides the current value, exactly as the authored stores that scored 60/60 |
| New simple instructions | Narrow-domain instruction data from an offline teacher (a declared training source, never serving responses), at least 2 layers, distillation |
| Saved and reloaded hard artifact within a budget | M3 plus the existing sealed bundles, and Lab 3's exact save/restore (0.08/0.27 ms) |

---

## 3. The missing mechanisms (designs)

### M1. Architectural exact relational memory (AERM)

**Why.** It has three evidence bases:
- exact identity is load-bearing (KVAR);
- authored version-ordered stores answer updates perfectly;
- small models cannot learn a text API (Toolformer), so memory must be architectural.

**What it is.** A new layer between the core's layers, with integer state:

**Addresses.**
- `k_h = hash_h(entity-span ids, relation id) mod P_h`, for h = 1..H hash heads.
- The table sizes `P_h` are distinct primes. This is the project's prime addressing, in its proven role of exact identity.
- The hashes are xor/shift (multiplier-free).
- The entity span and relation are *not* parsed by rules. The core's hidden state proposes them through two learned pointer heads over the last W tokens, trained with the gates.

**Record.** `(key, value span ids, value embedding, version, previous)`, in a fixed-capacity table with typed eviction (Evicted and Absent are distinct statuses; AGENTS.md chain-traversal rule).

**Write.**
- A learned gate `g_w(h_t)` decides whether to write.
- On a key hit it **overwrites** and increments the version, keeping `previous`.
- Version order decides "current". It is never a score.
- The write is deliberately an explicit, non-invertible operation. A group action is invertible, so no rotation can overwrite. Geometry supplies compatibility and transport; explicit writes supply the record lifecycle (§7).

**Read.**
- A learned gate `g_r(h_t)` and the same pointer heads form the query key.
- The current version's value embedding is added to the residual through a zero-initialised projection, and its token ids feed the copy distribution.
- Typed statuses (Hit, Absent, Evicted) are exposed to the core as embeddings, so it can abstain.

**Training.**
- End to end with the core.
- The gates use KVAR's working soft read-gate surrogate (`kvar-hard-successor-result`), annealed to hard.
- Dense supervision comes from synthetic relation dialogues (§4 D2) with gold write/read positions as an auxiliary loss. The lesson from Stage C: answer-only supervision fails at this scale.

**Serving (D11).** Hashing, table reads, integer compare and add. No products.

**Geometry.**
- Prime-sized multi-head tables give exact identity (Literature: Engram's design).
- **Geometric address channel (G, §7.3; on the main path since 15:27 UTC).** It replaces the earlier "fixed 600-cell/E8 fuzzy index":
  - records also carry a learned 2I key code with a quantized gain;
  - a query visits the cells `g_q·d` for d in a small support S, then follows exact postings to records;
  - D6 and D3's gain-controlled follow-up shape its representation and cost claims;
  - the exact hash channel above stays canonical either way.

**Novelty relative to KVAR.**
- It is integrated into the language core, not a side experiment.
- It has version order and typed statuses.
- It uses learned pointer heads instead of caller-supplied addresses, which is the missing piece named in §1.2.

### M2. Conditioned, anti-echo response training

**Why.**
- 74.6% of supervised positions lacked the prompt.
- Models echo the first fact: in 10/10 middle turns, the reply restates turn 1.
- Likelihood-trained dialogue copies context n-grams at about 5× the human rate, and unlikelihood training cuts this by 69% with no perplexity change (Literature 1911.03860).

**What it is.**
- **Turn-structured windows** that always contain the current user turn and as much history as fits, from the end backwards. Context is at least 512 (the stack trains 256-token windows at 1,720 tok/s, so 512 is affordable).
- **The recurrent state is carried across window boundaries** (truncated backpropagation through time, TBPTT). This is the recurrent core's structural advantage over attention: a bounded window with unbounded carried state.
- **Loss:** response-weighted cross-entropy, plus an unlikelihood penalty on n-grams copied from earlier turns unless the gold reply copies them, plus learned-EOS weighting.
- **Decode:** integer n-gram blocking and a minimum length, as in BlenderBot (2004.13637).
- **Data:** narrow-domain conversations (the user's name, family, places, preferences, simple tasks) generated offline by a teacher and filtered, plus TinyStories-Instruct-style data. Assertions, updates and queries are balanced, and distractors are included.

### M3. Geometry-coded hard artifact

**Why.** Rounding flips 15–23% of decisions at width 576, and table emulation costs about 4.3× the energy.

**What it is:**
- **Incoherence:** a randomized Hadamard transform before coding. It uses additions and subtractions only; at d = 4^k the normaliser is a shift (Literature: QuIP#, QuaRot 2404.00456).
- **Codes:** E8-lattice codes at 2 bits/weight (add-and-compare decode; the E8P table is 1 KiB), or grouped 4-bit codes with power-of-two scales (shifts).
  - Chosen by measured flips and relation retention, not by loss alone.
  - Always **quantization-aware**, with the codebook in the loop: tiny, heavily trained models are the hardest to quantize after training (2411.04330).
- **Kernels:**
  - lookup-table accumulation with tables held in registers, replacing 4 or more products per lookup (T-MAC);
  - runtime-by-runtime products via quarter-square tables;
  - B1's verified automata where exact state is served.
- **Audit:** the completed call-graph auditor (#1451).
  - Register-resident lookup tables need `unsafe` intrinsics, so they belong in one separately audited kernel crate, or a safe-Rust fallback.
  - **This is an owner decision** (D11 and `forbid(unsafe_code)`).

### M4. Event-gated exact state (deferred, optional)

**Design.**
- Lanes write only where a learned event gate fires; otherwise their transport is the identity.
- Evaluation is on state-tracking tasks (code variables, who-holds-what), not perplexity (Literature 2411.12537).

**Status.** It enters only after M1–M3, and only with new causal evidence (D9). The Stage C witness showed that ungated lanes rotate at almost every token.

**Design constraint** (§7). A lane transition is a group action, so it is invertible, and it cannot express reset or overwrite. Any such semantics need an explicit non-invertible write beside the lanes.

---

## 4. Decisive measurements (pre-registered)

| ID | Question | Arms and controls | Gate and decision | Cost, owner |
|---|---|---|---|---|
| **D0** | Which core? | Cycle-4 main comparison (frozen, published by the cloud session) | ROADMAP §2 rule: stack within 0.03 nats → the stack is the core; otherwise the native learner stays, and the stack goes to ablations. **Decided 05:50 UTC:** 1.998113 against 2.011149 (−0.013), at 789.3 against 788.1 tok/s. **The stack family is the core**, and the 1.68M native model is frozen as the baseline. The decision selects an architecture; the code-BPE weights do not transfer to the dialogue tokenizer (§7.1) | 0 local; cloud. Weights copied and verified on the SSD |
| **D1** | Is the geometric transport in the core load-bearing? | `rrarra` quaternion vs identity transport at **matched MLP width**, 2 seeds, 1,000 updates, cycle-4 data | Quaternion better by ≥0.02 in both seeds → keep. Within 0.02 → drop rotation (simpler serving). Worse → drop. **Decided 09:36 UTC: keep.** Identity transport costs +0.0711 and +0.0774 nats (2.599845 → 2.670985; 2.580265 → 2.657687) | About 3.4 h cloud; cloud track |
| **D2** | Does architectural exact memory give small models updated relations? | Core = B1's `rrar` (1.39M) with M1 AERM vs the same core with in-window reads only, at equal parameters (widened) and tokens. 3 seeds, plus #1017 text. Synthetic relation dialogues cover: assert, update, same-value reassertion, owner swap, role reversal, absent relation, prior-version query and intervening distractor turns (§7.2) | Updated-relation query accuracy ≥0.90 with AERM **and** ≥0.30 above dense-only in every seed; text ΔNLL ≤0.05. The gate is on the update-then-query-current class; the other classes are reported. Every failure is classified by the four-class trace (§7.2). Fail → AERM parked at this scale, and the track falls back to A plus the milestone's narrowed scope. **Result: the frozen gate FAILS on the margin** (0.146 / 0.122 / 0.102). The memory arm scores 1.000 on every class in distribution, where the control has 0.51 on First and ≤ 0.07 on Absent. **Held out, its learned read trigger does not fire**, so accuracy drops to 0.00–0.035, while the gold register would give 1.000 ([result](d2-aerm-probe-result-2026-09-28.md)) | About 1 day implementation, 1.6 h slot; Lab 1 |
| **D3** | Is a geometric index as good as a learned one at equal cost? | **Frozen T2 contest (complete locally):** stands as a **compression-fidelity screen** at its scope. Its decode cost is relabelled, because the harness scores every event (§7.1). **Deciding follow-up:** every arm gets the same charged gain channel, or none; equal total bits; a real cell index whose cost counts inspected events, cells, postings and LUT builds; correct-source admission reported beside fidelity to the dense ranking | Within 1 point of recall@s and 0.005 nats of the best ordinary index at equal bits, with **fewer inspected events** and a multiplier-free decode → G may use it. Otherwise G uses ordinary indexing. **Frozen run: NOT QUALIFIED at its scope** (#1456) | Lab 2 |
| **D4** | Does geometry-coded QAT fix the hard artifact? | The existing dialogue child: Hadamard + E8 (2-bit) vs Hadamard + grouped 4-bit vs nearest per-row. The #1433 legal-code arm is **measured: it fails** (2/10 relation answers against nearest's 4/10; 962 flips against 909; [result](dialogue-code-choice-result-2026-09-28.md)). The QAT objective is **fidelity to the continuous child**: distillation from it on its own complete-prefix trajectories, since #1433 shows corpus cross-entropy drifts away from the parent | Decision flips ≤5% (nearest: 909/3,914 = 23%), and the ten question-turn relations kept at least at nearest's 4/10, with Momo and green among them. The winner becomes the bundle coding for the milestone | Lab 3 (now unblocked) |
| **D6** | Does the 2I read representation discard what the reader uses, and does a gain channel restore it? | **Evaluation only, no fit** (§7.2). See the D6 notes below | Four named outcomes, defined below. **Amended by the owner, 15:27 UTC: the outcomes are evidence about the representation tested, not permission or prohibition for geometric reads.** Each constrains the design of G (below). The next reader experiment must target the failure D6 shows | Light job (≤2 threads, ≤1.5 GB, no model slot); Lab 2 |
| **G** (was D7) | The geometric read/address operator | **On the main path (owner, 15:27 UTC).** An always-on learned addressed read into the exact store and the context, built behind I4 with the stack's dense read as fallback. D2 names its first job: its trigger-gated read did not generalise, while its writes did. D6 and D3 shape its representation and cost claims. It is compared with an equal-bit ordinary channel (K) | Pre-registered before its first fit; replaces the fallback only if it wins at equal cost | Labs 1 and 2 |
| D5 | The milestone candidate | Core (D0/D1) + the exact store + G (or the dense fallback) + M2 data + M3 coding, 10–30M parameters, within an M1-week | ROADMAP §8 milestone (frozen) | The candidate fit waits for G's first comparison and D4. **Integration engineering proceeds now** (ROADMAP ruling 12) |

**D6 arms, population and metric.**
- **Parent:** #1438's same-dose, kernel-off checkpoint. Its anchor: read NLL 1.984753 ± 1e-3 and 27/32 complete answers. If that checkpoint is unavailable, use the step-15,672 parent with its own recorded anchor.
- **Arms.** Each substitutes the read score at evaluation. Age bias, NoRead, candidates, values/copy and decoding stay fixed.
  - **O:** the dot score (the anchor).
  - **U:** per-lane unit dot, unquantized.
  - **D:** 2I direction only, `Σ_l Re(g_q,l⁻¹·g_k,l)`, the fixed function without #1438's MLP.
  - **G:** 2I plus a 3-bit dyadic gain per lane, `Σ_l r̂_q,l·r̂_k,l·Re(g_q,l⁻¹·g_k,l)`, at 10 bits per lane.
  - **K:** per-lane k-means with 1,024 raw centroids, also 10 bits per lane.
- **Population:** the parent's selected read positions on development text (T2's protocol), plus the 32-row source panel.
- **Metric:** Δ(arm) is the fraction of positions whose top-1 read event differs from O's.

**D6 outcomes** (their reading amended by the owner, 15:27 UTC: evidence about the tested representation, not a veto).
- **Reported beside top-1:** the change in attention mass, NoRead mass, the read-out value vector (η and ε_v of §7.2) and the effect on complete answers. Top-1 agreement does not preserve these.
- **Magnitude is load-bearing** iff Δ(D) ≥ 0.05 and Δ(U) ≥ ½Δ(D).
- **The gain restores it** iff Δ(G) ≤ ½Δ(D) and Δ(G) ≤ Δ(K) + 0.01.

| Outcome | Condition |
|---|---|
| **GAIN** | Magnitude is load-bearing, and the gain restores it |
| **RESOLUTION** | Δ(D) ≥ 0.05 and Δ(U) < ½Δ(D): the snap loses the information, not the norm |
| **ORDINARY-BETTER** | Magnitude is load-bearing, and G fails either restoration criterion |
| **REPRESENTATION-ADEQUATE** | Δ(D) < 0.05: the coding keeps the top-1 read event. On its own this does **not** show where #1438's harm came from; the mass, value and answer effects below decide what else the coding changed |

- **Reported, not gated:**
  - the panel's correct-source top-1 and complete answers per arm, with one temperature per arm calibrated on a disjoint slice;
  - top-2 code collisions;
  - the per-lane gain distribution and clipping.
- **Stop rule.** Stop once the outcome is known, with no codebook sweep.

**Order.**
- **D0 is decided.**
- **Lab 2:** D3's frozen run is published (#1456). Next are D6, then D3's gain-controlled follow-up.
- **Cloud:** D1 is decided (keep).
- **Lab 1:** D2 is decided (frozen FAIL on the margin). Next are checkpoint save and reload for the stack plus store, and G's design once D6 reports.
- **Lab 3:** D4 is reported in #1458 and is under director verification.
- **G** is designed after D6, then pre-registered and compared with the dense fallback at equal cost.
- **D5:** the candidate fit waits for G's first comparison and D4. Integration engineering proceeds now (ROADMAP ruling 12).

---

## 5. What stops

- Geometry as a drop-in attention score or mixer: Lorentz, 2I, H4 reads and curvature (§1.3). No new score family without new causal evidence (D9).
  - **D6 is that causal test** for the read representation.
  - **Amended (owner, 15:27 UTC):** D6 shapes the geometric read/address operator G, which is on the main path. It neither permits nor forbids geometric reads as a family. A drop-in dense score replacement stays stopped.
- Exposure-only continuation of the 1.68M native model (already CLOSED).
- Finite-group lanes in the language path. The verified automata stay available as tools, and M4 is deferred.
- Per-product-table kernels (FAILED on energy).
- Single-seed, 32-row authored panels as capability evidence. D-gates use at least 3 seeds and fresh draws.
- Answer-only supervision for learned structure (Stage C).

## 6. Risks and open questions

**Risks:**
- **D2 may fail because the gates do not learn at 1.39M parameters.** Mitigation: the surrogate recipe, dense gold write/read positions, and a widened core. On failure, the milestone's "updated relation" category would need a larger core.
- **The narrow domain may still be too hard at 10–30M parameters.** No peer-reviewed 5–50M chat model exists. The milestone's thresholds stay frozen, and a miss is reported, not retrofitted.
- **Lattice QAT at this scale is unproven.** The literature is at 0.7B and above. D4 measures it.
- **The `unsafe` lookup-table kernels are an owner decision** (M3).

- **D6 may show that the coding keeps the top-1 read** (REPRESENTATION-ADEQUATE). That narrows where the loss lies, but it does not close the geometric-read question: mass, value and answer effects are reported beside it.

**Open questions:**
- **The tokenizer and corpus for D5:** one tokenizer for dialogue and stories (I5).
  - D0 selected the stack *architecture*.
  - Its weights were trained on the lab code BPE, a different token identity from #1017, even at the same vocabulary size. No weights or panels transfer between them.
  - The D2 core (`rrar`) already uses the #1017 token store.
- Whether cycle 5's product-key memory adds capacity cheaply enough to matter at D5. It re-enters after D0, on the chosen core.

---

## 7. External audit integration (Codex, 2026-09-28)

**Source.** An external Codex research packet delivered by the owner at about 05:50 UTC, imported verbatim with a claim ledger at [`docs/evidence/external-codex-audit-2026-09-28/`](../evidence/external-codex-audit-2026-09-28/README.md).
- The director read all 18 files.
- Its source claims were checked against the cited commits.
- Its two supplied probes were rerun locally, with identical checks and script hashes.
- One construction, the 2I orbit codebook, was recomputed independently.
- Nothing in it was a trained model or a language result.

### 7.1 Corrections accepted

| Claim in this synthesis or the ROADMAP | Correction | Evidence |
|---|---|---|
| "Geometry used as a drop-in score or mixer has been tested fairly" | The negatives retire the **parameterisations tested**. #1438 changed three things at once: it dropped per-lane magnitude, snapped to 120 roots and swapped the dot for a learned per-lane MLP. Its norm control (arm C) never ran. | `geometric_read.rs` lines 1–19 and 460–490; result record lines 61–66. Unit normalization provably can reverse rankings, even on exact 2I directions (ledger rows 5 and 12) |
| "The prose deficit follows capacity and exposure, not geometry" | This is a **working hypothesis**. The measured limits are real; their causal sufficiency is untested. D0 (capacity-matched) is consistent with it; D5 tests it | Ledger row 15 |
| D3: "a cheaper multiplier-free decode" from T2 | T2's harness **scores every previous event** (`arm_ranking` over `code_cache[..previous]`) while charging only the s retained. It measures compression fidelity, not index cost. Inspected and retained events must be reported separately | `joint-addressing-contest.rs` lines 338–345, 832 and 366–392 at `b0f70c63` |
| D3: geometric vs learned codebooks | H4 and E8 decode to **unit roots**, k-means to **raw centroids**, and there is no gain channel. The contrast is confounded by magnitude, so a gain-controlled follow-up decides D3 | `addressing_arms.rs` lines 713–725 |
| D0: "the stack is the core" | It selects the **architecture family**. Code-loss parity does not qualify dialogue, and code BPE and #1017 are different token identities | Ledger row 15 |
| Recall of the dense reader's top events as the index metric | The dense reader's favourites include its mistakes. **Correct-source admission** is reported beside fidelity | NOTE2 §1 |

### 7.2 What it adds to the measurements

- **D6: an information audit before any new geometric reader** (§4). Evaluation only, with fixed arms and a stop rule. It is the "new causal evidence" D9 requires before any read-score work resumes.
- **The four-class decision trace**, used for every D2 and D5 failure:
  1. the relevant record was unavailable (never written, or evicted);
  2. it was available but not selected (address or ranking);
  3. it was selected but the value or action was wrong;
  4. the latent result was right but the emission was wrong.

  Each class needs a different repair. Aggregate accuracy hides which one failed.
- **Counterfactual coverage in D2's generator:**
  - owner swaps and role reversals;
  - same-value reassertions against changes;
  - absent relations;
  - current against prior versions;
  - intervening irrelevant turns.

  The oracle labels are training and evaluation only. Runtime decisions come from text (AGENTS.md chain-traversal rule).
- **Error accounting for any selected read** (ledger row 11). The one-step bound `‖o−ô‖ ≤ 2Rη + R·min{2, e^{2ε}−1} + ε_v` separates three losses: omitted mass η (admission), score error ε (coding) and value error ε_v. D3's follow-up and D7 report η, ε and ε_v separately.
- **Whole-path cost** (Lab 3). Event sparsity does not remove dense projection or vocabulary-head reads. Every serving report keeps per-token parameter bytes beside event reads (R3, R5).

### 7.3 The mechanism it contributes: score-to-address compilation (G; on the main path since 15:27 UTC)

**Exact algebra** (ledger rows 7 and 10, reproduced). For a relation kernel κ on a finite group G with support S, records sharing a key cell and coefficient aggregate exactly:
- `N(q) = Σ_{d∈S} κ(d)·ρ(d)·M(q·d)`
- `Z(q) = Σ_{d∈S} κ(d)·C(q·d)`

NoRead applies when Z=0.

**What follows from it:**
- The query **visits |S| cells** instead of scoring every event. The read stage costs O(|S|·D), independent of history length. Encoding, postings, writes and the output head are extra and are charged.
- This gives geometry a computational job: reducing what is read.

**Three constraints the design keeps:**
1. **Binding** (ledger row 8). Sums and counts erase who-owns-what. Cells hold **exact postings** to AERM records, never summaries standing in for records.
2. **Invertibility** (ledger row 9). Transport and compatibility are group operations; overwrite and eviction are explicit writes.
3. **Magnitude** (ledger rows 5 and 12). Codes carry a quantized gain.
   - The orbit factorization lets an expanded codebook keep a small relative table `A[i,j,r]`: 1,920 entries in place of 230,400.
   - This is recorded as an option, not adopted.

**Occupancy caveat** (ledger row 13). With 120 cells, 8 probes and 256 uniform keys, a query sees about 17 candidates. With 120² cells it sees 0.14, and most queries are empty. Resolution is not free. Trained keys cluster, and the worst case (a full-scan fallback) is reported, never averaged away.

**Relation to the approved track.** This was first adopted as D7, AERM's optional approximate channel, replacing the earlier "fixed 600-cell/E8 fuzzy index". **The owner moved it to the main path at 15:27 UTC** as G, the geometric read/address operator. It is not a new learner. Its first fit is pre-registered before it runs, against an equal-bit ordinary channel.

### 7.4 Adjusted or not adopted

- **Lab allocation.** The packet assigns the attribution study to Claude. **D6 goes to Lab 2**, which owns both #1438's reader and evaluator and T2's query/key accessor; this matches the packet's own OpenCode allocation of "reusable traces … admission-versus-ranking accounting". Lab 1 runs #1433's endpoints and D2.
- **Superseded facts.** The packet predates D0: #1437's result was published at 05:50 UTC. The cycle-5 memory arms remain unrun.
- **Not adopted:**
  - margin certificates as a pruning mechanism (the packet's own result was 0/192 certified at coarse precision);
  - orbit-expanded codebooks before D6;
  - any new relation-kernel fit before D6.
- **Reported only:** the synthetic top-1 tables (25/121/112 of 192; 39–95 of 128) have no generator scripts in the packet, use random data and sometimes unequal bits. They motivate D6 and decide nothing.
