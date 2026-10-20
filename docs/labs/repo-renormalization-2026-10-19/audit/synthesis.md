# Synthesis: are we overcomplicating it?

*Revised 19 Oct 2026 after the adversarial review (`synthesis-review.md`). Findings accepted are fixed in place; partial rejections are listed under "Disputed" at the end.*

**Short answer: partly yes, but not for the reason first given.** The project has a lawful candidate for most transformer slots, and many were built and checked at finite controls. But nothing on the current model path has learned language: the best trained discrete model is below a unigram count table. The cause is **not established**. The first draft blamed a 16-bit token code; the project's own K16 record rejects that (training broke much of the aliasing, yet top-1 stayed at 65 and 463, `docs/labs/k16-outcome-call-2026-10-19/README.md`). What the evidence leaves open is ranking capacity, the read, the address key and the training schedule. Since 12 October most recorded work has been non-learning work, part of it owner-mandated rebuilding (D23–D32) and part of it discretionary gating. The owner's fear is right that the parts have never been stitched together. It is wrong if it means the remaining work is only assembly: a decisive, already-approved diagnostic (W2) is still queued, and every later step needs new build work and owner decisions (sections 4 and 6).

Paths are relative to the repo root. Audit inputs are under `docs/labs/repo-renormalization-2026-10-19/audit/`.

## 1. Stitching map

