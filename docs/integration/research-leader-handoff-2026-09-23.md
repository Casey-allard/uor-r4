# Research-leader handoff: UOR-R4 Geometric Language Model

This is a durable **navigation and decision record** for the temporary Kimi/DeepSeek research lead and the returning principal investigator. It is not a replacement for the owner, live repository, artifacts, or sealed evidence. Read the [root agent contract](../../AGENTS.md), [owner decisions](DECISIONS.md), [canonical plan](project-track.md), [current state](current-state.md), [direction assessment](model-direction-2026-09.md), [project map](../PROJECT_MAP.md), and relevant source before acting. Refresh `origin/main`, GitHub issues/PRs, the actual model ledger and physical free space. A dated result or “next” does not outrank those live authorities.

## Mission and non-negotiable distinction

The owner wants **one useful native Rust autoregressive Geometric Language Model** with learned context, conversation/memory, coding and reasoning on consumer M1-class hardware, eventually with measured lower wasted compute and energy. Serving should use bounded geometric routing, exact addressed memory, shared typed operators, integer/low-bit state and table reads. Under owner-adopted D0-b, offline Rust learning may use floating point and matrix products; final serving may use at-most-4-bit additive linear maps implemented without multiplier instructions or floating-point arithmetic. A dense transformer disguised as a lookup, a hosted teacher at serving, or an invented energy claim misses the objective. Geometric structure is a design priority, **not** measured semantic superiority.

Keep six responsibilities separate in every design and claim: (1) exact occurrence, address and version identity; (2) candidate admission and source ownership; (3) learned ranking/attention and relational transport; (4) mutable state and dependent computation; (5) lexical Generate/Copy/Stop; (6) actual loaded autoregressive behavior. A better ranking function cannot recover an omitted source, a signed relation cannot repair a missing lexical feature, and a successful typed fixture is not general prose. The [architecture source audit](architecture-2026-09/README.md) and [formal vocabulary](../formal_vocabulary.md) spell out source and claim boundaries.

## How the programme got here

| Period / path | What is retained | What it does **not** prove | Source of detail |
| --- | --- | --- | --- |
| Earlier exact-memory and native primitives | Prime/zeta/R4/H4 representations, exact addresses, deterministic typed memory/copy/arithmetic and frozen kernel contracts. | Exact storage and authored answers alone are not a learned language model. | [Takeover review](takeover-review-2026-09-19.md), [project map](../PROJECT_MAP.md), [source audit](architecture-2026-09/README.md) |
| Separate TinyStories `.rgm` / native-chat line | A local token predictor and useful tokenizer/training/export lessons. | Its prose score and performance do not automatically transfer to later geometric addressed-memory artifacts. | [Takeover review](takeover-review-2026-09-19.md), [evidence index](EVIDENCE.md) |
| September 13–19 learned read/update experiments | Bounded query-selectable reads, ordered occurrence matching, dependent updates, scheduling, spans and role repair. `%120` residue addressing, cold routes and bounded candidate admission were measured. | A static-residue count comparison is not a hard ceiling on a prefix-dependent trained memory; authored grammar panels are not general language. | [Canonical plan history](project-track.md), [takeover review](takeover-review-2026-09-19.md), [mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) |
| September 20–22 relational/structural memory | Relative H4/Q8 and role/scope/version mechanisms carry useful distinctions. Signed Q8 retains an answer-relevant order/sign distinction; the competent ordinary finite comparator ties on the authored computation task. Observation and computation were integrated across prior scoped scripts. | A geometric witness or a 38/38 authored lifecycle is not geometric superiority, open-ended reasoning or self-sufficient text understanding. Some earlier narrow-fit failures were supervision coverage, not geometric impossibility. | [Evidence index](EVIDENCE.md), [current-state history](current-state.md), [Hopf and role direction](structural-memory-hopf-direction-2026-09-20.md) |
| September 22–23 shared lexical line | A single native Generate/Copy/Stop artifact was fitted on source-separated prose and authored grounded cases. The later independent audit verified all reported panels after fresh-process reload. | Exact loaded parity does not make its language useful: greedy continuations collapse, and learned post-copy feedback dependence remains unverified beyond the tested crossed pair. | [Principal review](ordinary-lexical-principal-review-2026-09-23.md), [audit/correction](ordinary-lexical-audit-2026-09-23.md), [current state](current-state.md) |
| September 23 formulation milestone | Five matched arms re-tested the recorded per-position question inside the same grounded learner through one unchanged model source; the default path reproduces the delivered artifact bit for bit; on the served conditioning the prose-only window objective reaches 6.6841 and the per-position arm reaches 7.0327 with full grounded-panel retention. | The per-position objective is not the cause of the recovery (per-position minus window **+0.0641 [−0.0142, +0.1490]**), no arm beats the tuned two-token count reference (5.1217), and greedy generation still collapses. A repaired window objective that fully retains the grounded panels is untested. | [Result](ordinary-lexical-formulation-result-2026-09-23.md), [evidence](../evidence/ordinary-lexical-formulation-2026-09-23.json), [current state](current-state.md) |

