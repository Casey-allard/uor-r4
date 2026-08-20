# Persistent prompt-conditioned predictive state — specification (#835)

- **Issue:** #835 — "spec/#822-A: define persistent causal prompt/session state and its
  compiler–runtime boundary" (item A of S1 tracker #822, programme #820).
- **Parent tracker:** #822 (S1 — persistent prompt-conditioned predictive state).
- **Date:** 2026-08-20.
- **Status:** Frozen experimentable **contract** (reference specification). This document
  freezes the semantics an S1 conditioning arm must implement and the controls it must clear.
  It does **not** establish that persistent state improves prompt causality — that is the
  measured question of #834, and the deployed lowering/certification is #836.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md).
  Every labeled statement carries exactly one claim class (**Definition**, **Objective**,
  **Guarantee**, **Assumption**, **Empirical Criterion**); every **Guarantee** and
  **Empirical Criterion** carries a status (**Structural**, **Witnessed**, **Empirical**,
  **Assumed**, **Unproven**). Records are append-only.
- **Machine-checked evidence:** the reference model and its determinism, capacity, saturation,
  reset, tie-break, witness-replay and planted-negative controls run in
  `crates/uor-r4-api/tests/prompt_state_spec_835.rs` (default `cargo test`).

## 1. Problem and scope

#784 measured **0.0% full-depth context-code collisions** yet 11/15 distinct rows still
selected the same continuation: distinct prompts converge on a shared marginal continuation
(continuation-distribution convergence), and the deployed NGRAM/EXCT evidence path is chiefly
suffix-local. There is today no persistent whole-prompt state carried across generated tokens.
"Bidirectional anchors" is not yet a causal, bounded, packable semantics: without a reference
state transition, candidate/support effects, capacity/eviction rules, and witness fields, an
experiment cannot distinguish prompt information from suffix leakage or decoder noise.

**Definition (execution scope of this spec).** This is a **reference-only / off-serving-path**
specification in the sense of [`docs/conformance_execution_scope_830.md`](conformance_execution_scope_830.md):
it defines (a) an offline **reference model** of persistent prompt state with independent
reference semantics, and (b) the **deployed integer/table** contract a lowering must satisfy.
The reference model is not deployed-serving evidence. The deployed capability — the packed
section, the hot-path transition, and its regenerated conformance — is the scope of **#836**.

