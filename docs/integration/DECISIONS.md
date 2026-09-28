# DECISIONS

Append-only. Each entry is an owner decision with its rationale and scope.
Agents may draft entries; only the owner ratifies them.

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

