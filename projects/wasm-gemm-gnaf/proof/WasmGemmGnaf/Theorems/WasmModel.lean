/-
  Theorems: the formal WebAssembly layer.

  This module is an INDEX, not a source of new mathematics.  Every declaration
  below restates a proposition already proved in `WasmGemmGnaf/Wasm/` and closes
  it by `exact`-ing the original.  Restating rather than `export`ing is
  deliberate: the proposition is written out here in full, so SPEC §15's
  required-name list can be checked against the *statements* in one place
  instead of trusting that a re-exported name still says what it used to say.

  ## SPEC §15 declarations discharged here

  | SPEC §15 name                     | discharged by                             |
  |-----------------------------------|-------------------------------------------|
  | `Wasm.mem_successors_iff_step`    | `Theorems.mem_successors_iff_step`        |
  | `Wasm.costed_erase_iff_plain_run` | `Theorems.costed_erase_iff_plain_run`     |

  `Wasm.costed_erase_iff_plain_run` is discharged in the amended form recorded
  as `DEV-001` in `model/spec-deviations.json`: the literal SPEC §7.5
  biconditional is false for an arbitrary cost labelling, so the right-hand side
  carries the side condition `CostedLabelling`, and the unconditional intent is
  discharged separately by `Theorems.costed_run_iff_plain_run`.

  ## Additional proved results indexed here (not on the §15 list)

  `encode_decode_roundtrip`, `decode_is_encode`, `decode_error_or_encode`,
  `validate_bool_iff`, `successors_nodup`, `fault_decoding_ne_instantiation`
  (with the two injectivity halves), `costed_run_iff_plain_run`,
  `wasm_cost_table_total`.

  ## SPEC §15 Wasm declarations that remain OUTSTANDING

  * `Wasm.decode_sound`, `Wasm.decode_complete` — these are stated in SPEC §7.3
    against the vendored `DeclarativeBinaryRelation`, which this repository does
    not mechanize.  `Wasm/Binary.lean` proves the intrinsic pair
    `encode_decode_roundtrip` / `decode_is_encode` instead, and says so in its
    own doc comment; that is strictly weaker than a statement about the pinned
    declarative grammar and is therefore NOT claimed under the §15 names.
    Blocking obligation: `O-6`.
  * `Wasm.validate_iff_declarative` — `validate_bool_iff` decides
    `Wasm.DeclarativelyValid`, the *repository's* declarative judgment for the
    `i32` executable subset, not Core 3.0 validation.  Blocking obligation:
    `O-6`.
  * `Wasm.validation_progress` — no progress theorem exists for the modelled
    subset.  Blocking obligation: `O-6`.
  * `Wasm.bounded_tree_covers_every_branch` — absent.  Blocking obligation:
    `O-6`.
  * `Wasm.costed_initialization_erase` — absent; `Wasm/Erasure.lean` proves
    erasure for the reduction phase, not for initialization.  Blocking
    obligation: `O-6`.
  * `Wasm.profile_matches_pinned_revision` — absent.  Blocking obligation:
    `O-6`.

  `Wasm.runFuel_sound` and `Wasm.runFuel_complete_with_bound` exist at their
  §15 names in `WasmGemmGnaf/Wasm/Fuel.lean` and are not re-indexed here.
-/
import WasmGemmGnaf.Wasm.Binary
import WasmGemmGnaf.Wasm.Validate
import WasmGemmGnaf.Wasm.Step
import WasmGemmGnaf.Wasm.Fault
import WasmGemmGnaf.Wasm.Erasure

set_option autoImplicit false

namespace WasmGemmGnaf.Theorems

/-! ## Binary format: decode/encode round trip -/

/-- **Round trip.**  Decoding an encoded module returns exactly that module. -/
theorem encode_decode_roundtrip (m : Wasm.Module) :
    Wasm.decode (Wasm.encode m) = .ok m :=
  Wasm.encode_decode_roundtrip m

/-- **Decoder soundness.**  A byte string that decodes at all is the encoding of
the module it decodes to, so a malformed input never yields a wrong module. -/
theorem decode_is_encode {b : ByteArray} {m : Wasm.Module}
    (h : Wasm.decode b = .ok m) : b = Wasm.encode m :=
  Wasm.decode_is_encode h

