# Bounded semantic transitions and replayable plan witnesses — specification (#843)

- **Issue:** #843 — "compiler/runtime: learn and execute bounded semantic transitions with
  replayable plan witnesses" (item B of S4 tracker #826, programme #820).
- **Parent tracker:** #826 (S4 — semantic planning and geometry qualification).
- **Date:** 2026-08-22.
- **Status:** Frozen design contract for the build increments. This document freezes (a) the typed
  observation record and the deterministic induction that turns observations into a transition rule
  set, (b) the fixed capacities and the deterministic decline rules, (c) the packed R4G1 sections,
  (d) the deployed bounded planner and its plan-witness encoding, (e) the equal-budget arm/null
  bake-off, and (f) the measurement and run contract. It establishes **no** planning or reasoning
  capability — that is the measured question of the later increments and of #846.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md). Every
  labeled statement carries exactly one claim class (**Definition**, **Objective**, **Guarantee**,
  **Assumption**, **Empirical Criterion**); every **Guarantee** and **Empirical Criterion** carries a
  status (**Structural**, **Witnessed**, **Empirical**, **Assumed**, **Unproven**).
- **Execution scope:** two scopes, deliberately separated in the sense of
  [`docs/conformance_execution_scope_830.md`](conformance_execution_scope_830.md). The observation
  record, the inducer, the arm bake-off harness, and the benchmark instruments are
  **compiler / certifier-instrument, off-serving-path** (owned collections and f32 permitted). The
  lowered planner of §7 and the witness encoder of §8 are **normative deployed-serving** (fixed
  capacity, allocation-free steady state, P-4 operations only, `no_std`, no f32). Nothing in the
  first scope is deployed-serving evidence.
- **Predecessors consumed:** the frozen benchmark constitution and typed reference model of
  [`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) (#844, including
  its appended §11 Amendment A1); the normative scorer designation of #831; the RF-27 semantic-state
  reference model and the RF-08 bounded future-state planner.

---

## 0. Entry boundary and the binding instrument verdict

### 0.1 Inherited boundaries (not re-litigated here)

- **S1 PROMOTE (RF-31).** Prompt causality is established at +28.45‰ [25.57, 31.32]; the persistent
  prompt state `Ψ` (#835) is available as a conditioning surface.
- **S2 REVISE — honest abstention, no calibrator.** Every confidence object in this document is an
  **ordinal**, decline-oriented band. **Assumption.** No statement here reads a calibrated
  probability and no metric rewards a numeric confidence value. Abstention is emitted through the
  RF-30 typed decline surface.
- **S3 LIMIT — no free-running generation at this scope (#824).** Every measurement here is scored
  teacher-forced. **Assumption.** No result in this document is read as free-running coherence; the
  #824 boundary stands and any future generation re-entry re-measures against the unchanged
  #841 §6 bar.
- **S4 entry — AMEND + PROCEED (#826, 2026-08-22).** S4 proceeds in parallel with S2's open
  re-entry on the *established*, not promoted-generative, substrate.

### 0.2 The binding cheap instrument ran first, and returned DEGENERATE

#843's run contract makes the coverage/reachability instrument binding **before** any packed or full
fitting. It was run first, and it did not pass. Its four results, the bounded repair they force, and
the numbers behind them are recorded as **Amendment A1** in
[`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) §11 and are not
restated here. In summary: 13 of the 20 frozen horizon cells were 0/512 solvable; a structure-keyed
memorized-trajectory null saturated at valid-plan rate 1.0000 in every non-vacuous cell, placing the
#826 promotion statistic at ≤ 0 < δ_min by construction; five of six split axes had a single cell;
and `TaskInstance::id()` carried the generator seed, which made an id-keyed null score 0.000 and read
as healthy.

**Definition (consequence, frozen).** Increment 2 of this issue implements Amendment A1 and re-runs
the instrument. **No packed section is written and no arm is fitted until the instrument reports
non-vacuity ≥ 0.5 per frozen cell, axis cardinality ≥ 8 per axis per family, a content-derived
identity, and strongest-non-oracle-null headroom ≥ δ_min.** If the re-run does not clear those four,
the recorded outcome of #843 is the limiting diagnosis with typed planning kept **NOT ESTABLISHED**,
not a weakened gate.

### 0.3 The corpus-observation arm: a structural zero, recorded without a run