| Slot | Best-evidenced lawful replacement (status) | Strongest evidence | Gap |
|---|---|---|---|
| Tokenizer | 4,096-piece BPE `d36d3e87` (ACTIVE) | Every corpus, panel and artifact is pinned to it (`mechanisms/tokenizer.json`) | No vocabulary comparison and no number-aware tokenizer were ever trained (`tokenizer.json` never_tested) |
| Embedding | Address-structured injective cells A8 on the PrimeZeta injective codebook (ACTIVE) | At zero training, admission rose from 127 to 1,351 of 3,847 rows (`docs/labs/address-cells-preflight-2026-10-19`). PrimeZeta gives 4,096 of 4,096 tokens unique vectors (`docs/labs/prime-zeta-construction-2026-10-14`) | The carrier is 4 blocks × 16 roots = 16 bits per vector over a 12-bit vocabulary, so an injective code is possible (architect §3.1; `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`). The *seeded* initializer gave only ≤16 distinct vectors, capping top-1 at 1,986 rows at init (`k16-outcome-call`). Learning erodes A8's placement: Relation 1,351→551, control 857 (`docs/labs/a8-dose-2026-10-19`). Full 240-root E8 never used (`discrete-learning.json`) |
| Position / order | **No mechanism is assigned, and no lawful evidence exists.** The only related result is about **recurrent state transport**, not position: quaternion transport beat identity by 0.071/0.077 nats in an `rrarra` core at 1,000 updates (`docs/integration/transport-attribution-d1-2026-09-28.md`) | That core had softmax attention, a sigmoid/exp gate and dense MLPs, ran in the lab sandbox, and its identity arm had 332,928 fewer parameters; the record itself asks for a parameter-matched real-gate control. The quaternion arm was no better than Householder at one paired seed (`recurrence-gates.json`) | Not on the discrete path. Route-holonomy and icosian-holonomy reads are held proposals (`ROADMAP.md` retained table) |
| Attention / read | **Discrete:** Route, Flock and Relation (ACTIVE). **Float history (all negative or softmax-assisted):** the best softmax-free read, arm W, was +0.0172 BPB float / +0.0166 served and 15% slower (195.2 vs 228.4 tok/s), recorded **REJECT**; its q/k gradients were the flock-softmax straight-through surrogate and it ran through the CPU host fallback. Headline arms B and D missed by +0.040 and +0.259 (`docs/labs/softmax-free-read-2026-10-10`). The starling base-3 tree kept its own hit-rate target but recall was **not met** (0.329 vs softmax 0.999); its salience was trained against the dense teacher's attention with a distillation KL (`docs/labs/starling-tree-2026-10-12`). The "matched softmax on 3 of 5 seeds" row needed a dense bootstrap (`ROADMAP.md` retained table) | The integer Flock passes 25 CUDA controls (`docs/labs/discrete-driver-cost-2026-10-13`) | No discrete read's value has been isolated: Route got 20 proposals in 16,404 batches (`docs/labs/attention-state-2026-10-18`). Every float read that learned had softmax help in training (`starling-surrogate-2026-10-12`, `starling-distill-2026-10-12`) |
| Feed-forward / memory | Hamilton gate G2 and SpiralCore/FBS projection (READY). E8 orbit memory (PARKED) | Hamilton gate: RTX 5090 parity 43/0 and 18 rung-0 CUDA controls (`hamilton-gate.json`). E8 replacing half the MLPs cost +0.033 BPB at ~10× fewer operations, with a sigmoid gate and CPU steps (`docs/labs/e8-orbit-memory-2026-10-11`; `e8-orbit-memory.json`) | **None has ever been trained.** The discrete path has no feed-forward block (architect §5). The architect rates the F32 operators "RETIRE at tested scope for now" and finds no E8-memory site on the discrete path (see section 6, Disputed) |
| Normalization | **None.** Snapping to an E8 root and the unit-quaternion gate may play this role, but no record says so (`mechanisms/board.md`) | — | Not designed |
| Output head | Admission (K nearest of 256 cells) plus the nearest token by L1 distance (ACTIVE) | Best trained top-1 is **65 and 463 of 14,287** (K16-A seeds 1 and 2), with admission 2,237 and 3,167; both seeds fail G-Adm (`docs/labs/k16-outcome-call-2026-10-19`) | Below a **unigram count at 1,226** and a context-free successor table **T2 at 6,300** (`docs/labs/host-diagnostics-2026-10-17`). The address key takes only 5 values to order up to 256 cells, so the K-boundary is mostly decided by cell-id labelling (architect §3.2, a proof from `discrete.cu`). The writer scans the whole vocabulary, about 5×10⁶ comparisons per row, which breaks D5-b (architect §3.5) |
| Loss | Integer rank-margin hinge (ACTIVE) | The R objective collapsed to 19–20 occupied cells. The A objective kept 192–209 cells occupied (`k16-outcome-call`) | The hinge has no likelihood, so it has no BPB (`rank-margin.json` never_tested). Yet the M1 bar D39(1)(d) requires **≤ 0.90 BPB** (`DECISIONS.md` D39) |
| Optimizer / learning | Discrete counterfactual moves (ACTIVE). D37 float training is legal but unused | The row-parallel kernel is about 56× faster (`docs/labs/counterfactual-rows-parity-2026-10-18`). BVG-K scores 16 coordinates per launch (C1 qualified, `NOW.md` row 7; `docs/labs/bvgk-rowsk-scoring-2026-10-18`) | A full 16,404-batch run accepts **407 / 130 / 410 / 129** moves (`NOW.md` K16 receipts) and sees under half of TRAIN (1,049,856 of 2,425,925 targets; `discrete-learning.json`) |
| KV cache / context | An exact log plus a prime sieve (PARKED as exact-memory). D19 object store (READY). Discrete Hamming binding (VSA) is a never-tested lawful option (`vsa.json`) | The exact log did the retrieval where the learned pointer copied the most recent value (`eras/eras.md` #1558, #1559, #1607). The exact store scored 1.000 whenever the register was right (`exact-memory.json`). The object store has 60 tests and a conformance PASS (`object-store.json`) | "Address as the read" has never been built. Nothing consumes the object store |
| Arithmetic | D11 integer/table serving (ACTIVE per its dossier; this audit's board would say READY, `mechanisms/board.md`) | The engine is bit-exact and multiplier-free; zero-multiply audits pass: 44 native symbol ranges and 7/7 WebAssembly (`docs/labs/wasm-serving-runtime-2026-10-18`) | **No lawful model has been exported**, so the export gap is unmeasured. The served stacks missed M1: the 19.9M stack at 0.933 BPB ("NOT MET by 0.033") and the charted models at 1.20–1.24 BPB; they read every weight every token (D5-b) and schemas /1–/2 serve softmax through exp tables (`docs/labs/d11-serving-2026-10-10`; `lut-serving.json` rule_compat.D23) |

**Reading the map:** several lawful parts are built and qualified at finite controls (Flock, Hamilton gate, SpiralCore/FBS, object store, D11 engine). **Not built:** a position mechanism, normalization, "address as the read", a consumer of the object store, a lawful exporter for any trained trunk, and a sub-O(V) writer. **No two parts have ever been trained together.** The only assembly that has trained, the discrete path, has no feed-forward block, no position mechanism and a 5-value address key, and it is below unigram.

## 2. Never fairly tested vs failed only on configuration

**Already approved and queued (closest to a decision):**

1. **W2, the read-code capacity capture.** Already FIT'd, about $0.91, rides the same lease. It decides H_rep (representation wall) against H_law (learning-law wall) and **includes the admission-matched count-table arms** the headline bars lack (architect §6 W2; `NOW.md` sequence row 6, "queued"; D39 item 4).
2. **The admission-preserving A8 rerun.** Pre-registered and FIT, never run; the controller and a separate finite qualification producer have merged, and the qualification is NOT_RUN (`admission.json` never_tested; `NOW.md` line 17; `docs/labs/admission-anchor-controller-2026-10-19`).

**Never fairly tested, cheap (zero training, host-only, or a small check):**

3. **A full-vector or interior-block address metric** at a zero-update pre-flight, the same kind that measured A8. This is the lever architect §3.2 calls the *likely* branch (`admission.json` never_tested).
4. **A wider carrier** (more roots or blocks). Not cheap reuse: it needs new `build.rs`/`discrete.cu` code and a fresh CUDA qualification before any pre-flight (`discrete-learning.json` never_tested "Full 240-root E8 coverage").
5. **Read-off versus read-on ablations** for Route and Flock on existing checkpoints (`route-read.json`, `flock-read.json`).
6. **PrimeZeta against NormalNull and a quota-only injective null**, at zero updates (`prime-zeta-codebook.json`).
7. **Whether the secant surrogate gives descent** for the recurrence gates and SpiralCore/FBS, a small float check (`recurrence-gates.json`, `spiralcore-fbs.json`).
8. **4,096 versus 16,384 tokenizer pieces.** The trainer already exists (`tokenizer.json`).

**Never tested, moderate cost:**

- **Ranking with admission intact** (the RANK_BOTTLENECK branch): no run has had admission intact and then failed or succeeded on ranking (`admission.json`).
- **An occupancy-regularised rank objective**, the registered route after the K16-R collapse (`admission.json`, `rank-margin.json`).
- **Full-epoch and frequent-token revisit schedules**: a pass covers 1,049,856 of 2,425,925 targets (`discrete-learning.json`).
- **The finite relational head**: K16-A reached 16,404 and can satisfy its Stage M2 prerequisite; the architect says "KEEP on the K16 line" (architect §5).
- **A discrete Hamming binding (VSA) on the D32 writer** (`vsa.json` never_tested).

**Never tested, expensive or blocked:**

- **A trunk built only from lawful parts**, trained with D37 float in a new scanned module, then exported (`geometric-stack.json`: the 24-kernel extract is qualified at finite controls only). Also any lawful feed-forward block in a language fit. Both need an exporter that does not exist (section 4, Stage 4).
- **The forced-copy oracle for the pointer.** Named the cheapest decisive test, never run. It is interventional: it supplies the true slot to a bound model, so stored outputs cannot answer it, and running it on the softmax-pointer artifact would execute a banned model. It needs a lawful pointer first (`pointer-key.json` never_tested; `docs/labs/kimi-direction-2026-10-12`).

**Failed only on configuration** (closes the configuration, not the mechanism; D22):

- The A2 pilot, Dose 1, K16-A and K16-R all started from a seeded codebook with **≤16 distinct vectors** (`docs/labs/a2-pilot-diagnosis-2026-10-14`, `k16-outcome-call`). In K16, training broke much of the aliasing yet top-1 stayed low, so the seeding is a configuration flaw, not the established cause of the ranking failure.
- P8 (127 of 3,847) used uniform placement at K=8.
- A8 Q1 ran without the admission guard (`admission.json`).
- The starling and rank-table reads used softmax-tuned recipes and softmax-surrogate gradients (`flock-read.json`).
- E8 memory used a sigmoid gate and 2,000 CPU steps (`e8-orbit-memory.json`).
- Spin weights ran through a dense contraction, which a rule retired, not a result (`spin-weights.json`).
- M2 437/512 → 0/128 was panel fitting (`native-geometric.json`).
- Read binding was measured on a softmax read (`read-binding.json`).
- VSA used the banned CPU learner (`vsa.json`).

## 3. Non-learning work dominated after 11 October (partly owner-mandated)

Lab records by window, from `records/*.jsonl`, `kind=="lab-record"`. "Governance/ops" means the lines `harness-governance` or `compute-ops`. A "model-line result" has an outcome of positive, negative, inconclusive or diagnostic-stop. The 9–11 Oct and 12–18 Oct rows were recounted for this revision.

| Window | Model-line results | Model-line non-results (tooling/plan/process/review/blocked/ops) | Governance/ops | Results share |
|---|---:|---:|---:|---:|
| 9–11 Oct | 83 | 21 | 4 | 77% |
| 12–18 Oct | 34 | 116 | 157 | **11%** |
| 19 Oct | 4 | 21 | 7 | 13% |

All records by ISO week (all kinds; results = positive, negative, inconclusive or diagnostic-stop), recomputed on 20 Oct from the corrected catalog (`archive/catalog.jsonl`); the W36–W40 rows in the first revision came from the uncorrected records. The pre-October rows mix loose documents and lane results, so compare shares, not counts:

| Week | Results / total |
|---|---|
| W36 (31 Aug–6 Sep) | 59/99 |
| W37 | 38/48 |
| W38 | 18/48 |
| W39 | 91/211 |
| W40 | 74/92 |
| W41 (5–11 Oct) | 93/119 |
| **W42 (12–18 Oct)** | **34/309** (190 tooling; non-results to results about 8:1) |
| W43 (19 Oct) | 4/32 |

**In W42, 114 records are harness tooling, mostly ports such as `arena-*-port`, `check-*`, `claims-*` and `card-reading-port`.** Many of the 34 W42 "results" are kernel parity or qualification passes, not learning.

**Confound.** Part of this drop was ordered by the owner, not chosen by the labs. On 12–13 Oct, D23/D24/D25/D29/D30/D32 forced a substrate rebuild (D29 moved kernels out of every module), and D27 (13 Oct) made Rust-first mandatory, which drove the `*-port` harness work counted above as tooling (`DECISIONS.md` D27, D29, D32). The share is therefore an upper bound on discretionary process.

The training runs on the discrete path since 13 October number five: the A2 pilot; Dose 1 (stopped by the owner); rows Dose 2 (stopped at Q1); K16 (four runs); A8 (two arms, stopped at Q1). **None produced a KEEP.**

Further signals:

- **19 owner decisions in 8 days** (D23 on 12 Oct to D41 on 19 Oct, `docs/integration/DECISIONS.md`).
- **The K16 runs were preceded by a long gate chain**: 25/25 controls, C-R1..C-R6, G-Pre, custody readback, a census, PM dispatch and four owner-release records. Codex operated those four releases and the owner ratified them afterwards, which prompted D40 (`DECISIONS.md` D40 item 1; `NOW.md`). The runs themselves took "~1.2 h (~$2)" (architect §7).
- **STATUS and ROADMAP rows have become run logs** of receipt hashes (`STATUS.md`, `ROADMAP.md` M1 row).
- **The build closure is heavy.** The active model compiles a 347k-line core to obtain 1,381 lines of `spiralcore_operator.rs` (`code.json` notes).

## 4. Shortest credible path to a first simple conversation

**Binding constraints on every stage** (the first draft omitted these):

- **Holds.** Training, M2 and Stage B stay HELD; D37 releases no hold (`DECISIONS.md` D37 item 6; `ROADMAP.md` M2 row `held`). Every stage below that trains needs a hold release from the owner.
- **Gates.** No compute without FIT + PM dispatch + owner release where a hold applies (D39 item 7). Owner-identity releases are typed by the owner personally (D40).
- **Order.** D39 item 4 fixes W1 → W2 → W3 (BVG-K, registered before any float sheet) → W4 (float) only if W2 decides H_law and W3 fails. The path below follows that order; any reordering is an owner decision.
- **M1 definition.** D39 item 1 closes M1 only on "one saved discrete rung-0 artifact" passing (a)–(e). A float trunk (Stages 3–5) cannot close M1 without the owner redefining the milestone.
- **Float home.** A float path is "a separate module" with its own scanned allowlist entry (D37 item 6), "never `crates/uor-r4-training`" (architect §6 W4), which `code.json` lists as `held` and which carries the historical dense/sigmoid stack.
- **Distillation.** D41 item 2(d): "no student-side exception: training stays the D32 discrete integer path". `distill-d1` does not exist yet; D41 item 3 puts the driver math optimization and a ~1M-token pilot first.

**Stage 0. Run what is already approved** (≈$0.91 plus the A8 finite qualification).
- **Do:** W2 read-code capacity capture, including its admission-matched count-table arms; then the admission-preserving A8 finite qualification and rerun.
- **Bar:** W2's registered H_rep/H_law decision; A8 against its registered sheet and against the 857 control.
- **Decides:** H_rep → fix the read/address metric, do not fund float credit; H_law → continue to Stage 2 (architect §6 W2).

**Stage 1. Make the capacity measurable** (host work plus a zero-update GPU pre-flight; a wider carrier adds build and CUDA qualification).
- **Do:** full-vector/interior-block address metric at zero updates on A8 placement. Optionally build a wider carrier first.
- **Bar:** **not** G-Pre ≥ 4,894 at checkpoint 0, which every K16 arm already met at 14,287/14,287 with 16 occupied cells (`docs/labs/k16-rows-qualification-2026-10-18`). Use the architect's non-vacuous gates: distinct full vectors and the alias ceiling ≥ Bar (i) 1,941 net of admission (G-Cap), within-cell tie mass, and admission at checkpoint 0 above A8's 1,351 of 3,847 on the same panel (architect §6 W1 items 1–2).
- **Reuses:** `address_cells.rs`, `coverage_census.rs`, `attention_preflight.rs`.

**Stage 2. Pick the learning law in the D39 order** (W3 ≈ $12, then W4 ≈ $5–10 GPU plus a host build: $17–22 before W2 if both run; architect §6).
- **Do:** W3 BVG-K at K=1/4/16, two seeds, equal wall clock. Only if W2 = H_law and W3 fails: W4, one float-credit sheet in a new scanned module with a Stage-1 export-gap probe first. Add the admission guard to both. *Proposal for the owner, not a plan step:* running W3 and W4 as one matched race would save a cycle but overturns D39 item 4.
- **Bar:** integer metrics only (D39 item 5): DEV top-1 above unigram (1,226), then above the admission-matched T2 from W2; export gap within the two-seed band. Seed spread is large (K16-A: 65 vs 463), so the band must be set from it.

**Stage 3. Add the missing trunk slots** (new build; needs owner decisions on M1 definition and the architect's dispositions).
- **Do:** Hamilton-gate feed-forward, dyadic recurrence with quaternion transport, Flock/Relation reads, as D37 float in a new scanned module. This contradicts the architect's "RETIRE at tested scope for now" for the F32 operators and "no site" for E8 memory (architect §5). The override is weak: the only supporting evidence is the confounded transport result and finite-control qualification of the 24-kernel extract (`geometric-stack.json`). Treat this stage as a hypothesis test, not reuse.
- **Data and scale:** the old stack's 0.877/0.933 BPB came from the 82M-token chat-v0-p2 split and never met M1; a larger set was not reachable then (#1913, `geometric-stack.json`). A 1.39B-token chat corpus was added in era 22 (#1660, `eras.md`), but no trained stack used it. Scale is unestablished.
- **Bar:** the same integer DEV top-1 and admission metrics as Stage 2, on the same panel, two seeds, beating Stage 2's best by more than the measured seed band, with fresh transfer reported. BPB needs a likelihood definition first (section 5).

**Stage 4. Export and serve** (new build work).
- **Do:** write an exporter. None exists for this trunk: the recurrence mode is refused by export and QAT, and the Hamilton gate "still needs the icosian snap" (`recurrence-gates.json` rule_compat.D11); `training/src/stack_export.rs` is compiled out with `#[cfg(any())]` (`code.json`); integer schemas /1–/2 serve softmax (`lut-serving.json` rule_compat.D23). Then measure the export gap and tokens per second on the laptop.
- **Bar:** D11 audits green, the gap within the band, tokens per second reported (decode and context separately).
- **Reuses:** the wasm/native runtime (#2654) and the zero-multiply audits.

**Stage 5. Conversation** (needs a new owner decision if the student is float).
- **Do:** consume `distill-d1` hard targets once it exists. Under D41 item 2(d) the student is the D32 discrete path; a float student needs a new owner decision. Ground replies with the exact log plus prime sieve and the object store.
- **Bar:** M2 (≥ 256/512 saved and ≥ 40% fresh, existing panels and graders unchanged, `ROADMAP.md` M2 row), with deterministic graders only (D28). The multi-turn reply bar is BLOCKED under R14 (D39 item 1), so the multi-turn check needs a registered number before it can gate.

**What to stop or propose:**

- **Stop interpreting admission objectives on the 5-value endpoint key.** No objective can order 256 cells with a key that takes 5 values; the address metric is the lever (architect §3.2).
- **Stop rerunning the seeded ≤16-vector lineage** (alias ceiling 1,986 at init, `k16-outcome-call`).
- **Stop treating any run that loses to unigram or T2 as worth interpreting** beyond one line.
- **Stop logging receipts in STATUS and ROADMAP.**
- **Propose to the owner** (governance changes, not settled): one release per stage instead of per arm (this changes D39 item 7 and D40 practice); a freeze on discretionary harness work until a model passes Stage 2 (D27 ports stay mandated).

## 5. Risks and unknowns

| Claim | Type | Source |
|---|---|---|
| The address key takes 5 values to order up to 256 cells; the K-boundary is largely set by cell-id labelling | **Proof**, read from `discrete.cu` | architect §3.2 |
| The 16-bit carrier allows an injective code; the seeded initializer gave ≤16 distinct vectors | **Proof** (carrier) and **measurement** (init) | architect §3.1; `a2-pilot-diagnosis` |
| Aliasing does not explain K16-A's ranking failure | **Measurement** (Q4 census: ceiling ~9,000 rows, top-1 65/463) | `k16-outcome-call` (#2029 `6257392642`) |
| The trained discrete model is below unigram and T2 (65 and 463 vs 1,226 vs 6,300) | **Measurement** | `k16-outcome-call`, `host-diagnostics` |
| Non-results outnumbered results about 8:1 in W42 | **Measurement** from this audit's records; partly owner-mandated rebuilding (D27/D29/D32) | `records/*.jsonl` |
| The old float stack reached 0.877 float / 0.933 served BPB on one 19.9M stack and 1.20–1.24 on the charted models, all missing M1 | **Measurement**; the trained form used softmax, SiLU and matmul, **geometry was 0.05% of its arithmetic**, and serving read every weight per token | `geometric-stack.json`; `d11-serving-2026-10-10`; `eras.md` #1893, #2186 |
| Integer serving first cost 4.3× more energy | **Measurement**, on a dense configuration | `lut-serving.json` |
| Quaternion transport helps order in a lawful model | **Hypothesis.** The only result is confounded (softmax/sigmoid/dense core, parameter mismatch, 1,000 updates) | `transport-attribution-d1`; `recurrence-gates.json` |
| A softmax-free read can learn content ranking from scratch with hinge credit | **Hypothesis, the biggest unknown.** Every read that learned had softmax help; the best softmax-free read is a recorded REJECT | `softmax-free-read`, starling records |
| D37 float training of the table beats discrete moves | **Hypothesis.** Untested; W4 is conditional on W2 and W3 | D39 item 4; architect §6 |
| The export gap of a hinge-trained float model is small | **Hypothesis.** The architect expects a structurally non-zero gap, and no exporter exists | architect §3.3; `recurrence-gates.json` |
| M1 bar (d), ≤ 0.90 BPB, can be scored on a model with no likelihood | **Open definition.** Needs an owner ruling, for example a lawful integer calibration table over ranks (hypothesis) | `rank-margin.json`; D39 |
| A float student may consume `distill-d1` | **Not permitted** under D41 item 2(d) without a new owner decision | D41 |
| A norm-preserving Hamilton gate can suppress a signal | **Hypothesis**, a dossier reading only | `hamilton-gate.json` |
| Reaching near-frontier capability needs more data than any trained stack used. Data beat size before | **Measurement** for the past (#437, #1913). **Hypothesis** for the future | `eras.md` |
| The writer's O(V) scan breaks D5-b at larger vocabularies | **Proof**, by count of comparisons per row | architect §3.5 |

**Bottom line.** Some lawful parts exist and are qualified; several slots (position, normalization, export, address-as-read) do not. The cheapest decisive step is already approved and queued: W2, which says whether the wall is the representation or the learning law. After that, the D39 order (BVG-K, then float only if needed) picks the learning law. Everything beyond is new build work behind standing holds and owner decisions. Whether a lawful read can learn ranking without softmax remains the central unknown, and no stitching answers it before Stage 2 runs.

## 6. Disputed (review findings rejected in part)

- **Finding 8 (16-bit diagnosis), partly rejected.** Accepted that the "16-bit code" headline is refuted and removed. Rejected the implied conclusion that representation is cleared: the 5-value address key (architect §3.2, confirmed by the reviewer) and the seeded-init alias ceiling (1,986 rows, `k16-outcome-call`) remain supported structural limits, so "stop rerunning the seeded lineage" and "the address metric is the lever" stay.
- **Finding 16 (architect dispositions), partly rejected.** Accepted that the disagreement must be stated, and it now is (the section 1 map row and Stage 3). Rejected that "RETIRE" binds the synthesis: under D22 and the owner's status vocabulary there is no retired status, a negative closes a configuration only, and the architect's own wording is "at tested scope for now". The operators stay READY/PARKED per their dossiers.
- **Finding 17 (process confound), partly rejected.** Accepted the confound and reframed section 3. Rejected removing the conclusion: the owner-mandated rebuild explains the ports and kernel moves, not the per-arm gate chain before K16 or receipt logging in STATUS/ROADMAP, which are discretionary.