/-- **Decoder totality.**  Every byte string either produces a typed fault or
produces the unique module whose encoding it is. -/
theorem decode_error_or_encode (b : ByteArray) :
    (∃ f : Wasm.DecodeFault, Wasm.decode b = .error f) ∨
    (∃ m : Wasm.Module, Wasm.decode b = .ok m ∧ b = Wasm.encode m) :=
  Wasm.decode_error_or_encode b

/-! ## Validation -/

/-- The executable validator decides the declarative validity judgment of this
repository's Wasm subset. -/
theorem validate_bool_iff (m : Wasm.Module) :
    Wasm.validate m = true ↔ Wasm.DeclarativelyValid m :=
  Wasm.validate_bool_iff m

/-! ## Reduction: the executable successor enumeration is the relation -/

/-- **SPEC §15, `Wasm.mem_successors_iff_step`.**  The executable successor
enumeration is exactly the reduction relation — the anti-cheat property. -/
theorem mem_successors_iff_step (c : Wasm.Config) (e : Wasm.Event) (c' : Wasm.Config) :
    (e, c') ∈ Wasm.successors c ↔ Wasm.Step c e c' :=
  Wasm.mem_successors_iff_step c e c'

/-- The successor enumeration is duplicate free, so counting successors counts
distinct reducts. -/
theorem successors_nodup (c : Wasm.Config) : (Wasm.successors c).Nodup :=
  Wasm.successors_nodup c

/-! ## Fault disjointness -/

/-- The two failure domains of `Wasm.Fault` are disjoint: a decoding failure can
never be silently reported as an instantiation failure. -/
theorem fault_decoding_ne_instantiation
    (a : Wasm.DecodeFault) (b : Wasm.InstantiationFault) :
    Wasm.Fault.decoding a ≠ Wasm.Fault.instantiation b :=
  Wasm.Fault.decoding_ne_instantiation a b

/-- The decoding injection loses no information. -/
theorem fault_decoding_injective {a b : Wasm.DecodeFault}
    (h : Wasm.Fault.decoding a = Wasm.Fault.decoding b) : a = b :=
  Wasm.Fault.decoding_injective h

/-- The instantiation injection loses no information. -/
theorem fault_instantiation_injective {a b : Wasm.InstantiationFault}
    (h : Wasm.Fault.instantiation a = Wasm.Fault.instantiation b) : a = b :=
  Wasm.Fault.instantiation_injective h

/-! ## Cost erasure -/

/-- **SPEC §15, `Wasm.costed_erase_iff_plain_run`** in the `DEV-001` amended
form.  A costed run of `costedTrace` holds exactly when the ordinary run of
`eraseCosts costedTrace` holds *and* `costedTrace` is a labelling the machine
produces.  The side condition is necessary: without it the backward direction is
false, because `costedTrace` is universally quantified. -/
theorem costed_erase_iff_plain_run {P : Wasm.Profile} {module : Wasm.Module}
    {invocation : Wasm.RawInvocation} {costedTrace : List Wasm.CostedEvent}
    {observation : Wasm.ExecutionObservation} :
    Wasm.CostedRun P module invocation costedTrace observation ↔
      (Wasm.Run module invocation (Wasm.eraseCosts costedTrace) observation ∧
        Wasm.CostedLabelling module invocation costedTrace) :=
  Wasm.costed_erase_iff_plain_run

/-- **Cost instrumentation is transparent**, with no side condition.  This is
the unconditional content SPEC §7.5 intends: an observation is reachable by a
costed run exactly when it is reachable by the ordinary run of its own trace. -/
theorem costed_run_iff_plain_run {P : Wasm.Profile} {module : Wasm.Module}
    {invocation : Wasm.RawInvocation} {observation : Wasm.ExecutionObservation} :
    (∃ costedTrace : List Wasm.CostedEvent,
        Wasm.CostedRun P module invocation costedTrace observation) ↔
      Wasm.Run module invocation observation.trace observation :=
  Wasm.costed_run_iff_plain_run

/-! ## Cost-table totality -/

/-- **SPEC §7.5, cost-table totality and exclusivity.**  Every costed event has
exactly one contribution: totality fails if a rule is left uncosted, exclusivity
fails if a rule is costed twice. -/
theorem wasm_cost_table_total (t : Wasm.CostTableBody) (event : Wasm.CostedEvent) :
    ∃ contribution : Cost.DynamicVector,
      Wasm.EventContribution t event contribution ∧
        ∀ other : Cost.DynamicVector,
          Wasm.EventContribution t event other → other = contribution :=
  Wasm.wasm_cost_table_total t event

end WasmGemmGnaf.Theorems
