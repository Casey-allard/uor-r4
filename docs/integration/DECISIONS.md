# DECISIONS

Append-only. Each entry records its authority, rationale and scope. D0–D13
record owner direction. Under D14 the owner delegates prospective working-policy
decisions to the recorded council; immutable mission, evidence, unique-data and
spending boundaries still require owner direction. Preserve original decisions
and mark supersession explicitly rather than rewriting historical outcomes.

---

## D0 — What "no matmul at serving" means

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: D0-a — the multiplier-free-kernel reading.**

The project's objective is to eliminate *wasted compute*. The serving path must
execute no multiplier and no floating-point arithmetic, and must not perform a
dense contraction that touches every parameter for every token. Offline training
is unrestricted.

### Permitted at serving

- integer add, subtract, shift, rotate, bitwise AND/OR/XOR/NOT, popcount, compare
- bounded index arithmetic and bounds checks
- exact fixed-point arithmetic in Q1.15 / Q1.30 (add and shift only)
- **table and array reads of learned values** — the value was learned offline; the
  serving operation is a selected read
- bit-serial / LUT accumulation of low-bit (ternary or ≤4-bit) weights, where the
  multiplier circuit is absent by construction (T-MAC-class kernels)
- exact selected reads, writes and copies over addressed memory

### Excluded at serving

- IEEE `f32` / `f64` arithmetic and `libm` transcendental functions on the hot path
- the integer `*`, `/`, `%` operators on the hot path (permitted only through
  documented shift-and-add idioms such as `square_u64` / `div_small_positive`)
- any GEMM or matrix-vector product that multiplies and accumulates over a dense
  weight matrix
- any tabulated implementation of a **dense** contraction: if serving touches all
  entries of a learned weight store per token, it is a matrix product regardless of
  the opcode used

### Unrestricted offline

Floating point, gradients, `uor-matmul`, Adam and any other optimizer. Training may
freely compute linear maps; the constraint applies only to the served computation.

### The distinguishing test

**Per-token parameter sparsity.** For each token, count the fraction of the learned
parameter store that is read. Sparse selected access ⇒ compliant. Dense access ⇒
non-compliant, whatever arithmetic is used. This test, not the opcode, decides.

### Consequences

1. The lane tables (`discrete_tables[l]`, 120×120 trained by Adam) are
   **compliant**: reading one 120-entry row is a sparse selected read.
2. Two current serving operations are **not** compliant and must be repaired or
   removed: the JEPA projection in `learner/jepa_trainer.rs::predict_jepa_step_q30`
   (literal `i64` multiplies) and the `f64` softmax sampler in
   `native_capability_api.rs` (reachable whenever `temperature > 0.001`).
3. A ternary linear map trained offline and served by LUT accumulation **is**
   compliant. This is the precedent-backed path to competitive quality.
