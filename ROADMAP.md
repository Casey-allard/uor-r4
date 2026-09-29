# UOR-R4 Geometric Language Model — multi-lab roadmap

**Director:** Claude (Lab 1). **Updated:** 2026-09-29 01:50 UTC. This file owns lab
assignments, track status, the dead-path register and the cross-lab protocol. Measured
results and retained artifacts live in [current state](docs/integration/current-state.md).
Ordered responsibilities and acceptance live in the [canonical plan](docs/integration/project-track.md).
Owner decisions live in [DECISIONS](docs/integration/DECISIONS.md). Director rulings here
coordinate the labs. They do not amend an owner decision record, and the owner keeps
strategic authority.

## Lab 1 leadership and restart packet

**Updated September 29, 01:50 UTC.**
- **Lead:** Claude (Lab 1) is the permanent project lead: scientific direction, principal research, architecture, shared interfaces, promotion and merge decisions, within the owner's constraints.
- **Astra's temporary cover** began on September 28 at 22:45 UTC. It ended with the handback on September 29 at 01:35 UTC. Its record is preserved in [its history file](docs/integration/astra-temporary-cover-2026-09-28.md) and on [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5881962511).
- **Casey retains** the mission, spending, destructive-action authority and ratification of owner-level decisions.

| Lane | Owner | Current deliverable |
|---|---|---|
| Leadership, learner, dialogue, memory semantics, architecture, native artifact and session, promotion | Claude (Lab 1) | The I1 store in the dialogue stack's read path; S4's fit once #1483 clears |
| Integration, S1/QAT and S4 repairs, checks, delivery mechanics | Kimi (Lab 1 research-integration steward) | #1483's two failures, resolved under the [adjudication](https://github.com/UOR-Foundation/uor-r4/pull/1483#issuecomment-5882022226) |
| Geometric read and address | DeepSeek/OpenCode (Lab 2) | The G-decomposition result (running) and the causal change its dominant cause selects; then a synonym world for G v2 |
| Numerical fidelity, codecs, kernels, audits, cost | Anti-Gravity (Lab 3) | #1479, held for Lab 1's fresh-root re-run; D4-E8 through QAT |

### The artifact contract

**The integrated milestone is one saved model** that gives useful complete replies with:
- memory update;
- source and version distinctions;
- absence and conflict handling;
- reload survival;
- native D11 execution.

Loss, tracking accuracy, arithmetic parity and library availability are components of that milestone, not the milestone itself. There is no third learner and no parallel serving engine.

### Bottleneck and next decision

**Memory fails at address formation.** This was corrected at 01:49 UTC; see [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5882099700).
- G v1's reads fire, but held-out Updated is 0.03–0.16 against a target of at least 0.90.
- S2 recalls 0 of 10 facts.

**Why surface-form keys are not the cause.** In the D2 relation world, held-out templates change only the carrier text, and the entity and relation tokens are the same at write and at query. The failures there are therefore role-tagging or write-trigger failures, not surface-form mismatches.

**Lab 2's decomposition (`AddressCause`) selects the next causal change:**
- missed writes → address-completed writes, tested by evaluation only on the saved checkpoints;
- mis-tags → change how tags are formed;
- wrong values → clause closing;
- evictions → capacity.

**Canonical geometric keys (G v2)** address a different problem, different surface forms of the same entity or relation. They need a world with synonymous mentions, and their consumer is S2's memory trajectories.

### Live state

Verify it against GitHub; this file is not a job monitor.
- **Merged at 01:35 UTC:**
  - #1481 (`f4a1e171`, the S1/QAT follow-ups);
  - #1468 (`9869229b`, Lab 3's negative D4 study and codec primitives).

  Earlier: #1482 (`1c94969d`, Lab 2's AERM checkpoint save and load). The squash patches of #1481 and #1468 were checked equal to their PR diffs.
- **Open:**
  - #1483 (S4, Kimi).
  - #1479 (Lab 3). Its cost numbers are unqualified until Lab 1 re-runs them on a quiet machine into a fresh root. `d11-cost-profile-1` was resealed in place, so it is not original evidence.
- **Running:** Lab 2's `aerm-decompose` at `5f3de916`, in the model slot from 00:32 UTC to about 04:02 UTC (6 threads, 9 GB cap).
- **Pinned models:**
  - the S2 dialogue stack `8cb11d8f…`, the loss baseline;
  - the native child `98aca5ab…`, the reference for replies and memory;
  - `geometric_s1` `3eb1ebbb…`, the D4 codec target.

### Rules in force

- **Decisions:**
  - D11 and D0-b govern serving: no float, and no multiply or divide instruction, in the served numerical path.
  - D12: a missed gate prevents promotion only.
  - D9: new compute needs a named causal change.
- **Merges:**
  - The class A/B/C criteria are in the [policy](docs/integration/agent-execution-policy.md).
  - Lab 3 results merge only after Lab 1 re-runs them from committed code into a fresh root.
  - Nothing is auto-merged unconditionally. Before each merge, recheck the exact approved head and its blockers. Then merge through the protected queue, verify the delivered patch, and notify the consumer.
  - The shared GitHub account never turns self-review into a non-author review.
- **Machine and spending:**
  - One heavy job at a time, through `/Volumes/UOR-Workspace/locks/model-slot.json`; light jobs use at most 2 threads.
  - No paid or external compute, and no change of provider, plan, credential or spending limit.

### Restart procedure

1. Run `date -u`.
2. Read this packet, the latest #973 comments, the open PRs and the model-slot lock.
3. Recover existing workers, branches, jobs and sealed roots before assigning anything.
4. Staffing defaults:
   - zero additional Opus workers, and at most one bounded specialist;
   - no recursive spawning;
   - completion events, not polling.

**Next actions, in order:**
1. Resolve and review #1483.
2. When the slot frees:
   - Lab 1's #1479 cost and E8-reference re-runs, on a quiet machine;
   - then S4's pre-registered fit, if #1483 has merged.
3. Lab 2's decomposition, then the G v2 decision.
4. Wire the I1 store into the stack's read path, and measure it on S2's 10 memory trajectories.

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

Every parameterisation of geometry tested as a **continuous score or mixer** inside a language
model is null or negative (§5):
- quaternion transport against Householder or diagonal decay;
- Lorentz reads against Dot, across seeds and transfers;
- the 2I read score;
- snapped rotation codebooks;
- prime and CRT hashing.

**Scope** (ruling 11):
- These negatives retire the parameterisations tested, not geometric reads as a family.
- The 2I read dropped per-lane magnitude, snapped to 120 roots and swapped in a learned MLP all at once. Its norm control never ran, so its cause is unresolved until D6.

*Working hypothesis:* the native model's prose deficit follows capacity and exposure rather than
geometry. The measured facts:
- It has 1.68M parameters and scores 0.41 nats behind the 7.16M #1017 transformer.
- Another 30M targets of exposure left prose at 0/5.
- At matched capacity, the stack (with geometric transport) beat its transformer control on code, 1.998113 against 2.011149 (D0).

That capacity and exposure are *sufficient* to explain the deficit is untested; D5 tests it.