#843 asks whether *corpus / instruction observations* can induce useful transitions. The reachability
arithmetic answers this from source, at no run cost.

**Definition (observation record schema, as it exists).** The v4 corpus observation record
(`crates/uor-r4-graph-compiler/src/observation.rs`, `RECORD_SIZE = 88`) is (story token, next token,
top-8 tokens, top-8 weights, span, byte anchors), with a 16-byte probability sidecar
(target logprob, entropy, top-8 mass, target rank).

**Empirical Criterion (corpus coverage of the typed schema). Status: Empirical.** Of the ten typed
object kinds the planner requires (#844 §3.1 — state, action, precondition, effect, goal, constraint,
evidence/provenance, confidence, transition, plan witness), the v4 record provides fields for
**zero**. Coverage of the benchmark's typed state/action vocabulary recoverable from the corpus
observation store is therefore **0.000**, against δ_min = 0.05.

**Definition (exit, applied).** Under the long-run discipline (AGENTS.md, gate one: if the ceiling is
below the effect you are hoping for, do not launch), the corpus-induction arm **does not launch**.
This is a structural source-check result, not an empirical `UNAVAILABLE`: no observation shard is
read and no wall-clock is spent. Transition observations are instead compiled from the fitting split
of the repaired #844 generator (§3), which is document-disjoint from the held-out and sealed cells by
construction. **Reopening condition:** a corpus observation record that carries typed state/action
slots — a schema change, not a scale change — would make this arm measurable and is out of scope
here.

---

## 1. Problem, scope, and non-goals

A planner can appear successful through retrieval leakage, an oracle-like compiler, unbounded search,
or a reference-only floating-point path; packed semantics can also drift from the certifier or the
production engine. This issue's job is to make each of those failures visible and then to establish —
or to refuse — the strongest **non-geometric** production planning baseline.

**In scope (frozen here):** the typed transition observation record (§2); deterministic induction,
deduplication, conflict handling, and held-out enforcement (§3); the fixed capacities and the
deterministic decline rules (§4); the packed R4G1 sections (§5); the deployed bounded planner (§6);
the plan-witness encoding (§7); the equal-budget arm and null set (§8); reference/packed/production
differentials and planted mutations (§9); the measurement protocol and run contract (§10); and the
Amendment A1 generator repair this issue must land first (§2).