Do not fuse these artifact families into one claimed result. Historical branches, negative candidates and sealed roots are retained so a later model can inspect them. The [research ledger](../RESEARCH.md) and named experiment records own exact artifact, population and control scope.

## Latest measured seam (September 23 formulation milestone; refresh before a run)

The recorded per-position question was executed inside the one shared grounded Generate/Copy/Stop
learner, on protected main `79b5d50c`, through one **unchanged** model source
(`learner/transferable_lexical.rs` sha256 `03f83eb8…b250`); only the runner gained `--objective`,
`--curriculum`, `--prose-slots`, `--ground-slots`, `--ground-weight` and a read-only
`--condition`/`--artifact` scoring mode. The default path reproduces the delivered artifact **bit for
bit** (`olx-form-1`, sha256 `fe3e8a638cd50a19741065cb00ac787b63bb59f8ad5776fd8bfcb162c37a912a`,
development `7.372198274238398`), so every arm is matched to the delivered baseline through one path.

On the **served** window conditioning — the recurrence the served multi-token path actually executes —
and the same 5,376 development targets with paired document-cluster intervals: the per-position
objective with grounded supervision scores **7.0327** against the delivered 7.3722
(**−0.3395 [−0.4388, −0.2447]**) and **fully retains the grounded panels** (32/32 preflight A, 3/3
held-out, 2/2 preflight B, 36/36 training, 4/4 class, `REPLAY_OK 36 cases`); a **prose-only window
objective** reaches **6.6841** (−0.6881 [−0.7686, −0.6091]), crossing donor E (6.8701) for the first
time; a window objective with two grounded draws per step reaches **6.9686** (−0.4036
[−0.4668, −0.3412]) with 31/32, 1/3, 2/2. Per-position minus window at the same schedule is only
**+0.0641 [−0.0142, +0.1490]**, so **the per-position objective is retired as a necessary training
change** and retained as a diagnostic conditioning: the deficit was the grounded rehearsal mix and the
grounded-only final phase. The guard run's step probe shows the delivered schedule reaching its best
prose probe during the warm-up (7.1080 at step 800) and never better after grounded supervision
saturates.

**Erratum carried forward.** The retained audit's “about 1.17 bits” compared each artifact under its
own conditioning. Under the served conditioning the per-position prose-only artifact (`olx-channel-1`)
recovers **0.682 [0.581, 0.790]**, not 1.17; roughly 0.49 bits was the conditioning change, available to
serving only for the first generated token. The audit's data set, artifact and sealed numbers are
unchanged.

**What still fails.** The fit/tune-separated tuned two-token count reference at **5.1217** still beats
the best arm (6.6841) by **1.56 bits/target**, so the 64-coordinate ternary state under-uses the two
tokens available at the decision point — a local-recency/optimization fact, not a long-range transport
one. Greedy generation collapses on every arm; sampling NOT_RUN; energy UNAVAILABLE. The next bounded
milestone is to keep the served window conditioning, raise the grounded presence until the authored
panels are fully retained (the configuration `olx-form-3` was one case short of), pre-declaring 32/32
preflight A, 3/3 held-out and 4/4 class as acceptance, and then attack the local deficit. The
[result](ordinary-lexical-formulation-result-2026-09-23.md) and [evidence](../evidence/ordinary-lexical-formulation-2026-09-23.json)
own the numbers; the [plan](project-track.md) and [state](current-state.md) own sequencing.

## Geometric mechanism decision tree

Start with the missing information or behavior, not a favored manifold. For a local prose-quality failure, first inspect context exposure, objective, target distribution, optimization, export parity and a competent count/ordinary model. For an omitted distant source, fix admission/retention before changing the relation score. For an order, role or nested-scope collision with sources present, compare exact typed slots and a finite codebook to signed relative H4/Q8 under equal information and serving cost. For a witnessed interference/capacity limit, then consider a retained-fiber Hopf state or explicit coefficient bank.

