# ADR-0007: Stage interfaces for end-to-end integration

- **Status:** accepted (integration architecture). Individual stage promotions remain separate decisions and keep their board status.
- **Date:** 2026-11-15
- **Context owner:** the owner's direction of 2026-11-15: connect the mechanisms into one end-to-end training and serving process, replace matmul and the slow parts of softmax, attach e-means where it fits, and import the CIRISKernel window/checkpoint. As of [Amendment 1](#amendment-1-2026-11-15-claude-394), PLAN.md's owner directions do not yet record these words. That direction does not release anything CHARTER §3 bans.
- **Amended:** 2026-11-15 by claude-394, at Casey's request. See [Amendment 1](#amendment-1-2026-11-15-claude-394).

## Context

The project has 28 recorded mechanisms on [the board](../../mechanisms/README.md) — 12 ACTIVE, 5 READY, 11 PARKED — spread over two model paths (the discrete M1 path and the float-trained FBS trunk served by D11). Each was built against its own need, with its own state, its own selection rule and its own test. There is no shared stage boundary. The consequence, measured in the inventory of 2026-11-15:

- **Five selection rules and four aggregation rules for one conceptual operation.** The float trainer's Flock read selects by mean L1 and mixes by plain mean; the served Hamming read selects by popcount and mixes by margin weights; the discrete path selects by L1 + Hamming + trit sweeps; the parked engine uses exp-table weights with a `NoRead` slot.
- **Every integration is a bespoke port.** A mechanism joins the served path only when someone re-implements its arithmetic inside `crates/uor-r4-integer/src/stack/session.rs`.
- **No compaction anywhere.** The session refuses at `context`; the float decode slides; the discrete chat silently drops the oldest token; `StackChat` refuses and resets. None carries a summary.
- **Training and serving disagree about what the read is.** The served Hamming read is the only form whose selection rule is trained; the others are fixed rules wrapped around learned tables.

Global optimality over a portfolio is not reachable without (a) a common seam per stage, (b) a common metric, and (c) a rule that says when one mechanism may replace another. This ADR fixes those three; it does not promote any parked mechanism.

## Decision

### 1. Seven stage interfaces, one mechanism per stage per artifact

Every model is described as a composition of seven stages. Training and serving share one persistent native integer state and call the same typed operators (CHARTER §3.7, D85). Gradients and optimizer updates land in that representation, and floating point is legal only in separately scanned reference or diagnostic code. Each stage has one Rust interface, one artifact field, and one parity test.

| Stage | Interface (integer serving) | Reference implementation today |
|---|---|---|
| `Tokenizer` | `encode(&str) -> &[u32]`, `decode(&[u32]) -> String` | 4,096-piece tokenizer; 16,384-piece trainer is tooling |
| `Embedding` + `Codebook` | `encode(&[i64]) -> u32`, `decode(u32) -> &[i64]`, `export() -> TableImage` | token table; A8 address cells; prime-zeta addresses |
| `SequenceMixer` | `step(&mut self, x: &[i64], ctx: &mut dyn Context) -> &[i64]` | FBS recurrence + Hamilton gate; discrete block apply |
| `Position` | `apply(&self, pos: u32, q: &mut [i64; 4])` | SpiralCore transport spin in the recurrence (no RoPE) |
| `Read` | `select(&self, q: &Query, view: &ContextView) -> ReadOut` | Hamming copy/type/ask heads; flock; route; relation |
| `Context` | `push(span)`, `settle() -> Stub`, `merge(&Self)`, `view() -> ContextView` | none (the gap this ADR closes) |
| `Head` + `Loss` | `score(&[i64]) -> u32`; training loss | admission; whole-table nearest; rank-margin hinge |

`ReadOut` is fixed: `support: Vec<u16>` (positions, canonical order), `weights_q31: Vec<i128>` (the rank weights, summing to their own total), `payload: Option<Vec<i128>>` (a value row, or `None` for points). A read that cannot express its result this way is out of scope until it can.

Rules:

1. **One mechanism per stage per artifact.** The artifact names each stage's mechanism and table digest. The engine refuses an artifact whose stage it does not implement; it never silently falls back.
2. **Every implementation ships with a reference adapter.** The adapter reproduces today's behavior exactly, so the interface can land with bit-identical tests before any new mechanism is wired.
3. **No stage may change two things at once.** A unit replaces one stage implementation or adds one training arm; a unit that changes two stages is refused by its own ORDER.
4. **The schema carries the composition.** A new stage mechanism is a new `/N` schema field, not a flag buried in a table. `/11`–`/14` already follow this rule.