**Out of scope:** any geometry mapping or W(3,3) qualification (#845, which stays `on-hold` until this
issue records a non-degenerate baseline); the untouched-partition final verdict and the claim bounding
(#846); free-running generation re-entry (#824 boundary); corpus-schema extension (§0.3).

**Non-goals (from #843):** unbounded search or heap growth on the hot path; use of the benchmark
verifier, a hidden gold path, the teacher, or the network during deployed planning; and any claim of
geometry value.

---

## 2. Increment 2 — Amendment A1 (prerequisite, lands before anything is packed)

Amendment A1 is specified in [`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md)
§11.2. This section freezes only how it is verified here.

**Definition (the four gate instruments, built).** Each is a deterministic, teacher-free test over the
repaired generator, and each is reported per (family × frozen horizon):

| Instrument | Frozen threshold | Fails if |
|---|---|---|
| `non_vacuity_per_frozen_cell` | solvable fraction ≥ 0.5 in all 20 cells | any cell cannot separate arms |
| `axis_cardinality` | ≥ 8 distinct cells per §2.2 axis per family | a split axis is a one-element set |
| `identity_is_content_derived` | structurally identical instances share an id; seed excluded | an id-keyed null cannot fire |
| `strongest_null_headroom` | strongest non-oracle null ≤ 1 − δ_min in every cell | the promotion statistic is ≤ 0 by construction |

**Guarantee (A1 does not move a frozen number). Status: Structural** — a built test asserts
δ_min = 0.05, n = 512, H ∈ {1, 2, 4, 8}, `H_MAX` = 16, and `W_MAX` = 64 hold their #844 values after
the repair.

**Empirical Criterion (A1 gate). Status: Empirical — PASSED 2026-08-22 (increment 2).** All four
instruments pass, so increment 3 may begin. Measured values and the per-cell tables are recorded in
[`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) §11.5; the
instruments live in `crates/uor-r4-graph-compiler/tests/amendment_a1_843.rs` and run in the default
`cargo test`. Headline: non-vacuity 1.0000 at H ∈ {2, 4, 8} and 0.7500 at H = 1; every split axis at
8 cells or more; identity content-derived; strongest non-oracle null between 0.2773 and 0.9355,
against a ceiling of 0.95. A failure would have been reported as the recorded outcome, not routed
around.

**Definition (the horizon-1 cell, frozen — maintainer sign-off 2026-08-22).** Valid-plan rate cannot
separate planning from retrieval at horizon 1: a one-step answer is a deterministic function of the
observable state, goal, and operator set, and the fitting split must cover the whole operator pool
or induction has nothing to learn from. Measured, a retrieval baseline scored exactly 1.0000 there.
The H = 1 cell is therefore gated on the **correct-outcome rate** — a valid plan on a solvable
instance, or a correct `Decline(no_plan)` on one with no plan inside the horizon — and a quarter of
horizon-1 instances are generated beyond the horizon so that decision is real. On every H ≥ 2 cell,
where all instances are solvable, the correct-outcome rate *is* the frozen valid-plan rate. Full
reasoning: `compositional_planning_spec_844.md` §11.6.

**Watch item for increment 6.** The tightest cell is symbolic-transformation at H = 1: strongest null
0.9355, headroom 0.0645, only 1.29 × δ_min. The A1-d instrument prints a `TIGHT` marker for any cell
whose headroom falls under 2 × δ_min. A candidate has barely more than the effect floor of room
there, so that cell's reading carries the least weight in the §8 selection.

---

## 3. Typed transition observation records and deterministic induction

### 3.1 The observation record

**Definition (`TransitionObservation`, versioned).** One observed or generated typed transition
attempt, carrying everything the inducer and the leakage scan need and nothing from which a gold path
is recoverable:

| Field | Meaning |
|---|---|
| `sample_id` | content-addressed identity of the observation, derived from the typed content only — no seed, clock, RNG, or hash-iteration order |
| `family` | one of F1–F5 |
| `from_slots` / `to_slots` | typed slot valuations before and after the attempt (bounded integers) |
| `operator` | operator id and its typed parameter block |
| `outcome` | `Applied`, `PreconditionFailed`, `ForbiddenRegion`, or `Unknown` |
| `read_slots` | the slot mask the precondition actually read (the basis for rule generalization) |
| `effect_delta` | the typed per-slot delta observed, or `None` when not applied |
| `goal_ref` / `constraint_refs` | ids of the goal and forbidden-region predicates in force |
| `evidence` | provenance id plus ordinal support band (F4); empty otherwise |
| `polarity` | `Positive`, `Negative`, or `Conflicting` |
| `split_cell` | the (entity, vocabulary, topology, template, composition, horizon) cell this observation belongs to |

**Guarantee (no future-answer field). Status: Structural** — a built test asserts that no field, and
no combination of fields, exposes the gold terminal state or the gold action sequence: the record
describes one attempted step, never a plan.

**Definition (negative and conflicting examples, required).** `PreconditionFailed` and
`ForbiddenRegion` observations are first-class and are emitted at a frozen ratio alongside
`Applied` ones. A rule set induced from positive examples alone cannot express a precondition, so the
inducer refuses an observation set whose negative fraction is below a frozen floor.

### 3.2 Induction to a transition rule set

**Definition (what is induced — schematic, not grounded).** The induced object is an **operator rule
set**: for each operator, one or more rules of the form (precondition mask and comparison block,
typed effect delta, support count, ordinal band). Grounding — computing `s' = T(s, a)` — happens at
plan time by saturating integer arithmetic on the packed slot vector. **A grounded
`(state, action) → state` table is deliberately not the artifact:** it would scale with the reachable
state space, while a rule set scales with the operator vocabulary, which is what makes the fixed
capacities of §4 and the P-4 hot path of §6 achievable.

**Definition (deterministic pipeline, frozen).**

1. **Partition** observations by `sample_id` into content-addressed shards (plan §4.1 discipline); no
   HashMap-iteration-order, clock, or RNG dependence.
2. **Reduce** per shard in ascending `sample_id` order, then merge shards in ascending shard index —
   an ordered reduction, so the result is independent of shard count and thread count.
3. **Deduplicate** on the canonical key (`operator`, `read_slots`, comparison block, `effect_delta`);
   duplicates increment `support` rather than creating an entry.
4. **Detect conflict.** Two observations sharing (`operator`, `read_slots`, comparison block) but
   carrying different `effect_delta` are a **declared conflict**. The conflicting rule is **not**
   emitted into the deployed rule table; it is recorded in a conflict list with both deltas and their
   supports. Reaching a conflicted rule at plan time is `Decline(unknown)` — never a majority vote,
   never a silent default.
5. **Assign an ordinal band** from `support` against frozen thresholds. Ordinal only (§0.1).
6. **Enforce splits.** Every observation carries its `split_cell`; the inducer rejects any observation
   whose cell is in the held-out set or in the #844 §2.6 sealed set, and a leakage scan asserts the
   emitted rule set is derivable from fitting cells alone.

**Guarantee (induction determinism). Status: Structural** — a built test induces the same rule set
byte-for-byte from reordered shards, from a different shard count, and across repeated runs.

**Guarantee (no evaluation leakage). Status: Structural** — a built test asserts that a held-out or
sealed observation reaching the inducer is refused by name, and that the emitted set attributes
support to fitting cells only.

### 3.3 Increment 3 outcome (measured 2026-08-22)

Built as `crates/uor-r4-graph-compiler/src/semantic_transitions.rs` with the record in
`crates/uor-r4-graph-compiler/tests/semantic_transitions_843.rs`; the deployed-form primitives and
the §4.2 capacities live in `crates/uor-r4-graph-format/src/plan.rs`, which is `core`-only and
allocation-free. Thirteen guarantees asserted, all passing, in about 8 s.

Fitting on the low half of the topology axis (cells 0–3, 256 seeds per family, horizon 8):

| Family | Rules | Conflicts | Observations | Negatives |
|---|---|---|---|---|
| graph-navigation | 8 | 0 | 32 256 | 440 (13‰) |
| symbolic-transformation | 6 | 0 | 23 520 | 0 (0‰) |
| constraint-satisfaction | 8 | 0 | 32 512 | 1 516 (46‰) |
| multi-hop-evidence | 8 | 0 | 32 256 | 440 (13‰) |
| counterfactual-intervention | 8 | 0 | 27 840 | 0 (0‰) |

**The induced rule count equals the size of the whole shared effect pool in every family** — eight
for the grid families, six for symbolic transformation. That is the property the Amendment A1 pool
design exists to produce: an inducer fitted on the low half of the axis recovers every primitive it
will need on the high half, so a held-out cell is a novel *composition* and never a novel
primitive. A held-out cell is therefore hard rather than unsolvable, which is the only way the
§8 comparison means anything.

**Definition (where the negative floor applies, frozen).** The floor is checked only when the
observed tasks declared forbidden regions. Its purpose is to catch a *sampling* failure — a declared
boundary that the observation pass never probed, from which any induced precondition would be
unfalsifiable. Symbolic transformation and counterfactual intervention carry their whole topology in
the operator effect set and declare no forbidden regions at all, so their dynamics really are total,
there is no negative evidence to demand, and the floor does not apply. Making the floor
unconditional would have refused two of the five families for having easy dynamics, which is not a
sampling failure.

**Assumption (recorded).** The reference model's operators carry no declared precondition, so every
induced rule is unconditional and no `PreconditionFailed` observation arises on this benchmark. The
precondition machinery is exercised instead by the forbidden-region observations, and a built test
asserts the boundary does **not** turn into an operator precondition: the induced effect vocabulary
equals the declared operator effects exactly, with the boundary adding none and removing none. That
is what keeps a query-side constraint out of the artifact-side dynamics.

---

## 4. Fixed capacities and deterministic decline

### 4.1 Measured basis

The pre-repair reachable vocabulary, measured by BFS to `H_MAX` = 16 over 64 seeds per family, is:
542 / 409 / 538 / 545 / 543 distinct states and 4 / 3 / 4 / 4 / 4 operators for
F1–F5 respectively, with 1083–1924 grounded transitions per family and a union of 2577 states and
8715 grounded transitions. **The observed maximum BFS queue depth was 64, exactly `W_MAX`** — the
frozen frontier bound *binds* on this task shape, so a bounded-frontier arm reaches capacity and the
ordering of the retained frontier is decisive. That is the reason §8 carries a table-guided
best-first beam arm and not only a breadth-first one.

### 4.2 Frozen capacities

**Definition (deployed capacities, frozen — maintainer sign-off values).** Set at ≥ 4× headroom over
the measured basis of §4.1, with the two #844 values carried through unchanged:

| Constant | Value | Basis |
|---|---|---|
| `PLAN_HORIZON_MAX` | 16 | `H_MAX`, frozen by #844 §2.5 |
| `PLAN_FRONTIER_MAX` | 64 | `W_MAX`, frozen by #844 §2.5 |
| `PLAN_SLOTS_MAX` | 8 | typed slots per state |
| `PLAN_SLOT_BITS` | 16 | one bounded signed slot value; saturating arithmetic |
| `PLAN_ACTIONS_MAX` | 64 | measured ≤ 4 per family, ≤ 20 union; ≥ 3× headroom after A1 widening |
| `PLAN_RULES_MAX` | 256 | ≥ 4 conditional rules per operator at `PLAN_ACTIONS_MAX` |
| `PLAN_CONSTRAINTS_MAX` | 64 | measured ≤ 5 forbidden regions per instance |
| `PLAN_GOALS_MAX` | 8 | measured 1 per instance |
| `PLAN_VISITED_MAX` | 2048 | measured ≤ 545 reachable states per instance; 3.8× headroom |
| `PLAN_WITNESS_MAX_BYTES` | 4096 | `PLAN_HORIZON_MAX` steps × per-step record, ≥ 4× headroom |

**Definition (caller-owned scratch).** All of the above live in a caller-provided `PlanScratch`
whose size is a compile-time function of these constants — measured at ≤ 64 KiB. The planner
allocates nothing.

### 4.3 Deterministic decline, overflow, and conflict

**Definition (frozen rules).** Exceeding `PLAN_HORIZON_MAX`, `PLAN_FRONTIER_MAX`, `PLAN_VISITED_MAX`,
or any slot/action/rule capacity is `Decline(capacity)`, **never** a silent truncation. Slot and score
arithmetic **saturates**; it never wraps into a valid-looking value. A conflicted rule (§3.2 step 4)
or an unknown slot is `Decline(unknown)`. Exhausting the bounded search without reaching the goal is
`Decline(no_plan)`. An ordinal band below the frozen decline threshold is `Decline(low_confidence)`.
An inapplicable action inside a *submitted* plan is `Invalid(step_i, precondition)`, not a decline.

**Guarantee (totality and determinism of the deployed decision rules). Status: Structural** — built
property tests over the goal, transition, conflict, tie-break, saturation, and decline paths, plus
negative fixtures for each capacity boundary.

---

## 5. Packed R4G1 sections

**Definition (four new optional sections, frozen).** All carry
[`SectionId::OPTIONAL_BIT`](../crates/uor-r4-graph-format/src/types.rs), so an artifact without them —
or a reader that does not consume them — behaves exactly as before (absent-section identity), and
every historical artifact stays valid:

| Section | Id | Contents |
|---|---|---|
| `PSCH` | `OPTIONAL_BIT \| 0x13` | planning schema: version, slot count and widths, operator vocabulary, the §4.2 capacities as recorded values, ordinal band thresholds |
| `PTRN` | `OPTIONAL_BIT \| 0x14` | the induced rule table: per rule (operator id, precondition mask and comparison block, effect delta block, support, ordinal band), canonically ordered, plus the operator→rule index |
| `PGOL` | `OPTIONAL_BIT \| 0x15` | packed goal predicates and forbidden-region predicates |
| `PWIT` | `OPTIONAL_BIT \| 0x16` | the versioned plan-witness encoding of §7 |

**Definition (layout discipline, following the `PSTATE`/`SKMX`/`PSIB` precedent).** Fixed-width
little-endian records; a canonical sort key per table; borrowed zero-copy views over the artifact
bytes; lookup by binary search for the small tables (`PSCH`, `PGOL`) and by a fixed-capacity
open-addressed probe with a header-recorded checked `max_probe` bound for `PTRN`. Any hash used is the
multiply-free add/rotate/xor mixer already normative in `skipmix::hash_key` — unseeded, so identical
inputs hash identically on every platform. **No section lookup cost may be a function of how many
entries the table holds.**

**Guarantee (two-stage validation and fail-closed). Status: Structural** — stage-1 structural
validation (offsets, lengths, alignment, capacity headers, canonical ordering) and stage-2 semantic
validation (rule references resolve, no duplicate canonical key, no conflicted rule present, capacity
headers match §4.2) both reject corrupt or incompatible data with a typed error. Built negative
fixtures cover truncation, overlong lengths, non-canonical order, duplicate keys, out-of-range
operator ids, a capacity header exceeding §4.2, and an unsupported version.

**Guarantee (backward behavior). Status: Structural** — a built test asserts an artifact with these
sections stripped produces byte-identical non-planning output to the same artifact before they were
added.

**Definition (compatibility and migration).** Sections are versioned; absent sections preserve
historical behavior; any witness or API extension is additive or explicitly versioned; an unsupported
artifact era fails with a typed error rather than a best-effort read.

---

## 6. The deployed bounded planner

**Definition (execution scope).** Normative deployed-serving, in the #831 `R4Engine` path, with
caller-owned scratch and state. `no_std`, `forbid(unsafe_code)`.

**Definition (permitted operations, frozen).** Only the P-4 classes: XOR / AND / OR / NOT, shift and
rotate, popcount / cttz / ctlz, saturating and wrapping integer add-sub, integer comparison, and
fixed-offset table reads. **No multiply, no divide, no float** anywhere in the planning hot path.
Scores are saturating `ScoreQ` integers.

**Definition (the planning step).** From a packed state, the planner reads the `PTRN` operator index,
tests each candidate rule's precondition by mask-and-compare, applies its effect delta by saturating
integer add, tests the `PGOL` forbidden predicates by mask and popcount, and inserts the successor
into the bounded frontier. Membership in the visited set is an open-addressed probe with a checked
bound. Ties break by the canonical deterministic order already normative in
[`docs/scoring_semantics.md`](scoring_semantics.md) — no clock, RNG, or hash-iteration order.

**Guarantee (allocation-free steady state). Status: Structural** — asserted by an allocation-census
test in the style of `crates/uor-r4-core/tests/allocation_census.rs`.

**Guarantee (P-4 conformance). Status: Structural** — asserted by the machine-checked source scan
that already enforces the deployed kernel invariant.

**Guarantee (bounded work). Status: Structural** — expansions, candidate tests, table reads, and
integer operations are counted and asserted against the §4.2 capacities on every path, including
every decline path.

**Guarantee (byte-determinism). Status: Structural** — identical pinned inputs produce identical
witness bytes, asserted by the deterministic-rebuild gate.

**Assumption.** The planner may read the persistent prompt state `Ψ` (#835, RF-31) as a conditioning
surface and emits abstentions through the RF-30 typed decline surface. It never reads the benchmark
verifier, a gold path, the teacher, or the network — asserted by a built test.

---

## 7. The plan witness

**Definition (`PWIT` contents, frozen).** Versioned, self-contained, and replayable **without any
model output**: initial packed state; goal and constraint predicate ids and their packed bodies;
per step, the considered actions with their integer scores and canonical tie ranks; the per-step
transition evidence (rule id, support, ordinal band); the chosen action sequence; the terminal
verification result; and the decline reason when the episode is an honest abstention.

**Guarantee (independent replay). Status: Structural** — a built verifier that shares no code path
with the planner replays every transition and the terminal goal test from the witness bytes alone and
returns `Valid`, `Invalid{step, reason}`, or `Declined(reason)`. **A right answer reached through an
invalid intermediate step is rejected** (the #846 rule), asserted by a built negative.

**Guarantee (witness totality). Status: Structural** — every planning episode, including every
decline and every capacity boundary, emits a well-formed witness.

---

## 8. Arms, nulls, and the equal-budget rule

**Definition (arms, frozen — maintainer sign-off).** Three bounded planning arms, each lowered to the
same packed sections and the same P-4 operation set:

- **A1 — bounded breadth-first frontier.** Frontier-limited BFS at `PLAN_FRONTIER_MAX`.
- **A2 — bounded iterative-deepening depth-first.** Depth-limited DFS with an increasing bound to
  `PLAN_HORIZON_MAX`; frontier memory bounded by depth rather than width.
- **A3 — table-guided best-first beam.** Beam of width `PLAN_FRONTIER_MAX` ordered by an integer
  score read from the `PTRN` support and band columns and the packed goal predicate.

**Definition (nulls, frozen).** Four falsifiers, each of which must be non-degenerate — able to fire
and able to fail — before any reading is valid:

- **N1 — retrieval-only.** Nearest stored fitting instance by packed-state Hamming distance, replaying
  its stored plan.
- **N2 — direct-continuation.** The next action the emission path would produce, without planning.
- **N3 — memorized-trajectory.** Replay by **structural** key, with the generator seed excluded — the
  A1-c repaired identity. The pre-repair id-keyed form is retained only as a documented vacuous
  control and is never read as a baseline.
- **N4 — shuffled-state/action.** The same arm on a canonically permuted state and action labeling; a
  mechanism that survives shuffling was not using the semantics.

**Definition (upper reference, not a beat target).** The shortest-path oracle bounds what is
achievable; the promotion statistic of #826 is taken against the strongest **non-oracle** baseline.

**Definition (equal budget, frozen and enforced).** Every arm and every null runs under one shared
`Budget`: identical artifact byte ceiling, identical maximum expansions, identical maximum candidate
tests, identical maximum table reads, and identical maximum integer operations. **Guarantee
(budget parity). Status: Structural** — a built test asserts the recorded counters of every arm are
within the same declared ceilings, and a run whose counters differ is reported as invalid rather than
compared.

**Definition (selection, frozen).** The arm lowered as *the* non-geometric production baseline is the
one whose one-sided 95% lower confidence bound of (arm − strongest non-oracle null) is greatest,
**provided** it exceeds δ_min = 0.05 on every required axis and horizon under the Holm–Bonferroni
adjustment and meets the §4.2 and §6 budgets. **If no arm clears that bar, none is lowered as a
baseline and the recorded verdict is negative.**

---

## 9. Differentials and planted mutations

**Definition (three-way differential, frozen).** For every fixture instance: the **reference** arm
(`compositional_planning`, owned, f32 permitted), the **packed** arm (the same algorithm reading the
§5 sections), and the **production** arm (the §6 deployed planner in the `R4Engine` path) must agree
on the emitted plan, the decline reason, and the witness verdict. **Guarantee (reference, packed, and
production agree on valid fixtures; all three fail closed on corrupt or incompatible data).
Status: Structural** — a built differential test; disagreement fails the gate rather than selecting a
winner.

**Definition (planted mutations, frozen).** Each mutation is planted and the named detector must fire:

| Planted mutation | Detector that must fire |
|---|---|
| flipped bit in a `PTRN` effect delta | witness replay `Invalid` at the affected step |
| flipped bit in the `PTRN` operator index | stage-2 validation typed error |
| removed rule | `Decline(no_plan)` or `Decline(unknown)`, never a fabricated plan |
| duplicated rule with a different delta | conflict detection, then `Decline(unknown)` |
| corrupted witness step | independent replay `Invalid` at that step |
| corrupted terminal state in a witness | independent replay `Invalid` at the terminal check |
| corrupted manifest CID | loader fails closed |
| truncated section | stage-1 validation typed error |

**Guarantee (planted-negative detection). Status: Structural** — a built test asserts every row fires
its detector, and that a mutation which fires *no* detector fails the suite.

---

## 10. Measurement protocol and run contract

### 10.1 Protocol

Primary metric, sample size, horizons, effect floor, and multiple-comparison handling are the frozen
#844 §2.5 values, unchanged: **held-out valid-plan rate**; n = 512 instances per held-out cell per
horizon; H ∈ {1, 2, 4, 8}; δ_min = 0.05; Holm–Bonferroni across the (axis × horizon × family) grid.
Secondary metrics are the #844 §2.5 list, including the honest-decline rate and the resource envelope.
The #844 §2.6 sealed composition and topology cells are **not opened** by this issue; they remain
reserved for #846.

### 10.2 Run contract

    metric to move:       held-out valid-plan rate; current deployed value 0.000 (no deployed
                          planner exists; the deployed artifact is a teacher-forced
                          retrieval/continuation system per #824)
    reachability ceiling: corpus-observation arm - 0 of 10 typed object kinds present in the v4
                          88-byte record, coverage 0.000 < delta_min 0.05, so that arm does not
                          launch (section 0.3). Benchmark arm - pre-repair, 13 of 20 frozen cells
                          0/512 solvable and the strongest structure-keyed null saturated at
                          1.0000, so the ceiling on (arm minus strongest null) was at most 0 and
                          the fit did not launch either. After Amendment A1 the ceiling is
                          (1 minus strongest non-oracle null), required by the A1 gate to be at
                          least delta_min = 0.05 in every one of the 20 cells before any fitting.
    instrument + verdict: the four A1 gate instruments of section 2. All four must pass. The
                          pre-repair run of the same instrument returned DEGENERATE and is what
                          stopped the fit.
    exit rule:            lower an arm only if its one-sided 95% lower bound of
                          (arm minus strongest non-oracle null) exceeds delta_min = 0.05 on every
                          required axis and horizon under Holm-Bonferroni, and it meets the
                          section 4.2 capacity and section 6 operation budgets.
    if positive:          freeze that arm as the non-geometric production planning baseline;
                          record the trigger that releases #845 from on-hold; hand the sealed
                          partitions and the witness schema to #846.
    if negative:          publish the limiting coverage, induction, or planning diagnosis; keep
                          typed planning NOT ESTABLISHED; leave #845 on-hold; do not use geometry
                          to substitute for missing semantics or evidence.
    cost estimate:        the generator and all arms are synthetic, deterministic, and
                          teacher-free. The pre-repair probe over 4096 seeds x 5 families x 5
                          horizons ran in 15 s. The full grid (7 arms x 5 families x 4 horizons x
                          6 axes x n=512) is projected at minutes, not hours, on one core, with no
                          fixture and no network. No long-run publication is required; the exact
                          wall-clock, peak RSS, artifact bytes, expansions, and integer operations
                          are recorded with the results.

Positive and negative branches lead to different next actions, so the measurement has decision value.

---

## 11. Conformance mapping and built-capability order

**Existing ids, evidence language extended:** RF-01 (behavioral probes), RF-08 (bounded future-state
planning — reference), RF-12 (lowering reference regions to Boolean/mask/popcount/fixed-point),
RF-13 (packed zero-allocation kernels), RF-27 (semantic state space and typed transition dynamics —
reference), RF-28 (state transitions separated from language emission — reference), RF-32
(compositional-planning benchmarks — certifier instrument). **None of these is deployed-planning
evidence.**

**New built capability (only if built): RF-33 `bounded_semantic_transitions`** —
**normative deployed-serving** scope, covering the §6 planner, the §5 sections, and the §7 witness.
It is registered only when the §8 selection actually lowers an arm; a negative verdict registers no
new deployed capability. Built-capability order, unchanged from repository policy:
`model/ids.toml` → tagged Gherkin (`features/suites/bounded_semantic_transitions.feature`) →
failing marker in `crates/repo-conformance/tests/registered.rs` plus executable steps in root
`tests/bdd.rs` → implementation → `xtask check-model --write` to regenerate `CONFORMANCE.md`.
**`CONFORMANCE.md` is never hand-edited.**

**Term discipline.** Public language stays "typed state transitions", "bounded planning", and the
exact task and horizon measured, per #844 §4. No unqualified general-reasoning claim is made anywhere
in this issue's output.

---

## 12. Increment plan

| # | Increment | Scope | Ships |
|---|---|---|---|
| 1 | design freeze | this document and #844 §11 Amendment A1 | docs only |
| 2 | benchmark repair | Amendment A1 plus the four gate instruments (§2) | compiler, RF-32 surface |
| 3 | observation and induction | §3 record, inducer, dedup, conflict, split enforcement, leakage scan | compiler |
| 4 | packed sections | §5 `PSCH`/`PTRN`/`PGOL`/`PWIT`, two-stage validation, negatives, fuzz | format |
| 5 | deployed planner | §6 planner, §7 witness, `R4Engine` wiring, allocation census, P-4 scan, `no_std`, wasm | runtime |
| 6 | measurement and verdict | §8 arms and nulls, §9 differentials and planted mutations, §10 measurement, RF-33 if positive, results record | certify, conformance, docs |

Each increment is an independently green pull request through the full merge-queue ladder. An
increment that fails its own gate stops the sequence and is recorded, rather than being routed around.

---

## 13. What this issue establishes

**Claim status on completion.** A positive outcome establishes the **strongest non-geometric
production planning baseline** on the repaired #844 benchmark, at the exact typed tasks and horizons
measured, and nothing more: it is not a general reasoning claim, not free-running coherence (#824),
and not a calibrated-confidence claim (#823). A negative outcome establishes the limiting diagnosis
and keeps typed planning **NOT ESTABLISHED**. Either way, the S4 promotion verdict is #846's to make,
against the sealed partitions this issue does not open.