The owner’s S3/Hopf, S7/E8, harmonic/resonance, spin/chirality and scalar-influence ideas belong in that **conditional toolbox**. An S3-to-S2 Hopf observation loses fiber information unless the S1 fiber is retained; the quaternionic Hopf fibration is S3 → S7 → S4, not a reversible universal S3 → S2 → S1 → S3 chain. S7 unit vectors are 8 real coordinates, while a mutable harmonic **field** needs stored coefficients and a read/write law. Orthogonality under an integral does not make a single sampled wave crosstalk-free; E8's 240 roots are not 240 orthogonal registers. Normalize information, precision, update cost, candidate count and extraction cost against explicit role banks and existing H4 transport. [Structural-memory/Hopf review](structural-memory-hopf-direction-2026-09-20.md) contains the mathematical and experimental scope.

“Quantum spin,” supersymmetry, symmetry breaking, twin primes and the owner's topological-folding expression are prompts for precise **classical** representation hypotheses, not established physical effects in this model. In particular, `exp(i*pi) + pi^0 = 0` under ordinary arithmetic; equating the result with `0^0` depends on a chosen convention and does not introduce a quantum bit. Any proposed phase, chirality, fiber, or influence magnitude needs a defined state/action/observable, a counterexample it solves, and a matched ablation. Preserve exact `Z[phi]`, ordered n-lets and paired-H4/icosian identities when a variant uses them; a prime/hash label alone is not semantic distance.

## How the lead should make a research decision

1. Name one missing capability and trace the **actual** causal information from input through candidate admission, state, output and reload. Separate an instrument defect, unsupported supervision, representation loss, optimization failure and true capacity limit.
2. State the smallest falsifiable hypothesis, what observation would change the decision, and the strongest simpler comparator with the **same available information**. Keep a development panel open; freeze an independent acceptance population only after design selection. Use source-separated text and actual generated/loaded behavior when claiming language.
3. Check equations and priors against original papers, theorem statements and repository source. Use the browser and exact/symbolic tools when they answer a concrete question. Record a URL and mark what a source states versus what we infer. Do not treat analogy or another model's consensus as proof.
4. Design causal interventions that change one relevant input while holding others fixed; retain row-level prior controls, source ownership, timing, memory and bytes touched. A negative applies only to the tested mechanism/configuration/population **after** its instrument and comparator are audited. Do not discard a whole geometric family from a tiny, mis-scored or unfair ablation. Conversely, do not promote a narrow fixture near-miss to general language.
5. Give DeepSeek substantive design and implementation discretion within that question. Ask an independent K3 architecture reviewer for consequential mathematical pivots or promotion claims; the lead integrates disagreements and remains accountable. Delegate independent tasks with explicit source revision, file ownership, deliverable and claim boundary. Stop refining the same panel when it cannot change the architectural decision; advance the model along the [canonical dependency plan](project-track.md).

## Autonomous work, tools and durable memory

Kimi's [team](../../.kimi-code/TEAM.md), [research protocol](../../.kimi-code/RESEARCH-PROTOCOL.md) and [tool/storage runbook](../../.kimi-code/TOOLS-AND-STORAGE.md) specify roles, dispatch, GitHub, cost and safe cleanup. Use `rg`/source first, `gh` for live issues and protected PRs, focused Rust tests and independent deserialization for behavior, SymPy/Lean for exact mathematics when warranted, and signed-in browser access for primary literature. `uor_knowledge` is **read-only historical retrieval**, not writable project memory. Codex-only MCPs/plugins do not transfer automatically; verify a tool in the active client before assuming it works.

The **writable shared memory** is the protected repository: a scoped result/evidence record with artifact/source/data/control identity, `current-state.md` for the newest verified seam, `project-track.md` for dependency/issue sequencing, and a decision note only for a real architectural change. Update README, project map and owning GitHub issue where their public claim or status changes. Record a short local `~/.kimi-code/UOR-R4-HANDOFF.md` pointer with the merged revision, limitations and next decision. At the start of the next session, re-open that pointer **and** its cited protected record, query historical memory for relevant prior work, and check that the lead can recover the last result; fix the handoff if it cannot. Never put credentials, raw session traces or unverified results into shared memory.

Before substantial computation, project complete CPU/wall-time, RAM, temporary and retained disk, the cumulative model ledger, and the physical 128 MiB stop margin. The standing owner authorization permits **prospectively recorded necessary local extensions**, not paid compute or deletion of unique artifacts. Preserve the owner's checkout and sealed/model evidence. Deliver code/docs through a protected PR; an open PR is pending, not merged. Verify merged content before updating issue status. End each run with the measured result, strongest negative, exact limitation, resources and **one** evidence-supported next milestone. The owner can initiate that milestone with `Proceed UOR-R4`.