Geometry's defensible jobs are discrete and table-servable, which is also what R1–R3 demand:
1. **Exact non-abelian state (B1).**
   - Lanes over 2I ≅ SL(2,5) track the A5 word problem, which is NC¹-complete. Diagonal SSMs and constant-depth transformers cannot do this at arbitrary length, assuming TC⁰≠NC¹.
   - Snapped to 2I, a lane serves as a 120-state automaton: one byte of state and two table reads per token, with no drift.
   - Out-of-repository toy result: 100% accuracy to length 4,096, where float lanes drifted to 0.62–0.74 ([review §6.2](docs/integration/first-principles-review-2026-09-25.md)).
   - Reproduced in repository Rust (#1442), with one correction: the capability is the finite group, not the quaternion parameterisation. Ordinary reflection-pair lanes learn the same icosahedral group and minimise to the same 60-state automaton.
   - **B1 is closed** (#1447). No lane type is retained in the language path:
     - the fresh reflection-pair replication tracks A5 exactly, but fails its text gate by +0.069 nats in one seed;
     - Stage C (natural-text swap stories) is parked.

     The exhaustively verified automata stay available as tools.
2. **Geometric addressing (D5).**
   - Optimal spherical codes (the 600-cell, the E8 roots) can act as sparse indexes, decoded with adds and compares.
   - A group-relative score can **compile to addresses**: visit the cells `g_q·d` for a small support S, and follow exact postings to records. Geometry then reduces what is read (ruling 11; synthesis §7.3).
   - Both are conditional on D6 and D3's gain-controlled follow-up.
3. **Exact identity and versioned memory.** This is already load-bearing.

The tracks below put geometry in those three places, scale the learner, and serve it without
D10 exceptions.

## 2. Tracks and owners

**Three labs (owner charter, 2026-09-28).** The numbering is kept.
- **Lab 1, Claude main:** lead and integration.
- **Lab 2, OpenCode:** geometric read and addressing.
- **Lab 3, Anti-Gravity:** numerical fidelity, codecs, kernels, audits and measured cost.

The roles and the standing merge and review criteria are in the [execution policy](docs/integration/agent-execution-policy.md#three-lab-organization-and-shared-operating-policy-owner-charter-2026-09-28).

**Not part of the three-lab organization, with no future assignments:**
- Lab 4 Codex, whose allowance is exhausted. It returns only by owner direction. [.codex-lab/README.md](.codex-lab/README.md) is its record.
- The Claude cloud track. Lab 1 absorbed its work (§9, 2026-09-28).

| Track | Owner | Goal | Current hypothesis | Status | Next decision point |
|---|---|---|---|---|---|
| **T1 Learner and exact state** | Lab 1 Claude (with the retired cloud track's work, absorbed 2026-09-28) | A single main-line learner at #1017 scale, with geometric state where it earns it | (a) The capacity-matched stack closes the native gap. (b) Exact 2I tracking lanes add A5-class tracking at ≤0.05 nats LM cost. (c) Fixed H4/E8 codebooks address sparse parameter memory as well as learned keys do. | (a) **Decided, 05:50 UTC 09-28:** the stack scored 1.998113 against its control's 2.011149 at full exposure, so **the stack is the main line** (D0). Weights are on the SSD, SHA-256-verified. (b) **Closed** (#1447): no lane type is retained in the stack, and Stage C is parked. (c) Implemented and held, NOT_RUN (#1437, merged). **D1: keep the transport. D2: frozen gate FAIL on the margin; the store works, the learned read does not generalise** ([result](docs/integration/d2-aerm-probe-result-2026-09-28.md)). **Next:** wire I1's store (six views) into the stack, then Lab 1's decision on Lab 2's G proposal (D6 delivered in #1464). S2 is the dialogue baseline (§2b). | §4.1 |
| **T2 Geometric addressing (D5×D6)** | Lab 2 OpenCode | Decide whether geometry can be the sparse index for event memory | A fixed 600-cell/E8 cell index retrieves the dense read's top events as well as LSH, IVF/k-means, PQ and learned kNN at equal bytes touched, with a cheaper multiplier-free decode. | The frozen contest is published: **NOT QUALIFIED** at its scope (#1456, with the ruling-11 corrections). It is a compression-fidelity screen, because it scores every event, and its geometric arms are confounded by magnitude. **D6 delivered** (#1464): ORDINARY-BETTER on the native dot read. **Next:** G v1, language-to-address generalization for the exact store (the outcome is on #973); D3's gain-controlled follow-up is pre-registered by Lab 2. | §4.2 |
| **T3 Numerical fidelity and measured efficiency** | Lab 3 Anti-Gravity | Numerical fidelity, codecs, kernels, execution audits and measured M1 cost for the main-line model under R1–R5. The stack's writer, loader and integer forward belong to Lab 1 (S1) | LUT-accumulation kernels, exact 2I lanes and table products serve the stack without D10 exceptions, losing ≤0.02 nats. J/token is set by bytes touched. | `uor-chat`, blocked kernels and streaming delivered ([#1434](https://github.com/UOR-Foundation/uor-r4/pull/1434), [#1436](https://github.com/UOR-Foundation/uor-r4/pull/1436)); width-576 `uor-chat` ([#1450](https://github.com/UOR-Foundation/uor-r4/pull/1450)). **The auditor is repaired** ([#1451](https://github.com/UOR-Foundation/uor-r4/pull/1451)), with failing and benign sentinels. **First J/token measured** (owner run, 2026-09-28): the integer path is 0.00413 J/token against 0.00096 for its float parent, whole-system marginal: about 4.3× the energy and about 2.6× slower ([note](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5862960224)). [#1452](https://github.com/UOR-Foundation/uor-r4/pull/1452) (capability API, WASM and M1 cost) is **not accepted**: its M1 figures were packager fallback defaults ([review](https://github.com/UOR-Foundation/uor-r4/pull/1452#issuecomment-5874859825)). **D4 is open:** #1458's results were unmeasured and not accepted. **The stack's serving port is S1 (Lab 1).** **Next:** D4 against the S1.0 target through the QAT hook, with Lab 1 re-running its gating measurement; the salvageable parts of #1452 and #1458 as narrow PRs. | §4.3 |
| **T4 Native dialogue and conversion fidelity** | **Lab 1 Claude** (owner transfer from Lab 4 Codex, whose allowance is exhausted; the study is unchanged) | Recover learned relations through integer conversion, and dialogue learning on the native path | Response-aware legal-code choice recovers relations lost at conversion; conversion changed 909 of 3,914 greedy decisions. | **#1433 executed and FAILS its retention rule** ([result](docs/integration/dialogue-code-choice-result-2026-09-28.md)): response NLL 2.796 against nearest's 2.849, but relation answers at the question turn fall to 2 of 10 (nearest 4, continuous 9), with 962 flips against 909. Nearest-hard stays the baseline. **Next:** D4 (Lab 3) with a fidelity objective; the T4 question passes to D4 and D5. | §4.4 |

**Main-line consolidation rule (director; decided 2026-09-28 05:50 UTC: the stack passed):**
- **If the stack comes within 0.03 nats of its transformer control at full exposure:**
  - it becomes the single main-line learner;
  - dialogue learning (T4), read and addressing work (T2) and serving (T3) retarget it;
  - the 1.68M native model is frozen as the retained baseline.
- **Otherwise:** the native model stays the main line, and the stack goes to ablations under its pre-registered card.
- **Either way:** no lab starts a third learner.

**Integration interfaces** (director, 2026-09-28; details in the [plan](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5863390710) §3):
- **I1, base checkpoint** (T1): `StackConfig` or the native config, weights, tokenizer identity and training provenance, in a sealed report root.
- **I2, native serving bundle** (Lab 1 exports and serves through S1; Lab 3 audits and measures): `uor-r4.integer-serving-bundle/1`.
  - The default profile serves the native model. A `stack` profile is added only if the stack wins.
  - The bundle carries ≤4-bit maps with shift-friendly scales, shift-and-add grid codes, integer biases, sealed tables, optional `verify_exact` lane automata, and `numerical_contract: "D11"`.
  - Conformance: fidelity ≤0.02 nats with the decision-flip rate, the repaired auditor clean on one serving binary, and identical tokens after save and reload.
- **I3, session** (Lab 1, `uor-chat`): one conversation API for every profile, with a declared context budget and eviction policy, per-turn starting seeds, transactional settings, and rejection of unsupported commands.
- **I4, addressed reads** (T2): `admit` and `rank`, measured separately, with bytes touched per token and decode cost.
- **I5, dialogue data and panels** (T4): sealed and versioned.
  - **Decided in §2b (2026-09-28):** two separate identities. The dialogue line uses the #1017 tokenizer, and the code line uses the code BPE.
  - D0 selected the stack *architecture*. The two tokenizers are different token identities even at the same vocabulary size, so no weights or panels transfer between them.
  - The D2 core and S2 (#1470) already use the #1017 token store.

### 2a. The approved track (owner, 2026-09-28: "Approve, start with D2"; amended 15:27 UTC)

The [whole-project synthesis](docs/integration/whole-project-synthesis-2026-09-28.md) (#1453) owns the design.

**The track:**
- a 10–30M recurrent core: the stack (D0), keeping its quaternion transport (D1);
- **M1**, the exact version-ordered store with typed statuses. D2 found that it works when addressed correctly, and that its learned trigger-gated read does not generalise;
- **G, the geometric read/address operator: on the main path** (owner, 15:27 UTC):
  - an always-on learned addressed read into the exact store and the context;
  - built behind I4, with the stack's dense read as fallback;
  - D6 and D3 shape its representation and cost claims;
  - it replaces the fallback only if it wins at equal cost;
- **M2**, conditioned anti-echo response training;
- **M3**, a geometry-coded D11 artifact (D4);
- **M4**, event-gated state, deferred.

**Owner, D12 (2026-09-28): geometry moves to the core.** The native engine's mechanisms are ported into the stack one at a time, each trained in and measured at equal capacity:
1. the 2I transport (**S4**, now);
2. the exact store (I1);
3. prime/UOR-addressed store keys;
4. zeta phase channels.

Reopened with them: the finite-group tracking lanes (with S4), and the geometric sparse index plus trained-in 2I/E8 read codes (inside G). Gates promote; they never kill. The [geometric toolbox](docs/integration/geometric-toolbox-2026-09-28.md) keeps every demonstrated geometric piece available.

Geometry sits in five places: the core's transport (D1, S4), exact identity, exact automata (the B1 lanes, reopened), weight coding (D4), and the addressed read (G).

| ID | Question | Owner | Status |
|---|---|---|---|
| D0 | Which core? | Cloud (retired) | **Decided:** the stack |
| D1 | Is the transport load-bearing at matched width? | Cloud (retired) | **Decided: keep.** Identity costs +0.0711 and +0.0774 nats (2 seeds). Merged in #1460 |
| D2 | Does exact memory give small models updated relations? | Lab 1 | **Not yet promoted** (its margin gate missed against a strong recency control; D12). The store is perfect in distribution (1.000 on every class). Held out, the learned read trigger fails ([result](docs/integration/d2-aerm-probe-result-2026-09-28.md)). **Active:** the store is I1 (#1472), and reads come through G |
| D3 | Geometric against ordinary index, at equal bits and fewer inspected events? | Lab 2 | **Not yet promoted** at its scope (#1456; confounded by magnitude, ruling 11). **Reopened inside G** (D12), with magnitude carried and the geometry trained in |
| D4 | Does geometry-coded coding fix the hard artifact? | Lab 3 | **Open.** [#1458](https://github.com/UOR-Foundation/uor-r4/pull/1458)'s D4 numbers (142 flips, +0.0125 nats, 7/10) are **constants in its test source, not measurements**. It has no pre-registration, fit, report roots or replies, so the result is **not accepted** ([review](https://github.com/UOR-Foundation/uor-r4/pull/1458#issuecomment-5873489608)). Lab 3 is to pre-register and run it |
| D6 | What does the 2I read representation discard? Evaluation only | Lab 2 | **Delivered** (#1464): ORDINARY-BETTER on the native model's dot read. **Evidence, not a veto** (owner, 15:27 UTC) |
| G | The geometric read/address operator | Lab 2 (research and implementation); Lab 1 (decision and integration) | **Main path** (owner, 15:27 UTC). G v1 (#1469) is running its frozen run. It carries the reopened geometric sparse index and trained-in read codes (D12) |
| S4 | Can the transport be exact 2I, trained in? | Lab 1 | **Being implemented.** Post-hoc snap on S2 costs +0.0221. Three-way outcome (D12): within 0.02 of its control it is adopted as the served transport; 0.02–0.06 it keeps developing (longer, annealed, 2I×2I); above 0.06, diagnose first. It is never retired on one run |
| D5 | The milestone candidate (§8) | All | The fit waits for G's first comparison and D4. Integration engineering proceeds now (ruling 12) |

### 2b. The artifact contract

This is the single integration target. Unknown fields are marked, never filled from an older artifact.

**Decided on 2026-09-28** in the three-lab reconciliation, under the owner's charter. They are Lab 1's decisions unless marked "owner". Earlier states are in git history and §9.

| Field | State |
|---|---|
| Model family and configuration | Geometric stack `rrarra`, quaternion transport, Lorentz reads (D0, D1). **S4, adopted 2026-09-29: the main line's recurrent transport is the trained-in snap to the binary icosian group 2I**; the memory-port and QAT phases continue from arm A with the snap on (codec and serving profile row). **Decided: the first dialogue baseline uses the measured cycle-4 shape:**<br>• width 288, 6 heads, MLP 749, context 256;<br>• 7,153,860 parameters at vocabulary 4,096.<br>It is the only stack shape with a language-model result and an export path. The 10–30M milestone shape is chosen after that baseline's panel result and S1's measured cost |
| Tokenizer and dialogue protocol | **Owner charter, 2026-09-28: two identities, kept separate.** Lab 1 chose #1017 for the dialogue line.<br>• **Dialogue line:** the #1017 tokenizer (`d36d3e87…`, CID `blake3:3f42bcfc…`, with `<\|bos\|>`, `<\|eos\|>` and `<\|unk\|>`); the literal-role protocol (`uor-r4.literal-role-dialogue/1`, `blake3:0099a613…`); the chat-v0 corpus (`uor-r4-chat-corpus/v1`, train tokens `4a554b0e…`, heldout tokens `ca5ccf07…`).<br>• **Code line:** the lab code BPE (merges `7453bfa0…`), for the cycle-4 reference only.<br>No weights, panels or exports move between the two lines without an explicitly implemented migration. A code-loss result never qualifies conversation |
| Pinned baselines | Preserved unchanged through experimental work:<br>1. **Dialogue, native:** the width-576 full-prefix child `98aca5ab…`, at development response NLL 2.773887 on the frozen 161-response panel, plus its nearest-hard integer child, #1433's baseline: packed manifest `548540e0…` and bundle `dialogue-child-bundle-1/bundle.json` `55b3fa54…`.<br>2. **Code, stack:** the cycle-4 `geometric_s1` `3eb1ebbb…` (1.998113), with its S1.0 exports: round to nearest `9ad33133…` and GPTQ `43e97b05…`.<br>3. **Dialogue, stack: S2 `8cb11d8f…`**, pooled development response NLL **2.520916** on the same panel (2026-09-28, [result](docs/integration/s2-dialogue-stack-baseline-2026-09-28.md)). It is the main line's loss-level dialogue baseline. The native child stays the reference for complete replies and memory recall, where S2 is worse (0 of 10 recalls) |
| Memory semantics and persistence | **Owner charter, 2026-09-28: the product store keeps six typed distinctions** (the scoped-memory contract):<br>• current;<br>• previous record;<br>• previous distinct value;<br>• initial occurrence;<br>• absent;<br>• evicted.<br>The store serializes with the session: save, reload, then identical subsequent tokens. The D2 probe kept only current and previous distinct, which is probe-only and not adopted. **Implementation: Lab 1 (I1).** The six-view store and a sealed inference checkpoint (`stack_store`, `stack_checkpoint`) are **merged (#1472)**. They are not yet wired into the model; the memory port row says how |
| Memory port (the read interface) | **Lab 1 decision, 2026-09-29.** This contract is between the stack, G (Lab 2) and I1. Only the rows marked NOT IMPLEMENTED are unbuilt; the rest follows the D2/G v1 probe (`stack_aerm.rs`).<br>• **Placement:** one port after a declared split layer, recorded in the checkpoint.<br>• **Address, the prime router** ([ADR-0003](docs/adr/0003-fixed-zeta-prime-route-attention.md); owner direction, 2026-09-29):<br>&nbsp;&nbsp;– every content atom has a registered prime, and a clause's data is the product of its atoms, so alike records share factors that gcd sieves out;<br>&nbsp;&nbsp;– the **semiprime of the clause's two key atoms is the routing expert** that keys the store. It is the same whichever role the head gave each atom, by unique factorization;<br>&nbsp;&nbsp;– the value is routed through the expert, not multiplied into it, so direction survives for relations whose values are names;<br>&nbsp;&nbsp;– ordered n-lets carry order and repeated factors;<br>&nbsp;&nbsp;– a read prefers the completion with the least divisor count, a single prime atom.<br>• **G's side:** per position, a role (other, entity, relation or value) and a clause decision (none, assert or correct) from the residual at the split. This is the language-to-address compiler that #958 lacked at its stage 4. G may replace how these are formed. G v2 canonical codes map to primes from a registry range disjoint from the token atoms.<br>• **Store:** I1's `StackStore`, with the six distinctions, replaces the probe's `RelationStore` on the main line.<br>• **Reads:** address-driven, G v1's policy with no learned read trigger. Wherever the open clause has an entity and a relation, all four views are read: current, previous record, previous distinct value and initial. Each view is a register that holds `(status, value)` until its next read.<br>• **Re-entry:** per register, the probe's zero-initialised status embedding and value projection add to the residual at the split, and a zero-initialised gated copy boost acts on the logits. The copy pointer advances through a multi-token value as it is emitted: **NOT IMPLEMENTED** (the probe copies single-token values).<br>• **Writes:** the clause decision is taken when entity, relation and value are complete. Whether it stays a learned trigger or becomes address-completed follows Lab 2's decomposition ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5882099700)). A question that carries a value must not write, so the decision includes mood.<br>• **Training:** gold roles and decisions drive the store (teacher forcing), and the heads train with auxiliary cross-entropies. No gradient passes through the store. Evaluation uses the model's own outputs.<br>• **Serving:** the store is exact host-side integer code. The D11 session needs a fixed-arena store without steady-state allocation: **NOT IMPLEMENTED**.<br>• **Persistence:** the store state is saved in the I1 checkpoint. Save, reload, then identical reads and tokens |
| Reader and addressing | **Fallback:** the stack's dense `a` read (Lorentz).<br>**G** (Lab 2, behind the read interface): its first target is **language-to-address generalization**. In D2, writes transferred to unseen phrasings, but reads did not.<br>D6 (Lab 2, delivered in #1464), on the native model's dot read: ORDINARY-BETTER. Lane magnitudes were load-bearing, and 2I directions with a 3-bit gain did not restore the read decisions. That is evidence at its scope, not a veto; it carries to the stack's Lorentz read only after a stack-side check |
| Codec and serving profile | D11. **Interim format:** 4-bit maps in groups of 32, plus grid-code scalars. On `geometric_s1` it costs +0.0362 nats with round to nearest and +0.0257 with GPTQ, of which the head is +0.012.<br>**Owner, 17:00 UTC:** Lab 1 adds QAT; D4 (Lab 3) targets the same gap at 4.25 bits per weight; the milestone uses whichever first reaches ≤ 0.02 nats. **Lab 1:** the hook is `MapCodec` (merged in #1466); D4 codecs plug in through it.<br>The post-hoc 2I transport snap (+0.026 on `geometric_s1`, +0.022 on S2) is not adopted. **The trained-in snap (S4) is adopted (Lab 1, [05:34 UTC 09-29](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5884326637); merged in #1483):** the pre-registered fit (`kimi-s4-fit-20260929`; manifests `2aabdc73…`, `e4926422…`, `56d29a49…`) scored arm A snapped **2.547953579954328** (`2b3b5681…`, `transport.json` = icosian) against arm B free **2.5373633745806394** (`fcc3099b…`), a **+0.010590** gap inside the ≤ 0.02 band, so no second seed. Both arms ran executable `fbeedde1…`; arm A's step-0 unsnapped score reproduces S2's 2.520916. Scope: one seed; the 161-response development panel; 512 updates after a schedule restart; S2 `8cb11d8f…` stays the loss-level reference; replies differ in 36 of 38 requests. The D11 icosian serving kernel (S1.4) is not yet built |
| Exporter, loader, session | **Lab 1 (S1, absorbed from the cloud track):**<br>• S1.1, the D11 engine in `uor-r4-integer::stack`, bit-identical to D10: **merged (#1467)**, with a FULL PASS ARM64 audit;<br>• S1.2, the I2 `stack` bundle and loader, with reload equality: **next**;<br>• S1.3, the M1 audit.<br>I3, the `uor-chat` stack profile, follows S1.2 on the dialogue line, whose tokenizer has BOS and EOS. Store serialization in the session: **NOT IMPLEMENTED** (I1) |
| Checkpoint (I1) | `StackConfig`, weights, tokenizer and protocol identity, training provenance and store state, in a sealed root; save → reload → identical logits and store reads (`stack_checkpoint`, merged in #1472; an inference checkpoint, not an optimizer resume). **None for the milestone yet.** S2 `8cb11d8f…` (#1470) is the first dialogue stack model |
| Evaluation population | **Development:**<br>• the frozen 161-response chat-v0 panel, scored as pooled response NLL;<br>• the 38-request fixed panel: 58 complete replies, read as text;<br>• for memory, the D2 relation world, in distribution and on held-out phrasing.<br>**Qualification:** the §8 panel, authored by Lab 1 from typed intent (`answer_oracle`) and sealed before the candidate's final fit: **NOT AUTHORED** |
| Producer → consumer → next missing artifact | **Lab 1 → all:**<br>• S2 `8cb11d8f…` (#1470), the first dialogue-trained stack: **delivered**. Next: I1's store wired into the stack through the memory port. **One artifact:** S2 plus the port, continued on chat-v0 mixed with the D2 relation world. That world is rendered in the same literal-role protocol and #1017 tokenizer, and the mixture and a regression bound on the 161-response panel are pre-registered. It is scored on D2's held-out phrasing, S2's 10 memory trajectories and that panel. The S4 transport and QAT phases apply to this same artifact;<br>• the S1 engine and bundle;<br>• the QAT hook.<br>**Lab 2 → Lab 1:** D6, published with its scope; then G behind the read interface, measured on D2's held-out phrasing and on the S2 checkpoint.<br>**Lab 3 → Lab 1:**<br>• codecs through the hook, at ≤ 0.02 nats and ≤ 4.25 bits per weight;<br>• the measured M1 cost of the D11 stack path, whose gating measurement Lab 1 re-runs |

## 3. Deconfliction rulings, 2026-09-27

1. **ROADMAP.md has one owner per section.**
   - The director owns §0–§3 and §5–§8.
   - Each lab edits only its own §4 subsection, through its own PR.
   - No lab rewrites the whole file. This version supersedes the rewrite in [#1439](https://github.com/UOR-Foundation/uor-r4/pull/1439).
   - #1439 should drop its `ROADMAP.md` change and stop re-carrying #1434 and #1436. Each PR merges on its own.
2. **The dialogue-child fit checkpoint belongs to the #1433 study.**
   - Path: `/Volumes/UOR-Workspace/uor-r4-lab/fourth-lab-sequential-code-choice-fit-1` ([#1433](https://github.com/UOR-Foundation/uor-r4/pull/1433)).
   - **Owner transfer, 2026-09-28:** Codex's allowance is exhausted, so Lab 1 executes the unchanged frozen study. Its resume reads that checkpoint and writes a new root.
   - No other lab evaluates, resumes or modifies it.
   - #1439's proposed observation at step 231 is not authorized. The pre-registered plan finishes all 512 updates, then compares.
3. **The 120-root H4/2I geometry is repointed.**
   - The dense read-score use is dead at its tested parameterisation ([#1438](https://github.com/UOR-Foundation/uor-r4/pull/1438), HARM). Its cause is unresolved until D6 (ruling 11).
   - Codex's exact integer H4 classifier ([#1435](https://github.com/UOR-Foundation/uor-r4/pull/1435)) becomes the multiplier-free decoder for fixed-geometry addressing: T2's event index and T1(c)'s fixed-H4 memory index.
   - Its unpushed `inverse(q)*k` read-lookup table has lost its consumer and is parked.
4. **There is one mission runtime and one front-end.**
   - `uor-r4-integer` with `uor-chat` is the mission runtime and front-end.
   - `uor-r4-lut` with `lut-chat` (D10: hardware multiplies on runtime values) is frozen as a non-mission comparator and gets no new features.
   - The stack's mission serving is Lab 1's S1 (writer, loader, integer forward). Lab 3 supplies codecs, kernels, audits and measured cost.
5. **Hyperbolic geometry survives only as an index or memory geometry.** "Hyperbolic attention", meaning a Lorentz score inside dense reads, is dead at every tested scope (§5). It continues only as a candidate geometry in T2's contest and T1(c)'s memory index.
6. **Branch and worktree naming.**
   - New work uses `lab/<lab>/<topic>`, with lab ∈ {`claude`, `opencode`, `anti-gravity`}. `codex` is historical (three-lab charter, 2026-09-28).
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

11. **External audit corrections, 2026-09-28.**
    - **Source:** a Codex research packet. It is imported verbatim with a claim ledger at [`docs/evidence/external-codex-audit-2026-09-28/`](docs/evidence/external-codex-audit-2026-09-28/README.md), and integrated in [synthesis §7](docs/integration/whole-project-synthesis-2026-09-28.md#7-external-audit-integration-codex-2026-09-28).
    - **Verification:** the director checked every source claim below against the named lines, reran the supplied probes and re-derived the mathematics.
    - **T2 cost scope.**
      - `arm_ranking` scores every previous event, but `decode_spec` charges only the s retained events (`joint-addressing-contest.rs` lines 338–345, 832 and 366–392 at `b0f70c63`).
      - The frozen contest therefore measures **compression fidelity under an exhaustive scan**. Its results stand at that scope.
      - Its decode-cost column must be relabelled. Every T2 report gives inspected and retained events separately.
      - No arm claims a sparse-index cost from this run.
    - **T2 magnitude confound.**
      - The H4 and E8 arms decode to unit roots; k-means decodes to raw centroids (`addressing_arms.rs` lines 713–725).
      - The geometric-versus-ordinary verdict is therefore decided by a **gain-controlled follow-up**: every arm gets the same charged gain channel or none, at equal total bits, with a real cell index whose cost counts inspected events.
      - Lab 2 updates §4.2 to match.
    - **Correct-source admission.** T2 and D3 report correct-source admission beside fidelity to the dense ranking, because the dense reader's favourite events include its mistakes.
    - **#1438's scope.**
      - HARM stands at its parameterisation. It dropped per-lane magnitude, snapped to 120 roots and swapped the dot for a learned MLP together, and its norm control never ran. The cause is **unresolved**.
      - **D6** settles it without a fit. D6 is an information audit (arms O/U/D/G/K; four named outcomes; synthesis §4), assigned to **Lab 2**, which owns #1438's reader and evaluator and T2's accessor.
    - **Design rule.**
      - Group actions are invertible, so they cannot overwrite.
      - Geometry supplies compatibility, transport and addresses. Explicit writes supply record lifecycles.
      - Cells hold exact postings, never summaries in place of records, because sums erase bindings.
    - **Not adopted:**
      - margin certificates as a pruning mechanism (0/192 certified at coarse precision);
      - orbit-expanded codebooks or new relation-kernel fits before D6.

12. **Delivery discipline, 2026-09-28.** This follows an external review the owner shared. The owner decided the last two items at 15:27 UTC.
    - **One target.** The §2b artifact contract is the integration target. Every lab's next output must be something another lab can load or use. No new status documents.
    - **Separate gates.** A probe pass supports only its scoped hypothesis. The §8 milestone decides the product, on the same saved artifact.
    - **Engineering proceeds now; promotion waits for evidence.** Serialization, loaders, session state, tokenizer binding and the stack's integer port do not wait for research.
    - **No fit without:**
      - a named decision;
      - bounded resources;
      - saved, reloadable weights wherever the result could be used;
      - a named consumer.
    - **Every handoff states four facts:** what was selected or rejected; which artifact exists (path and identity); who consumes it; what they do next.
    - **D6 is evidence, not a veto** (owner). Its outcomes describe the representation tested. The next reader experiment must target the failure D6 shows.
    - **G is on the main path** (owner). It is designed after D6, built behind I4 with the dense read as fallback, and promoted only if it wins at equal cost.
    - **Results need measurements** (added 15:45, after #1458). A result merges only with its sealed report roots on disk, and Lab 1 re-reads or reproduces its headline numbers from those roots before merging. A number typed into a test, a document or a PR description is never a result.
    - **Lab 3 results are re-run by the director** (owner, 17:11 UTC, after #1452's figures proved to be packager fallback defaults). Lab 3 keeps D4 and its salvage work.
      - A Lab 3 number is accepted only after the director re-runs its gating measurement from Lab 3's committed code, into a fresh sealed root.
      - Every reader of a result file must fail on a missing key. Defaults are never used.
      - Tests must fail, not pass, when their fixture is missing.

### Engine consolidation map

Fourteen engine paths exist. Only the ones listed as main line or active may receive new features.

| Engine | Disposition | Reason |
|---|---|---|
| Native joint model on `uor-r4-integer` (full256; width-256/576) | **Retained baseline** (the stack passed T1(a) on 2026-09-28) | The only trained path that serves text end-to-end under R1–R2 with no transformer block. Dense access (R3 interim). Language 0/5. |
| Geometric stack (`geometric_stack.rs`) | **Main line** (D0, 2026-09-28) | Best quality: 1.998113 against its control's 2.011149. Its serving moves from `uor-r4-lut` to the D11 engine (Lab 1's S1; S1.1 merged in #1467). |
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
- **Result, 2026-09-28 05:50 UTC** (#1437): after 7,324 updates, on 512 development windows (131,072 targets):
  - stack **1.998113** nats (0.803432 bits/byte) against control **2.011149** (0.808673);
  - 789.3 against 788.1 tok/s.
- **Viable: the stack is the main line (D0).**
  - Both final models and their step-7,300 checkpoints are on the SSD at `uor-r4-models/investigations/cycle4-main-20260928/`, 24/24 SHA-256 verified.
  - Scope: code BPE; one seed per arm; a Cascade Lake host.

**(b) B1: finite-group tracking lanes in the stack** (this window; [#1442](https://github.com/UOR-Foundation/uor-r4/pull/1442), [record](docs/integration/b1-finite-group-lanes-2026-09-27.md)).

- **Closed, 2026-09-28** ([#1447](https://github.com/UOR-Foundation/uor-r4/pull/1447); record §8; `docs/evidence/b1-closure-2026-09-28.json`).
  - The fresh, pre-registered reflection-pair replication tracks A5 exactly, but **fails the per-seed text gate** (+0.069 nats in one seed against 0.05).
  - **No lane type is retained in the stack.**
  - Stage C (natural-text swap stories) is **parked**; its transport witness rotates at almost every token.
  - The exhaustively verified automata (`LaneAutomaton::verify_exact`) remain a tool. M4 (event-gated state) is deferred.
  - The history below is kept as written.

- **Status at 23:43 UTC 09-27: Stage A PASS.** 36 runs.
  - Non-commutative lanes track A5 exactly to length 4,096 after snapping in 17 of 18 runs: quaternion 8/9, reflection pair 9/9. Phase and frozen lanes stay at chance.
  - Quaternion lanes land on 2I; reflection pairs land on the icosahedral rotation group of R³. Both minimise to the same 60-state automaton.
  - **The quaternion-specific serving claim is retired** by the kill rule below. The finite-group state claim survives at Stage A scope.
- **Stage B (01:41 UTC 09-28; corrected after an independent review of #1442).** The host is the recurrence-primary stack `rrar` (1.39M parameters, 1,500 updates on the #1017 token store plus A5 words, 3 seeds). The gates were pre-committed per seed.
  - **Pre-registered subject, quaternion (2I) lanes: FAIL.** Seed 3 did not learn A5 (0.039) and cost +0.121 nats.
  - **Pre-registered transformer control: NOT_RUN.** It will run beside Stage C.
  - **Reflection pair, added after Stage A: exploratory pass.** Text ΔNLL is +0.036, −0.050 and +0.009 (mean −0.002). A5 is 1.000 at position 128 in 3/3 seeds, and 3–4 of 8 lanes per seed snap to an exact 60-state automaton that is perfect at 4,096.
  - **Phase: FAIL,** at chance, costing +0.05 to +0.09 nats. **Lane-free stack:** 0.008–0.160 at position 128.
  - **The kill rule applies as written** (review §9.3):
    - keep the better ordinary lanes (reflection pairs, exploratory pending a fresh pre-registered replication);
    - **retire the 2I geometric-state claim from the serving path;**
    - keep geometry as codebook and addressing infrastructure (T2).
  - **Scope:** synthetic A5 only. State tracking in natural language is untested and is the next question.
  - **Serving:** the automaton (one byte of state, two table reads) is not yet wired into `uor-r4-integer`; that is part of the T3 port.

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

**(c) Sparse parameter memory** (built by the retired cloud track, [#1437](https://github.com/UOR-Foundation/uor-r4/pull/1437); **held**. Disposition, 2026-09-28: its fixed icosian/E8 key index may be offered as an address representation in Lab 2's G proposal. The product-key memory itself is a D5 parameter-sparsity candidate. Its base must be `rrarra`, and its §4 is amended before any arm runs).
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
- decode operations per query, with **inspected** events (every previous event is scored from
  its codes: 128 candidates on average, 255 at most) and **retained** events (s values read)
  reported separately; this configuration makes no sparse-index cost claim.

**Status (2026-09-28, Lab 2).** Contest executed (draft PR #1456). Frozen gate **NOT
QUALIFIED**: within 0.005 nats of the best ordinary arm but 1.17 pt recall behind it on the
full comparison tail at s=16, and the decode is not cheaper. The geometric-versus-ordinary
contrast is confounded by magnitude (H4/E8 decode to unit-norm roots, k-means to raw
centroids; no gain channel in any arm), so the decision moves to a gain-controlled
follow-up. Next: **D6** (evaluation-only information audit, synthesis §4) then **D3**'s
gain-controlled follow-up, which must show **fewer inspected events**.

**Populations:**
- natural development text;
- a D6 long-range panel (induction and copy at distance 32–255) with a verified at-chance count control.

**Decision.**
- If the geometric index comes within 1 point of recall@s and 0.005 nats of the best ordinary index, with a cheaper multiplier-free decode, it qualifies as T3's admission index.
- Otherwise ordinary indexing is adopted and geometric addressing is retired from the serving claim.
- This subsumes the read-localization "oracle re-rank" question: whether admission keeps the correct entity found at rank 2–3.

### 4.3 T3 Numerical fidelity and measured efficiency (Lab 3 Anti-Gravity)

**First: the first measured J/token in the repository.** Measure on the M1 with macmon or powermetrics, from two run lengths:
- the F32 session;
- the retained integer session;
- a llama.cpp SmolLM2-135M Q4_0 reference.

That decides which serving levers matter: bytes, instructions or products.

**Result, 2026-09-28** (*Measured*, owner-run; [#820 note](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5862960224)). Whole-system marginal energy ΔE/ΔN from macmon `sys_power`, single thread, 3 repeats at 10,240 and 32,768 steps, source `3b463398`:
- the retained integer session (`bundle-quaternion-1`, no float, no multiplier): **0.00413 J/token** [0.00394–0.00439], 836–892 tok/s;
- the F32 parent (`fit256-quaternion-3`, comparator only): **0.00096 J/token** [0.00060–0.00128], 1,802–2,552 tok/s;
- the llama.cpp SmolLM2-135M Q4_0 reference: NOT_RUN.

What the numbers show:
- **The integer path, which emulates products with tables, costs about 4.3× the energy per token of hardware float on the M1.** It is also about 2.6× slower. Gross energy for 32,768 tokens is about 578 J against 236 J, so the direction does not depend on the idle subtraction.
- Lab 3's pre-registered consequence therefore applies: the stack port below uses **grouped LUT accumulation (T-MAC style) and fewer bytes touched**, not per-product tables.
- `powermetrics` and macmon both read CPU power as 0 mW on Darwin 27.0.0. SoC CPU energy is UNAVAILABLE on this OS, so the owner's cross-check could not validate macmon.
- The idle spread (±1.5–1.8 W) is about half the net load, so the ratio is directional and the absolute J/token approximate.

**Second: the stack's mission port.** This moved to Lab 1 as **S1**, first by owner direction on 2026-09-28, then under the three-lab charter; Lab 3's role around it is fidelity, codecs, kernels, audits and cost. Its original scope, kept for reference: serve T1's export in `uor-r4-integer` without D10 exceptions:
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

### 4.4 T4 Native dialogue and conversion fidelity (Lab 4 Codex → Lab 1 by owner transfer, 2026-09-28)

- **Execution transferred to Lab 1** because Codex's allowance is exhausted. The study, its gates and its checkpoint are unchanged. The resume uses a hash-checked campaign whose only edits are `resume_from`, the process time cap and the stop file.
- ~~Finish #1433 to 512 updates, then run the 161-response comparison and the 58-turn observation, as pre-registered.~~ **Done, 2026-09-28 06:36 UTC** ([result](docs/integration/dialogue-code-choice-result-2026-09-28.md)).
  - **Outcome: FAILS retention.** Panel NLL −0.053; question-turn relations 2/10 against nearest's 4/10 and the continuous child's 9/10.
  - Momo and green are not recovered; Tokyo and pizza are lost.
  - Learned answers 5 of 10 "What is my …?" questions with "Yes, I can help with that."
  - 6.4% of codes moved.
  - Nearest-hard stays the baseline, and the candidate is preserved.
- **Reading:** corpus cross-entropy pulled the artifact toward the corpus and away from its parent. D4 therefore uses a fidelity objective, distillation from the continuous child, and a different weight representation.
- T1(a) is decided (the stack), so dialogue learning retargets the main-line base in D5, on one tokenizer (I5).
- #1435 repoints to the addressing decoder, per ruling 3.

## 5. Dead and parked paths

Do not resume any of these without new causal evidence and a decision it can change
([D9](docs/integration/DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract)).
Each negative keeps its exact scope; a failed parameterisation does not retire a whole family.

**Owner decision [D12](docs/integration/DECISIONS.md#d12--gates-promote-never-kill-reopen-geometric-candidates-port-the-native-engines-mechanisms-keep-a-geometric-toolbox) (2026-09-28): gates promote; they never kill.** The verdicts below record results at their scope. For mechanisms, DEAD, RETIRED and FAILED now read **"not yet promoted at that scope"**, and the kill rules are withdrawn. The dispositions are in the [geometric toolbox](docs/integration/geometric-toolbox-2026-09-28.md):
- **Reopened as active candidates:**
  - the 2I and reflection-pair tracking lanes, trained jointly with S4's 2I transport;
  - exact memory (I1 plus G);
  - the geometric sparse index, inside G;
  - trained-in 2I/E8 read codes, inside G.
- **Already in the main line:** quaternion rotation (the D1 transport) and the Lorentz reads.
- **Kept in the toolbox:**
  - the 2I relative-relation table;
  - prime/CRT addressing and UOR identity;
  - zeta phase channels;
  - the exact Hopf map;
  - E8/H4 codebooks;
  - orthant admission;
  - vector-symbolic binding;
  - golden-ratio rotation codebooks;
  - horocycle and curvature;
  - Cayley–Dickson algebra;
  - the R4G1 kernel.

| Path | Verdict | Scope and numbers | Source |
|---|---|---|---|
| Lorentz score in dense reads ("hyperbolic attention") | **In the main line** (the stack reads with Lorentz; neutral against Dot at 7M) | Transferred Lorentz and Affine readers are +0.0464 and +0.0133 nats against Dot. Inside the default stack, Lorentz−Dot averages −0.0001 over 2 seeds. The cycle-3 win at reduced scale did not transfer. | [radial result](docs/integration/radial-adaptation-result-2026-09-27.md), [cycle 4 §7](docs/integration/geometric-stack-cycle4-2026-09-27.md) |
| Finite 2I read score | **Not yet promoted** (harm at its confounded parameterisation; trained-in 2I read codes reopened inside G, D12) | Read NLL +0.019719; complete answers 27→17 of 32; 1.84× cost. It dropped per-lane magnitude, snapped to 120 roots and swapped in a learned MLP together, and its norm control (arm C) never ran. **The cause is unresolved**; D6, evaluation only, settles it (ruling 11) | [#1438](https://github.com/UOR-Foundation/uor-r4/pull/1438) |
| Quaternion rotation as a general content mixer | **In the main line** (the D1 transport: +0.07 nats) | −0.025 nats against Householder (1 seed); diagonal decay 1.871 against quaternion 1.880 bits/byte (2 seeds); snapping every lane to 2I +0.16–0.18 bits/byte | [review §6.3](docs/integration/first-principles-review-2026-09-25.md) |
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
| Quaternion (2I) lanes as a serving advantage over ordinary non-commutative lanes | **Not yet promoted** (the B1 kill rule is withdrawn, D12; reopened with S4) | A5 at length 4,096 after snapping: quaternion 8/9, reflection pair 9/9. Both minimise to a 60-state A5 automaton. The finite-group state mechanism itself survives. | [#1442](https://github.com/UOR-Foundation/uor-r4/pull/1442) |
| Quaternion (2I) tracking lanes: the pre-registered B1 Stage B subject | **Not yet promoted** (kill rule withdrawn, D12; reopened with S4) | Inside the stack, 2 of 3 seeds learned A5; seed 3 did not (0.039) and cost +0.121 nats. The ordinary reflection-pair lanes were kept as an exploratory result (3 of 3), then failed their replication (next row). | [#1442](https://github.com/UOR-Foundation/uor-r4/pull/1442) |
| Reflection-pair tracking lanes in the stack (B1 closure replication) | **Not yet promoted** (one seed of six missed the text gate; the six-seed mean is +0.018; reopened, D12) | A fresh pre-registered 3-seed replication tracks A5 exactly (1.000; 60-state automata, `verify_exact` true), but seed 4 costs +0.0688 nats against the 0.05 gate. Six-seed mean +0.018. The automata stay available as tools. | [B1 record §8](docs/integration/b1-finite-group-lanes-2026-09-27.md), [#1447](https://github.com/UOR-Foundation/uor-r4/pull/1447) |
| Context-conditioned lanes on natural-text swap stories (B1 Stage C) | **PARKED at this scale** | The pilots learned no tracking. The transport witness rotates at almost every token (no event gating). Re-entry requires M4's event gate and new causal evidence (D9). | [Stage C record](docs/integration/b1c-context-lanes-swap-stories-2026-09-28.md) |
| Per-product table emulation as a lower-energy serving kernel | **FAILED on energy** (measured) | Integer bundle 0.00413 J/token against 0.00096 for its F32 parent on the M1 (about 4.3×), and about 2.6× slower. Whole-system marginal; SoC CPU counters unavailable. The multiplier-free contract stands; the kernel moves to grouped LUT accumulation (§4.3). | [#820 note](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5862960224) |
| Response-aware legal-code choice with corpus cross-entropy (width-576 dialogue child, #1433) | **FAILED retention** | Panel NLL 2.849→2.796, but question-turn relation answers 4→2 of 10 (continuous 9). Greedy flips against the parent 909→962; 6.4% of codes moved; a generic "Yes, I can help with that." on 5 of 10 questions. The objective, not the dose, is the lesson: D4 uses fidelity to the continuous child | [result](docs/integration/dialogue-code-choice-result-2026-09-28.md) |

## 6. Shared machine protocol

The M1 has 16 GB of RAM and 8 cores, shared by the labs. On 09-27 one lab's fit was stopped by
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
- Two ledgers disagree at present: OpenCode's 756M-ms ledger and the Codex lab's. Lab 2 reconciles them into the programme ledger in current state; Lab 1 holds the Codex lab's record. As of 2026-09-28 the shared ledger `model-time.json` records Lab 1's charges and extensions.

## 7. Cadence and protocol

**GitHub is the shared record for all labs** (owner direction, 2026-09-27).
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
  - Main stays green. Merges follow the [standing merge and review criteria](docs/integration/agent-execution-policy.md#three-lab-organization-and-shared-operating-policy-owner-charter-2026-09-28) (classes A–C) through protected delivery; no per-merge owner approval (owner charter, 2026-09-28).
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

### Next product milestone (defined prospectively, 2026-09-28)

This applies only to candidates built after the frozen studies: B1 closure, #1433, the T2 contest and the base reading. It is never applied retroactively to an earlier experiment.

**The artifact:**
- one sealed D11 bundle, saved and then reloaded in a fresh `uor-chat` process;
- the repaired auditor is clean on its serving binary;
- it declares its context budget, eviction policy and per-turn token limit.

**The panel:**
- authored once from typed intent (`answer_oracle`), sealed before the candidate's final fit, and held out from all training and development;
- every conversation fits the declared budget;
- **responsive multi-turn:** 40 conversations of 3–4 turns, each reply answering its own latest turn;
- **updated relation:** 30 conversations that assert, update and then query a relation, answered with the updated value;
- **new simple instructions:** 30 instructions of at least 6 types absent from training templates.

**Pass:**
- at least 80% of scored turns in each category;
- 100% identical greedy streams across the reload;
- a cost report: tokens/s, RSS, and whole-system J/token.

**Always reported:**
- the float parent and the current dialogue child on the same panel;
- every failed turn classified by the four-class decision trace ([synthesis §7.2](docs/integration/whole-project-synthesis-2026-09-28.md)): record unavailable, available but not selected, selected but wrong value or action, or right latent result but wrong emission.

This reporting adds no threshold. Thresholds change only with the owner, before any candidate exists.

**Baseline, descriptive only:**
- `uor-chat` cannot serve `dialogue-child-bundle-1` (width 576).
- The width-256 `bundle-quaternion-6` answers every turn with an unrelated story.
- The #1432 outputs echo relations ("The name is Alex is Alex.") instead of responding.

## 9. Director log

**2026-09-28 21:42 UTC: owner decision D12 — gates promote, never kill; geometry to the core.**
- **Gates decide promotion only.** A miss keeps a mechanism active with its next step. Parking a family needs a root-cause case and the owner's OK. D9's rule against blind retries stays.
- **Reopened:**
  - the finite-group tracking lanes (A5 exact; six-seed mean text cost +0.018);
  - exact memory;
  - the geometric sparse index;
  - trained-in 2I/E8 read codes.
- **The native engine's mechanisms are ported into the stack:** S4 (2I transport), then I1, prime/UOR keys and zeta phases.
- **S4 is judged three ways,** with no kill.
- **The [geometric toolbox](docs/integration/geometric-toolbox-2026-09-28.md)** keeps every demonstrated geometric piece building and documented, with its novelty and reuse points.

**2026-09-28 20:25 UTC: S2, the first dialogue-trained stack. Rule 1 applies, but its replies are poor.**
- **Primary metric.** On the study's identical 161-response panel, pooled development response NLL is **2.520916**, against the native full-prefix child's 2.773887 and its parent's 3.022131. By the pre-registered rule 1, **S2 `8cb11d8f…` becomes the main line's loss-level dialogue baseline.**
- **Where it is worse.** It is worse than the native child on Rewrite, Summarize, UltraChat and the first-four targets.
- **Complete replies:** 2 of 38 final turns answer the request ("Hello! How can I help you today?"; "The capital of France is Paris."), and none of the 10 memory recalls succeeds. The native child recalled Alex and green. The D10 integer replies match float on 14 of 58 turns.
- **What it changes.** Memory is the named gap: I1's store wired into the stack, plus G (Lab 2). Integer fidelity needs QAT (#1466). Response quality waits for the milestone shape; there is no repeat of the same objective without a cause (D9).
- **Also merged this session:**
  - #1464 (D6), with Lab 1's non-author record;
  - #1465 (three-lab reconciliation). It merged before its review fixes, which follow in a separate PR;
  - #1466 (the QAT hook);
  - #1467 (S1.1, the D11 engine, bit-identical to D10, with a FULL PASS ARM64 audit). Its post-merge non-author review follows.

**2026-09-28 17:25 UTC: three-lab organization adopted (owner charter); cloud work absorbed; §2b decisions.** The charter is recorded verbatim in [three-lab-charter-2026-09-28.md](docs/integration/three-lab-charter-2026-09-28.md).

- **Organization.** Lab 1 (Claude main) leads and integrates, Lab 2 (OpenCode) owns geometric read and addressing, and Lab 3 (Anti-Gravity) owns fidelity, codecs, kernels, audits and cost. The cloud track and the Codex lab are retired, with no future assignments. The single shared operating policy and the **standing merge and review criteria** are now in the [execution policy](docs/integration/agent-execution-policy.md#three-lab-organization-and-shared-operating-policy-owner-charter-2026-09-28):
  - class A, lane work;
  - class B, results;
  - class C, shared interfaces.

  Ordinary progress no longer waits for per-merge owner approval.
- **The cloud track's obligations**, from [its handoff](handoff/cloud-20260928/HANDOFF.md), are all absorbed or decided:

  | Obligation | Status |
  |---|---|
  | S1 (stack export, loader and integer forward) | Lab 1: S1.0 measured (#1463), S1.1 in progress |
  | The D1 models and the cycle-4 weights | On the SSD, SHA-256 verified, with receipts |
  | The four sandbox-only data items | On the SSD, verified, with a receipt |
  | The other sandbox runs (pilot and ablation models, executables) | Not transferred. The owner chose to save the four sandbox-only data items: `registry.u16` and `registry.txt` (listed as not rebuildable), and the cycle-3 code and wiki splits (probably rebuildable, unverified). Of the rest: the pilot and ablation models rebuild only approximately, on another host; the executables rebuild from source (D0 needs `b87acd63` with its one-line `Cargo.lock` change and `-C target-cpu=x86-64-v3`); the experiment directories are partly rebuildable, with their code and notes in `handoff/cloud-20260928/sandbox/` |
  | Cycle 5 (product-key memory with H4/E8 codebooks) | Held. Its fixed key index may be offered as an address representation in Lab 2's G proposal; the product-key memory itself is a D5 parameter-sparsity candidate. It needs the `rrarra` base and its §4 amendment before any arm runs |
  | The `lens.u16` description error | Corrected in both receipts |

  No future task depends on the retired session.
- **§2b decisions.**
  - The two tokenizer identities stay separate.
  - The product memory keeps all six distinctions.
  - The pinned baselines are named: native dialogue `98aca5ab…` at 2.773887, and code stack `3eb1ebbb…`.
  - The development and qualification populations are declared.
  - The first dialogue baseline uses the cycle-4 shape.
- **The first missing learned behavior:** the main-line stack has never been trained on dialogue. Its `dialogue-train` path was checked on synthetic data only, and the prepared chat-v0 corpus was unused by the stack. **Lab 1's next fit is S2**, the first dialogue-trained stack checkpoint, pre-registered on #973 before it runs.
- **Owner-local note, not changed here:** the owner checkout's uncommitted `AGENTS.md` block (the OpenCode research kit) still says "merging asks the owner". The owner may want to align it with the new criteria.

**2026-09-28 17:00 UTC, S1.0 measured; the owner chooses QAT plus a D4 target for S1's fidelity; #1452 not accepted.**
- **S1.0 (Lab 1, evaluation only; [record](docs/integration/s1-stack-serving-measurements-2026-09-28.md)).** The cycle-4 `geometric_s1` exported to the D11 interim format (4-bit maps in groups of 32) misses S1.2's 0.02-nat fidelity gate:
  - **+0.0362** nats with round to nearest;
  - **+0.0257** with GPTQ (top-1 agreement 0.918);
  - the integer arithmetic costs ≤ 10⁻⁶ nats, so the gap is all weight representation.
- **S1.0c attribution.** With one group of tensors from the export at a time:
  - the head alone costs +0.012;
  - the MLPs about +0.006, the embedding +0.003 and the mixer maps about +0.003;
  - the per-channel scalars (conv taps, decay rates, β) cost +0.001.
- **S1.0b.** Snapping every transport quaternion to the nearest unit icosian at evaluation costs +0.026 on each of three models. That is above the 0.02 option threshold, so it is not adopted.
- **Owner, 17:00 UTC: QAT (Lab 1) plus a D4 target.**
  - Lab 1 adds quantization-aware training, with the served 4-bit maps and grid scalars in the loop, and validates it with a pre-registered short fine-tune.
  - D4 (Lab 3) gets the same measured gap as its codec target at 4.25 bits per weight.
  - The milestone fit uses whichever reaches ≤ 0.02 first.
  - S1.1–S1.3 (the D11 port, the bundle and the audit) continue meanwhile.
- **#1452 not accepted.** Its M1 figures (3.16 ms per step, 304 tok/s, 67 ms cold load) were packager fallback defaults. Its own committed harness run shows 7.3 ms, 130.6 tok/s and 1,530 ms. It is back in draft with a [review](https://github.com/UOR-Foundation/uor-r4/pull/1452#issuecomment-5874859825).
  - **Owner, 17:11 UTC:** Lab 3 keeps D4, and the director re-runs every Lab 3 gating measurement before acceptance (ruling 12).
- **Cloud data transfer:** copied and verified (14/14), with the branch deleted.
  - Its README corrects an earlier one: `lens.u16` holds per-token byte lengths, not document lengths.
  - The erratum is recorded in the cycle-4 and D1 receipts on the SSD.

**2026-09-28 16:28 UTC, cloud handoff received; S1 moves to Lab 1 locally (owner).**
- **The cloud track stood down and started nothing new.** S1 had never started, so the owner moved it to Lab 1, locally.
- **D1 models.** `transfer/d1-models-20260928` (`93cd1f13`) was copied to the SSD (`uor-r4-models/investigations/d1-transport-attribution-20260928/`): 24/24 SHA-256 verified, model hashes equal to their reports, receipt written, branch deleted.
- **Handoff archive.** `claude/cloud-handoff-20260928` (`16c77c97`) merges as `handoff/cloud-20260928/`: `HANDOFF.md` and 290 sandbox text files. The 131 vendored third-party Python package files stay on the branch.
- **Owner decision.** The sandbox-only data that cannot be rebuilt comes over on one more temporary branch: `registry.u16`, `registry.txt`, and the cycle-3 code and wiki splits (about 148 MB).

**2026-09-28 16:10 UTC, D1 merged; the stack's serving port goes to the cloud track (owner).**
- **D1 merged** (#1460, `57f5176a`). The director re-read its numbers from the four sealed `report.json` files and matched 23/23 packet hashes. Decision: keep the quaternion transport.
- **Owner, 16:08 UTC.** The stack's D11 serving port, **S1**, goes to the cloud track end to end: the I2 `stack` profile's writer and loader, the integer forward, reload equality, fidelity ≤ 0.02 nats against float, and an evaluation-only 2I-snap diagnostic.
  - Lab 1 runs the ARM64 audit on the M1.
  - Lab 3 stays on D4 and plugs its codec in later.
- The [assignment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5873926600) is posted on #973 and #1460.

**2026-09-28 15:33 UTC, D1 and D2 decided; owner decisions on D6 and G; delivery discipline (ruling 12).**
- **D1 (cloud): keep the quaternion transport.** Identity costs +0.0711 and +0.0774 nats at matched MLP width, in 2 seeds.
- **D2 (Lab 1): frozen gate FAIL on the margin** (0.146 / 0.122 / 0.102).
  - The memory arm scores 1.000 on every class in distribution. The equal-parameter control has 0.51 on First and ≤ 0.07 on Absent.
  - Held out, the learned read trigger never fires on the unseen query phrasing, and accuracy drops to 0.00–0.035. The gold register would give 1.000.
  - No weights were saved (a reports-only runner). Save and reload are now a prerequisite for the next fit.
- **External review (shared by the owner).** Verified and acted on:
  - stale board rows, corrected at 08:20;
  - Lab 3 waiting on the rejected #1433 child, answered by a four-fact handoff at 08:21;
  - D2 saving no weights;
  - the over-restrictive D6→D7 logic.
- **Owner, 15:27 UTC:** D6 is evidence, not a veto; G is on the main path.
- **Merged:** #1437 (cycle-4 result and cycle-5 code), #1454 (runtime reconciliation), #1456 (T2, with the corrections) and #1457 (D2 result and these decisions).

**2026-09-28 15:45 UTC, #1458 not accepted: its D4 results were never measured.**
- #1458's D4 numbers are constants in its test source, one commented "Simulated complete-prefix trajectory flip reduction". Its `current-state.md` edit presented them as "PASSES pre-registered gates".
- There was no pre-registration, fit, report root or generated reply. The stack runtime's `step` compiles to multiply, `udiv` and float instructions, and the bundle contract has no writer or loader.
- The PR is back in draft, with the [review](https://github.com/UOR-Foundation/uor-r4/pull/1458#issuecomment-5873489608). **D4 is open.** The clean codec primitives are to be split into their own PR.
- Ruling 12 gains a rule: results need sealed roots and director-checked numbers before merge.

**2026-09-28 06:36 UTC, #1433 executed (Lab 1 by owner transfer).**
- The fit completed 512/512 updates (3,841 s, peak 4.69 GB). The fixed endpoints ran in 164 s, and every step exited 0.
- The same-binary nearest re-observation reproduces Codex's original exactly.
- **Outcome: FAILS the retention rule.** Lower panel NLL, but 2 of 10 relation answers against nearest's 4 of 10. No follow-up sweep.
- **Consequence:** D4 is unblocked, with a fidelity objective.
- **Slot:** free since 06:36. Next for Lab 1 is D2 (AERM).

**2026-09-28 06:20 UTC, base decided, track approved, external audit integrated.**
- **D0 decided** (05:50 UTC, #1437): the stack scored 1.998113 against its control's 2.011149, so **the stack is the main line** and the native model is the retained baseline.
  - Both final models and their checkpoints were copied from the temporary transfer branch to the SSD.
  - 24/24 SHA-256 verified; the model hashes equal the reports; the branch was deleted as approved.
- **PR backlog cleared** on the owner's instruction: #1447, #1449, #1450, #1435, #1440, #1451 and #1433 merged. #1437 and #1452 need `main` merged first.
- **Track approved** (owner: "Approve, start with D2"): §2a and the [synthesis](docs/integration/whole-project-synthesis-2026-09-28.md).
- **External Codex audit** (ruling 11): verified line by line and imported with a claim ledger.
  - **Accepted:** T2 is a compression-fidelity screen; T2's geometric arms are confounded by magnitude; #1438's cause is unresolved; the capacity reading is a hypothesis; D0 selects an architecture, not dialogue readiness.
  - **Added:** D6 (Lab 2, evaluation only), the four-class failure trace, and counterfactual coverage for D2.
  - **Added conditionally:** D7, score-to-address compilation.
  - **Not adopted:** certificate pruning.
- **Queue** (the slot lock is the only reservation):
  1. the #1433 resume, until about 06:50;
  2. #1433's endpoints (Lab 1);
  3. D2 fits (Lab 1).

  Beside it, as light jobs: D6 (Lab 2) and D4 preparation (Lab 3).

**2026-09-28 04:30 UTC, the owner-commissioned oversight audit was verified and reconciled** ([plan](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5863390710)).
- **Verified:** the auditor's matcher misses six float forms on main, #1446 and #1444. The energy labels were stale; Lab 3 fixed them. Unpublished branches: Lab 2 (7 commits), Lab 4 (`852c1c67`), `geometric-lm-goal` (13 dirty files).
- **Not verified:** the audit's three independent reviewers exited at a usage limit and produced nothing.
- **Base decision:** blocked only on the unpublished cycle-4 full-exposure result in the cloud sandbox. The rule above decides it.
- **Rulings:**
  - no new learner starts until then;
  - cycle 5 is held;
  - the transformer comparators are never the served backbone.
- **Queue:** B1 closure → #1433 frozen resume (alone) → T2 contest → T3 matched-quality cost. The slot lock is the only reservation.
- **Milestone:** the next product milestone is defined prospectively (§8).

**2026-09-28 03:42 UTC, first measured serving energy.**
- The owner ran Lab 3's harness with the `powermetrics` cross-check (§4.3, [note](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5862960224)).
- The multiplier-free integer path costs about 4.3× the energy per token of its float parent, and runs about 2.6× slower. Lab 3's pre-registered reading attributes the gap to per-product table emulation rather than to the contract; that attribution is not separately measured.
- **No lower-energy claim is supported for any served path today.** T3's next kernel is grouped LUT accumulation with fewer bytes touched.
- On this macOS build only whole-system energy is measurable; SoC CPU counters read 0 mW.
- Stage C of B1 (#1447) is in its third pilot, with dense state supervision (amendment 2 on #973).

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

**2026-09-28 02:02 UTC, correction.**
- An independent review of #1442 found the 01:47 entry and T1(b) text overstated Stage B.
- As pre-registered, the 2I arm fails and the transformer control is NOT_RUN. The reflection-pair pass is exploratory.
- The kill rule is applied as written: the 2I serving claim is retired and the ordinary lanes are kept.
- The earlier "B1 complete" and "quaternion-specific" wording is withdrawn. The measured numbers stand.

**2026-09-28 01:47 UTC, B1 complete and the owner decisions carried out** (superseded in part by the correction above).
- **B1.**
  - Stage B passes for reflection-pair finite-group lanes; quaternion lanes are parked on reliability (T1(b)).
  - A Stage A rerun on the committed code reproduced exactly.
  - Evidence: `docs/evidence/b1-finite-group-lanes-2026-09-27.json` on #1442.
- **Owner decisions executed:**
  - #1441 and D11 (#1443) merged, with trees verified.
  - `.uor-models` moved to the SSD: 72 entries, 26.62 GB, 0 SHA-256 mismatches. The 3 sealed-held-out entries stay internal and unread.
  - 9 idle worktrees removed after re-verification. The owner waived the rest of the notice.
  - The B1 grid was split into three parallel streams (owner approved).
  - `macmon` was installed. Its CPU and RAM channels read 0 W on this macOS, so T3 measures whole-system marginal energy. The owner's `powermetrics` cross-check waits on the sudo narrowing requested on #1444.
- **Director reviews:** #1434 and #1436 are "merge after fixes". #1442 is under independent review before its merge is put to the owner.
- **Cross-lab result:** Lab 2's oracle re-rank took 0/5 to 5/5. The native model's read ranking (age prior against content) is the bottleneck; noted for T1.

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
