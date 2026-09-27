# UOR-R4 Geometric Language Model — multi-lab roadmap

**Director:** Claude, Lab 1 window. **Updated:** 2026-09-27 23:58 UTC. This file owns lab
assignments, track status, the dead-path register and the cross-lab protocol. Measured
results and retained artifacts live in [current state](docs/integration/current-state.md).
Ordered responsibilities and acceptance live in the [canonical plan](docs/integration/project-track.md).
Owner decisions live in [DECISIONS](docs/integration/DECISIONS.md). Director rulings here
coordinate the labs. They do not amend an owner decision record, and the owner keeps
strategic authority.

## 0. Mission and hard runtime rules

**North star: geometric intelligence.** The goal is a working geometric language model whose
runtime uses integer, fixed-point and geometric operations only: no transformer, no matrix
multiplication and no floating point. Training, data preparation, analysis and offline tooling
may use floats and matrix products. None of those may leak into the serving path.

The owner reaffirmed this target on 2026-09-27: "Keep the native, multiplier-free serving
target" ([#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279)).

The rules below apply to every lab. A serving change that breaks any of R1–R4 does not enter
the mission path.

| Rule | Meaning | Evidence required |
|---|---|---|
| **R1: no float** | No f32/f64 instruction and no libm call in any served symbol. | An instruction audit of the release binary (`scripts/serving_multiplier_check.py`, `scripts/audit_zero_matmul_serving.py`). |
| **R2: no multiplier** | No integer multiply or divide instruction in served kernels ([D0-b](docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted)). Products of runtime values use product or quarter-square tables, or exact geometric structure (signed permutations, ℤ[φ] shift-add). D10's hardware-multiplier exception is not adopted by any lab. | The same audit. |
| **R3: no matmul** | End state: no dense per-token access to a learned weight store ([D5](docs/integration/DECISIONS.md#d5--per-token-parameter-sparsity-is-the-terminal-serving-invariant)). In the interim, ≤4-bit maps executed as adds, shifts and table reads are permitted as labeled stepping stones. Each must be reported with its dense per-token parameter reads, and each must be owned by a track that is replacing it with addressed or sparse access. | Per-token parameter reads in every serving report. |
| **R4: no transformer** | No served backbone whose primary mechanism is stacked dense all-pairs attention plus MLP. A model counts as a transformer when most of its token-mixing layers are dense all-pairs reads. Converted open-weight transformers (D10's SmolLM2) may serve only as comparators or offline teachers. Recurrence-primary hybrids (most token mixing recurrent) are interim, and their reads move to bounded geometric or exact addressing (D5/D6). | An architecture statement in the PR. |
| **R5: honest cost** | An integer path is not an efficiency result. Energy may be claimed only when measured on the M1 in J/token. Bytes touched per token set the energy floor, not multiplier count ([first-principles review §5](docs/integration/first-principles-review-2026-09-25.md)). | Measured J/token, RSS and tokens/s. |

## 1. Director's reading: where geometry can earn its place

*This is a working hypothesis synthesised from the evidence in §5, not a measured claim.*

Geometry has failed as a **continuous score or mixer**. Every matched test of that role inside
a language model is null or negative (§5):
- quaternion transport against Householder or diagonal decay;
- Lorentz reads against Dot, across seeds and transfers;
- the 2I read score;
- snapped rotation codebooks;
- prime and CRT hashing.

The native model's prose deficit follows capacity and exposure, not geometry:
- It has 1.68M parameters and scores 0.41 nats behind the 7.16M #1017 transformer.
- Another 30M targets of exposure left prose at 0/5.

Geometry's defensible jobs are discrete and table-servable, which is also what R1–R3 demand:
1. **Exact non-abelian state (B1).**
   - Lanes over 2I ≅ SL(2,5) track the A5 word problem, which is NC¹-complete. Diagonal SSMs and constant-depth transformers cannot do this at arbitrary length, assuming TC⁰≠NC¹.
   - Snapped to 2I, a lane serves as a 120-state automaton: one byte of state and two table reads per token, with no drift.
   - Out-of-repository toy result: 100% accuracy to length 4,096, where float lanes drifted to 0.62–0.74 ([review §6.2](docs/integration/first-principles-review-2026-09-25.md)).
   - Reproduced in repository Rust (#1442), with one correction: the capability is the finite group, not the quaternion parameterisation. Ordinary reflection-pair lanes learn the same icosahedral group and minimise to the same 60-state automaton.
2. **Fixed geometric addressing (D5).**
   - Optimal spherical codes (the 600-cell, the E8 roots) can act as sparse indexes.
   - They need no learned keys, decode with adds and compares, and give sparse parameter or event access.
3. **Exact identity and versioned memory.** This is already load-bearing.

The tracks below put geometry in those three places, scale the learner, and serve it without
D10 exceptions.

## 2. Tracks and owners

Lab numbering is fixed as: **Lab 1 Claude, Lab 2 OpenCode, Lab 3 Anti-Gravity, Lab 4 Codex**.
Codex is the owner-authorized fourth lab ([.codex-lab/README.md](.codex-lab/README.md)); the
three-lab brief does not assign it work.

| Track | Owner | Goal | Current hypothesis | Status | Next decision point |
|---|---|---|---|---|---|
| **T1 Learner and exact state** | Lab 1 Claude: this window, plus the cloud lab track on `claude/blissful-wozniak-girwwq` | A single main-line learner at #1017 scale, with geometric state where it earns it | (a) The capacity-matched stack closes the native gap. (b) Exact 2I tracking lanes add A5-class tracking at ≤0.05 nats LM cost. (c) Fixed H4/E8 codebooks address sparse parameter memory as well as learned keys do. | (a) Full-exposure run in the cloud sandbox, ETA ≈07:00 UTC 09-28. (b) Stage A PASS (#1442); quaternion-specific claim retired; Stage B paused. (c) Implemented, NOT_RUN ([#1437](https://github.com/UOR-Foundation/uor-r4/pull/1437)). | §4.1 |
| **T2 Geometric addressing (D5×D6)** | Lab 2 OpenCode | Decide whether geometry can be the sparse index for event memory | A fixed 600-cell/E8 cell index retrieves the dense read's top events as well as LSH, IVF/k-means, PQ and learned kNN at equal bytes touched, with a cheaper multiplier-free decode. | Never run (D5 names it; nobody has run it). | §4.2 |
| **T3 Mission runtime and measured efficiency** | Lab 3 Anti-Gravity | Serve the main-line model under R1–R4, and measure its real cost on the M1 | LUT-accumulation kernels, exact 2I lanes and table products serve the stack without D10 exceptions, losing ≤0.02 nats. J/token is set by bytes touched. | `uor-chat`, blocked kernels and streaming delivered ([#1434](https://github.com/UOR-Foundation/uor-r4/pull/1434), [#1436](https://github.com/UOR-Foundation/uor-r4/pull/1436)). No J/token has been measured in the repository. | §4.3 |
| **T4 Native dialogue and conversion fidelity** | Lab 4 Codex | Recover learned relations through integer conversion, and dialogue learning on the native path | Response-aware legal-code choice recovers relations lost at conversion; conversion changed 909 of 3,914 greedy decisions. | [#1433](https://github.com/UOR-Foundation/uor-r4/pull/1433) at 231 of 512 updates. | §4.4 |

**Main-line consolidation rule (director, pending T1(a)):**
- **If the stack comes within 0.03 nats of its transformer control at full exposure:**
  - it becomes the single main-line learner;
  - dialogue learning (T4), read and addressing work (T2) and serving (T3) retarget it;
  - the 1.68M native model is frozen as the retained baseline.
- **Otherwise:** the native model stays the main line, and the stack goes to ablations under its pre-registered card.
- **Either way:** no lab starts a third learner.

## 3. Deconfliction rulings, 2026-09-27

1. **ROADMAP.md has one owner per section.**
   - The director owns §0–§3 and §5–§8.
   - Each lab edits only its own §4 subsection, through its own PR.
   - No lab rewrites the whole file. This version supersedes the rewrite in [#1439](https://github.com/UOR-Foundation/uor-r4/pull/1439).
   - #1439 should drop its `ROADMAP.md` change and stop re-carrying #1434 and #1436. Each PR merges on its own.
2. **The dialogue-child fit checkpoint belongs to Codex.**
   - Path: `/Volumes/UOR-Workspace/uor-r4-lab/fourth-lab-sequential-code-choice-fit-1` ([#1433](https://github.com/UOR-Foundation/uor-r4/pull/1433)).
   - No other lab evaluates, resumes or modifies it.
   - #1439's proposed observation at step 231 is not authorized. The pre-registered plan finishes all 512 updates, then compares.
3. **The 120-root H4/2I geometry is repointed.**
   - The dense read-score use is dead ([#1438](https://github.com/UOR-Foundation/uor-r4/pull/1438), HARM).
   - Codex's exact integer H4 classifier ([#1435](https://github.com/UOR-Foundation/uor-r4/pull/1435)) becomes the multiplier-free decoder for fixed-geometry addressing: T2's event index and T1(c)'s fixed-H4 memory index.
   - Its unpushed `inverse(q)*k` read-lookup table has lost its consumer and is parked.
4. **There is one mission runtime and one front-end.**
   - `uor-r4-integer` with `uor-chat` is the mission runtime and front-end.
   - `uor-r4-lut` with `lut-chat` (D10: hardware multiplies on runtime values) is frozen as a non-mission comparator and gets no new features.
   - The stack's mission serving is T3's port, fed by T1's export format.
5. **Hyperbolic geometry survives only as an index or memory geometry.** "Hyperbolic attention", meaning a Lorentz score inside dense reads, is dead at every tested scope (§5). It continues only as a candidate geometry in T2's contest and T1(c)'s memory index.
6. **Branch and worktree naming.**
   - New work uses `lab/<lab>/<topic>`, with lab ∈ {`claude`, `opencode`, `anti-gravity`, `codex`}.
   - Existing PR branches keep their names.
   - The `codex/` prefix currently hides three different labs, so every PR body must name its lab.
7. **Claims.** [frontier-geometric-transfer-synthesis-2026-09-27](docs/integration/frontier-geometric-transfer-synthesis-2026-09-27.md) overstates serving.
   - It says "0 soft attention matrices, 0 dense MLPs".
   - The served path in fact has full 256-token reads and dense per-token parameter access.
   - Its "5.8450 nats advantage" is a synthetic repeated-sequence recall test, not a language result.
   - Its owner should correct it when it is next touched. Until then this roadmap and current state govern.
8. **The reads-only stack is a transformer comparator (R4).**
   - The reads-only `aaaaaa` stack is six dense all-pairs reads with MLPs: a transformer with a different score and no RoPE.
   - Its lead over the control comes mainly from its age bias, its NoRead slot and dropping RoPE ([cycle 4 §7](docs/integration/geometric-stack-cycle4-2026-09-27.md)).
   - It stays a valid upper reference, and cycle-5 results on it remain valid index evidence.
   - The main-line decision in §2 uses a recurrence-primary pattern such as `rrarra`, not `aaaaaa`. The owner M1 scripts should not switch their default to `aaaaaa`.
9. **Serving audits must cover everything served.** Audit gaps are assigned to T3:
   - no opcode audit exists for `uor-r4-lut`, `uor-r4-simd` or the stack engine;
   - `audit_zero_matmul_serving.py` hard-codes owner-checkout paths;
   - it covers no `lorentz::*` or `sampling::*` symbols, and its `step_conversational\b` pattern cannot match `step_conversational_into`;
   - the integer Min-P sampler has a u128 multiply (`crates/uor-r4-integer/src/sampling.rs:120`), reachable through the library but not the CLI.

10. **Owner decisions, 2026-09-27 23:55 UTC** (put to the owner as prompts, each accepted as recommended):
    - **D10 is superseded programme-wide.** A D11 decision record (protected PR) makes native, multiplier-free, transformerless serving (R1–R5) the only serving contract for every lab. Converted transformers are comparators or offline teachers only. `lut-chat` is frozen.
    - **`.uor-models` moves to the SSD.** Lab 1 performs the move:
      - a SHA-256-verified copy to `/Volumes/UOR-Workspace/uor-r4-models/`;
      - a symlink at each old path;
      - deletion of the internal originals after verification;
      - a record under `.uor-cleanup/2026-09-27/`.

      Directories a running job is using are skipped until it ends.
    - **The director may remove other labs' idle worktrees.** Eligible worktrees have 0 dirty files, 0 unpushed commits and no process using them. Removal follows a 2-hour notice on #820, and the branch stays on GitHub.
    - **#1441 merges now.**

### Engine consolidation map

Fourteen engine paths exist. Only the ones listed as main line or active may receive new features.

| Engine | Disposition | Reason |
|---|---|---|
| Native joint model on `uor-r4-integer` (full256; width-256/576) | **Main line until T1(a)**, then the retained baseline if the stack passes | The only trained path that serves text end-to-end under R1–R2 with no transformer block. Dense access (R3 interim). Language 0/5. |
| Geometric stack (`geometric_stack.rs`) | **Main-line candidate** (T1(a)) | Best quality. Its serving must move from `uor-r4-lut` to R2 kernels (T3). |
| `uor-chat` and the integer conversation adapter | **Active: the single mission front-end** | Its zeta, Hopf and prime state is telemetry only and never enters scoring; do not claim it as geometry. It is width-256 only. |
| `uor-r4-lut`, `lut-chat`, `uor-r4-simd`, D10 LUT export | **Frozen comparator** | Hardware multiplies; transformer blocks (R2, R4). |
| TinyStories GeometricProse `.rgm` | **Frozen reference** | The only measured positive geometric ablation (JEPA +0.374 BPB, S2 read-out +0.338 BPB), but n-gram-class quality. Its float sampling and VSA multiplies break R1–R2. |
| TLA/R4G1 graph runtime | **Frozen contract** | Keeps its own no-multiply contract; not language evidence. |
| #1014/#1017 float transformer references | **Comparator/teacher only** | R1, R4. |
| Finite 2I read kernel; kappa conversion; low-bit core, A1–A4, TLX; router; #973 probes | **Parked or dead** | §5. |

Other consolidation debts:
- There are eight chat front-ends. Only `uor-chat` is extended.
- The Lorentz distance is implemented six times. New work reuses `crates/uor-r4-integer/src/lorentz.rs` for serving and `geometric_stack.rs` for training.
- There are five count-table models. `ngram.rs` is the one baseline in active use.

## 4. Track details

Each lab keeps its own subsection current: hypothesis, status, next decision and owned paths.

### 4.1 T1 Learner and exact state (Lab 1 Claude)

**Owned paths:**
- `crates/uor-r4-training/src/geometric_stack.rs`, `stack_memory.rs`, `stack_export.rs`, `stack_dialogue.rs`;
- the new `stack_tracking.rs`;
- `crates/uor-r4-training/examples/geometric-stack.rs`;
- the cycle-4/5 notes and packets.

**(a) Capacity: the stack at full exposure** (cloud track, [cycle 4](docs/integration/geometric-stack-cycle4-2026-09-27.md) §6).
- Setup: 7.15M-parameter stack against a 7.16M transformer control, 29,999,104 target visits each.
- At 1,000 updates the stack led by 0.158 nats, 2.6097 against 2.7680, on code.
- Pre-registered reading:
  - within 0.03 nats of the control → viable, with integer serving next;
  - more than 0.03 nats behind → ablations;
  - more than 2× slower → kernel work.

**(b) B1: finite-group tracking lanes in the stack** (this window; [#1442](https://github.com/UOR-Foundation/uor-r4/pull/1442), [record](https://github.com/UOR-Foundation/uor-r4/blob/lab/claude/b1-tracking-lanes/docs/integration/b1-finite-group-lanes-2026-09-27.md)).

- **Status at 23:43 UTC 09-27: Stage A PASS.** 36 runs.
  - Non-commutative lanes track A5 exactly to length 4,096 after snapping in 17 of 18 runs: quaternion 8/9, reflection pair 9/9. Phase and frozen lanes stay at chance.
  - Quaternion lanes land on 2I; reflection pairs land on the icosahedral rotation group of R³. Both minimise to the same 60-state automaton.
  - **The quaternion-specific serving claim is retired** by the kill rule below. The finite-group state claim survives at Stage A scope.
- **Stage B:** a one-seed pilot only.
  - With lanes the stack scores A5 1.000 in context, against 0.008 at position 128 without.
  - Text NLL is +0.052 nats, against a 0.05 gate.
  - The 3-seed grid is **NOT_RUN**; it was paused for the owner's storage directive.

Pre-registered before any run:
- **Mechanism.**
  - A token-conditioned side channel of K quaternion lanes: `h_t = q[token_t] ⊗ h_{t−1}`, with `h_0 = 1`, no additive input and no decay.
  - `q = normalize(raw)` is freely parameterised, not near-identity. The repository's near-identity form never learned A5 in the review.
  - The lane read-out feeds the residual stream.
  - Serving: token→icosian tables, a 120×120 Cayley table and a 120-row read-out table per lane. That is exact, multiplier-free and sparse.
- **Stage A, implementation gate, not a claim.**
  - Reproduce the review's out-of-repository toy result in repository Rust: A5 over 3 generators, trained at length ≤32 with a curriculum, tested to 4,096.
  - Arms: 2I lanes, commutative phase lanes, Householder-pair lanes and no transport.
  - 3 seeds, plus the snapped automaton.
  - Pass: 2I lanes exact at 4,096 when snapped, in at least 2 of 3 seeds, with commutative lanes at chance.
- **Stage B, decisive.**
  - Train the stack with and without lanes on natural text plus A5 windows.
  - LM gate: development NLL within 0.05 nats of the lane-free stack at equal tokens, over 3 seeds.
  - Tracking gate: A5 accuracy inside the context is ≥0.99 with lanes. The lane-free stack and the transformer control are reported at the same lengths.
  - Kill rule, from review §9.3: if 2I lanes fail to match the strongest non-diagonal control's tracking at lower serving cost, or cost more than 0.05 nats of LM, retire the geometric state claim from the serving path.

**(c) Sparse parameter memory** (cloud track, [#1437](https://github.com/UOR-Foundation/uor-r4/pull/1437)).
- A product-key memory replaces one MLP.
- The index is either learned (Dot or Lorentz sub-keys) or fixed (the 120 icosians, the 240 E8 roots).
- Five pre-registered arms, with 0.02-nat decision rules. NOT_RUN.

### 4.2 T2 Geometric addressing, D5×D6 (Lab 2 OpenCode)

**Question.** At equal bytes touched per query, does a fixed geometric index select the
causally available events that the dense read would weight most, as well as the best ordinary
index does?

**Instrument.** Start offline, on frozen read-head queries and keys dumped from a retained model:
- the native Dot model at step 15,672 now;
- the stack's reads once T1(a) resolves.

**Budgets.** Score s ∈ {4, 8, 16, 32} of up to 255 events.

**Arms:**
- recent-s;
- random-hyperplane LSH, 3 seeds;
- IVF/k-means, 3 seeds;
- PQ with asymmetric distance;
- a 600-cell cell index on 4-D key blocks with multi-probe, decoded by #1435's exact classifier;
- E8-root cells on 8-D blocks;
- a Lorentz distance index, only where the keys are distance-trained.

**Metrics:**
- recall@s of the dense top-1 and top-3 events;
- captured attention mass;
- greedy decision equivalence;
- NLL change when the read is restricted to the admitted set;
- decode operations per query.

**Populations:**
- natural development text;
- a D6 long-range panel (induction and copy at distance 32–255) with a verified at-chance count control.

**Decision.**
- If the geometric index comes within 1 point of recall@s and 0.005 nats of the best ordinary index, with a cheaper multiplier-free decode, it qualifies as T3's admission index.
- Otherwise ordinary indexing is adopted and geometric addressing is retired from the serving claim.
- This subsumes the read-localization "oracle re-rank" question: whether admission keeps the correct entity found at rank 2–3.

### 4.3 T3 Mission runtime and measured efficiency (Lab 3 Anti-Gravity)

**First: the first measured J/token in the repository.** Measure on the M1 with macmon or powermetrics, from two run lengths:
- the F32 session;
- the retained integer session;
- a llama.cpp SmolLM2-135M Q4_0 reference.

That decides which serving levers matter: bytes, instructions or products.

**Second: the stack's mission port.** Serve T1's export in `uor-r4-integer` without D10 exceptions:
- table products;
- a rounded fixed-point quaternion scan;
- exact 2I lanes;
- Dot or Lorentz reads through tables;
- integer sampling.

Gate:
- an R1/R2 audit that is clean;
- a fidelity loss of ≤0.02 nats against the float model at equal inputs;
- the decision-flip rate reported.

**Ongoing.** `uor-chat` stays the single front-end.

### 4.4 T4 Native dialogue and conversion fidelity (Lab 4 Codex)

- Finish #1433 to 512 updates.
- Then run the 161-response comparison and the 58-turn observation, as pre-registered.
- After T1(a), retarget dialogue learning to the main-line base, or record why the native path stays.
- #1435 repoints to the addressing decoder, per ruling 3.

## 5. Dead and parked paths

Do not resume any of these without new causal evidence and a decision it can change
([D9](docs/integration/DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract)).
Each negative keeps its exact scope; a failed parameterisation does not retire a whole family.

| Path | Verdict | Scope and numbers | Source |
|---|---|---|---|
| Lorentz score in dense reads ("hyperbolic attention") | **DEAD as a score** | Transferred Lorentz and Affine readers are +0.0464 and +0.0133 nats against Dot. Inside the default stack, Lorentz−Dot averages −0.0001 over 2 seeds. The cycle-3 win at reduced scale did not transfer. | [radial result](docs/integration/radial-adaptation-result-2026-09-27.md), [cycle 4 §7](docs/integration/geometric-stack-cycle4-2026-09-27.md) |
| Finite 2I read score | **DEAD (HARM)** | Read NLL +0.019719; complete answers 27→17 of 32; 1.84× cost | [#1438](https://github.com/UOR-Foundation/uor-r4/pull/1438) |
| Quaternion rotation as a general content mixer | **PARKED** | −0.025 nats against Householder (1 seed); diagonal decay 1.871 against quaternion 1.880 bits/byte (2 seeds); snapping every lane to 2I +0.16–0.18 bits/byte | [review §6.3](docs/integration/first-principles-review-2026-09-25.md) |
| Count-prior blend | **DEAD** | +0.004928 [−0.026, +0.041] against the `(prev,cur)` table | [count-blend result](docs/integration/ordinary-lexical-count-blend-result-2026-09-23.md) |
| Exposure-only continuation of the 1.68M native model | **CLOSED** | Step 15,672: continuous prose 0/5 and 0/5; integer 0/5 and 1/5 | [continuation result](docs/integration/language-continuation-result-2026-09-26.md) |
| Termination-weighted objective | **INERT** | 3 unique rows against the 4 required; prose 0/5 | [termination review](docs/integration/termination-objective-review-2026-09-27.md) |
| A1–A4 local selector tuning | **PARKED** | A4 scored 0/12 complete answers | D8 |
| orthant64 bounded admission; recent64 training | **FAILED / WITHDRAWN** | 17 of 32 source answers in both arms | D9 |
| Learned sparse-read selector | **DEAD** | Lost to a plain recent cache, 319 against 323 | [sparse-read result](docs/integration/native-sparse-read-result-2026-09-24.md) |
| Prime and zeta mechanisms in predictive paths; prime/CRT/Galois hashing | **RETIRED from prediction** | No better than simple tabulation | [phase 2](docs/integration/geometric-lab-phase2-2026-09-26.md), [review §9.6](docs/integration/first-principles-review-2026-09-25.md) |
| VSA, coarse lattice tier, lane tables (Sep 19 model) | **DEAD** | VSA −0.0013 BPB; lattice tier net-negative at 64.2% of the artifact | current-state archive |
| Curvature in converted dot heads; horocycle position prior | **DEAD** | Curvature stays flat at t≈0.02, and forcing it costs 0.0011–0.0015 bits/byte; horocycle 3.264 against 3.226 | [phase 2](docs/integration/geometric-lab-phase2-2026-09-26.md) |
| Golden-gate rotation codebooks for compression | **PARKED** | At practical sizes, covering is no better than random, with a hole near the identity | [review §6.4](docs/integration/first-principles-review-2026-09-25.md) |
| D10 converted SmolLM2 as the served model | **OUT OF MISSION (R4)** | Comparator or teacher only; no real SmolLM2 checkpoint was ever converted | [D10](docs/integration/DECISIONS.md#d10--a-converted-open-weight-backbone-is-the-interim-chat-vehicle-serving-arithmetic-restated) |
| `uor-r4-lut`/`lut-chat` with hardware multiplies on runtime values | **FROZEN comparator (R2)** | Integer stack gap 0.011–0.013 nats, measured under D10 only | [cycle 4 §8](docs/integration/geometric-stack-cycle4-2026-09-27.md) |
| 2I relation lookup table for the parked read (`inverse(q)*k`) | **PARKED** | Its consumer, #1438, is parked | Ruling 3 |
| Quaternion (2I) lanes as a serving advantage over ordinary non-commutative lanes | **RETIRED** (B1 kill rule) | A5 at length 4,096 after snapping: quaternion 8/9, reflection pair 9/9. Both minimise to a 60-state A5 automaton. The finite-group state mechanism itself survives. | [#1442](https://github.com/UOR-Foundation/uor-r4/pull/1442) |

## 6. Shared machine protocol

The M1 has 16 GB of RAM and 8 cores, shared by four labs. On 09-27 one lab's fit was stopped by
storage and swap pressure while another lab's fit ran.

**The model slot.**
- A heavy job is training or evaluation above 2 GB RSS, or above 2 threads for more than 10 minutes. Only one heavy job runs at a time, machine-wide.
- Claim the slot before starting by writing `/Volumes/UOR-Workspace/locks/model-slot.json`.
  - Fields: `{lab, branch, pid, started_utc, expected_end_utc, threads, rss_cap_gb}`.
  - Delete it when the job ends.
- If the slot is taken, queue. Never kill or pause another lab's job.

**Light jobs.** Builds, unit tests and small probes may run alongside, at ≤2 threads and ≤1.5 GB, with `CARGO_BUILD_JOBS` ≤ 3.

**Storage** (owner direction, 2026-09-27):
- **Internal drive: keep at least 60 GiB free.**
  - Below 40 GiB, no lab starts a new build cache or heavy job on the internal drive until it has migrated.
  - At 25 GiB, running jobs checkpoint and stop.
  - It had 27 GiB free at 23:40 UTC 09-27. The table is on [#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5860924838).
- **Build caches:** one per lab, at `/Volumes/UOR-Workspace/BuildCaches/<lab>`, set through `CARGO_TARGET_DIR`. This replaces the earlier per-topic caches, since each cache can be 10–25 GB. Prefer release builds.
- **Checkpoints, derived data and reports:** `/Volumes/UOR-Workspace/uor-r4-lab/<lab>-<topic>`.
- **Migration:**
  - Never move a directory an active job is using.
  - Each lab migrates only its own material.
  - Regenerable caches, and clean, fully pushed, idle worktrees, may be moved or removed once no process is using them.
  - Unique artifacts get a verified copy (SHA-256 manifest) and a symlink at the old path, recorded under `.uor-cleanup/<date>/`. Deleting the internal original of a unique artifact needs the owner's approval, because the SSD would then be the only copy.
- **SSD:** keep at least 30 GiB free inside `UOR-Workspace`. Ask the owner before resizing the image.
- `UOR-Workspace` is an APFS sparse image on the X10 Pro SSD. Never point Cargo at the ExFAT volume itself.
- **Every status report includes both `df` numbers,** internal `/System/Volumes/Data` and SSD `/Volumes/UOR-Workspace`.
- Never delete another lab's worktree or artifacts, or unique research. The one exception is the owner-granted director removal of clean, fully pushed, idle worktrees after a 2-hour notice (ruling 10).

**Ledgers.**
- Report model compute and orchestration separately per work unit.
- Two ledgers disagree at present: OpenCode's 756M-ms ledger and the Codex lab's. The Lab 2 and Lab 4 leads should reconcile them into the programme ledger in current state.

## 7. Cadence and protocol

**GitHub is the shared record for all four labs** (owner direction, 2026-09-27).
- **The lab board** is one [#820 comment](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5860922002), edited in place by the director (`gh api -X PATCH repos/UOR-Foundation/uor-r4/issues/comments/5860922002`). It has one row per lab:
  - track and current unit;
  - branch and PR;
  - last pushed SHA and time;
  - uncommitted file count;
  - model-slot use;
  - internal and SSD storage;
  - next decision.

  The director refreshes it at each work-unit boundary and after every lab report.
- **Start of each unit:** read the board, then post a work card in the existing owning issue (#973 for model work, #962 for dialogue, #820 for process). It names the lab, track, branch, PR, owned paths, deliverable, decisions and resources.
- **Commit in small steps.**
  - Push at the end of every unit and before any run longer than 10 minutes.
  - Open a draft PR from the first commit.
  - Every PR body names its lab.
- **Results go into git** as a doc plus an evidence JSON. Large artifacts stay on the SSD, with their SHA-256 and path recorded in the evidence.
- **End of each unit:** post a status report in the same issue, including both `df` numbers.
- **Worktree audit at each work-unit boundary** (director):
  - run `git fetch --prune`;
  - for every `git worktree list` entry, record the branch, ahead/behind against `origin/<branch>` and the dirty file count, with `--no-optional-locks` so nothing is written into another lab's worktree;
  - post the findings on #820 and ask each owning lab to commit and push.

  No lab commits, resets or pushes another lab's worktree. The owner checkout's dirty files belong to the owner.

- **Zoom out** when stuck for about 45 minutes, or when a work unit ends:
  - reread §0–§1 and your track;
  - write 3–5 sentences on whether the work serves geometric intelligence;
  - choose continue, pivot or prune;
  - mark any dead path dead here.
- **Status report at the end of each session:**
  - what changed;
  - the hypothesis tested and its result;
  - roadmap edits;
  - GitHub and disk state;
  - the next step;
  - anything needed from the director or another lab.

  Post it in the owning issue ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973) for model work, [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) for programme), and in your §4 subsection when the track state changes.
- **Delivery.**
  - Feature branches, draft PRs for cross-lab review, pushed at least at the end of every session.
  - Main stays green, with merges only after director review and owner approval under protected delivery.
  - Partial results say `References #N`.
- **Testing.** Smoke tests, property checks and one decisive experiment per hypothesis. Quarantine flaky or peripheral failures and note them. No testing loops.

## 8. Capability responsibilities

These are canonical in the [project plan](docs/integration/project-track.md); [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) is the tracker.

| Order | Responsibility |
|---|---|
| 01 | [#1139](https://github.com/UOR-Foundation/uor-r4/issues/1139) Contextual phrase and role binding |
| 02 | [#1140](https://github.com/UOR-Foundation/uor-r4/issues/1140) Shared state transitions and compositional emission |
| 03 | [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) Integrate the geometric model and learn general prose (active) |
| 04 | [#962](https://github.com/UOR-Foundation/uor-r4/issues/962) Conversation and identity-scoped durable memory |
| 05 | [#954](https://github.com/UOR-Foundation/uor-r4/issues/954) Grounded correctness, conflict handling and abstention |
| 06 | [#955](https://github.com/UOR-Foundation/uor-r4/issues/955) Generalized multi-step reasoning |
| 07 | [#1088](https://github.com/UOR-Foundation/uor-r4/issues/1088) Executable Rust coding and controlled workspace use |
| 08 | [#963](https://github.com/UOR-Foundation/uor-r4/issues/963) Complete-path M1 latency, energy and memory |
| 09 | [#964](https://github.com/UOR-Foundation/uor-r4/issues/964) Scoped serving, geometry and artifact guarantees |
| 10 | [#1172](https://github.com/UOR-Foundation/uor-r4/issues/1172) Native capability API and WASM runtime |
| 11 | [#1173](https://github.com/UOR-Foundation/uor-r4/issues/1173) The native model in GitHub Pages AI Studio |
| 12 | [#965](https://github.com/UOR-Foundation/uor-r4/issues/965) Qualify, release and iteratively improve the local model |

General prose, reasoning, coding, chat quality, frontier capability and complete-path energy
savings all remain unqualified.

## 9. Director log

**2026-09-27 22:45 UTC, first assignments.**
- **Survey.**
  - Four labs are active.
  - Anti-Gravity's #1439 rewrote this file.
  - Codex and Anti-Gravity target the same checkpoint.
  - The H4 geometry is in three labs.
  - There are two serving contracts and two chat front-ends.
- **Rulings:** §3.
- **Assignments:** T1–T4 (§2).
- **Dead-path register:** created (§5).
- **Mission guard:**
  - D10's backbone and runtime-multiplier exceptions are out of mission (R2, R4).
  - The reads-only `aaaaaa` stack is a transformer comparator, not a main-line candidate (ruling 8).
- **The single decisive experiment still missing programme-wide** is D5's addressing contest. It is assigned to Lab 2 as T2.

**2026-09-27 23:55 UTC, owner decisions.**
- The owner asked to be prompted with every decision, each with a recommendation.
- Four were put and all accepted:
  - supersede D10 (a D11 PR follows);
  - migrate `.uor-models` to the SSD;
  - allow director removal of idle worktrees after notice;
  - merge #1441.

  See ruling 10.

**2026-09-27 23:43 UTC, owner direction: GitHub record and storage.**
- **Lab 1 delivery:** B1 was committed and pushed at `09e537a4`, draft #1442, with a #973 status report.
- **Board:** the #820 lab board was created, to be edited in place.
- **Worktree audit:** posted on #820.
  - Anti-Gravity's `codex/geometric-lm-goal` was never pushed and holds 13 dirty unique files.
  - Codex's `852c1c67` is on no origin branch.
  - `canonical-address-routing` has 2 untracked files, owner unclear.
- **Storage:** the internal drive has 27 GiB free, below the new 40 GiB rule.
  - The internal drive holds `.uor-models` (26 GB) and 14 worktrees (17 GB). Nearly all build caches are already on the SSD.
  - §6 rules and §7 protocol adopted.
  - Lab 1's cache moved to the per-lab path `BuildCaches/claude`.

The [previous roadmap](https://github.com/UOR-Foundation/uor-r4/blob/dccef74b/ROADMAP.md) and the
[historical roadmap](https://github.com/UOR-Foundation/uor-r4/blob/bc03f2d7ffde99608da370808eca542360e54508/ROADMAP.md)
are preserved in history.