4. The root `README.md` and `AGENTS.md` wording ("no mathematical matrix products,
   including … tabulated lookup/add contractions") is **narrower than this decision**
   and currently conflicts with the shipped implementation. Updating that wording is
   a stable-goal change and must go through owner-directed protected delivery; until
   it does, this file is the normative statement of intent and the conflict is known.

---

## I1–I5 — Serving efficiency invariants

Recorded with D0. These are measured, not asserted, and are reported together.

| ID | Invariant | Measurement |
|---|---|---|
| **I1** | Sparse per-token parameter access | fraction of the learned store read per token |
| **I2** | Low bytes/token | bytes streamed per token; bits per stored weight |
| **I3** | Bounded candidate scoring | candidates scored per token (target ≪ vocabulary) |
| **I4** | No multiplier in the kernel | static AST check + dynamic op census |
| **I5** | Exact addressed memory, no recomputation | provenance/version reads per token |

I3 and I5 are the properties a dense transformer cannot match at equal quality and
are the basis of any efficiency claim.
---

## D0-b — What "no matmul at serving" means, adopted

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: D0-b. Bounded integer/ternary linear maps are permitted at serving, executed with no
multiplier in the kernel.** This supersedes D0-a, whose strict reading banned the mathematical
linear map even when tabulated.

### The serving contract

- **No floating point** at serving, and no `libm` transcendentals.
- **Weights at most 4 bits**, ternary preferred.
- Allowed: integer add, subtract, shift, bitwise, popcount, compare, table and array reads, bounded
  index arithmetic, exact fixed-point.
- **No multiplier instruction in the kernel.** A linear map is legal only in a form that executes
  as adds/subtracts/shifts/table reads.
- **Energy measured, not estimated**, on a named machine, when the value is claimed.

### Why D0-a was withdrawn

The strict reading banned the one operation every architecture that *learns a representation* uses,
and thereby excluded every published precedent (BitNet b1.58, MatMul-free LM, T-MAC) while leaving
only logic-gate networks, whose best published sequence result is 5.00 BLEU on 16-token MT. Under
that reading the project was in genuinely unexplored territory with no learnable substrate of
sufficient capacity, which is not a viable path to a chat model.

D0-b preserves the actual objective — no multiplier, tiny RAM, no GPU, local, measured energy —
while allowing the substrate that reaches chat quality. The multiplier constraint stays; the
mathematical ban is what changes.

### How it is enforced

1. `scripts/serving_multiplier_check.py` — a per-function **zero-check** over a `--release` binary:
   a declared serving symbol whose instruction range contains no multiply mnemonic cannot execute
   one. Sound where counting is not.
2. `scripts/energy_per_token.py` — refuses to report a power figure that is not physically
   plausible, which is what stops an unpopulated `CPU Power` field from becoming a published
   result.
3. Every low-bit weight matrix uses a **power-of-two per-row scale**, so `y = (Σ ±x) << shift` and
   the layer is multiplier-free by construction rather than by compiler grace.

### Consequence

The native learned core may use low-bit linear maps. `learner/lowbit.rs` implements the substrate:
ternary weights at 2 bits each with per-row power-of-two scales, trained in float, served in exact
integer arithmetic, with a test asserting the serving path equals the floating reference.

---

## D1 — Falsification before extension

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

Method is Socratic and falsify-first. Before any new geometric mechanism is added,
every existing mechanism must show a **measured** contribution against a matched
control, or be retired from the critical path. A mechanism that cannot be shown to
change held-out bits-per-byte is not carried forward on grounds of elegance,
correctness of its mathematics, or architectural priority.

Corollary: the per-mechanism ablation table (Card P7) is a precondition for
Stages 3–6, not an optional diagnostic.

---

## D2 — How a mechanism's contribution is judged

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: an ablation is a measurement, not a verdict. Magnitude and mechanism class are
judged separately, and a geometric mechanism is not retired on a near-zero delta.**

Owner direction: *"you probably need to weight falsification misses that are very small vs if
the mechanism truly works coherently, and not immediately rule it out because it may be a
better mechanism that maintains our project goal of a geometric language model."*

### Two independent axes

1. **Magnitude**, against an equivalence margin `epsilon` (default **0.01 BPB**, about one
   third of the 0.0316 BPB gap to the matched Kneser-Ney 5-gram, so anything below it cannot
   be decisive for the competitive question):
   `MAJOR+ / MINOR+ / NEGLIGIBLE / MINOR- / MAJOR-`.
   A paired bootstrap CI can exclude zero on a practically meaningless effect. **Passing the
   CI test is not evidence that a mechanism matters, and failing the magnitude test is not
   evidence that it is a dead end.** Statistical and practical significance are reported
   separately and never conflated.

2. **Mechanism class**, declared per mechanism and never inferred from the number:

   | Class | Role | A NEGLIGIBLE delta means |
   |---|---|---|
   | `primary-carrier` | expected to carry predictive signal | a defect to diagnose |
   | `modulator` | expected small, consistent shaping | acceptable |
   | `selector` | routing / candidate selection | **nothing** — BPB ablation is the wrong instrument, because an alternative path substitutes; use decision-flip rate and forced-choice tests |
   | `enabler` | observation channel whose value appears only once composed with a component that does not yet exist | expected, and uninformative about the mechanism |
   | `count-table` | a count statistic, not a learned geometric mechanism | removable on resource cost |

3. **Decision-flip rate** is reported alongside BPB. A mechanism can be BPB-neutral while
   changing a large fraction of decisions, which matters for the *kind* of errors rather
   than their average. Measured examples from the first run: `lattice_coarse` is
   BPB-negligible yet flips **9.5 %** of decisions; `vsa` flips 0.7 %.

### Decision rule

- **Never retire** an `enabler` or `selector` on a near-zero or negative delta. Repair the
  wiring or change the instrument and re-measure.
- Only a `count-table` or a correctly wired `primary-carrier` that is measurably
  net-negative is a removal candidate, and then on resource cost as much as accuracy.
- Record the identified wiring defect with the delta, so a small number is never read as a
  mechanism verdict. Example: the `vsa` delta measures a codebook that is **not** consistent
  with the learned representation, not the VSA mechanism.

### Consequences

- The 2026-09-19 `vsa` and `lanes` results are **not** retirement verdicts. `vsa` is a
  wiring defect with an identified cause; the required action is to learn a consistent
  codebook and re-measure.
- Interaction must be accounted for before any removal: zeroing the JEPA weights also
  changes the fiber the S2 readout consumes, so single-mechanism deltas do not sum. A
  pairwise factorial or Shapley attribution over the mechanism set is the correct
  instrument once the set is stable.


## D3 — Takeover clarification and policy synchronization

Owner: Casey · Confirmed directly in the September 19, 2026 takeover conversation.

The owner reconfirmed that offline training may use matrix products, all recorded decisions are owner decisions, and add/subtract implementations are allowed while geometric routing should remain the focused attempt. D0-b remains adopted; D1 is interpreted together with D2. The stable policy, AGENTS and live entry documents are synchronized through protected delivery. Frozen runtime contracts retain their scope.

This ratification does not turn the historical rationale into a theorem: not every possible learned representation requires a linear map, and a small failed recurrent experiment does not prove local training or an entire mechanism family impossible. See the [source/literature corrections](takeover-review-2026-09-19.md). The arithmetic allowance is an owner engineering decision independent of those broader claims.

---

## D4 — Goal decomposition and the geometry gate

Owner: Casey · Drafted by: the run lead · Date: 2026-09-24 · **Ratified by owner instruction, 2026-09-24 session.**
Full rationale and the staged plan: [direction-decision-2026-09-24.md](direction-decision-2026-09-24.md).

Two goals had been conflated. They are now separated.

- **Goal S — serving arithmetic.** No floating-point matmul and no dense transformer at serving. **Kept.** It is
  theorem-backed for the permitted operator class; implementation discipline is strong. Two defects remain open: the
  served path is dense per token, and no quality-matched whole-path energy measurement exists.
- **Goal R — geometric predictive advantage.** No geometric mechanism has beaten an information- and
  compute-matched ordinary control on any tested task in this repository. Goal R is therefore **re-scoped from an
  assumption to a gated hypothesis**: geometry is retained by default only where already load-bearing (identity,
  addressing, version authority, exact serialization) and as a *candidate* sparse-access structure / parameterisation
  (quaternion/H4 4× under equivariance; exact finite-table closure, which adds inductive bias but not capacity). It
  earns a **causal serving role only by beating a matched ordinary control on the same probe at matched capacity and
  cost.**

This is a change of emphasis, not of goal: the terminal objective (geometry replacing float matmul in a fully
transformerless model) stands as an objective, not a measured result.

## D5 — Per-token parameter sparsity is the terminal serving invariant

Owner: Casey · Drafted by: the run lead · Date: 2026-09-24 · **Ratified by owner instruction, 2026-09-24 session.**

D0-a's distinguishing test — **per-token parameter sparsity** ("if serving touches all entries of a learned weight
store per token, it is a matrix product regardless of the opcode used") — survives D0-b as the **end-state**
contract. D0-b continues to permit bounded ≤4-bit additive/table linear maps as the *interim experimental*
substrate, but a dense per-token map must be **reported as non-compliant with the end state** and cannot be claimed
as the target architecture.

Rationale: dense streaming of a 4-bit matrix is a bandwidth-bound GEMM in disguise; "geometry replaces matmul"
retains technical content only if geometry replaces the **dense access** — routing that selects which learned rows
are read. This also converts Goal R into a cheap, fair, measurable contest (geometric routers — prime/zeta/H4/VSA —
versus ordinary LSH/learned-kNN at matched access budget, judged by top-K decision-equivalence), which has never been
run. The current served artifact is D0-b-compliant and D5-non-compliant (611,814 inspections/step); this is recorded,
not retroactively banned.

## D6 — The target objective is a long-range information probe

Owner: Casey · Drafted by: the run lead · Date: 2026-09-24 · **Ratified by owner instruction, 2026-09-24 session.**

The target objective is a panel on which the **previous two tokens are insufficient** — induction / copy-at-distance
/ agreement at K = 32…512 — where a tuned order-2 count prior is **at chance by construction**. The order-2 count
prior (`C`) is retained as a **standing local control**, not a target: beating a servable memoryless table on the
project's own docs is not a language milestone. Each panel must include a natural-text long-range companion, an
equal-work stateful n-gram control, source-disjoint held-out episodes, and a *verified* (not asserted) at-chance
count control. This replaces the proxy objective identified in
[repo-review-direction-2026-09-23.md](repo-review-direction-2026-09-23.md) §0.3.

## D7 — Adopt the integrated attention-language programme

Owner: Casey · **Ratified by direct owner instruction, September 24, 2026:** “solidify your above plan as the goforward plan (in github so other contributors know it is too) and then proceed with the steps you recommended above”.

The [principal attention programme](principal-attention-plan-2026-09-24.md) is the go-forward implementation roadmap. Its next milestone is **one jointly learned attention-language artifact**, combining exact event history, learned geometric candidate admission, a mutable relative-frame energy, shared sparse state operators and normalized sparse Generate/Copy/Stop output. Develop the ordinary and geometric paths together on the same information, episodes and measured access budget. The roadmap continues through compositional sessions, quality scaling, complete-path laptop qualification and replacement by useful workload.

This decision supersedes the earlier sequencing that deferred geometric implementation behind further isolated metadata/cache gates. It does not change D0-b, D4's requirement for evidence before a geometric-advantage claim, D5's terminal sparsity constraint, or D6's long-range information objective and controls. Geometric code can be developed and trained from the start; promotion still requires the declared comparisons. Focused correctness checks and diagnoses serve the integrated endpoint. They are not successive substitute capability milestones.

The owner also authorized local HDD cleanup. Remove specifically identified regenerable material after checking use and preservation; retain unique source, research, artifacts, negative results, histories and user data. Existing local resource-extension authority remains in force, with complete prospective accounting and no paid external compute.

## D8 — Correct the training method and reference ladder

September 24, 2026. **Implementation decision under the owner's direct request**
to assess the supplied stuck-point review holistically and make warranted
updates. This adopts the supported correction in the
[independent assessment](stuck-point-review-response-2026-09-24.md), not every
threshold, claim or policy suggested in the attachment.

The [canonical plan](project-track.md) now owns a persistent reference → native
joint learner → discretization → bounded admission/transport → integer export
ladder. Stop using successive local auxiliary selector fixes as the main learning
strategy. Preserve A1–A4, exact event/version memory, candidate ownership,
separate admission/ranking, shared typed operators and matched ordinary controls.

Restore #1017 as the pinned bounded language reference/optional offline teacher
and #1014 as the actual historical attention-ablation evidence. Their ordinary
transformer computation remains outside target serving. The previously revealed
test is a regression benchmark; it is not a fresh final holdout. Use a reusable
offline Rust autodiff tool, explicitly connect language loss to context/state,
declare the soft-to-hard estimator and mask, and verify hard-path behavior during
training. A library dependency alone is not a gradient or capability result.

This supersedes D7's current local-training implementation sequence while
preserving its integrated endpoint. D0-b and D4–D6, the native transformerless
goal, geometric evidence requirements, eventual per-token sparsity, exact-memory
contracts and no-hidden-provider boundary remain unchanged. No new Python model,
paid/external compute, automatic six-week freeze or universal token-count theorem
is adopted. Meaningful exposure/seed budgets and numerical gates precede each
comparison and use measured throughput. Model compute and orchestration time are
reported separately. The live state is concise and its prior contents remain
archived. Complete the authorized rung before changing mechanism; consequential
direction changes require evidence, independent review and protected delivery.


## D9 — Prevent experiment loops and preserve the context contract

September 25, 2026. **Owner-directed process correction.** After challenging
whether the 256/256 agreement had been reverted, the owner directed a system to
prevent repeated narrow tests, interacting-variable loops and spending without
progress toward the integrated model. The principal investigator accepts
responsibility for allowing a failed admission experiment to become the next
training objective.

The completed orthant64 campaign retained **256-token training, evaluation and
session limits**, but admitted **at most 64 selected events** from that history.
These are different controls. Both candidates failed the original source-retention
gate. The earlier 64-token training mismatch was not reintroduced; full direct
memory access was deliberately restricted. The later recommendation to train
recent64 is withdrawn before execution. Its diagnostic results and the failed
orthant64 artifacts remain evidence at their original scope; no result becomes
PASS and no diagnostic override is promoted.

**Operational correction:** apply the mandatory
[progress-control rules](agent-execution-policy.md#progress-control--owner-correction-september-25)
through root AGENTS, the machine policy, the research skill and the DeepSeek
workflow. One short existing-issue work card names the integrated deliverable,
blocker, causal change, distinct outcome decisions, fixed conditions, necessary
checks and whole-cycle cost. Repetition requires new causal evidence. Additional
validation must protect a named unresolved risk. A sound negative with no such
new cause parks that branch and returns work to an independent implementation
of the integrated model. A failed command may still be repaired and resumed;
there is no arbitrary global retry quota and no weakening of frozen acceptance.

**Current implementation direction:** preserve the accepted learned-code parents
and full access to all causally available events within the 256-token window.
Build the quantized R4 transport and integer execution bridge for that model,
retaining the matched ordinary arm and measuring state/output drift and actual
loaded generation. Sparse admission is deferred as a separate optimization,
requiring a demonstrated implementation need, a candidate-recall learning
mechanism and a discriminating comparison before another fit. A finite full
256-token reader is bounded, but does not establish scalable sparse attention,
D5 parameter sparsity or efficiency. A full-access numerical implementation can
proceed without first obtaining a 64-candidate retention pass.

This refines D8 sequencing after the completed admission decision. The useful
transformerless geometric-model goal, D0-b arithmetic contract, D4–D6 evidence and
terminal sparsity requirements, exact memory, language learning and matched
ordinary controls remain in force. No new model training is part of this process
correction. The [current execution contract](current-state.md#active-execution-contract)
is authoritative for the next implementation; historical experiment schedules do
not override it.

## D10 — A converted open-weight backbone is the interim chat vehicle; serving arithmetic restated

**Superseded by [D11](#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded) on 2026-09-27.** The text below is the historical record.

Owner: Casey · Drafted by: the lab lead · Date: 2026-09-26 · **Owner decision in the 2026-09-26 lab session, recorded on
branch `claude/blissful-wozniak-girwwq`; it reaches `main` only through protected delivery.** Context:
[lab phase 2](geometric-lab-phase2-2026-09-26.md) §6.

**Scope clarification, September 27:** the owner subsequently answered the
fourth lab's explicit D10 question: **“Keep the native, multiplier-free serving
target.”** That lab does not adopt the converted transformer backbone or hardware
serving-multiplier exceptions below; offline Rust learning remains permitted
under D0-b. The numbered record below preserves the September 26 decision in the
other lab session, not a global revocation or restatement of that session's
instructions. See the [shared owner clarification](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279)
and [current execution direction](current-state.md#fourth-lab-shared-research-and-integration).

1. **Backbone.** A converted open-weight instruct model (SmolLM2, Apache-2.0) is accepted as the chat backbone. The
   rule "no transformer backbone at serving" is restated as: **no floating point and no dense float matmul at serving,
   and learned weight maps execute without a multiplier instruction** (≤4-bit table, add, subtract and shift kernels,
   as in D0-b).
2. **Multipliers.** Delegated by the owner to runtime speed. Hardware integer multiplication is allowed where both
   operands are runtime values or fixed non-learned constants: attention scores, gating, value mixing, normalization,
   RoPE and softmax normalization. The systems review measured table-driven products at about 1.8× the cost of a
   hardware multiply on the same core. Learned weight maps stay multiplier-free, including their scales (shift-add).
3. **SIMD.** Chosen for speed. One small audited SIMD crate may use `unsafe` for NEON table and dot-product kernels.
   The crates that declare `forbid(unsafe_code)` keep it. Every `unsafe` block carries a safety comment and an
   equivalence test against the portable kernel.
4. **D5 is unchanged.** Dense per-token access to a learned weight store remains non-compliant with the end state and
   is reported as such. The converted backbone is the interim chat vehicle. Geometric routing to sparse parameter
   access (memory layers) remains the terminal target, as do the hyperbolic memory and index layers of the phase-2
   roadmap.
5. **Paid compute is not yet authorized.** The owner asked for its size and cost; the answer is in the phase-2 note §6.
6. The frozen R4G1/TLA runtime contract is unaffected.

## D11 — Native, multiplier-free, transformerless serving for every lab; D10 superseded

Owner: Casey · Drafted by: the director (Claude, Lab 1) · Date: 2026-09-27 · **Owner decision in the
2026-09-27 director session, accepted as prompted ("Supersede D10").**

Context:
- the owner's geometric-intelligence brief: no transformer, no matrix multiplication and no floating point in the runtime;
- the owner clarification of 2026-09-27, "Keep the native, multiplier-free serving target" ([#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279));
- the operational rules R1–R5 in [ROADMAP.md](../../ROADMAP.md) §0.

1. **One serving contract for every lab.**
   - **R1:** served computation has no floating point and no `libm`.
   - **R2:** no integer multiply or divide instruction in any served kernel, as D0-b is written. Products of runtime values use product or quarter-square tables, or exact geometric structure (signed permutations, ℤ[φ] shift-add).
   - **R3:** dense per-token access to a learned weight store is not the end state (D5). ≤4-bit additive maps remain labelled interim stepping stones, reported with their per-token parameter reads.
   - **R4:** no transformer backbone. A model whose token mixing is mostly dense all-pairs reads counts as a transformer.
   - **R5:** energy is claimed only when measured.
2. **D10 is superseded.**
   - Its converted open-weight backbone (SmolLM2) is not a served model for any lab. Converted transformers are comparators or offline teachers only.
   - Its hardware-multiplier exception for runtime values is withdrawn.
   - `uor-r4-lut`, `lut-chat` and the D10 export are frozen as non-mission comparators. The audited `unsafe` in `uor-r4-simd` stays confined to that comparator code.
3. **Preserved.**
   - D10-era results keep their exact scope, for example the cycle-4 integer-retention reports measured under D10.
   - The frozen R4G1/TLA contract is unaffected.
   - Offline training may still use floating point and matrix products (D0-b, D3).
4. **Enforcement.**
   - Every mission serving change carries the R1–R2 instruction audit of its release binary and reports per-token parameter reads (R3).
   - The runtime track (T3) extends the audit to every served symbol, including the Lorentz and sampling paths (ROADMAP ruling 9).


## D12 — Gates promote, never kill; reopen geometric candidates; port the native engine's mechanisms; keep a geometric toolbox

Owner: Casey · Drafted by: Lab 1 (Claude main) · Date: 2026-09-28 · **Owner decision in the Lab 1 session, answering four prompts.** Each answer picked the recommended option. The owner also added a synthesis instruction, quoted in item 5.

Context. Near misses on one configuration of mechanisms still under construction had become eliminations of whole mechanism families. Examples:
- **B1's 2I tracking lanes** tracked A5 exactly, then were closed for a +0.069-nat text cost in one seed, against a 0.05 limit, under a kill rule.
- **D2's exact memory** scored 1.000 in distribution, then was marked FAIL because the margin gate missed against a strong control.
- **The geometric sparse index** was ruled "not qualified" in a contest later found to be confounded by magnitude.
- **2I codes** were "rejected" after post-hoc tests only (D6, S1.0b).
- **The native geometric engine** was set aside at 1.68M parameters, and never compared at matched capacity.

1. **Gates promote; they never kill.**
   - A pre-registered gate decides only whether a mechanism enters the served or main-line model now.
   - A miss keeps the mechanism active, with its next diagnosed step recorded.
   - **Parking a mechanism family** needs a written root-cause case and the owner's OK.
   - [D9](#d9--prevent-experiment-loops-and-preserve-the-context-contract) stays: every retry needs a real causal change, not an extra seed or dose.
   - Earlier FAIL, DEAD and RETIRED labels on mechanisms now read **"not yet promoted at that scope"**. The measured numbers keep their exact scope.
   - Kill rules attached to earlier gates are withdrawn, for example B1's "retire the geometric state claim from the serving path".
2. **Reopened as active geometric candidates, each with a named next step** (ROADMAP §2a and §5):
   - **B1 2I tracking lanes:** trained jointly with the trained-in 2I transport (S4). Lab 1.
   - **D2 exact memory:** as I1's store, with G's learned reads. Labs 1 and 2.
   - **The geometric sparse index:** re-tested inside G, with magnitude carried and geometry trained in. Lab 2.
   - **2I (or E8) read codes, trained in, not snapped post hoc:** inside G. Lab 2.
3. **The native engine's mechanisms are ported into the main-line stack's core,** one at a time. Each is trained in and measured at equal capacity:
   - **First:** the 2I transport (S4, now).
   - **Then:** the exact store (I1); prime/UOR-addressed store keys; zeta phase channels.

   One learner is kept. Geometry moves from the edges of the model to its core.
4. **S4 is judged three ways, with no kill:**
   - **Within 0.02 nats of its control:** it becomes the served transport now.
   - **0.02–0.06:** it stays on the geometric track, with named next steps: longer training, an annealed snap, the finer 2I×2I set.
   - **Above 0.06:** diagnose before the next step.

   The 2I transport is never retired on one run.
5. **A geometric toolbox.** The owner's instruction, quoted:

   > leave closed mechanisms that demonstrated real geometric capabilities that are absent from the rest of our mechanisms so that we have their tools and pieces available to fix other problems later - use your context and project understanding synthesis to evaluate them and their novelty

   Lab 1 keeps such mechanisms' code building and documented, with their demonstrated capability, novelty and reuse points, in the [geometric toolbox](geometric-toolbox-2026-09-28.md).

---

## D13 — Record the September 29 two-track owner plan

Authority: the owner's approval recorded in the
[September 29 plan of record](../plans/2026-09-29-path-to-chat.md), delivered by
[#1507](https://github.com/UOR-Foundation/uor-r4/pull/1507). This entry records that
existing ratification; it does not assert that the planned council, experiments,
runner, cleanup or integrations subsequently completed.

The approved scientific programme has two tracks: native geometric chat with
retrieval before scale, and offline geometric conversion/distillation of local
open-weight teachers. Four candidate mechanisms are flock selection, harmonic
features, quaternion/2I transport and lattice weight codes. Track B's source
transformers remain offline teachers/comparators until an actual converted
runtime satisfies D11. The exact persisted conversation log is the durable
memory; learned/prime addresses index it. Context is malleable with separately
declared learning, evaluation, access and state dimensions.

The initial retention rule uses paired geometric/ordinary arms, at least two
seeds, and 0.02 nats or 0.03 accuracy tolerance. Staged measured parity and later
prospective amendments follow D14; prior one-seed evidence retains its original
scope. Instruction qualification uses new wordings of trained instruction types.
Local teacher work and local CPU/GPU compute are allowed within measured host
and cumulative budgets; paid/external compute is not adopted.

The plan authorized GitHub lab boards and two-hour notice for cleanup of
validated regenerable caches/clean merged worktrees, while unique data remains
protected. Its permanent lead, fixed roster, rigid working rules and provisional
assignment order are superseded by D14. Its “kill” terminology terminates a
declared run/advancement decision, not the existence of a mechanism family.
Unfinished work and historical results remain recorded at their exact scope.

## D14 — Durable autonomous labs and correctable governance

Authority: direct owner instructions in the September 29 durable-lab planning
and implementation session. The owner requested GitHub as source of truth,
unlimited joining/returning labs, continuation after token loss, SSD/session
repair, resource hygiene, extended standing goals, expert/adversarial research
and broad council authority to correct working restrictions. Subsequent owner
choices fixed the council and cadences below and authorized implementation and
dispatch. This entry does not claim those mechanisms are already deployed.

1. **GitHub and recoverability.** Protected main holds accepted source, decisions
   and research. Issues/PRs hold live work and proposals; atomic coordination
   records hold lab/task leases. Local memory/search indexes are derived views.
   Every lab publishes completed work, source/evidence identities, limitations,
   costs and the next dependency. No chat is indispensable project memory.
2. **Unlimited peer labs.** Labs may join, disappear and return; Claude, Codex,
   Anti-Gravity and OpenCode/DeepSeek are currently available. No permanent
   provider director, fixed lab cap or special-provider review monopoly remains.
   Task ownership is a renewable claim; machine admission bounds concurrent
   jobs. Older lead/roster statements are preserved as historical instructions.
3. **Council discretion.** Consequential shared architecture, interface,
   promotion and policy changes use three seats, at least two non-author seats,
   and two recorded votes on an identified proposal revision. The council may
   prospectively change working constraints, research methods, schedules,
   evaluation design and priority with evidence, explicit objections, impact,
   budget and rollback. The mission and D11/D5 final-runtime target, honest
   evidence, preservation of unique material and paid/external spending remain
   owner boundaries. No vote retroactively converts a failed result to a pass.
4. **Staged research.** Use source-bound smoke, measured development parity,
   paired candidate gates and separate final qualification. Keep context/access
   contracts explicit. Gates govern promotion at their tested scope; retries
   need new causal evidence. Preserve useful geometric pieces and history.
   Cheap agents handle routine work; independent experts address consequential
   mathematics, learning, systems and evidence questions. No repeated broad
   audit or ceremonial test/review campaign is a default requirement.
5. **Cadence and failover.** Active heartbeats every five minutes, twenty-minute
   claims, recoverable checkpoints every thirty minutes and before quota/context/
   connection boundaries. Expired leases are suspect, not proof a worker died.
   Verify process/job/source identity before adoption; preserve live jobs and
   unique/unpushed work. Blocked labs advance independent ready dependencies.
6. **Resource stewardship.** Use verified durable runner admission, an
   append-only cumulative ledger and actual host/volume identity. Necessary
   local budget extensions remain prospectively recordable under standing
   authority. Two-hour notice plus revalidation precedes removal of manifested
   regenerable caches or clean merged worktrees. Unique material is not routine
   cleanup. Separate disk headroom, RAM/unified memory and paid compute.
7. **Protected delivery.** Exact-head independent review and a delivery
   coordinator precede the merge queue. Shared-policy changes use council
   authority unless they cross an owner boundary. A required server check is
   adopted as the target enforcement mechanism; until administrator setup and
   a real smoke verify it, report procedural enforcement only. Shared-account
   review provenance remains explicit. No direct main, admin bypass or shared
   force-push. Verify delivered changes; close only completed acceptance.
8. **Portable clients.** The [lab protocol](../labs/protocol.md),
   [operations](../labs/operations.md), [plan](../labs/plan-2026-09-29.md) and
   [extended goals](../labs/README.md) implement this charter. Adapters identify
   manual/unavailable/unverified capabilities. A prompt cannot keep an offline
   model reasoning; durable submitted jobs and recoverable records bridge that
   gap. README remains a professional overview rather than an activity stream.

## D15 — Converted students may become served candidates after a D11 audit; runtime and energy claims are measured

**September 30 clarification:** read this historical decision with D17 below.
The owner directly reaffirmed that conversion must remove the transformer
architecture. A numerical/access audit alone does not authorize a transformer
backbone. The original account and measurements below are preserved.

Authority: the owner's answers to three prompts in the Claude-lab session on 30 September 2026, about 01:30 UTC; each answer picked the recommended option. This is an owner decision under D14's boundaries, because it amends D11. It follows D13 and D14 as recorded in [#1521](https://github.com/UOR-Foundation/uor-r4/pull/1521), and refines D13's sentence that Track B's source transformers stay offline teachers and comparators until a converted runtime satisfies D11.

1. **D11 §2 is amended.**
   - A converted open-weight student (Track B) may become a **served candidate** once an audit of its release binary shows D11's R1–R4:
     - no floating point and no integer multiply or divide in served kernels;
     - token mixing that is not mostly dense all-pairs reads;
     - weights of at most 4 bits, read from tables;
     - reported per-token parameter reads.
   - Until that audit passes, it remains a comparator or offline teacher.
   - The rest of D11 is unchanged, and the D10 exception stays withdrawn.
2. **D11 R1–R2 are unchanged, and cost claims are measured.**
   - A runtime claim needs measured ms/token on this M1, against the D10 NEON engine and an ordinary 4-bit model of equal quality. An energy claim needs measured J/token against the same.
   - Analytic byte and operation tables are hypotheses.
   - Closing the serving-kernel gap is a named item for the fidelity and cost lab (Anti-Gravity).
   - R2 is revisited only if a measured, independently re-run gap cannot be brought below 2× with threads and table layout.
   - The only same-artifact figure so far is *self-reported* until re-run: about 5.40 ms/token for D11 against 1.11 for D10 on the S2 model. The D10 figure and the identity of the logits have not been independently re-run.
3. **Energy.** The first J/token (`sudo powermetrics`, run by the owner) is taken at A3's D11 export, next to D10 and an ordinary 4-bit model of equal quality.

## D16 — Working rules from the 29 September council (council authority under D14)

**September 30 correction:** D17 supplies the prospective, unambiguous parity
decision function and coordination authority. The originally merged wording is
retained below; its merge did not establish the missing council/delivery gates.

Authority: proposed by the Claude lab from the adversarial council of 29 September (23 agents: evidence briefs, four proposals, twelve red-team verdicts, a judge and a completeness critic). **These rules take effect when the D14 council records two approving votes from non-author seats on the PR that carries this entry.** They change working rules only, prospectively. The evidence and the recommended experiments are in the [council verdict](council-verdict-2026-09-29.md), which feeds the [lab plan](../labs/plan-2026-09-29.md).

1. **Applying the parity rule so that it always decides.**
   - For each seed or disjoint draw i, **d_i** is the geometric arm's metric minus its paired ordinary arm's metric, with loss in nats or the accuracy drop.
   - **Keep:** the geometric form stays if the mean of the d_i is at most the tolerance (0.02 nats or 0.03 accuracy).
   - **Replace:** the ordinary form takes the main-line slot if every d_i exceeds the tolerance. The geometric form goes to the D12 toolbox, never deleted.
   - **Mixed:** run exactly one more seed or draw, then the mean of the three decides.
   - **Advantage:** a geometric advantage is claimed only if every d_i is below minus the tolerance.
   - For training-free arms, disjoint window draws stand in for seeds.
   - A kill rests on the ordinary arm (for example dot top-k), so that sparsity is never confounded with geometry.
2. **Retrieval instruments must defeat fixed untrained rules before they freeze.** For A1 these are "the latest open value" and "the latest 2-word continuation", each below 0.6 on every gated cell.
3. **Credit for geometry needs the matched ordinary arm.**
   - E8 weight coding is credited only through E8P against RHT-plus-scalar codes at equal bits.
   - Harmonic attention is compared with Taylor-2 at equal feature count, and it enters serving only on a measured win over a byte-matched window.
   - The owner's Lie-group case is tested with an arm that can differ from Taylor-2: a RoPE-plane-aligned or SU(2)/Wigner-D basis.
   - The quaternion pillar gets a decayed or gated recurrent Track B arm.
4. **Records relabelled** (measurements unchanged):
   - **B3 root:** the instrument of `b3-e8-smollm2-mlp/attempt-full-32layers` is disputed. Its float reference scored 9.45 nats/token on SmolLM2, consistent with #1017 token IDs (max 4095) fed to a 49,152-token model. It is not a verdict on E8 until a re-run with a float-NLL validity band.
   - **#1505:** `keys-1` was the originally pre-registered, under-exposed probe, and `keys-9` is the full-exposure run under the amended pre-registration.
   - **The R1 panel:** only its memory score (0/10) is stored.
5. **One shared selector and one Track B host.**
   - `crate::flock` (the OpenCode/DeepSeek lab) serves A1's reads and pointer, B0, B2 and the later D11 port.
   - Reported Track B numbers come from the shared candle host once its parity gate passes. Model-source is its oracle only.

## D17 — Transformer-free conversion, deterministic parity decisions and integration recovery

Authority: the owner's direct September 30 answer in the Codex integration and
storage session: **“Keep the transformer-free requirement; conversion must
remove the transformer architecture.”** The owner also requested continuing
goals for every lab, explicit integration of accumulated branches, and internal
drive cleanup. The working-rule and migration corrections below require the
three-seat, two-non-author D14 council on their exact delivery head. This entry
does not retroactively claim that #1522 passed its pre-merge gates.

1. **The serving architecture remains transformer-free.** A converted student
   is eligible only after the conversion removes the transformer architecture
   and the actual complete runtime satisfies D11/D5 at their adopted scopes.
   Integer arithmetic, table-coded weights, sparse reads or passing a kernel
   audit alone do not establish architectural compliance. D15 is read with this
   explicit owner boundary. Offline teachers/comparators remain permitted;
   capability, arithmetic, parameter-access cost and measured energy require
   their separate evidence. The owner reply above establishes this boundary,
   not every other claim attributed to an earlier conversation.
2. **Parity rule v2 is prospective and has disjoint cases.** Define degradation
   as `d = loss_geometric - loss_ordinary` for loss, or
   `d = accuracy_ordinary - accuracy_geometric` for accuracy. Positive means the
   geometric arm is worse. Freeze two paired seeds (or disjoint draws for a
   training-free comparison), the metric and tolerance before observing them.
   Use tolerance 0.02 nats or 0.03 accuracy unless a new reviewed design changes
   it prospectively. Evaluate the first two differences as follows:
   - Both `d <= tolerance`: retain the geometric candidate at this scope.
   - Both `d > tolerance`: select the ordinary candidate for this scope and
     preserve the geometric candidate in the toolbox with its diagnosed result.
   - Exactly one exceeds tolerance: take one additional predeclared paired
     seed/draw. Retain geometry if the mean of all three differences is
     `<= tolerance`; otherwise select the ordinary candidate at this scope.
   Equality belongs to retention. A nonfinite or unavailable observation is not
   an automatic pass or replace decision; diagnose it under the work card.
   Claim an advantage only when every evaluated paired difference is strictly
   below `-tolerance`. Retention is not advantage or whole-model promotion.
   This changes future decisions only; preserve old verdicts and measurements.
   Examples at tolerance 0.02: `[0.02,0.02]` retains; `[0.021,0.03]` replaces;
   `[0,0.03]` requires a third draw even though its two-draw mean is 0.015;
   adding 0.03 gives mean 0.02 and retains, while adding 0.031 replaces.
3. **Delivery and work ownership are transferable.** The
   [integration queue](../labs/integration-queue.md) routes existing work before
   expansion. Lab boards communicate claims; the deployed atomic state records
   ownership. The currently leased steward manages a shared execution slot;
   Codex is not a permanent allocator. The September 29 council's experiment
   table is a historical recommendation. Live dependencies, causal work cards,
   resource reservations and current reviewed policy determine admission.
4. **One scoped repair of the policy-migration deadlock.** #1522 changed the
   policy files before its required receipts were available. The deployed
   coordinator correctly fences new claims and delivery against the older
   policy. Its migration operation requires a reviewed merged policy, creating
   a circular prerequisite for the corrective PR. For **#1529 only**, the D14
   council may authorize the existing Codex claim for #1520, epoch 4, to deliver
   this documentation correction and complete goals under a retained successor
   work card. Preserve exact head/base, executed documentation checks, two
   non-author reviews, three identified council votes and the incident record.
   After recording those receipts, use GitHub's ordinary protected queue with
   the reviewed head pinned. No direct main push, admin merge, fabricated status,
   force push, source/model change or resource admission is included. Report
   the standard coordinator's policy-currency rejection honestly; the manual
   steward checks every other delivery requirement and records queue/result
   identities. After merge, verify the delivered patch, release the old claim,
   reconcile attempts, and adopt this exact policy merge using its genuine
   Class C receipt before new claims or execution. This one-use exception ends
   on successful migration; later changes use the normal coordinator.

## D18 — One retrieval question for 1–14 October; Track B, cost and memory-port work parked with re-entry conditions

**Owner-approved on 30 September 2026** (menu choice "Approve", recorded on [#820](https://github.com/UOR-Foundation/uor-r4/issues/820)). In force for 1–14 October. Tracked on [#1552](https://github.com/UOR-Foundation/uor-r4/issues/1552).

**Authority.** Proposed by the Claude lab from the 30 September direction review: four evidence briefs, three proposals, capability and cost critiques, and a judge. The review was carried out by the Claude lab alone under the owner's 20:20 UTC ruling that a lab may review its own work. The full review is kept in [direction-review-2026-09-30.md](direction-review-2026-09-30.md).

It changes the programme's priorities for 1–14 October only, prospectively. It changes no measurement and no earlier verdict.

### 1. The question

For these two weeks both labs and the laptop answer one question: **can the native geometric stack bind and copy a value it has never seen, stated earlier in the same conversation, and does that survive D11 serving?**

The question rests on two things:

- **Measured.** Every English memory score is 0/10, and R1's development relation recall is 0/64.
- **Hypothesis.** The §8 conversations fit in 256 tokens. The longest is 137 words against a ≤140-word proxy, and the exact #1017-tokenizer count is pending. If that count fails, this decision is reopened.

### 2. A1, run as pre-registered, with one budget amendment

The pre-registration is #1511, comments 5898059603 and 5902211457, and `a1_gate` in `milestone_world_v2.rs`.

- **Instrument check first.**
  - Run it on the real #1017 tokenizer.
  - `instrument_freeze_ok` must be true: R-recency and R-nlet each below 0.6 on every gated cell.
  - Every MQAR distance bucket must be populated, and every probe item must fit in 256 tokens.
  - The digest is posted before any treatment run.
  - A leaking cell is reported, not gated. There is no second revision of the instrument.
- **Arms.**
  - P: plain Lorentz read.
  - P+ptr: Lorentz pointer.
  - T: the transformer control, in round 1.
  - C: the Dot control of the better arm.
  - Post hoc on fixed weights: a flock sweep and `top:1`.
  - Seed 2 for the best arm and its Dot control. D17 parity rule v2 decides between them.
- **Budget (the amendment).** The budget is an equal step count, fixed from a smoke measured under the same thread and core concurrency the arms will use. The 30-minute wall is a stop only. An arm stopped by the wall is reported and excluded from parity comparisons, with no extension.
- **Gate (unchanged).** On the development-phrasing × development-value cell, MQAR recall ≥0.9 at distances 16, 64 and 200 AND open-relation recall ≥0.9.
- **Kill (unchanged).** Every arm, including T and T at 2× steps, below 0.5 at distance 16. Track A retrieval then moves to the exact log plus the prime sieve, with no third round.

### 3. The four outcomes, decided on 14 October

- **A — retrieval is learned and served.** An arm passes `a1_gate`, and served retrieval is within 0.03 of the model's own float (item 5). The §8 panel is requested only if D11-served development cells reach ≥0.80 in Responsive, Instruction and Relation, the 38-request memory score is ≥8/10, and greedy streams are identical across a fresh-process reload.
- **B — scale or phrasing.** Retrieval passes on the pure-retrieval cell (train phrasing × development value), but Responsive or Instruction stays below 0.80, or development × development relation is below 0.6. The owner then chooses among size (10–30M), Metal coverage of the Lorentz and pointer ops, and teacher data.
- **C — read defect.** T beats the best stack arm by ≥0.2 at distance 64 after the seed-2 pair. The parity rule is then applied to the read.
- **D — no learned retrieval at this scale, T included.** The item 2 kill applies. Week 2 produces a design memo for the log-plus-sieve route, and no new training.

A result on the development cells is a phrasing-transfer proxy. Only the sealed §8 panel qualifies an artifact.

### 4. Data unlock and the conditional 7M fit

- **`PrefixPolicy::TruncatedPrefix{keep_last}`.**
  - It must reproduce FullPrefix exactly: 14,826 eligible of 129,486 runs; 1,048,098 of 56,650,286 response tokens.
  - There are no mid-response windows.
  - It is adopted only if, on two seeds, the truncated arm is ≥0.02 nats better than FullPrefix on the 161-response panel AND does not regress the relation and abstention cells.
- **The 7M fit, only on outcome A of A1.**
  - Setup: the S4 arm A trunk, the A1 winner and the item 4 data policy, with M-world v2 at ≥30% of tokens.
  - The work card fixes the learning rate and the rule for a guard failure before the run. R1's lr 1e-3 already failed the 2.597954 guard. A guard failure means the model is not the §8 candidate; the retrieval readouts are still reported.
  - The fit stops after 12M new positions unless development Instruction is ≥0.40 and pure retrieval is ≥0.5. Otherwise the result is outcome B.
  - One continuation, to 10 tokens per parameter cumulative, is allowed only under the same rule.

### 5. Serving

- **Porting the pointer.** Only the trained A1 pointer is ported into `IntegerStackSession`. #1540's head-0 boost is a different mechanism.
- **Gate.**
  - The zero-multiply audit stays FULL PASS, with the pointer inside the audited roots.
  - D11 still equals D10 on the non-pointer path.
  - D11-served MQAR and open-relation recall are within 0.03 of the model's own training forward pass. D10 refuses pointer models, so it is not the comparator here.
  - Served recall more than 0.10 below float makes QAT with the pointer the next untested step. It is not run inside this window.
- **Flock.** Flock is recorded for parity only and is not ported for serving at 256 tokens. #1528 selects over scores the caller has already formed, so it saves no bytes there (derived).
- **Eviction, on outcome A only.** A sink-plus-sliding-window eviction policy is added to the D11 session, because §8 requires a declared eviction policy.
- **Energy.** The D15 item 3 J/token capture, with its comparators, is taken at that export.

### 6. Parked, not killed (D12)

Each item keeps its records and its re-entry condition.

- **Track B conversion:** the #1518 chase, B0, B1, B2, harmonic arms and the B3 rerun. It re-enters on owner direction.
  - #1518's FAIL record stays immutable.
  - A prospective successor host gate is defined here but not run: dense NLL on the D3 held-out split (596 articles, 71,714 targets) within a band of the independent referee's 3.8425 bits/token, frozen before any run (±0.01 proposed), plus a flock identity arm within 1e-5 nats of dense.
  - It amends D16 item 5 only on owner approval.
  - A B3 rerun first needs a one-matrix exhaustive-encoder check.
- **QAT and codec reruns on the old lineages.** They re-enter on item 5's kill.
- **Kernel-speed, K-cost and energy work,** except the capture in item 5. Re-entry: a served model answers at least half of the development turns, or a D5 design exists.
- **Memory port, AERM, G v2 probes, I1 integration.** Re-entry: outcome D, or §8 passes.
- **A2 beyond 256 tokens, age buckets and ring buffer.** Re-entry: the §8 token count fails, or §8 passes.
- **Teacher T3/T4 generation.** Only the teacher-ceiling measurement may run, scored by the frozen M-world oracle, and only if R1-X shows the limit is phrasing.

### 7. Claims

This line claims no chat, no geometric advantage and no runtime or energy saving.

- **Measured.** D11 is about 3–5× slower than D10 NEON (self-reported), reads 100% of its weights per token, and has no valid J/token.
- **The runtime-cost thesis is not tested by this decision.** It needs a D5 selected-weight-access mechanism, and no such mechanism is built.

### 8. Process

- One immutable work card per run, posted before launch.
- A wall is a stop, not a budget.
- Reviews follow the owner's 20:20 rule: a recorded review at the exact head, and self-review is allowed.
- A PR merges only after its exact-head compile and test run has completed and passed.
- STATUS.md and current-state.md are corrected when this entry is delivered, and then at each outcome.

### 9. Owner rulings recorded with the approval (30 September)

- **Labs.** Only the Claude lab and the OpenCode/DeepSeek lab remain. Codex, Kimi and Anti-Gravity were removed by the owner; their merged work keeps its scope.
- **Reviews.** A lab may review its own PR. Each review is recorded at the exact head with its scope and evidence, and the merge follows the exact-head compile and test run.
- **§8 token count.** The owner runs a count-only #1017-tokenizer check on the sealed panel. The Claude lab supplies the tool, tested only on a non-panel fixture. No lab reads the panel. A count above 256 reopens item 1.
- **Cost design.** The DeepSeek lab writes a one-day, no-compute design memo for a D5 selected-weight-access mechanism (prime or semiprime routing, or another candidate) as input to the 14 October decision. Nothing is run.
- **#1546.** The low-bit trainers' Adam guard is not needed by the current stack, so it is **not restored**. The four legacy `uor-r4-core` learner tests that depended on it are marked ignored, with a pointer to #1546 and this entry, so that `main`'s test signal reflects the current stack. The tests and their records are kept.

### 10. Outcome — D, reached on 1 October (owner-confirmed)

- **The kill in item 2 was met.** Every A1 arm scored below 0.5 MQAR recall at distance 16 on the development cell, the transformer control at 2× steps (5,180) included, which scored 0. The owner confirmed outcome D on 1 October; the record and roots are on #1552.
- **Measured reading.**
  - Without a pointer head, no read retrieves (MQAR 1/109).
  - Pointer arms score 0.28–0.44 MQAR, falling as about 1/N. That matches the untrained "most recent value" rule (0.36): they copy recency and do not bind a query to its key.
  - Lorentz-vs-Dot read parity is undecided at two seeds. The D18 Lorentz-scored pointer did not run.
- **Consequences.**
  - No new A1 training.
  - The exact-log plus prime-sieve design memo starts at once rather than in week 2.
  - The 7M fit (item 4) and the pointer's D11 port (item 5) do not run.
  - The pointer head stays in the toolbox (D12).
- **Item 1's premise also failed.** The owner-run §8 count needs up to 317 positions with a 32-token reply budget, against 256. The owner kept this decision and re-entered A2 as source work (#1557).