### 2. The selection protocol

"Global optimality" needs a global measure, so selection is a constrained comparison on one frozen corpus, not a per-stage maximum:

1. **Exactness first.** An exact mechanism must be bit-identical to its reference on the frozen corpus. A lossy mechanism states what it discards and must pass the witness-pair test for the reads it claims to preserve.
2. **Quality second.** Frozen panels, pre-registered, the same evaluator for every arm. A promotion needs the panel and its control, not a single run.
3. **Cost third.** `tokens/s`, `bytes/state`, `tables`, and `work/token` on the same corpus and machine. A cheaper arm that loses exactness or quality is not a candidate.
4. **Ablations are staged.** Stage-by-stage first, then the interacting pairs (`Context × Read`, `Codebook × Embedding`; `Read × Position` left with I4, Amendment 1), because the inventory shows these are where the rules disagree.

### 3. The mechanism map

Statuses are the board's; this ADR changes none. "Action" is the integration step, not a promotion.

| Mechanism | Role | Status | Stage | Action |
|---|---|---|---|---|
| Tokenizer (4,096) | tokenizer | ACTIVE | `Tokenizer` | Wrap as the reference adapter; keep the 16,384 trainer behind a flag and a separate decision |
| D11 integer serving | arithmetic / runtime | ACTIVE | all | The serving contract; every stage implementation compiles into it |
| SpiralCore/FBS routes | sequence mixing | ACTIVE | `SequenceMixer` | Reference adapter; keep as the default trunk |
| Hamilton gate (G2) | feed-forward gate | ACTIVE | `SequenceMixer` | Inside the mixer; expose as a gate table, not a stage |
| Recurrence gates | gates | READY | `SequenceMixer` | Second mixer arm; parity against the FBS decay table |
| Discrete geometric learning | optimizer | ACTIVE | `Loss`/trainer | Second training arm; no change to serving |
| Rank-margin hinge | loss | ACTIVE | `Loss` | Reference loss. The JEPA arm is parked (Amendment 1) |
| Admission | output head | ACTIVE | `Head` | Reference head; nearest and pointer are arms |
| Whole-table nearest | output head | ACTIVE (in recipe) | `Head` | Reference for the FBS path; measure admission-vs-nearest per corpus |
| Prime-zeta codebook | embedding | ACTIVE | `Codebook` | Candidate encoder; already a baseline start |
| Address cells (A8) | embedding | ACTIVE | `Codebook` | Candidate encoder; e-means fits its placement fit |
| Routed read | attention | ACTIVE | `Read` | `ReadOut` adapter; tree descent returns positions |
| Flock read | attention | ACTIVE | `Read` | `ReadOut` adapter; the only fully softmax-free selector |
| Relation read | attention scoring | ACTIVE | `Read` | Adapter with a relation-typed support |
| Hamming read (ternary/XOR form) | attention | READY | `Read` | Reference for the copy, ask and place heads; the read-index unit targets the copy/ask forms. The **place head** is a write/placement operation, not a position-and-weight read, so `ReadOut` does not yet carry it — it joins the relation read on the I1 risk list |
| Hopf kept-fibre map | attention scoring | READY | `Read` (scoring) | Telemetry-only; do not gate a read on it |
| Spin-native weights | matmul replacement | READY | `SequenceMixer` | Arm; its 1.5148 BPB came through a banned dense multiply, so it needs a compliant form first |
| Ternary/XOR | matmul replacement | READY | `Read`/`SequenceMixer` | Arm for sign-code reads; dense ternary is still dense (D25) |
| Object store (D19) | durable memory | READY | `Context` | Backing store for stubs and checkpoints; not a read |
| Exact memory | key-value memory | PARKED | `Read`/`Context` | Its trained forms missed the bar or used softmax; the exact-address form is a candidate for `Context` stubs |
| Pointer key | copy head | PARKED | `Read` | Softmax pointer; not a candidate until expressed as a rank read |
| Read binding | auxiliary loss | PARKED | `Loss` | Arm only if the teacher marks survive D41 |
| Hopf sector routing | attention scoring | PARKED | `Read` | Reopen condition unchanged |
| E8 orbit memory | feed-forward layer | PARKED | `SequenceMixer` | Arm; +0.033 BPB within bar, needs a non-dense form |
| Carrier credit | optimizer | PARKED | `Loss` | No arm until a live objective exists |
| Legal construction | optimizer | PARKED | `Loss` | Discrete-path arm only |
| VSA | key-value memory | PARKED | `Context` | Its 4,096-bit codes fit stub identity; blocked by the banned CPU learner |
| Geometric stack | whole model | PARKED | — | Not a stage; preserve as evidence |
| Native geometric engine | whole model | PARKED | — | Not a stage. Its JEPA trainer uses a cross-entropy term and an f64 Adam, so it cannot be reused as written (Amendment 1) |
| R4G1 graph | whole model | PARKED | `Codebook`/`Read` | Harvest its induction as a codebook builder; do not revive the compiler as the model |
| E8 address memory (since 31 Oct) | feed-forward table | no status | `SequenceMixer` | Arm; inconclusive at claude-366 |
| Pair experts / pair rows (since 31 Oct) | reply memory | no status | `Read`/`Head` | Keep opt-in; they are outside the vector state and must not masquerade as one |
| Zeta modules (`zeta_zeros`, `zeta_projection`) | address phase | no status | `Codebook`/`Position` | Address/phase only; no semantic claim (ADR-0003's retained scope) |

The map is a superset of the board's 28 rows: it splits the Hamming form out of ternary/XOR and adds the whole-table nearest readout, which is a form of the admission head.

### 4. The four attachments

**e-means → `Codebook` trainer.** e-means (arXiv 2611.09501) is a mini-batch k-means with EMA centroid counts/sums, a cosine-annealed step, dead-cluster revival and a last-epoch-mean finalization. It changes the *solver*, not the centroid or the serving arithmetic. It attaches behind `Codebook::fit`, at the four k-means sites already in the tree: `crates/uor-r4-training/src/addressing_arms.rs` (Lloyd), `crates/uor-r4-graph-compiler/src/induction.rs` (spherical mini-batch), `crates/uor-r4-core/src/transformerless/compiler.rs` (sampled RVQ), `crates/uor-r4-training/src/joint_admission.rs` (PQ-ADC). Gate: the exported integer table is bit-identical across two runs, and the final MSE is no worse than Lloyd at no more than one third of its iterations. The paper's scaling laws are not adopted: they were fitted at `n ≥ 10⁵`, `k ≥ 256`, and say they are not optimal for small or low-dimensional data.

**CIRISKernel → `Context`.** The transferable part is the checkpoint/purge discipline: a span may be replaced by a stub only when its answer can no longer change (for the kernel, `n − t + 1` clean receipts; for us, the read-invariance criterion), and the stub keeps the digest, the fields the read consumes and the conclusion. The concrete form is a ring of `w` spans with a settlement gate before overwrite, a mergeable fold for branched sessions, and an overflow policy that replaces today's silent drop/reset. Gates: bit-identical tokens against a full-history session; a witness-pair suite for every read that claims preservation; `bytes/token` reported.

**RoPE: withdrawn (Amendment 1).** As first written, RoPE would have been a `Position` arm compared against transport spin. The mechanism board (`docs/mechanisms/README.md`, its Position and positional-encoding rows) carries position as SpiralCore transport spin, "no RoPE", so the arm is withdrawn. `Position` has one mechanism, transport spin. Reopening RoPE would need an owner decision.

**JEPA: parked (Amendment 1).** The JEPA trainer (`crates/uor-r4-core/src/native_geometric/learner/jepa_trainer.rs`) optimises `L_CE + λ·d_FS + λ_fiber·d_U(1)` with an f64 Adam. The `L_CE` term is cross-entropy, which CHARTER §3.2 names in its ban, and the optimiser's float state breaks §3.7. So the trainer cannot attach behind `Loss` as written. Its latent terms (`d_FS`, `d_U(1)`) may be proposed again as a `Loss` arm under three conditions: their loss, gradient and update are native Q31, nothing in them is cross-entropy or a normalised exponential, and they are judged on served replies.

**Graph compiler → `Codebook` builder.** The graph compiler is historical. The JEPA trainer (parked above) and the induction code live in `crates/uor-r4-core/src/native_geometric`, although the board's whole-model row *Native geometric engine* remains PARKED. Its preserved induction (content-addressed seeding, spherical mini-batch k-means, bit-identical convergence) is a usable `Codebook` builder and an e-means fit site. The compiler pipeline is not revived as the model.

### 5. Unit sequence

Each unit is one PR, one stage, one gate; I6 is the exception, composing the stages rather than replacing one.

| Unit | Stage | Deliverable and gate |
|---|---|---|
| **I1** | `Read` | The one `Read` interface and `ReadOut` that D86 step 6 needs. Its reference implementation is the native Q31 first-pick read (`Q31FirstPickRead`, claude-389 to claude-393), shared by training and `IntegerStackSession`, with bit-identical replay. The Hamming copy and ask heads are the second implementation. Flock, route and relation adapters wait until a second consumer needs them (Amendment 1). |
| **I2** | `Context` | Ring + settled stub + mergeable fold; overflow policy; full-history equivalence. **Required gate:** the codex-3 witness pair (two histories with the same sum and different top-1, PR #4015) or its successor must pass for every read the stub claims to preserve. Imports the CIRISKernel discipline. |
| **I3** | `Codebook` | **Deferred until D86 step 6** (Amendment 1). e-means behind `fit`; integer export; bit-identical table and the ≤⅓-iteration gate; applied first to one site (address cells or induction). |
| **I4** | `Position` | **Withdrawn** (Amendment 1): transport spin is the one position mechanism. |
| **I5** | `Loss` | **Deferred until D86 step 6, and parked until it is CE-free and native** (Amendment 1). |
| **I6** | assembly | `ModelSpec` + artifact fields naming every stage; one end-to-end run on the frozen corpus with the cost ledger, judged by its replies. It is the same run as D86 step 6, not a second one. |

I1 is the critical path, because it is where this ADR and D86's native read meet. I2 follows once a model holds a conversation long enough to overflow its context. The paused read-index work (the exact code→positions index for single-code heads) is I1b, after I1.

## Amendment 1 (2026-11-15, claude-394)

At Casey's request, after a zoomed-out read of this ADR posted on [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6903557510). The amendment adds no rule. It brings the ADR in line with rules already in force and points its first unit at D86.

"D86 step 6" here means step 6 of the path the D86 review set out, which is recorded in [PLAN.md](../../../PLAN.md)'s owner direction of 2026-11-15: one model trained and served natively, judged by its replies. The D86 row in DECISIONS.md records the donor release and does not list the steps.

1. **Training.** The ADR first said "training may use float". CHARTER §3.7 and D85 say otherwise: gradients and optimizer updates land in the native representation, and float is reference code only. §1 now says so.
2. **RoPE is withdrawn** from `Position`. The mechanism board carries position as transport spin, "no RoPE". I4 is withdrawn with it.
3. **JEPA is parked** as a `Loss` arm. Its trainer's `L_CE` term is cross-entropy, which §3.2 bans by name, and its f64 Adam breaks §3.7. I5 waits for a CE-free, native form.
4. **I1 is narrowed** to the one `Read` interface D86 step 6 needs, with the native Q31 first-pick read as its reference, so that this ADR and D86's read line meet instead of forking.
5. **I3 and I5 are deferred until D86 step 6.** The evidence says the model's gaps are data (about 39k usable TRAIN windows, D86's review) and a read that averages the fact it found (claude-387/388). Neither is a codebook solver or a loss term. Interface units that move no reply are L-008's pattern.
6. **The codex-3 witness pair is a required gate** for I2 and for any future compaction or summary of context. A stub that keeps a fold but not the selection loses the answer, the same failure as the averaged read.
7. **The context line's owner direction** (e-means, CIRISKernel) has no row yet in PLAN.md's owner directions or DECISIONS.md. Its verbatim words should be recorded there (L-001). The direction as quoted releases nothing that §3 bans.

The mechanism map's statuses are unchanged.

## Alternatives considered

- **Keep integrating bespoke.** Rejected: five selection rules already disagree, and each port costs a unit.
- **One unified model with every mechanism.** Rejected: the mechanisms interact; a single build cannot attribute an effect, and the parked ones have reopen conditions.
- **Optimize each stage locally and sum the wins.** Rejected: the inventory shows the read, position and context interact; local maxima do not compose.
- **Adopt e-means' scaling laws as-is.** Rejected: fitted at a scale we do not have, and explicitly not optimal for tiny or low-dimensional data.
- **Revive the graph compiler as the model.** Rejected: it is historical; its induction is reusable, its pipeline is not.

## Consequences

- **Easier:** a mechanism can be swapped without touching `session.rs`; every swap has a parity test and a cost entry; the portfolio question becomes measurable.
- **Harder:** the interface must be rich enough for seven mechanisms without becoming a union of special cases; `ReadOut` may need a relation-typed variant; the context stub needs a serialization format.
- **Riskier:** schema churn (`/N` per stage), qualification debt (each adapter needs its own evidence), and the temptation to promote on one panel. The frozen corpus and the metric vector must be fixed before I1 lands.
- **Revisit if:** a stage cannot be expressed without floats in training or serving; a parity adapter cannot reach bit-identity; or a mechanism's reopen condition changes its stage. Revisit the e-means scaling laws only after a corpus of at least `10⁵` rows and `k ≥ 256` exists.
