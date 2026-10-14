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
  wiring or change the instrument and re-measure. **Bounded by [D20](#d20--geometry-stays-first-the-no-loss-rule-is-suspended-only-after-exhaustion-so-a-measurement-can-decide),
  which specifies the exhaustion precondition under which this clause and the next are suspended so
  that one pre-registered contest can decide.**
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
correction. The [current execution contract](../history/current-state-2026-09-25-to-2026-10-02.md#active-execution-contract)
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
and [current execution direction](../history/current-state-2026-09-25-to-2026-10-02.md#fourth-lab-shared-research-and-integration).

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
   - **Bounded by [D20](#d20--geometry-stays-first-the-no-loss-rule-is-suspended-only-after-exhaustion-so-a-measurement-can-decide)**:
     after the exhaustion precondition is met and recorded, this clause and the FAIL relabelling below
     are suspended for one pre-registered geometry-vs-matched-control contest, and the no-loss rule
     resumes on its report.
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

## D19 — Grounded conversation and durable memory first

**Authority, October 1:** the owner requested: “Please solidify your plan as the
active plan for the project and restructure the github roadmap and issues list
as you recommend”, protect mechanisms from poorly planned/executed tests,
“always merge your prs”, and proceed autonomously with Claude and
OpenCode–DeepSeek working concurrently. The owner's selected first product
priority is **“Grounded conversation and durable memory first.”** The retained
public work card is [#1563](https://github.com/UOR-Foundation/uor-r4/issues/1563);
the [programme coordination record](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5925730048)
publishes the authorization and division of work. This entry records that
direction; it does not attribute every implementation detail to an owner quote.

1. **One active plan.** [project-track.md](project-track.md) now owns the active
   scientific sequence and capability obligations; current-state owns changing
   results; ROADMAP/STATUS/lab entry are navigation. The September 29 schedules
   and D18's fortnight-only next-action/roster restrictions are superseded where
   they conflict. D18 outcome D, A1's stopped training, original failed gates
   and Track B's parked disposition remain. No sealed panel is opened.
2. **Next useful mechanism.** Complete a learned saved compiler/store/emitter
   path, then durable grounded conversation on the same model. Retain geometric
   query/write alignment, compositional state and selected-access discovery as
   shared-interface research. D11/D0-b/D5, Rust-native implementation, exact
   identity and typed geometry are unchanged. Full alpha still requires useful
   coding/reasoning; frontier capability remains an objective.
3. **Admissibility.** The [October policy](mechanism-admissibility-2026-10.md)
   makes D12/D17/D9 operational: assess the learning opportunity, instrument,
   information, control and consumer before interpreting a negative. A failed
   promotion stays failed. A materially changed successor may re-enter with a
   causal prediction and bounded work card; no automatic family retirement or
   blind retry follows. D17v2's exact prospective retention rule remains in
   force. Existing E3 thresholds and §8 acceptance are not weakened.
4. **Development integration is not promotion.** E3 v11 and a small lexical
   compiler may enter a labelled end-to-end diagnostic to localize failures
   before reaching their old component gate. This is consistent with the
   concurrent owner-authorized E4 v12 on #1552. Source/category/answer oracles
   cannot stand in for predicted-input capability. Changes to semantic labels
   must preserve Assert/Correct/reassertion/query behavior or explicitly declare
   a narrower development scope, not silently discard temporal obligations.
5. **Coordination and delivery.** Codex is reauthorized by this direct request
   alongside Claude and OpenCode–DeepSeek. Kimi/Anti-Gravity remain historical;
   this does not restart clients or transfer live source/jobs. Live issue claims
   and verified workers decide ownership. The current owner rule from D18 §8–9
   remains: recorded exact-head review (self-review allowed), actual relevant
   compile/tests, then protected merge; delivery-evidence is advisory. This
   owner-authorized plan gets adversarial specialist review with authorship
   disclosed; it does not fabricate non-author council votes. Future ordinary
   decisions follow the shared protocol subject to the newer owner rules.
6. **Cadence and resources.** Keep GitHub updates, five-minute active heartbeat
   and thirty-minute recoverable checkpoints; use the documented manual
   coordination fallback when the deployed coordinator/policy is unverified.
   Owner-directed FIFO starts within the aggregate eight-thread/11 GiB envelope,
   cumulative charges, physical storage/RAM checks and prospective local
   extensions remain. Small bounded checks do not require a heavy-job slot;
   preserve existing runs' declared reservations and verified host limits. No paid compute, unique-material
   deletion, protected-branch bypass or new universal test/timer regime is
   authorized. A PR's passing transport statuses are not test execution.
7. **Issue structure.** Reopen transferred capability tracking where needed and
   give each obligation a current parent, dependencies, explicit acceptance and
   ready/blocked/parked status. Preserve original bodies/results in history.
   Do not close a capability because a component or its tracking migration
   completed. Close a delivered scoped task only against its complete evidence.

**Alternatives and objections.** More frozen E3 fits do not repair missing spans,
persistence or autoregressive integration; a whole new learner would duplicate
the exact-store work. A fixed ten-relation compiler risks becoming a scripted
assistant, so it is the first integration boundary only, with explicit later
scope/compositional/language obligations. Geometry parity is not superiority;
exact storage is not language understanding. Preserve these objections in the
acceptance and use measured consumer behavior to choose successors.

**Effect, review and rollback.** Effective on protected delivery of this change;
exact-head reviews/checks and merge identity are recorded on its PR/#1563.
The documentation task adds no model compute or new numerical result. Reverting
this scheduling change would restore the prior ordering, not invalidate new
evidence, delete artifacts or restore obsolete serving exceptions. Any later
working change records the affected interface, costs and next discriminator
prospectively. The existing sealed milestone stays fixed unless the owner
explicitly changes it before a new candidate.

---

## D20 — Geometry stays first; the no-loss rule is suspended only after exhaustion, so a measurement can decide

Owner: Casey · Drafted by: DeepSeek lab (agent) · Date: 2026-10-05 · **Owner decision, 2026-10-05.**

Owner direction, quoted: *"move 1, please make the change, but document it clearly that geometric
mechanisms will need to be prioritized and novel invention of geometric mechanisms will likely be
needed, but if we have exhausted everything, we can suspend the rule until we have something
working so we can measure and then we build the fix."*

### 1. Priority is reaffirmed, and the expectation is raised

- **Geometric mechanisms stay first.** The objective is unchanged: a geometric language model,
  useful conversation/memory and coding/reasoning, ultimately frontier capability on consumer
  M1-class laptops. D11/D5 serving, R4/S3/H4 state and transport, exact `Z[phi]`, typed
  paired-H4/icosian geometry and UOR identity remain the declared mechanisms.
- **Novel invention is expected, not merely tolerated.** D2's `enabler` class exists precisely
  because a geometric mechanism's value may appear only once composed with a component that does
  not yet exist. **Inventing that component is project work, not a diagnostic excuse.** A mechanism
  parked as `enabler` carries an obligation to name the missing component and attempt to build it.
  The measured record (`docs/research/retired-mechanisms-catalogue-2026-10-01.md` Part 0) shows the
  opposite happened: items marked retired between 2026-09-19 and 2026-09-28 were **un-retired by D12
  before any re-measurement**, and that gap — *"un-retired on policy, never re-tested"* — is named
  there as the largest single risk to the programme. **D20 closes that gap by making re-measurement
  the price of retention.**

### 2. What "exhausted everything" means — the precondition, stated objectively

Suspension under §3 requires **all five** of the following, recorded on the owning issue. Any one
missing means the precondition is not met and the rule stays in force.

1. **Every** mechanism in the barrier-assessment ledger (`synthesis.md` §3: ACTIVE, SAVED OPTION,
   DORMANT-BUT-PROMISING) has a result from a **parameter- and compute-matched ordinary control** —
   not a component result, not a non-learned-rule comparison, not an unreached code path.
2. Every such result carries **≥3 seeds**. (The assessment's own rule; its completeness critique
   found it violated at 2 seeds.)
3. Every candidate's wiring is **verified reached in a real run** — a firing counter or equivalent —
   so that "no effect" cannot mean "never executed". The `unless_query` precedence guard that was
   written into an unreached policy arm, and the closed-label fallback that fired **0 times in 60
   conversations**, are the recorded examples of why this is a precondition and not a formality.
4. Each non-promoted mechanism has a **written root-cause case naming what would change the answer**,
   not a wiring attribution alone.
5. **Novel geometric candidates have been invented and tested to the same standard.** This is the
   owner's explicit requirement: exhaustion is not "every existing mechanism failed", it is
   "we built the missing components and invented the new geometric mechanisms, and they were
   measured". A programme that has not attempted invention has not exhausted anything.

### 3. The suspension — bounded, and triggered only by §2

Once §2 is recorded complete, **and only then**, these three clauses are suspended:

- `DECISIONS.md:158` (D2) — the near-zero-delta exemption.
- `DECISIONS.md:192` (D2) — the *negative*-delta exemption.
- `DECISIONS.md:438/443` (D12) — "gates never kill", and the FAIL → "not yet promoted at that scope"
  relabelling.

**What the suspension is FOR — and it is not elimination.** With those clauses in force, a measured
gap can be answered by re-attributing it to wiring and moving on: D2:192 says *"repair the wiring or
change the instrument and re-measure"*, and `AGENTS.md` adds that *"a failed experiment does not
demote the whole architecture."* **The effect is that a gap produces no obligation.** Suspending them
makes a gap **BINDING**: it must yield a **named missing component and a repair attempt**, recorded,
rather than being a non-event that quietly defers.

**Purpose, in the owner's words: suspend "until we have something working so we can measure and then
we build the fix."** The suspension exists to make the measurement **actionable** — it converts a
null into a repair plan. **It does not, and cannot, retire a geometric option** (§4). It lapses on
report, and §1 continues to govern throughout.

**Nothing else is suspended.** D11/D5 serving constraints, evidence discipline, ≥3-seed rules,
resource admission, preservation of negative results and protected delivery all remain in force.

### 4. The measurement — and the rule that a comparison may NEVER rule out geometry

**Owner correction, 2026-10-05:** *"comparison ruling out is AWFUL for real research so do not let the
test claiming a transformer did better rule out the geometric options... we need to make the geometry
work first, then we can tune it to beat the transformers."*

**A comparison is a DIAGNOSTIC, not a verdict.** The owner's research order is binding:

1. **Make the geometry work.** A matched-control comparison is run to find **where** a geometric
   mechanism is incomplete, not to decide whether it survives.
2. **Then tune it to beat the transformers.** The competitive question becomes meaningful only once
   the mechanism functions. Asked earlier, it measures the missing component rather than the ceiling.

**Therefore, explicitly and without exception:**

> **No result from this or any other comparison retires, parks, demotes or deprioritizes a geometric
> option.** A geometric mechanism that loses a matched-control comparison is **incomplete**, and the
> correct response is to identify and build the missing component and re-measure — never to eliminate
> it. Prior art agrees: *"comparisons that pit convolution-free cells against a convolution-equipped
> Mamba measure the missing convolution, not the recurrence."*

This supersedes the drafting agent's earlier proposal of "a tie is a loss → RETIRED", which was
**wrong** and would have converted the project's own diagnostic instrument into an elimination
machine. The owner rejected it.

**The instrument.** One contest on `mqar-bench`, with the change it requires: `enum ArmSpec` has a
single variant and `arch: StackArch::Geometric` is hardcoded (`mqar-bench.rs:449-465, :570`), so the
bench has **never** compared the geometric stack to a parameter-matched ordinary model — only to
non-learned rules. Equal parameters, equal tokens, equal data, **≥3 seeds**.

**The DIAGNOSTIC bands** (replacing the rejected elimination rules):

```
d = geometric - matched ordinary control, held-out nats

d >= 0            -> the mechanism WORKS at this scope. Promote it, record the effect, proceed
                     to the competitive tuning stage.
d < 0 (a gap)     -> the mechanism is INCOMPLETE. The gap is a LOCALISER, not a verdict.
                     Required output: the missing component, named, with the evidence that
                     identifies it, and the next attempt to build it.
                     RETIREMENT IS NOT AN AVAILABLE OUTCOME.
```

**What the gap is used for.** A gap localises *where* to look — by split, by cell, by layer, by
condition — exactly as `first_piece 0.269 / full 0.032 / full|first 0.117` localised the transport
failure to the read-out rather than the address. **That is the entire purpose of running a
comparison at this stage.** The output of a losing run is a **repair plan**, never a casualty list.

### 5. The exit condition — "something working", in the owner's words

The owner's purpose for the suspension: *"until we have something working so we can measure and then
we build the fix."* **The exit is a WORKING GEOMETRIC MECHANISM, not a competitive win.**

The suspension lapses when either holds:

- **A geometric mechanism works** on the target task — it functions, measurably, on its own terms.
  **The no-loss rule resumes, the working mechanism is carried forward, and the competitive tuning
  stage begins**: only now does beating the matched control become the question.
- **A repair plan is recorded but the fix cannot be built within the bounded window** — then the
  suspension lapses with the missing component named, the mechanism stays a **live candidate** (D12),
  and the next attempt is scheduled. **This is a pause, not a retirement.**

**"Something working" is a positive result, and it is the only thing that ends the suspension
favourably.** If the geometry does not yet work, the answer is to build the missing component — not
to stop, and not to rule the mechanism out.

### 6. Consequences

- **D2's escape hatch is narrowed, not removed.** `enabler` and `selector` retain their meaning; what
  changes is that an `enabler` claim now requires the missing component to be **attempted**, and a
  `selector` claim requires its own instrument (decision-flip rate), per D2's existing table.
- **D12's "never kill" is unchanged in substance and is now reinforced.** A comparison cannot kill a
  geometric mechanism: a losing comparison yields a **repair plan**. Parking a mechanism still needs
  the written root-cause case and the owner's OK, exactly as D12 requires.
- **`formal_vocabulary.md:99`** — the `E8 = H4 x H4` row's *"architectural load-bearing is assumed;
  held-out advantage remains unproven"* — is now **scheduled for test under §4** rather than standing
  as a permanent exemption. The vocabulary's distinction between structural priority and measured
  advantage stays normative.
- **Nothing here changes the mission, the serving contract, or any historical result's scope.**

**Effect, review and rollback.** Effective on protected delivery of this change. The documentation
task adds no model compute and no new numerical result. Reverting D20 would restore the prior rules
and would not invalidate new evidence or delete artifacts. The suspension is bounded by §3, triggered
by §2 and ended by §5; it is not a standing repeal.

Refs #820, #1552, #1746.

## D21 — Three negatives on one line force a pivot; DeepSeek trains the pointer fix, Codex stops the constraint line

Owner: Casey · Drafted by: Claude lab (agent) · Date: 2026-10-09 · **Owner decision, 2026-10-09** (chosen options recorded on the tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028)).

**Why.** In the eight hours to 03:00 UTC on 10 October, 33 DeepSeek PRs and 15 Codex PRs merged, and neither milestone's headline number moved:
- M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029): open reply panel 43/232, v5 memory 10/40.
- M2 [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030): 8/512, unchanged since 7 October.

Each PR was honest and recorded. But each line kept producing a narrower next step whatever the result, which the progress-control rule (D9, AGENTS.md) is meant to stop.

### 1. The three-negatives rule (all labs, standing)

- **Trigger:** after **three consecutive merged PRs on one line of work** that do not move the owning milestone's headline acceptance number, the lab's next action on that line is a **pivot card** on the milestone issue, before any further work on it.
- **What the card does:** it chooses one of two things.
  - **Stop:** archive the line with its negatives.
  - **One decisive run:** a run that changes the model and is scored on the milestone's acceptance panel, with a pre-registered bar and its cost.
- **What doesn't count:**
  - Diagnosis, instrumentation, readings, plans and attribution PRs count toward the three. They do not reset it.
  - Fixing a crash or a broken measurement only extends the count.
- **What resets the count:** only a change in the headline number.
- **Headline numbers:** M1 reply panel and v4/v5 memory panel scores and served BPB; M2 complete correct replies on the 512 panel; for other milestones, the first acceptance item.

### 2. DeepSeek (M1): train the pointer fix

The diagnosis is done.
- #2123, #2127, #2128 and #2133 measured that the copy pointer attends the sentence frame and never the slot that holds the value.
- The plan in #2129 names the change: label the value's run as `bound` and distractors as `competing` in `ReadSupervisionGroup`, then train with `gate_supervised_loss`.
- Its step 1, the CPU check of `p_copy` at the value's positions, is #2133.

The next DeepSeek piece on this line is that supervised fine-tune:
- **Training data:** generated training dialogues (`milestone_world_v2`), **never the frozen v5 rows**.
- **Scoring:** the frozen v5 memory panel and the open reply panel, with the bar pre-registered on #2029 and the cost charged.
- **Delivery:** one PR with the result. No further read-only PRs on the pointer line come before it.

### 3. Codex (M2): stop the protected / discrete-constructor line

- **Archive the line as negative:** protected joint learning, Prefix transactions, discrete feedback, direct legal construction and constraint-solver basis diagnosis (#2079, #2084, #2088, #2101, #2109, #2117, #2121, #2125, #2132). The archive keeps its implementations and evidence.
- **Next M2 piece:** it must aim directly at the 8/512 number. For example: ordinary gradient training of reply completion on the saved native learner, scored on the frozen 512 panel, with a pre-registered bar and a stop rule.

### 4. Scope

D21 changes no acceptance criterion, serving contract or spending cap. It is prospective: earlier records keep their scope.

## D22 — A negative closes a configuration, never a mechanism; five closures reopened

Owner: Casey · Drafted by: Claude lab (agent) · Date: 2026-10-10 · **Owner decision, 2026-10-10** (chosen in the Claude lab session: "D22 + reopen orders").

**Why.** An audit of every closure since D21 (DeepSeek M1, Codex M2, Claude M4) found that the records were honest and scoped, but the test design and the D21 count closed novel mechanisms on tests that could not judge them:
- **Codex:** the protected/discrete constructor was **never scored on the panel**. It stopped on solver numerics (LU pivots 9e-11 and 7e-12 against an absolute 1e-10).
- **Codex:** native Context and prototype learning was judged on 24 rows with the loss that later proved to be the fault.
- **DeepSeek:** the token-identity pointer was keyed on the wrong context, which is why its hit rate was 0.003, and the line was archived on that run.
- **DeepSeek:** read-binding supervision met its own target (bound mass 0.73 → 0.90, wrong values 17 → 12) and was counted as a negative because a 40-row panel moved one row.
- **DeepSeek:** a 10 % recall dose doubled memory at no measurable cost and was rejected by one row.
- **Claude:** the learned rank table closed 54 % of its gap and was prescribed a stop.
- **Meanwhile,** what was kept leaned to ordinary levers: a frozen-upstream adapter reached 437/512 on the development panel and 0/128 on fresh rows.

D21 stays. These rules amend how it counts and what may be closed.

### 1. What a negative closes

- **A negative result closes the tested configuration only:** the recipe, dose, window, key, objective, base and steps. It never closes the mechanism. Records state the configuration as the scope.
- **Stopping a novel project mechanism** (prime/zeta/R4/H4/icosian geometry, exact addressed memory, pointer/copy, read binding, flock or softmax-free reads, protected or discrete constructors, native geometric operators) needs an **owner decision** posted on the milestone. The pivot card proposes it; it does not decide it.
- **A tooling, numerical or infrastructure failure** (a solver tolerance, a crash, a missing CLI field) is a **blocker to fix**, not a negative. It does not count toward the three.

### 2. What resets the count

- A run that **meets its own pre-registered mechanism target** (for example bound mass, wrong-value failures, transfer) **without a significant headline loss** is **KEEP and iterate**. It resets the line's count, as a headline move does.
- A result that **improves the headline but misses its bar** (for example within the bar's noise, or more than halfway to it) is kept as the line's new base. It is not discarded.

### 3. Minimum test power

- Every pre-registration states the panel's noise (seed-to-seed spread or a binomial standard error). **No bar is narrower than that noise; no single-row bars.**
- A decisive run uses **≥ 2 seeds**, or a panel of **≥ 200 fresh rows**, or both.
- A mechanism is **trained in, not bolted on**: from pretraining, or a full fine-tune long enough for the base to adapt. A 2,000-step add-on to a base never trained with the mechanism is a probe, not a decisive run.
- **Ordinary levers** (data dose, learning rate, rows, updates, seeds) **cannot serve as a novel line's decisive run.**

### 4. Transfer before KEEP

- Every KEEP that raises a development-panel number also reports a **fresh, held-out transfer** number. On M2 that is fresh rows of new combinations, as in #2164. A development gain with zero transfer is recorded as **panel fitting**, not a milestone move.

### 5. Reopened (orders posted on the milestones)

- **Codex, M2 #2030:**
  - **(a) The protected/discrete constructor (D21 §3 reversed).** Fix the pivot rule (scale-aware, or an exact refactor of the two failing bases) and score at least one legal displacement on the frozen 512 panel and the fresh 128.
  - **(b) Joint Context + prototype learning.** Train from the saved parent with the objective that works (pooled ranking), on all 512 rows, ≥ 2 seeds, scored on the 512 and the fresh 128.
- **DeepSeek, M1 #2029:**
  - **(c) Exact addressed memory.** Add the memory fields to `DialogueSettings` (source work, not an owner blocker) and train a memory-equipped stack in, not bolted on.
  - **(d) The identity pointer, with a real key** (the slot or entity address, for example a prime/UOR address, not the six tokens around the value), trained from the start of the dialogue fine-tune with ≥ 2 seeds.
  - **(e) Read-binding supervision, powered** (≥ 3 seeds or ≥ 200 fresh rows, wrong-value rate as the primary metric).
  - **(f) Adopt the 10 % recall dose (#2151) as the M1 base.**
- **Claude, M4 #2032:** the softmax-free read continues on its new line (arm W, wide learned flock). A next step on it fixes the training mismatch: a straight-through gradient that matches the rank forward, and hyperparameters tuned for the read.

## D23 - No softmax anywhere in the active project

**Authority, October 12:** Casey: "no softmax anywhere ever no matter what".
The direction is relayed on [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6124029555).

- Softmax is prohibited in training, serving, sampling, objectives, auxiliary
  losses, teachers and new comparator runs. No temporary-bootstrap, training-only
  or runtime-only exception exists. This supersedes earlier unrestricted-training
  language and permissions for softmax reference runs.
- Log-softmax, implicit softmax inside cross-entropy and mathematically equivalent
  normalized-exponential implementations, including table-based or renamed forms,
  do not evade the prohibition. Inspect the complete executed output/objective/
  gradient path before claiming compliance; a softmax-free reader is insufficient.
- This decision does not prohibit every unrelated scalar exponential or select a
  substitute learning law. A replacement must retain the geometric architecture,
  coherent shared learning and identity/orientation contracts, with prospective
  TEST FITNESS before model compute. Existing owner compute holds remain.
- Historical source, artifacts and measurements remain preserved as inactive
  evidence, not authorized execution. Earlier measured numbers are not erased.
  Existing code is not certified compliant by adopting this policy.
- Codex #2246 sampled producer uses softmax and is **BLOCKED** pending a compliant
  replacement. Its focused reference tests were implementation evidence only;
  no training or CUDA execution was performed. This is not a mechanism negative,
  model success or D21/D22 count reset.
- NVRTC is a compiler, not a mathematical operator. Compliance depends on what
  the complete program computes, irrespective of compilation backend.

The [delivery record](../labs/no-softmax-2026-10-12/README.md) records scope and
limitations. This policy is mandatory agent governance, not a comprehensive
Rust/CLI interlock or a claim of completed project-wide softmax removal.

## D24 - No CPU model compute, and no GPU on non-decisive work

**Authority, October 12:** Casey, restating the rule in the same session:
"no softmax at all anywhere", "no CPU runs at all - rewrite for GPU", "do not
waste GPU runs on bullshit that doesn't matter". Recorded with D23 as one
governance block; D23 carries the softmax prohibition, this carries the rest.

- **Model compute never runs on the CPU.** Training, evaluation, and every part of
  the forward that scoring, normalization or the value mix touches are GPU work. A
  path that silently falls back to the host - as a routed read does today when
  `cuda_covered()` is false - is a **defect**: fix it with a kernel. It is not a
  measured cost of the mechanism and not a result to report.
- **A host-fallback execution is not mechanism evidence.** Both the arithmetic and
  the step time are wrong, so a fit produced that way may be cited only as a scoped
  reading of that flawed configuration. Existing numbers are not erased; they are
  not evidence for the mechanism.
- **GPU time is spent only where it can change a decision.** A configuration these
  rules already exclude, or a comparison whose baseline they forbid, does not get a
  run. Such a campaign is cancelled, not extended.
- **Every prohibited softmax site is replaced by a designed geometric mechanism** -
  exact `Z[phi]`/zeta-phase interference, typed H4/icosian pairing, Lorentz or
  address geometry, or exact hard selection by address - with a prospective TEST
  FITNESS review before model compute, per D23.
- **Interlock.** `uor-harness ratchet d23` (Rust, #2346; supersedes the retired
  `scripts/check_no_softmax.py`) ratchets the executable tree against
  `scripts/no-softmax-baseline.json`: a softmax site may not be added, and the
  baseline may only decrease. D23 recorded that no interlock existed; this is it.
  It is not a claim that project-wide removal is complete.
- **Cancelled under this rule (recorded):** the M1 #2029 invariant-addressed router
  campaign stopped at 2 of 3 arm-A seeds. Arms A and C normalize over the kept
  candidates with softmax, arm B *is* the full softmax read, and the routed read
  executes host-side - all three violate D23/D24. The retained reading is
  `icloud:UOR-R4/results/deepseek/router-abandoned-2026-10-12.tar`
  (md5 f411b7d1870afdabcd2195ce27a8c157), with each fit's report.json, config.json,
  manifest and full log. Model weights were not retained: the configuration is
  excluded, and the pod volume was non-canonical with no canonical companion. This
  is not a mechanism negative and does not count under D21/D22.

Historical records keep their text as evidence; they are not rewritten.

## D25 - Geometric operator phase and current lab direction

**Authority, October 12:** the owner's decisions recorded on
[#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6133330776),
amended by the [late owner clarification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135511346),
and the owner's explicit instruction in Codex chat to reconcile the
instructions and tracker, fix the operator plan, protected-merge it, keep the
training hold and preserve everyone's work. The reconciliation claim is
[#2029 comment 6135490872](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135490872).

- D23's prohibition also includes softplus, sigmoid and tanh: no
  exponential-family squashing in any executed model, training, gradient,
  output, sampling, teacher or comparator path, including equivalent renamed
  or table forms. The SwiGLU sigmoid exception is closed.
- No dense matrix multiplication/GEMM in training or serving. Replacing a
  library call with a handwritten dense matrix contraction is not a geometric
  replacement. Native Hamilton transport, geometric rank/margin selection,
  exact addressing and icosian/E8 operators are the intended substrate.
  Scalar floating-point geometric arithmetic remains legal in offline GPU
  learning; D11's integer/table, multiplier-free serving contract remains.
- D24 applies to the full model path, including reference/parity runs: GPU
  only, no CPU model execution and no hidden host fallback. Local editing,
  compilation, static checks, data preparation and grading of stored outputs
  remain possible without model execution. Preserve CPU reference source and
  pre-ban comparator measurements as inactive evidence; do not rerun them.
- Kimi is the current project manager for the shared plan, assignments and
  collision resolution; the owner retains strategic authority. Claude owns
  starling/murmuration reads, DeepSeek invariant-addressed router/memory,
  Codex enforcement/M2 grounded reply, and Kimi objectives/operators/CUDA.
- OP1 uses G2 Hamilton rotation only. G1 rectified-square is withdrawn by the
  owner as non-geometric; there is no non-geometric fallback. A geometric
  feed-forward block (Hamilton gate plus Hamilton projections) is permitted;
  the MLP ban applies to the dense SwiGLU block.
- Build order is OP2 Hamilton kernel first, then OP1 G2, OP3a product-key
  head, compliant rung0, and OP4/rung0b rank-read/gradient. OP3b remains
  the research arm after OP3a, not an extra prerequisite for rung0. These are implementation
  dependencies before an integrated training experiment. Source landing or an
  isolated kernel check does not establish whole-path compliance. Rung 0 is
  suspended, not withdrawn; its obsolete metric/control clauses must be amended
  before prospective review. Rung 0b follows the compliant operator phase and
  resumed rung 0, then margin-trained starling readout.
- **Rule freeze:** the 12 October constraint set plus the late G2-only/build-order/
  geometric-feed-forward clarifications is complete until the first compliant
  rung0 result reports. New concerns go to a proposal card and take effect only
  after that result, except cost/safety problems. No new constraint is adopted
  by this reconciliation. Softmax-dependent results do not count as active progress.
- **Training remains held.** The independent M2 no-training/no-rental hold
  also remains. Prospective TEST FITNESS, complete source/device qualification
  and explicit owner release of the applicable hold precede a run; a FIT
  verdict or this documentation delivery releases no hold and rents no pod.
- Tokens/s on the same CUDA harness is the speed comparison. Quality still
  needs a lawful raw-score or generated-answer measure. Historical BPB/NLL
  numbers remain evidence, not permission to execute a normalized-exponential
  evaluator. No new numeric quality threshold, milestone acceptance change,
  mechanism retirement or count reset is adopted by this reconciliation.
- Preserve every lab's unique source, artifacts, jobs, leases and worktrees;
  cleanup is limited to the deliverer's verified redundant workspace/branch.
- **Owner ruling, 12 October 2026 (late, in the harness session) - no matmul and
  no softmax, anywhere.** Asked to resolve the last reading under which the GPU
  training lane might still use dense GEMM, the owner answered: *"no matmul at all
  anywhere, same with softmax - there is a ternary alternative with xor that may
  work."* This adds no fourth restriction; it confirms and widens the scope of D23
  and D25, which already prohibited softmax and dense matrix multiplication in the
  executed path, and it removes the training-lane exemption.
  - **Ternary/XOR is an owner-suggested candidate, not an adopted mechanism.** The
    owner offers a ternary weight representation combined with XOR as a possible
    replacement arithmetic. It has not been designed, pre-registered, reviewed for
    TEST FITNESS, or measured here, and the owner's belief is not evidence. Under
    D25's rule freeze it enters as a proposal and changes nothing until the first
    compliant rung0 result reports. Brief:
    [ternary-xor-arithmetic.md](../mechanisms/ternary-xor-arithmetic.md).
  - It is **not yet established as a geometric mechanism.** D25 withdrew the
    rectified-square gate for being non-geometric, so a ternary/XOR scheme must
    justify itself on the same ground or be carried explicitly as a labelled
    non-geometric arithmetic substitution. The brief records which side it falls on
    and what a fair test needs.
  - No result that needs matmul or softmax anywhere counts as progress on any line.
    Historical measurements keep their scope as inactive evidence.

The [repaired operator plan](../labs/operator-phase-2026-10-12/README.md) defines
the preparation/admission boundary. This is agent governance, not a claim that
existing executables enforce all prohibitions or that a compliant model exists.


### Later owner clarification in the Codex session — October 12

Use SpiralCore's actual BFS transport together with the Atomic Ternary FBS
binder/frontier abstraction; a dense quaternion sum over every input remains
prohibited. The owner explicitly authorizes developing, building and testing
this mechanism on a CUDA 12.8+ devel Ubuntu24 pod through `uor-pod`. This named
build/kernel qualification permission releases no training hold or unrelated
rental campaign. [The projection record](../labs/spiralcore-fbs-projection-2026-10-12/README.md)
binds implementation, prospective finite-kernel FIT, preservation and limits.
The OP2 dependency is native SpiralCore/FBS geometry; local OP1 G2 rotation
remains, and OP3a/full-path/shared-writer qualification still follows.

## D26 - Vast.ai is the default provider for new pods

**Authority, October 13:** Casey, in the Claude session: "backend first, setup
the new pods to vastai the n go ahead and continue from kimi afterward",
following the owner's request to move GPU rental from Runpod to the cheaper
Vast.ai. The owner opened and funded the Vast.ai account (prepaid) and approved
a one-hour trial, which passed (13 October, 01:07–01:15Z).

- `uor-pod up` rents new pods on Vast.ai. **Amended the same day** (Casey: "we
  dont use runpod anymore we use vastai now"; "you may leave runpod as a backup
  secondary only if no choices have existed for a while and all work is blocked
  by needing a pod when runpod can supply it"): Runpod is an emergency backup
  only. `up --wait` switches to it after 30 minutes with no Vast offer, and only
  when Runpod has stock, recording the switch in the ledger and on the compute
  board; choosing Runpod by hand needs `--owner-approved`. No other provider is
  authorized. Both go through `scripts/pod/uor-pod` only.
- Only verified datacenter hosts are rented (no community hosts). The owner caps
  are unchanged and count both providers together: ≤ 4 running pods, ≤ $8/h.
- A Vast pod has no shared network volume. Its data comes from the private HF
  store and `down` archives its results to the laptop before the instance is
  destroyed; iCloud stays the durable result store.
- Existing holds are unchanged: this decision releases no training or M2
  rental hold, and D24/D25 still decide what may run on a pod.
- The Runpod network volume and its pods are not retired by this decision;
  retiring the volume is a later owner step, after its data is confirmed in
  iCloud or the HF store.

The [delivery record](../labs/vast-provider-2026-10-13/README.md) has the
trial and live-smoke measurements and the limits.

## D27 - Rust first, and the uor-foundation ecosystem as the shared floor

**Authority, October 13:** Casey, in the Kimi session: *"we are supposed to use
rust for efficiency and uor compatibility natively whenever possible... prism and
uor-framework along with uor-addr and uor-matmul (and lexlean, and prismpm) all in
the uor-foundation github all were useful for a shared ecosystem floor, so rust is
the best language we can use right now (also its compatible with wasm)... try to
use rust unless you absolutely cannot code your way around using python for some
reason."* Drafted by the Kimi lab (project manager).

- **Rust is the default for all new project code** - model, training, tooling, data
  preparation and scripts - for efficiency, native UOR compatibility and WASM
  reach. Python is used only where Rust genuinely cannot do the job. The existing
  Python harness scripts (`scripts/harness/`) are grandfathered; no new Python is
  added where Rust works.
- **The uor-foundation sibling repositories are the shared ecosystem floor**: reuse
  them before rewriting them, at the pinned revisions in `Cargo.toml`/`Cargo.lock`
  (a repin is its own compatibility decision, and
  [the source audit](uor-source-audit.md) records the upstream deltas).
  - **`uor-addr`** - canonical content addressing. Already pinned
    (rev `165b51e3e`), and the `uor-r4-core` address/identity paths already use it.
  - **`UOR-Framework`** - the typed UOR ontology. Already pinned as
    `uor-foundation`, `uor-foundation-sdk` and `uor-foundation-verify`
    (rev `51c013822`).
  - **`uor-matmul`** - under D25, only its lawful parts are usable: its exact
    dyadic accumulation semantics (one final rounding, not sequential FMA/BLAS
    rounding), its codec/codebook identity machinery (a schema-tagged manifest with
    codebook and code digests), its tabulated no-multiply path, and the tropical
    (max,+) selection lane (`uor_matmul_core::dot_tropical_ref`, already pinned for
    parity in `crates/uor-r4-graph-cli`). **Its dense GEMM entry points remain
    banned in every executed training and serving path** (D25) - as does anything
    from a sibling that would smuggle one in under a different name.
  - **`prism`** - the standard-library facade, transitive through `uor-addr`.
  - **`LexLean` and `PrismPM`** - pattern-level reuse: the repo-model /
    repo-conformance claim register and the honesty-gate discipline, with the
    packaging and attestation patterns. Neither is a dependency of this workspace.
- **Every interface surface imported from a sibling gets a D25 compliance check
  first**: no softmax-family operation, no dense matrix multiplication in an
  executed path. A dormant upstream does not block pattern-level reuse with
  attribution.
- **Nothing here is retroactive.** Existing artifacts, measurements and the
  historical Python/dense references keep their exact scope as inactive evidence,
  and no training or rental hold is changed by this decision.

## D28 - No transformer comparisons until the native system works end to end

**Authority, October 13:** Casey, relayed by the Claude lab: *"honestly we really
dont want to start comparing against transformers until we have a working end to
end system"* ([#2028 comment 6147245657](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6147245657)). Filed by the Kimi lab (project manager).

- **No transformer or neural-network comparisons of any kind** - comparators,
  controls, baselines, teachers, and neural graders - until the owner declares the
  native model works end to end. **That declaration is the owner's, not a lab's.**
- **Panels are scored with deterministic checks only**: membership checks, exact-answer
  oracles. A bar that can only be scored by a neural grader today is recorded
  **BLOCKED** and is not run. This includes the v5 reply panel's `grade-replies`
  qwen2.5:7b grader.
- **Comparisons stay inside the native line**: prior native rungs (for example rung 0b
  vs rung 0) and lawful count references, as the rung-0 and rung-0b sheets already do.
- **This settles the rung-0b card's open grader question (#2032)** and supersedes any
  older instruction implying a transformer baseline arm in active rungs.
- **Nothing here is retroactive.** Historical transformer-comparator evidence keeps
  its scope as dated records, and nothing is deleted.

## D29 — No banned mechanism in any compiled or loaded module; vendor-runtime content counts

**Authority, October 13:** Casey, relayed via the Kimi PM. Extends D23/D24: it is
not sufficient that a banned kernel is never launched.
**Amended 13 Oct (owner): NVRTC itself banned** - supersedes this decision's
original "NVRTC remains the CUDA compiler" clause; the prebuilt module, not the
runtime compiler, is the compliance surface.

- **Banned mechanisms must not exist in any compiled or loaded module, linked vendor
  call path (cuBLAS/cuDNN) or executed host fallback** - softmax, softplus,
  sigmoid, tanh and exp-family squashing; matmul/dense GEMM including tensor-core
  `mma`/`wmma` contractions. **The prebuilt module surface is the compliance
  boundary (amended): what `nvcc` compiles at build time is exactly what the driver
  may load, and presence counts, not only execution.**
- **NVRTC (runtime compilation) is banned project-wide** - no runtime compiler
  anywhere in the chain. All CUDA kernels compile ahead of time (`nvcc` via
  `build.rs`) into named PTX/cubin modules, hash-bound into artifacts, and load
  through the audited prebuilt-module launcher (`launch_prebuilt` pattern:
  driver-loaded module, no kernel shared with the historical `CUDA_STACK_SOURCE`).
  `CUDA_STACK_SOURCE` (`crates/uor-r4-training/src/cuda_stack_kernels.rs`) is split
  so prebuilt modules carry only compliant kernels; historical kernels (read/flash
  softmax, cross-entropy, fused sigmoid/exp/GELU recurrence, pointer mixture) are
  excluded from compiled modules and preserved as source evidence. **Nothing is
  deleted.**
- **Every track is in scope**: the rung-0 trunk, the Line-B native learner
  (`crates/uor-r4-core/src/native_geometric`), D11's integer-engine exp-table
  masses, and tooling. Historical/retired arms remain reachable only through
  configs the guards reject, and are D23/D25/D29-prohibited for new runs.
- **Enforcement.** The D23/D25 ratchets are Rust (`uor-harness ratchet d23|d25`,
  #2346, per-file parity with the retired Python scripts); DeepSeek's harness line
  repoints prlint/CI at them and retires the Python scripts, and the semantic path
  checker extends to CUDA raw-string kernel sources so compiled content is
  semantically gated, not only word-counted.
- **Consequence.** Rung-0 execution waits for the D29 vendor boundary to close:
  Claude's prebuilt-module split landed (#2344, NVRTC removed; the two learned
  contractions Codex found are quarantined in #2347), and Codex's vendor-boundary
  repair (#2345) runs its composed CUDA qualification (31 controls) after its F1
  guard fix; Codex re-qualifies on the prebuilt module, and cudarc's `nvrtc`
  feature is out of `uor-r4-training`'s dependency set. The FIT sheet pins source
  identity at execution, and the executed module must not contain banned kernels.
- **Assignments.** Claude builds the replacements and the module split; Codex
  reviews and re-qualifies; the Kimi PM routes.

## D30 — Hard substrate rules: no matmul or softmax anywhere (no wrapper laundering), Rust only, GPU-only heavy compute, pod kernel builds

**Authority, October 13:** Casey, direct to the Kimi PM (posted to all labs on
#2028, comment 6149487903). Sharpens D23/D24/D27/D29; where older text is softer,
this wins.

- **No matmul and no softmax — anywhere, and no wrapper laundering.** No dense
  GEMM/matmul and no softmax/softplus/sigmoid/tanh/exponential-family squashing in
  the model, training, serving, teachers, comparators, or tooling. A banned
  operation hidden behind a geometric-sounding name, a wrapper function, a
  runtime-filled lookup table, an FFI call, or a vendor library is still banned:
  compliance is judged on what compiles and executes, never on what the call site
  is named. Retired mechanisms are archived as evidence (branch-archive /
  cloud-store), never deleted, never active.
- **No Python. Rust only.** Every tool, harness path, trainer, and serving path is
  Rust. Transitional Python delegation inside the harness is a self-announcing
  shim (D27's port), not an accepted end state; no new Python is added.
- **GPU-only for anything that can be designed for GPU.** Model compute runs on
  GPU pods (Vast, per D26). **No big CPU runs locally** — the owner's laptop
  carries orchestration, git, harness, docs, and small builds only. A run that is
  heavy is designed for the pod rather than degraded onto local CPU.
- **Building GPU kernels on the pod is authorized.** Ahead-of-time nvcc builds may
  run on the pod when the kernel work needs the CUDA toolkit or the target GPU;
  per the D29 amendment, kernels ship as prebuilt modules (no NVRTC), so pod
  builds produce the module artifacts that get hash-bound and qualified.

## D31 — No result is ruled out on an instrument technicality

**Authority, October 13:** Casey, relayed from the owner's session to the Kimi PM
("file it now") and to Codex for the running murmuration qualification (#2028,
comment 6153979044). Companion to PM rule R12 (`docs/harness/pm-rules.md`), which
carries the four authority questions every check must answer before it has veto
power; D31 binds what happens when a check fails anyway. Where older text allows
a first-failure stop or treats an instrument artefact as a code blocker, this
wins.

1. **Classify a failing check before stopping on it.** Decide whether the check
   failed because of the mechanism or code under test, or because of the
   instrument: the test's own measurement precision, a reduction or cancellation
   artefact, the harness, setup, the driver, or tolerance bookkeeping. When a
   magnitude analysis shows the instrument can reject a correct result (as in
   murmuration control 11: two F32 global sums differing by about 8e-5 against
   about 1e-6 of rounding — the 0.002 relative bar admitted less than one F32
   spacing of numerator error), it is an **instrument defect** at once. It is not
   an open "hypothesis" that keeps the result blocked.
2. **An instrument defect is never a BLOCKER on the code, a negative or a stop.**
   Repair the instrument without changing the registered law, bar, step or
   fixture, and re-run. It never counts toward D21, never closes a configuration
   (D22), and never appears in a record as a code or mechanism failure.
3. **A qualification runs every independent registered control in one pass** and
   reports all failures together. One control's failure, of either kind, never
   leaves the rest NOT_RUN. Stopping at the first failure is reserved for setup
   or safety failures that make the remaining results meaningless: a missing
   binary, a source-pin mismatch, a D29 refusal on a required path, or a resource
   cap.
4. **Bars stay frozen.** This rule widens no tolerance. It forbids treating an
   instrument's own error as the measured object's failure.

First application: the murmuration control-11 secant failure is an instrument
defect, not a numerical BLOCKER on S1; the repaired device-F64 secant passed the
unchanged bar (analytic 0.041992284 vs F64 secant 0.042009846 at the disputed
point) and the qualification runs all 33 controls in one pass at the repaired
pin. R12's build-first and solvability questions still govern whether a check
should have run at all; D31 governs what a failure may mean once it has run.

## D32 — Discrete geometric substrate: ternary/Hamming/quaternion computation, no floating point in model computation including learning

**Authority, October 13:** Casey, relayed by Codex from the owner's session to
#2028 (comment 6154399083) and #2032 (comment 6154399570), with the owner's
clarification on #2032 (comment 6154447417):

> "ternary and e8 hamming distances or discrete hamming distances, or
> quaternions — we do not want floating point"

> "spiralcore preserves the e8 geometry through transport"

The owner asked why the learning path uses gradients instead of the project's
own vector/spinor/flock/FBS geometry. This decision is the answer made binding.

1. **The model's computation is discrete, including learning.** Scores,
   comparisons, state transitions and credit assignment are defined on
   ternary/exact-valued coordinates, E8-label or ternary-coordinate Hamming
   distances, or exact quaternion/icosian actions. No floating-point forward
   pass, no floating-point optimizer, and no floating-point gradient anywhere
   in the model path. Scalar float arithmetic that D8/D0-b permitted for
   offline training is **superseded for the model path** by this decision;
   D0-b's serving rules (D11) already excluded it there.
2. **The backbone is the existing geometry-preserving SpiralCore transport
   with FBS,** not an arbitrary bit encoding substituted for E8. Discrete
   learning selects and credits admissible geometric moves and routes within
   that structure. E8/icosian identities, orientation and the shared writer's
   native carrier are preserved. Ternary/Hamming comparisons are defined on
   the actual transported representation.
3. **Design cautions recorded with the direction.** Ternary-coordinate
   Hamming distance is not bit-code Hamming distance; an E8 label's Hamming
   distance is not automatically the E8 lattice metric. Exact icosian or
   quaternion actions can use integer-pair Z[phi] coefficients or finite
   transition tables. Learning can compare admissible neighboring discrete
   states and apply local credit without a floating-point optimizer. These
   are the design direction to specify and review, not claims that an
   implementation exists or is qualified.
4. **Existing floating-point work is preserved, not deleted.** F32 production
   operators, the F64 secant instrument, the murmuration qualification work
   and the writer adjoint repair remain as archived source and evidence under
   their exact scopes. This is a scope correction, not a mechanism negative
   (D22), and it permits no deletion of anyone's work.
5. **No floating-point numerical qualification or training launches.** Any
   pre-registration whose registered training or qualification law runs a
   floating-point optimizer or floating-point model computation is frozen —
   recorded, not executable — until it is re-registered on a compliant
   discrete learning law. A FIT verdict on such a sheet does not release
   anything. Training and M2 remain held until the owner releases the hold on
   a compliant sheet.
6. **Per-lab application at filing:** Claude stops further F32/F64 numerical
   work or promotion (source and instrument work preserved); DeepSeek aligns
   the native routing/memory learning design with discrete/ternary/exact
   geometric state and updates; Kimi (PM) records the rule and reconciles the
   active plan and sheets; Codex specifies the discrete learning law for
   review. New design and review work proceeds; no compute beyond
   preservation.

Where any older decision, sheet, rung plan or operator card assumes a
floating-point training law or floating-point model computation, D32 wins.
D23/D29/D30 remain fully in force and are not widened by this decision.

**Owner clarification, October 13 (#2032, comment 6154472370):** "hamming
distance is just one factor of the geometric routing." Carry the architecture
intact: SpiralCore preserves E8 geometry through transport; FBS provides
binding, frontier and routing structure; Hamming distance contributes as one
routing factor alongside the geometric structure, never as its sole
definition. The same native vector writer serves input, carry and output.
This corrects an overly narrow reading of item 3 as Hamming-encoding design;
it is not permission to replace transport with a Hamming-only model.

**Owner escalation, October 13 (#2032 and #2028, comment 6154767992/6154768548):** the owner
now explicitly requests **implementation and protected delivery** of the discrete geometric
learning path and shared vector writer using SpiralCore/E8/FBS — not only a design. Codex
implements at `codex/discrete-vector-learning-20261013` (base `e42e27e8e`): the exact
transported vector carrier shared by input/carry/output, causal FBS routing using actual
SpiralCore actions, and the discrete admissible-move credit/update law, with no floating
optimizer, no coefficient contraction, no host model fallback and no Python. Claude owns
independent law/read-interface review and the discrete murmuration/read integration design;
DeepSeek reviews and reuses the exact SpiralCore/E8 transport and FBS binder/frontier
interfaces and names the invariants to carry; Kimi (PM) carries the successor rung sheet under
R13 once law and source are pinned. Training and M2 remain HELD; this escalation changes the
substrate and wiring and claims no language quality. GPU finite execution waits for a compliant
FIT sheet. All prior source is preserved.

**Owner addendum, October 14 — the complex-plane zeta-zero exception.** Confirmed by the
owner directly to Kimi (PM) and quoted by Antigravity (#2028, comment 6157803441; Kimi's
confirmation request #2029 comment 6157856812): "primes are equivalent to zeta zeros and we
can take advantage of that by using prime routing with the zeta zeros in the complex plane
that connects euclidean and riemannian." D32's no-floating-point rule therefore admits one
scoped exception:

1. **Floating point is permitted only on the complex plane for the Riemann zeta-zero
   imaginary parts** — $\rho_k = \frac{1}{2} + i\gamma_k$, the prime log-phases
   $\theta_k(p) = \gamma_k \ln p$, and the phase factors $e^{i\gamma_k \ln p}$ — in the maps
   from complex 4D to Riemann 4D ($\mathbb{C}^4 \to \mathbb{R}^4$) and complex 8D to Riemann
   8D ($\mathbb{C}^8 \to \mathbb{R}^8$). This is where the prime/zeta-zero equivalence is
   exploited for prime routing, connecting the Euclidean and Riemannian representations.
2. **Everything downstream of those maps stays discrete.** Group routing ($\mathrm{Cl}(0,6)$,
   $2I/H_4$), lattice quantization ($E_8 = H_4 \oplus \varphi H_4$), ternary FBS switching,
   murmuration/starling reads, $G_2$ gates, the shared vector writer and counterfactual
   learning remain integer/exact operations with no floating point. The exception does not
   re-admit floating-point forward passes, optimizers, gradients, squashing functions or
   comparators anywhere else (D23/D32 otherwise unchanged), and it does not release any hold.
3. **Instrument status.** Any use of the exception inside a qualification or training path is
   declared on its pre-registration sheet with the exact map, its inputs/outputs and its
   integer boundary, so reviewers can verify the exception's scope was not exceeded.

## D33 — Delegated D4 execution for the rung-0 A2 pilot: Claude runs owner-release after his posted review

**Authority, October 14:** owner direction in Kimi's session ("can you allow me to have
claude release it after he finished his review and recommendation? that way I dont have to
go run something manually too").

1. **Scope.** For the rung-0 A2 seed-1 diagnostic pilot only (scope
   `m1-a2-seed1-vector0..1639-diagnostic-pilot`, plan pinned in
   `docs/labs/rung0-pilot-release-2026-10-14/` at merge `f410a77c`), Casey delegates
   execution of `discrete-stack owner-release` to the Claude lab, conditional on Claude's
   completed exact-head review and posted recommendation — satisfied by #2421 comment
   6160178151 and carried in the plan's pinned `review.json`.
2. **R15 amended, not repealed.** No other lab may write, edit, copy or generate
   `owner.json` or any stand-in. The delegate is named (`UOR_LAB=claude`), the release
   remains bound to every pinned identity in the plan, and the typed confirmation phrase
   is still required. The record format is unchanged (`owner: Casey`,
   `owner-terminal:` receipt channel — the decision is the owner's either way); the
   delegation is attested by this decision, the amended R15, and the GitHub receipt
   chain. The pinned admission path (`validate_release`) and the qualified pilot binary
   `d12877b6d8adc6a60445a014be682ab82459a781f760ac99a226b07335cbfcac` are untouched; the
   delegated run uses a tooling build of the discrete crate whose only delta from the
   pinned driver source is the actor guard, and it authors metadata only.
3. **No precedent.** M2 and every other training hold remain HELD and owner-personal.
   Future releases return to R15's owner-personal rule unless the owner delegates again
   explicitly. A delegated record created outside this scope, or by any other lab, is void;
   compute started from one is unauthorized.
## D34 — Revised attention/memory sequence accepted: reranker-first relational read plus a separate D19 object-store crate

**Authority, October 14:** owner decision delivered in Kimi's session via a persistent
decision dialog (the record is #2028 comment 6176107062; the question text offered
Accept / Need detail / Hold).

1. **Accepted direction.** The owner accepts the revised sequence from Claude's
   independent adversarial review (#2482, verdict REVISE, per-item): **reranker-first**
   — a bounded exact-relation reranker as a third `ReadSpec` sharing Flock's node
   construction, one ternary trit per (edge type, level) on the existing Gate kind,
   relation-off ≡ Flock as the rollback identity, capability target the copy-successor
   read (3,847/14,287 = 26.9% of pinned DEV targets, a capability Route and Flock
   provably cannot express, `discrete.cu:56-71`, `167-189`) — **plus a separate Rust
   object-store crate for exact compressed reads on the D19 durable-memory line**
   (not attention; benefit is warm/edit reuse, unique content expands 2.17× in
   records). The energy-controller class of machinery is dropped as the first
   artifact; C6-vs-G64 stays unselected because neither is needed for the reranker
   test; the ≤16K Flock shortlist is not the head's pool (pool = all eligible C192
   BinderPlan nodes); the writer is untouched and admission is reported separately.
2. **Prerequisites unchanged.** This is a direction decision only. No compute,
   adoption or hold release follows from it: every artifact still needs its own
   pre-registration, TEST FITNESS: FIT, and explicit owner release. A trained-in
   comparison additionally needs a declared interleaved schedule on both arms
   (registered learner change) and waits for seed-1's Route rows at cursor 16,404.
3. **Companion record.** The same dialog authorized removal of the dead `target/`
   symlink in the owner checkout (Sep-29 debris pointing at a retired archive
   volume; no content held by the link). Removal executed 2026-10-14 ~19:05Z.