Non-goals (from #822/#835, honored): no access to future target/completion tokens at inference;
random sampling, repetition penalties, and prompt-template injection are **not** persistent
semantic state; no commitment to W(3,3) or any geometry before the #834 bake-off.

## 2. State object and lanes

**Definition (persistent prompt state `Ψ`).** The persistent prompt state is a fixed-capacity
tuple `Ψ = (L_sfx, L_seg, L_role, L_ent, L_hist, L_con, clk)` of six bounded **lanes** plus a
monotone integer step counter `clk`. `Ψ` refines the semantic-state binding `S` of
`docs/formal_vocabulary.md` §3 (today a "frontier, rolling context code, token shortlist"
approximation in `RuntimeState`); it adds no floating-point or heap state to the deployed path.
Each lane is a fixed-capacity ring of typed **slots**; a slot is
`{ key: u64, weight: ScoreQ, born: u32, touched: u32, contribution_id: u32 }` with integer
fields only.

**Definition (the six lanes and their causal interventions).** Each lane is justified by one
falsifiable intervention on the frozen S1 suite `s1-causal-prompt-pairs`
(`crates/uor-r4-api/capability_suites/causal_prompt_pairs.json`; primary metric
`causal-influence-delta`) or its secondary `s1-continuity-text`:

| Lane | Symbol | Carries | Bounded deployed representation | Binding causal intervention |
|---|---|---|---|---|
| Local suffix | `L_sfx` | rolling last-k token code (the current NGRAM/EXCT surface) | fixed-k ring of token codes | `suffix-only` control: a suffix-only arm must **not** reach the paired-prompt delta |
| Prompt segment | `L_seg` | quantized digest of each supplied prompt segment | fixed-capacity segment-key ring | `prompt-swap` control: swapping the prompt to an unrelated one collapses the delta |
| Role / turn | `L_role` | chat role and turn index of the active span | small fixed enum + counter | role-shuffle: permuting role tags must not raise the delta |
| Entity / topic | `L_ent` | quantized topic/entity signatures observed in the prompt | fixed-capacity signature ring | entity-substitution: replacing entities changes the intended slots |
| Session history | `L_hist` | decayed digest of prior turns in a continued session | fixed-capacity decayed ring | `shuffled-state`: destroying carryover order collapses continuity |
| Explicit constraints | `L_con` | caller-declared constraints / forbidden regions | fixed-capacity constraint ring | constraint-drop: removing a constraint restores a forbidden continuation |

**Guarantee (bounded state). Status: Structural** (reference model;
`prompt_state_spec_835.rs::state_is_fixed_capacity`). Every lane has a compile-time capacity
`C_lane`; `Ψ` never grows beyond `Σ C_lane` slots. Deployed, `Ψ` is a field of the pre-allocated
`RuntimeState` and adds zero steady-state allocation. **Guarantee (deployed allocation-freedom).
Status: Unproven** here — it is a requirement the #836 lowering must satisfy and is established
by that issue's allocation census, not by this reference specification.

## 3. Initial fold and per-token transition (causal)

**Definition (initial fold `Φ₀`).** Given the complete supplied prompt `p = (t₀ … t_{m-1})` and a
prior state `Ψ⁻` (empty for a fresh request; the continued state for a session), the initial fold
`Φ₀(p, Ψ⁻) = Ψ₀` folds the whole prompt left-to-right into the lanes before the first generated
token. `Φ₀` reads the entire prompt (this is the "bidirectional" reading of the prompt) but reads
**no** completion token. It is deterministic and order-defined.

**Definition (per-token transition `T_ps`).** For each generated position `i`, given the state
`Ψ_i`, the just-consumed token `t` (the position's teacher-forced or previously-emitted token),
and the immutable artifact `A`, the transition
`T_ps(A, Ψ_i, t) = Ψ_{i+1}` updates the lanes and increments `clk`. `T_ps` reads only past and
current tokens; it never reads a future completion token. Its deployed shape mirrors the existing
hot-path step `infer_step(graph, state, token, output)` /
`R4G1Runtime::predict_step` (`docs/inference_contract.md` §2): `Ψ` is mutated in place in the
pre-allocated `RuntimeState`.

**Guarantee (causal / no future access). Status: Structural** (reference model;
`prompt_state_spec_835.rs::transition_is_causal`). `T_ps` and `Φ₀` are pure functions of
`(A, Ψ, tokens_≤i)`; a test constructs two continuations that differ only in a future token and
asserts identical state and contributions at every position `≤ i`.

**Definition (allowed deployed operations).** The deployed `Φ₀`/`T_ps` execute only the permitted
operation classes of `docs/inference_contract.md` §3 — bitwise, shift/rotate, popcount/bit-count,
integer add/sub (saturating/wrapping), comparison, and fixed-offset table reads. No multiply,
divide, float, clock, network, or RNG. Lane decay is a right shift (a division-free halving
schedule), never a multiply or divide; eviction and tie-breaking are comparisons; digests are
XOR/rotate/popcount folds. The offline compiler that *learns* lane parameters, decay schedules,
and quantized residual tables is unconstrained (f32, allocation, tensors) per §5.

## 4. State → support, evidence, and score contributions (decode-independent)

**Definition (contribution lowering).** `Ψ` influences prediction only by emitting bounded signed
`ScoreQ` **residual contributions** into the normative accumulator of
[`docs/scoring_semantics.md`](scoring_semantics.md) (i32, Q16.16), applied by saturating integer
addition in the fixed canonical order, each carrying a unique 32-bit `contribution_id` under the
no-double-counting rule. `Ψ` reuses the existing residual taxonomy — it introduces no new
contribution kind:

- candidate **support** widening/narrowing → `InteractionResidual` (co-occurrence between an
  active lane slot and a candidate),
- constraint proximity (from `L_con`) → `ConstraintPenalty` (emitted non-positive),
- session/goal consistency (from `L_hist`, `L_ent`) → `GoalReward` (emitted non-negative),
- uncertainty from lane saturation/eviction pressure → `UncertaintyPenalty` (non-positive).

**Guarantee (independence from decoding). Status: Structural** (reference model;
`prompt_state_spec_835.rs::contributions_independent_of_decode`). The contributions `Ψ` emits at
position `i` are a function of `(A, Ψ_i, candidate set)` and are identical under greedy, seeded-
sampling, or teacher-forced decoding of the *same* consumed tokens. Sampling changes which token
is consumed next, hence the next state; it never changes the contribution computed for a given
`(Ψ_i, candidate)`. This is the RF-28 "separate state transitions from language emission"
property instantiated for prompt state.

**Definition (resolution-path attribution).** A token whose selection is moved by a `Ψ`
contribution is attributed on the existing `graph` `ResolutionPath`
(`uor-r4-api::capability_suite::ResolutionPath`); `Ψ` adds no new path and binds the one
normative scorer `uor-r4-graph-format::scoring_semantics@1.0.0`
([ADR-0001](adr/0001-normative-r4g1-scorer.md), #831). A position where no lane is active
resolves exactly as today (EXCT/NGRAM/root-prior), so the state is additive over the baseline.

## 5. Compiler-learned versus deployed-integer boundary

**Definition (offline / deployed split).** Two disjoint quantity sets:

- **Compiler-learned (offline; f32/allocation permitted).** Lane capacities `C_lane`, decay shift
  schedules, key-quantization tables, and the quantized residual weight tables that map a lane
  slot + candidate to a `ScoreQ` contribution. These are learned by behavioral probing (RF-01)
  during compilation and frozen into the artifact. They are **Objectives** of the compiler, never
  runtime invariants.
- **Deployed fixed-point/table (hot path; integer/table only).** The lane rings, the `clk`
  counter, the `ScoreQ` contributions, and fixed-offset reads into the frozen residual tables.
  Every deployed operation is enumerated in §3.

**Definition (allowed hot-path operation enumeration).** The complete deployed operation set for
`Φ₀`, `T_ps`, and the contribution lowering is exactly: `XOR/AND/OR/NOT`, left/right shift and
rotate, `popcount`/`cttz`/`ctlz`, `saturating_add`/`saturating_sub`/`wrapping_add`/
`wrapping_sub`, integer comparison, and fixed-offset immutable table reads — the closed set of
`docs/inference_contract.md` §3. Any lowering using an operation outside this set is
non-conforming.

## 6. Capacity, eviction, decay, saturation, tie-breaking, and typed errors

**Definition (deterministic eviction and decay).** When a lane is at capacity and a new slot must
be inserted, the **lowest-priority** slot is evicted, priority being `weight` after decay, with
ties broken by the tie-break order below. Decay is applied on each `T_ps` step as an arithmetic
right shift of `weight` by the lane's decay shift (a halving schedule), floored at zero; a slot
whose decayed weight reaches the floor and is not refreshed is evictable. No two distinct slots
in a lane share a `key` (a repeated key refreshes `touched` and re-accumulates `weight` under
saturation, it does not add a second slot).

**Definition (overflow, saturation, decline).** All weight arithmetic uses saturating `ScoreQ`
addition: `ScoreQ::MAX`/`ScoreQ::MIN` clamp without panic (`docs/scoring_semantics.md` §5). When a
required lane cannot admit a slot without violating its capacity or ordering invariants, the
transition returns a typed **decline** rather than silently dropping evidence.

**Definition (canonical tie-break).** Slot ordering, eviction ties, and any candidate ranking that
`Ψ` participates in use the one canonical order of `docs/scoring_semantics.md` §6: **`ScoreQ`
descending, then key/id ascending**. This is total and platform-independent.

**Guarantee (typed errors, no panic). Status: Structural** (reference model;
`prompt_state_spec_835.rs::invalid_capacity_and_index_are_typed_errors`). Constructing a lane with
zero capacity, inserting at an out-of-range index, or requesting an undefined lane returns a
focused error value; no reference-model path panics on a recoverable input. The deployed lowering
carries the same obligation as a **Guarantee (Unproven here; #836)** discharged by that issue's
tests.

**Guarantee (determinism). Status: Structural** (reference model;
`prompt_state_spec_835.rs::determinism`). `Φ₀` and repeated `T_ps` from the same
`(prompt, artifact parameters, prior state)` produce byte-identical state and byte-identical
contribution sequences. Identical pinned inputs ⇒ identical bytes (byte reproducibility); there is
no HashMap-iteration, clock, or RNG dependence.

## 7. Reset and continuation semantics

**Definition (request boundaries).** State lifetime is caller-owned and versioned:

- **Native single request.** `Ψ⁻` is empty; `Φ₀` folds the prompt; `Ψ` is discarded at end of
  request. No cross-request carryover.
- **OpenAI-compatible chat.** Each turn's supplied `messages` are folded in order by `Φ₀`;
  `L_role`/`L_hist` capture role and prior-turn digests. A new `messages` array with no server-
  side handle re-folds from empty (stateless-compatible); a caller that passes a state handle
  continues `L_hist`/`L_ent` under the decay schedule.
- **Streaming.** Streaming changes only *when* tokens are emitted, not the state: `T_ps` runs per
  emitted token exactly as in non-streaming; a mid-stream cancel discards `Ψ` at teardown.
- **Supported WASM hosts.** A host without persistent-state support (or an artifact without the
  §8 section) runs with all lanes empty and declines the state feature — behavior is identical to
  the pre-state baseline; no host is required to allocate to support the feature.

**Guarantee (reset clears; continuation preserves). Status: Structural** (reference model;
`prompt_state_spec_835.rs::reset_and_continuation`). A reset yields the empty-`Ψ` fold of the same
prompt; a continuation with handle `h` reproduces the decayed carryover deterministically.

## 8. Artifact format, versioning, and old-artifact behavior

**Definition (proposed optional `PSTATE` section).** Compiler-learned parameters are carried in a
new **optional** R4G1 section `PSTATE` in the optional-kind space of
[`docs/transformerless/R4G1.md`](transformerless/R4G1.md) §3 (`kind & 0x80 != 0`; a concrete
kind byte is assigned by #836 when the section is implemented, alongside `NGRAM`/`FWDA`). It is
gated by a `feature_bits_required` bit exactly as the existing optional lexical sections are.
`PSTATE` carries, in fixed field order: a schema version, the per-lane capacities and decay
shifts, the key-quantization table identities, and the quantized residual-weight table (all
integer; byte-layout constraints identical to the packed-array discipline of the NODE/EMIT
sections). Its bytes are content-bound and participate in byte reproducibility.

**Guarantee (old-artifact behavior preserved). Status: Assumed** (contract for #836).
An artifact without a `PSTATE` section, or a client that does not advertise the feature bit,
behaves **identically** to the current baseline: all lanes are empty, `Ψ` emits no contribution,
and every position resolves as it does today. This is the versioned-optional-section rule of
R4G1.md §8; it is stated here as the compatibility obligation the #836 implementation must
discharge (behavioral equivalence with the baseline on the absent-section path, verified by a
round-trip test in #836), and is not claimed proven by this specification.

**Definition (request-state ABI).** Any request-carried state handle is a versioned, caller-owned
opaque token; an ABI change requires versioned negotiation and a migration/recompile note. The
reference model treats the handle as an integer session id.

## 9. Reference model and witness fields

**Definition (executable reference model).** The reference model is the deterministic realization
of §§2–7 in `crates/uor-r4-api/tests/prompt_state_spec_835.rs`: the `Ψ` tuple, `Φ₀`, `T_ps`, the
contribution lowering, and eviction/decay/tie-break, using integer `ScoreQ` arithmetic. It is a
**reference/f32-free** model in the RF-27/RF-28 sense (owned, offline, off the serving path); it
is the semantics #834 fits arms against and #836 lowers, not a deployed artifact.

**Definition (witness schema).** Every transition emits a bounded, replayable **witness** record
sufficient to reconstruct the state change and every contribution without the teacher:
`{ step: u32, token: u32, lane_touched: u8, slot_key: u64, weight_before: i32, weight_after: i32,
evicted_key: Option<u64>, contribution_id: u32, contribution_kind: u8, contribution_q: i32 }`,
serialized as a fixed-width little-endian byte record. An independent verifier replays the witness
sequence and reproduces `Ψ` and the contribution stream.

**Guarantee (witness replay). Status: Witnessed** (reference model;
`prompt_state_spec_835.rs::witness_replay`). Replaying the emitted witness sequence reconstructs
the final state and the full contribution stream byte-for-byte from a fresh empty `Ψ`.

## 10. Budgets (projected before fitting)

**Objective (deployed budgets, projected).** Projected from the fixed capacities before #834
fits any implementation-specific arm (these are ceilings the lowering targets, not measured
results):

- **Artifact bytes:** `PSTATE` ≈ `Σ_lane (C_lane · slot_stride)` for the residual tables plus a
  small fixed header; targeted at the same order as the existing optional `NGRAM` section, not a
  multiple of the NODE/EMIT core.
- **Table reads / token:** `O(active_lanes · C_lane)` fixed-offset reads, bounded by `Σ C_lane`;
  no unbounded scan.
- **Operations / token:** bounded by the same `Σ C_lane` with a small constant per slot (a fold, a
  compare, a saturating add); no per-token growth with sequence length.
- **Caller-owned state:** `Σ_lane (C_lane · slot_bytes)` + `clk`; a single fixed-size struct,
  independent of prompt length once folded.

These budgets bind the #834/#836 fit: an arm exceeding a projected ceiling must either re-specify
the capacity here (append-only) or be declined.

## 11. Benchmark and negative controls (falsifiers)

**Empirical Criterion (S1 causal benchmark). Status: Empirical** (to be measured in #834; here it
is the frozen protocol, not a result). The primary S1 statistic is `causal-influence-delta` on
`s1-causal-prompt-pairs`, teacher-forced, EXCT-disabled, document- and template-disjoint: the
paired-prompt next-token divergence induced by `Ψ` must exceed the `prompt-swap` and `suffix-only`
null lower bounds with a positive confidence-bound margin, on two domains, surviving paraphrase
within a frozen stability tolerance. The secondary `s1-continuity-text` (`continuity-top1`) must
exceed the suffix-only floor with the emission binding intact (`shuffled-emission` control
separates). Metric values are exact integer fractions (`numerator/denominator`), never floats
(`uor-r4-api::capability_suite::MetricStatus`).

**Definition (planted negatives / falsifiers).** The benchmark must **fail** on a non-causal
implementation. Two planted negatives are mandatory, expressed with the frozen control vocabulary
`uor-r4-api::capability_suite::ControlKind`:

- **`PromptSwap` / `SuffixOnly`** (causal-influence nulls): a `Ψ` that carries no prompt-segment
  information cannot separate from these controls.
- **`ShuffledState`** and a **constant-state** control: destroying typed-state carryover, or
  replacing `Ψ` with a fixed constant, collapses the delta to the null.

**Empirical Criterion (anti-vacuity guard). Status: Structural** (the guard itself is tested;
`prompt_state_spec_835.rs::planted_negatives_have_teeth`). Before any comparison, the causal arm is
asserted **non-degenerate** against the control (`is_degenerate_control` must be false for a genuine
causal reference model); and a constant-state / shuffled-state reference model is asserted
**degenerate** (`is_degenerate_control` true) — i.e. the instrument distinguishes a causal from a
non-causal state, so a zero reading means "no effect", not "broken harness". Two empty runs compare
equal and are rejected.

## 12. Repository conformance mapping

**Definition (RF mapping).** This specification extends the evidence of existing capability IDs; it
introduces **no new built RF capability** and adds **no** `model/ids.toml` row and **no**
`CONFORMANCE.md` regeneration (the #832 precedent for a spec/infrastructure leaf that binds
existing capabilities):

- **RF-27** (semantic state space and typed transition dynamics — reference/f32) and **RF-28**
  (separate semantic state/transitions from emission — reference/f32): `Ψ`, `Φ₀`, `T_ps`, and the
  decode-independent contribution lowering are the persistent-prompt instantiation of `S`/`T` and
  of the state/emission separation.
- **RF-01** (unsupervised intervention and counterfactual behavioral probes): the per-lane causal
  interventions and the planted negatives of §11.
- **RF-21** (R4G1 compilation quality gates) and **RF-22** (R4G1 pathology filter): the compile-
  side learning of lane parameters and the synthetic-input controls.

**Definition (built-capability order for #836).** When #836 lowers the *selected* arm into a
deployed capability, it follows the required order — `model/ids.toml` row → tagged Gherkin →
failing marker/behavior test → implementation → regenerated `CONFORMANCE.md` — and either extends
an existing suite or justifies a new capability then. The dormant/unselected arms remain ledgered
with activation or retirement gates. Generated conformance is never hand-edited.

## 13. Acceptance-criteria status

Requirements are frozen as a contract; the reference-model evidence file is
`crates/uor-r4-api/tests/prompt_state_spec_835.rs`.

- [x] Reference semantics determine the same state and score contributions from the same prompt,
  artifact, and prior state — §6 determinism (Structural); test `determinism`.
- [x] Every lane has a causal benchmark intervention and a bounded deployed representation —
  §2 table; §11.
- [x] Absent sections preserve historical behavior; invalid capacities/indices fail with typed
  errors — §8 (Assumed compatibility obligation for #836); §6 typed errors (Structural); test
  `invalid_capacity_and_index_are_typed_errors`.
- [x] The specification includes byte/operation/state budgets and explicit witness replay
  semantics — §10 budgets; §9 witness (Witnessed); test `witness_replay`.
- [x] No deployed transition uses multiply, divide, floating point, heap allocation, clock,
  network, or RNG — §3/§5 allowed-op enumeration (a **Definition** of the deployed contract; the
  deployed-path proof is discharged by #836's allocation census and P-4 scan).

## 14. Verification

- Reference-model unit/property tests (determinism, causality, capacity edges, saturation, reset,
  tie-breaks, witness replay, typed errors) and the prompt-shuffle / constant-state planted
  negatives: `cargo test -p uor-r4-api --offline --test prompt_state_spec_835`.
- Claim-wording gate over this document: `python3 scripts/check_claim_wording.py`.
- Register/deferral/limits unaffected (no new capability row):
  `cargo run -q -p xtask -- validate`.

## 15. Claim status and next action

**This document freezes an experimentable contract; it does not establish that persistent state
improves prompt causality.** The next action is **#834**, which fits prompt-conditioned evidence
arms against these causal and equal-budget controls and selects (or rejects) an arm; a selected
arm is then lowered and certified in **#836**. A negative #834 verdict narrows or retires the S1
conditioning claim under the #822 kill/redesign criterion and is first-class completion evidence.
