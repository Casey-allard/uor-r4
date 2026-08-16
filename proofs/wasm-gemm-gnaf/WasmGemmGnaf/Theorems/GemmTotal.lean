/-
  Theorems: the GEMM ABI, the raw-invocation carrier, and the total classifier.

  This module is an INDEX.  Every declaration restates a proposition already
  proved in `WasmGemmGnaf/Gemm/` and closes it by `exact`-ing the original, so
  SPEC §15's required-name list is checkable against full statements in one
  place.

  ## SPEC §15 declarations discharged here

  | SPEC §15 name                   | discharged by                          |
  |---------------------------------|----------------------------------------|
  | `Gemm.classify_total`           | `Theorems.classify_total`              |
  | `Gemm.raw_invocation_roundtrip` | `Theorems.raw_invocation_roundtrip`    |
  | `Gemm.raw_invocation_surjective`| `Theorems.raw_invocation_surjective`   |
  | `Gemm.abi_roundtrip`            | `Theorems.abi_roundtrip`               |
  | `Gemm.classifier_exact_domain`  | `Theorems.classifier_exact_domain`     |
  | `Gemm.reference_total`          | `Theorems.reference_total`             |
  | `Gemm.mandatory_family_nonzero_witnesses` | `Theorems.mandatory_family_nonzero_witnesses` |
  | `Gemm.observation_covers_status_and_full_c` | `Theorems.observation_covers_status_and_full_c` |

  Also indexed, though not on the §15 list: `classify_deterministic` and
  `valid_descriptor_wellFormed`, which together say the classifier is a function
  and that a `valid` verdict really carries a well-formed descriptor; and the
  remaining SPEC §8.4 required proofs `valid_reference_nonempty`,
  `deterministic_mode_unique` and `reference_memory_safe`, together with the
  four rejection theorems that discharge "trapped and uncaught-exception
  observations are rejected by every released problem case".

  ## SPEC §15 Gemm declarations: the input-finiteness pair is now CLOSED

  * `Gemm.valid_input_finite`, `Gemm.raw_input_finite` — **discharged** in
    `Universal/EnumerateInputs.lean`.  `Foundation.Fintype` demands an actual
    duplicate-free `List` covering the carrier, and `Gemm.rawInvocations` is
    that list: `ptr` and `len` range over `Gemm.allUInt32`, `bytes` over
    `Gemm.byteArraysOfSize len.toNat`, and the SPEC §8.3 lawfulness proof is
    attached by a dependent filter-map.  `Gemm.mem_rawInvocations` and
    `Gemm.rawInvocations_nodup` are proved structurally — by induction on the
    construction, never by evaluating a list of about `256 ^ (2 ^ 32)` entries.
    SPEC §8.4's `instance problem_input_fintype : Fintype
    problem.RawInvocation` is the same obligation in instance form and is
    supplied globally, with no instance hypothesis and no `Classical.choice`.
    `Universal/Competitor.lean` still *writes* `[Foundation.Fintype
    (Gemm.RawInvocation P)]` as a binder; that binder is now satisfied by an
    instance in the environment rather than assumed.

  ## Scope of what the reference obligations do and do not say

  `Theorems.reference_total` and its companions are statements about the
  *reference relation* `Gemm.Reference.Accepts` — the specification a candidate
  must meet.  They are not statements about any compiled Wasm module: that link
  is `GNAF.compile_refines`, omitted under `BI-002`/`O-6`.  In particular
  `mandatory_family_nonzero_witnesses` exhibits raw invocations that the
  *classifier and reference arithmetic* accept and evaluate to a nonzero result
  in every mandatory combination; it does not claim the released compiler can
  emit code for them.

  Note on the name `classify_total`: `WasmGemmGnaf.GNAF.classify_total` is a
  different, unrelated theorem about `GNAF.Machine`.  The §15 requirement is the
  `Gemm` one, which is what is re-indexed below.
-/
import WasmGemmGnaf.Gemm.ABI
import WasmGemmGnaf.Gemm.Classify
import WasmGemmGnaf.Gemm.Reference

set_option autoImplicit false

namespace WasmGemmGnaf.Theorems

/-! ## The ABI header round trip -/

/-- **SPEC §15, `Gemm.abi_roundtrip`.**  Encoding a header and decoding it again
returns exactly that header. -/
theorem abi_roundtrip (h : Gemm.RawHeader) :
    Gemm.decodeHeader (Gemm.encodeHeader h) = some h :=
  Gemm.abi_roundtrip h

/-! ## The raw-invocation carrier -/

/-- **SPEC §15, `Gemm.raw_invocation_roundtrip`.**  Decoding the canonical
encoding of a raw invocation returns that invocation. -/
theorem raw_invocation_roundtrip {P : Wasm.Profile} (raw : Gemm.RawInvocation P) :
    Gemm.decodeRawInvocation P (Gemm.encodeRawInvocation raw) = .ok raw :=
  Gemm.raw_invocation_roundtrip raw

/-- **SPEC §15, `Gemm.raw_invocation_surjective`.**  Every lawful
`(ptr, len, bytes)` triple really is a raw invocation: the carrier is not
narrowed by the problem implementation. -/
theorem raw_invocation_surjective {P : Wasm.Profile}
    (ptr len : UInt32) (bytes : ByteArray)
    (hlen : bytes.size = len.toNat)
    (hrange : ptr.toNat + len.toNat ≤ 2 ^ P.addressBits)
    (hpages : Wasm.pagesFor (ptr.toNat + len.toNat) ≤ P.maxPages) :
    ∃ raw : Gemm.RawInvocation P,
      raw.body.ptr = ptr ∧ raw.body.len = len ∧ raw.body.bytes = bytes :=
  Gemm.raw_invocation_surjective ptr len bytes hlen hrange hpages

/-! ## The classifier -/

/-- **SPEC §15, `Gemm.classify_total`.**  The classifier is total: it returns a
classification for every raw invocation body, with no partiality and no default
rejection arm reached by falling off the end. -/
theorem classify_total {P : Wasm.Profile} (raw : Gemm.RawInvocationBody P) :
    ∃ classification, Gemm.classify P raw = classification :=
  Gemm.classify_total raw

/-- The classifier is a function, hence deterministic. -/
theorem classify_deterministic {P : Wasm.Profile} (raw : Gemm.RawInvocationBody P)
    {c₁ c₂ : Gemm.Classification P}
    (h₁ : Gemm.classify P raw = c₁) (h₂ : Gemm.classify P raw = c₂) : c₁ = c₂ :=
  Gemm.classify_deterministic raw h₁ h₂

/-- **SPEC §15, `Gemm.classifier_exact_domain`.**  Every byte-level well-formed
descriptor whose views fit memory and whose tag combination is supported
classifies as valid, unless it violates the explicitly enumerated alias
predicate (part of `DescriptorWellFormed`) or the resource predicate
(`ResourceOk`).  The classifier therefore rejects nothing outside the enumerated
grounds. -/
theorem classifier_exact_domain {P : Wasm.Profile} (raw : Gemm.RawInvocationBody P)
    (h : Gemm.RawHeader)
    (hdec : Gemm.decodeHeader raw.bytes = some h)
    (hmal : Gemm.headerMalformed h = none)
    (htag : Gemm.tagUnsupported h = none)
    (hcomp : Gemm.headerCompatible h)
    (hdim : Gemm.dimensionsRepresentable h)
    (hwf : Gemm.DescriptorWellFormed (Gemm.descriptorOf h) (Gemm.windowOf P raw))
    (hres : Gemm.ResourceOk P raw) :
    ∃ invocation, Gemm.classify P raw = .valid invocation :=
  Gemm.classifier_exact_domain raw h hdec hmal htag hcomp hdim hwf hres

/-- Conversely, a `valid` classification really does produce a descriptor
satisfying every clause of the well-formedness predicate: the exact-domain
theorem above is not achieved by weakening what `valid` means. -/
theorem valid_descriptor_wellFormed {P : Wasm.Profile}
    {raw : Gemm.RawInvocationBody P} {inv : Gemm.ValidInvocation P}
    (h : Gemm.classify P raw = .valid inv) :
    Gemm.DescriptorWellFormed inv.descriptor.body inv.descriptor.window :=
  Gemm.valid_descriptor_wellFormed h

/-! ## The reference relation (SPEC §8.4) -/

/-- **SPEC §15, `Gemm.reference_total`.**  The reference relation is total over
raw invocations: every raw invocation — malformed, truncated, unsupported,
resource-invalid or valid — has an accepted observation, so no downstream
correctness claim can be vacuous for want of a witness. -/
theorem reference_total {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) :
    ∃ observation, Gemm.Reference.Accepts problem raw observation :=
  Gemm.reference_total problem raw

/-- **SPEC §8.4, `Gemm.valid_reference_nonempty`.**  A `valid` classification has
an accepted observation. -/
theorem valid_reference_nonempty {P : Wasm.Profile} (problem : Gemm.Problem P)
    {raw : Gemm.RawInvocation P} {invocation : Gemm.ValidInvocation P}
    (h : Gemm.classify P raw.body = .valid invocation) :
    ∃ observation, Gemm.Reference.Accepts problem raw observation :=
  Gemm.valid_reference_nonempty problem h

/-- **SPEC §8.4, `Gemm.deterministic_mode_unique`.**  Under a deterministic mode
two accepted observations of one raw invocation carry the same semantic
observation. -/
theorem deterministic_mode_unique {P : Wasm.Profile} (problem : Gemm.Problem P)
    {raw : Gemm.RawInvocation P} {a b : Wasm.ExecutionObservation}
    (hmode : problem.ModeDeterministic raw)
    (ha : Gemm.Reference.Accepts problem raw a)
    (hb : Gemm.Reference.Accepts problem raw b) :
    Gemm.semanticFor problem raw a = Gemm.semanticFor problem raw b :=
  Gemm.deterministic_mode_unique problem hmode ha hb

/-- **SPEC §8.4, `Gemm.reference_memory_safe`.**  Acceptance implies that every
candidate-call write stayed inside the sanctioned regions. -/
theorem reference_memory_safe {P : Wasm.Profile} (problem : Gemm.Problem P)
    {raw : Gemm.RawInvocation P} {observation : Wasm.ExecutionObservation}
    (h : Gemm.Reference.Accepts problem raw observation) :
    Wasm.CandidateCallMemoryWritesWithin observation (Gemm.windowOf P raw.body)
      (problem.SanctionedWriteRegions raw (Gemm.semanticFor problem raw observation)) :=
  Gemm.reference_memory_safe problem h

/-! ## Anti-vacuity (SPEC §8.3) -/

/-- **SPEC §15, `Gemm.mandatory_family_nonzero_witnesses`.**  Every mandatory
scalar-kind × compatible arithmetic-mode × transpose × layout-class combination
has a `1×1×1` raw invocation that classifies `valid`, returns status `0`, starts
from `C = 0`, and evaluates under that combination's exact declared arithmetic
to the **nonzero** result `1` of its kind.  `Gemm.mandatoryCases_covers` proves
the family really does contain every such combination. -/
theorem mandatory_family_nonzero_witnesses (P : Wasm.Profile) :
    ∀ c ∈ Gemm.mandatoryCases,
      ∃ (raw : Gemm.RawInvocation P) (inv : Gemm.ValidInvocation P) (h : Gemm.RawHeader),
        Gemm.classify P raw.body = .valid inv ∧
        Gemm.decodeHeader raw.body.bytes = some h ∧
        h.aTag.toNat = c.kind.tag ∧ h.bTag.toNat = c.kind.tag ∧
        h.cTag.toNat = c.kind.tag ∧ h.accTag.toNat = c.acc.tag ∧
        h.modeTag.toNat = c.mode.tag ∧
        (Gemm.descriptorOf h).transposeA = c.transposeA ∧
        (Gemm.descriptorOf h).transposeB = c.transposeB ∧
        (Gemm.descriptorOf h).shapeC = ⟨1, 1, 1⟩ ∧
        Gemm.cBitsAt (Gemm.descriptorOf h) (Gemm.entryStore raw.body) ⟨0, 0, 0⟩ = 0 ∧
        Gemm.referenceElem (Gemm.descriptorOf h) (Gemm.entryStore raw.body) ⟨0, 0, 0⟩
          = Gemm.oneBits c.kind ∧
        Gemm.oneBits c.kind ≠ 0 ∧
        Gemm.referenceStatus raw.body = 0 :=
  Gemm.mandatory_family_nonzero_witnesses P

/-- **SPEC §15, `Gemm.observation_covers_status_and_full_c`.**  For a validated
raw invocation, every byte of the declared `C` range and every byte of the
status-detail range of an accepted observation is pinned to the reference value:
the scratch mask hides neither. -/
theorem observation_covers_status_and_full_c {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) {o : Wasm.ExecutionObservation}
    (hv : Gemm.refValid raw.body = true) (h : Gemm.Reference.Accepts problem raw o) :
    ∃ e f : Gemm.Store,
      Gemm.semanticFor problem raw o = .returned (Gemm.referenceStatus raw.body) e f ∧
      ∀ i, i < raw.body.len.toNat →
        ((Gemm.regionsOfRaw raw.body).c.Mem i ∨
          (Gemm.regionsOfRaw raw.body).statusDetail.Mem i) →
        f i = Gemm.referenceFinalStore raw.body i :=
  Gemm.observation_covers_status_and_full_c problem raw hv h

/-! ## Trapped and uncaught-exception observations are rejected -/

/-- **SPEC §8.4.**  A trap before the `gemm` entry boundary is rejected. -/
theorem reference_rejects_trappedBeforeEntry {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (t : Wasm.Trap)
    (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Gemm.Reference.Accepts problem raw (.trappedBeforeEntry tr t store eff) :=
  Gemm.reference_rejects_trappedBeforeEntry problem raw tr t store eff

/-- **SPEC §8.4.**  An uncaught exception before entry is rejected. -/
theorem reference_rejects_thrownBeforeEntry {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (v : Wasm.ExceptionValue)
    (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Gemm.Reference.Accepts problem raw (.thrownBeforeEntry tr v store eff) :=
  Gemm.reference_rejects_thrownBeforeEntry problem raw tr v store eff

/-- **SPEC §8.4.**  A trap inside `gemm` is rejected. -/
theorem reference_rejects_trappedAfterEntry {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (entry : Wasm.ObservableStore)
    (t : Wasm.Trap) (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Gemm.Reference.Accepts problem raw (.trappedAfterEntry tr entry t store eff) :=
  Gemm.reference_rejects_trappedAfterEntry problem raw tr entry t store eff

/-- **SPEC §8.4.**  An uncaught exception inside `gemm` is rejected. -/
theorem reference_rejects_thrownAfterEntry {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : Gemm.RawInvocation P) (tr : List Wasm.Event) (entry : Wasm.ObservableStore)
    (v : Wasm.ExceptionValue) (store : Wasm.ObservableStore) (eff : Wasm.ObservableEffects) :
    ¬ Gemm.Reference.Accepts problem raw (.thrownAfterEntry tr entry v store eff) :=
  Gemm.reference_rejects_thrownAfterEntry problem raw tr entry v store eff

end WasmGemmGnaf.Theorems
