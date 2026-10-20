import WasmGemmGnaf.Gemm.Classify
set_option autoImplicit false

/-!
# Gemm: the observation contract (SPEC §8.3)

> Scratch is candidate-private workspace: candidate-call writes may target it,
> its final bytes are masked out of `SemanticObservation`, and it is freshly
> initialized from the raw bytes for every invocation. C and status-detail are
> fully observable. All nonscratch bytes outside the permitted C/status ranges
> must equal their `gemmEntryObservableStore` values. Thus success may change C,
> status, and scratch. A descriptor-level `invalid`, `unsupported`, or
> `resource-exhausted` result may write only a separately validated status
> range; if no trustworthy status range exists it writes nothing. … A runtime
> checked-overflow or arithmetic-exception may write validated status and
> scratch but must leave C equal to its entry value. Neither case can use
> workspace as a hidden cross-invocation channel.
> `SanctionedWriteRegions raw terminal` is therefore the closed phase-dependent
> function `∅`, `status`, `C ∪ status ∪ scratch`, or `status ∪ scratch` for
> those four cases respectively. The reference equality separately masks scratch
> only after complete descriptor validation.

This module fixes that closed function and proves the consequences SPEC names:
that the full C range and the status range are observable, that the
rejected phase changes nothing, and that a runtime failure leaves C at its entry
value.
-/

namespace WasmGemmGnaf.Gemm

/-- The ABI-visible byte store, as a total function of the byte offset relative
to `ptr`.  Modelling it as a function keeps the observation algebra free of
length side conditions. -/
def Store : Type := Nat → UInt8

instance : Inhabited Store := ⟨fun _ => 0⟩

namespace Store

/-- Two stores agree on a byte. -/
def Agree (a b : Store) (i : Nat) : Prop := a i = b i

theorem agree_refl (a : Store) (i : Nat) : Agree a a i := rfl

theorem agree_symm {a b : Store} {i : Nat} (h : Agree a b i) : Agree b a i := h.symm

theorem agree_trans {a b c : Store} {i : Nat} (h₁ : Agree a b i) (h₂ : Agree b c i) :
    Agree a c i := h₁.trans h₂

end Store

/-- A byte belongs to one of the listed regions. -/
def InRegions (rs : List ByteRange) (i : Nat) : Prop := ∃ r ∈ rs, r.Mem i

instance (rs : List ByteRange) (i : Nat) : Decidable (InRegions rs i) :=
  inferInstanceAs (Decidable (∃ _ ∈ _, _))

theorem inRegions_nil (i : Nat) : ¬ InRegions [] i := by simp [InRegions]

theorem inRegions_of_mem {rs : List ByteRange} {r : ByteRange} (hr : r ∈ rs)
    {i : Nat} (hi : r.Mem i) : InRegions rs i := ⟨r, hr, hi⟩

theorem inRegions_mono {rs ss : List ByteRange} (h : ∀ r ∈ rs, r ∈ ss) {i : Nat}
    (hi : InRegions rs i) : InRegions ss i := by
  obtain ⟨r, hr, hmem⟩ := hi
  exact ⟨r, h r hr, hmem⟩

/-! ## The four terminal phases (SPEC §8.3) -/

/-- The closed set of terminal phases that determine the sanctioned write set. -/
inductive TerminalPhase
  /-- A descriptor-level rejection with no trustworthy status range: nothing is
  written. -/
  | noTrustworthyStatus
  /-- A descriptor-level `invalid`/`unsupported`/`resource-exhausted` result with
  a separately validated status range. -/
  | descriptorRejected
  /-- A successful GEMM. -/
  | success
  /-- A runtime checked-overflow or arithmetic-exception. -/
  | runtimeFailure
  deriving DecidableEq, Repr, Inhabited

/-- **SPEC §8.3's closed phase-dependent function**
`∅`, `status`, `C ∪ status ∪ scratch`, `status ∪ scratch`. -/
def sanctionedWriteRegions (R : Regions) : TerminalPhase → List ByteRange
  | .noTrustworthyStatus => []
  | .descriptorRejected => [R.statusDetail]
  | .success => [R.c, R.statusDetail, R.scratch]
  | .runtimeFailure => [R.statusDetail, R.scratch]

@[simp] theorem sanctioned_noTrustworthyStatus (R : Regions) :
    sanctionedWriteRegions R .noTrustworthyStatus = [] := rfl
@[simp] theorem sanctioned_descriptorRejected (R : Regions) :
    sanctionedWriteRegions R .descriptorRejected = [R.statusDetail] := rfl
@[simp] theorem sanctioned_success (R : Regions) :
    sanctionedWriteRegions R .success = [R.c, R.statusDetail, R.scratch] := rfl
@[simp] theorem sanctioned_runtimeFailure (R : Regions) :
    sanctionedWriteRegions R .runtimeFailure = [R.statusDetail, R.scratch] := rfl

/-- The sanctioned sets grow along `∅ ⊆ status ⊆ status ∪ scratch ⊆
C ∪ status ∪ scratch`. -/
theorem sanctioned_monotone (R : Regions) :
    (∀ r ∈ sanctionedWriteRegions R .noTrustworthyStatus,
        r ∈ sanctionedWriteRegions R .descriptorRejected) ∧
    (∀ r ∈ sanctionedWriteRegions R .descriptorRejected,
        r ∈ sanctionedWriteRegions R .runtimeFailure) ∧
    (∀ r ∈ sanctionedWriteRegions R .runtimeFailure,
        r ∈ sanctionedWriteRegions R .success) := by
  refine ⟨?_, ?_, ?_⟩ <;> intro r hr <;> simp_all

/-- Only `success` sanctions writing `C`. -/
theorem c_sanctioned_iff_success (R : Regions) (phase : TerminalPhase)
    (hne : R.c.isEmpty = false)
    (hdc : R.statusDetail.Disjoint R.c) (hsc : R.scratch.Disjoint R.c) :
    (∀ i, R.c.Mem i → InRegions (sanctionedWriteRegions R phase) i) ↔ phase = .success := by
  constructor
  · intro h
    cases phase with
    | success => rfl
    | noTrustworthyStatus =>
      obtain ⟨i, hi, _⟩ := ByteRange.overlaps_self R.c hne
      exact absurd (h i hi) (inRegions_nil i)
    | descriptorRejected =>
      obtain ⟨i, hi, _⟩ := ByteRange.overlaps_self R.c hne
      obtain ⟨r, hr, hmem⟩ := h i hi
      simp only [sanctioned_descriptorRejected, List.mem_singleton] at hr
      subst hr
      exact absurd ⟨i, hmem, hi⟩ hdc
    | runtimeFailure =>
      obtain ⟨i, hi, _⟩ := ByteRange.overlaps_self R.c hne
      obtain ⟨r, hr, hmem⟩ := h i hi
      simp only [sanctioned_runtimeFailure, List.mem_cons, List.not_mem_nil,
        or_false] at hr
      rcases hr with rfl | rfl
      · exact absurd ⟨i, hmem, hi⟩ hdc
      · exact absurd ⟨i, hmem, hi⟩ hsc
  · rintro rfl
    intro i hi
    exact ⟨R.c, by simp, hi⟩

/-! ## Write confinement -/

/-- Every changed ABI-visible byte lies in the sanctioned regions. -/
def WritesWithin (entry final : Store) (rs : List ByteRange) : Prop :=
  ∀ i, entry i ≠ final i → InRegions rs i

theorem writesWithin_refl (s : Store) (rs : List ByteRange) : WritesWithin s s rs := by
  intro i h
  exact absurd rfl h

/-- Enlarging the sanctioned set never invalidates confinement. -/
theorem writesWithin_mono {entry final : Store} {rs ss : List ByteRange}
    (h : ∀ r ∈ rs, r ∈ ss) (hw : WritesWithin entry final rs) :
    WritesWithin entry final ss :=
  fun i hi => inRegions_mono h (hw i hi)

/-- "if no trustworthy status range exists it writes nothing". -/
theorem noTrustworthyStatus_writes_nothing {entry final : Store} {R : Regions}
    (h : WritesWithin entry final (sanctionedWriteRegions R .noTrustworthyStatus))
    (i : Nat) : entry i = final i := by
  by_cases hc : entry i = final i
  · exact hc
  · exact absurd (h i hc) (by simpa using inRegions_nil i)

/-- "A runtime checked-overflow or arithmetic-exception … must leave C equal to
its entry value." -/
theorem runtimeFailure_preserves_c {entry final : Store} {R : Regions}
    (hdc : R.statusDetail.Disjoint R.c) (hsc : R.scratch.Disjoint R.c)
    (h : WritesWithin entry final (sanctionedWriteRegions R .runtimeFailure))
    {i : Nat} (hi : R.c.Mem i) : entry i = final i := by
  by_cases hc : entry i = final i
  · exact hc
  · obtain ⟨r, hr, hmem⟩ := h i hc
    simp only [sanctioned_runtimeFailure, List.mem_cons, List.not_mem_nil,
      or_false] at hr
    rcases hr with rfl | rfl
    · exact absurd ⟨i, hmem, hi⟩ hdc
    · exact absurd ⟨i, hmem, hi⟩ hsc

/-- A descriptor-level rejection may write only the status range. -/
theorem descriptorRejected_preserves_outside_status {entry final : Store}
    {R : Regions}
    (h : WritesWithin entry final (sanctionedWriteRegions R .descriptorRejected))
    {i : Nat} (hi : ¬ R.statusDetail.Mem i) : entry i = final i := by
  by_cases hc : entry i = final i
  · exact hc
  · obtain ⟨r, hr, hmem⟩ := h i hc
    simp only [sanctioned_descriptorRejected, List.mem_singleton] at hr
    subst hr
    exact absurd hmem hi

/-- "All nonscratch bytes outside the permitted C/status ranges must equal their
`gemmEntryObservableStore` values." -/
theorem success_preserves_outside {entry final : Store} {R : Regions}
    (h : WritesWithin entry final (sanctionedWriteRegions R .success))
    {i : Nat} (hc : ¬ R.c.Mem i) (hs : ¬ R.statusDetail.Mem i)
    (hw : ¬ R.scratch.Mem i) : entry i = final i := by
  by_cases hcc : entry i = final i
  · exact hcc
  · obtain ⟨r, hr, hmem⟩ := h i hcc
    simp only [sanctioned_success, List.mem_cons, List.not_mem_nil, or_false] at hr
    rcases hr with rfl | rfl | rfl
    · exact absurd hmem hc
    · exact absurd hmem hs
    · exact absurd hmem hw

/-! ## Scratch masking -/

/-- Mask the candidate-private workspace out of the observable store. -/
def maskScratch (R : Regions) (s : Store) : Store :=
  fun i => if R.scratch.Mem i then 0 else s i

/-- The reference equality masks scratch only after complete descriptor
validation, i.e. only in the two post-validation phases. -/
def semanticObservation (R : Regions) (phase : TerminalPhase) (s : Store) : Store :=
  match phase with
  | .noTrustworthyStatus => s
  | .descriptorRejected => s
  | .success => maskScratch R s
  | .runtimeFailure => maskScratch R s

@[simp] theorem maskScratch_of_mem {R : Regions} {s : Store} {i : Nat}
    (h : R.scratch.Mem i) : maskScratch R s i = 0 := by simp [maskScratch, h]

@[simp] theorem maskScratch_of_not_mem {R : Regions} {s : Store} {i : Nat}
    (h : ¬ R.scratch.Mem i) : maskScratch R s i = s i := by simp [maskScratch, h]

theorem maskScratch_idempotent (R : Regions) (s : Store) :
    maskScratch R (maskScratch R s) = maskScratch R s := by
  funext i
  by_cases h : R.scratch.Mem i <;> simp [maskScratch, h]

/-- **The full C range is observable**: masking never touches a C byte, because
scratch is disjoint from C. -/
theorem c_observable {R : Regions} (hsc : R.scratch.Disjoint R.c) (s : Store)
    {i : Nat} (hi : R.c.Mem i) : maskScratch R s i = s i := by
  apply maskScratch_of_not_mem
  intro hmem
  exact hsc ⟨i, hmem, hi⟩

/-- **The status range is observable.** -/
theorem status_observable {R : Regions} (hss : R.scratch.Disjoint R.statusDetail)
    (s : Store) {i : Nat} (hi : R.statusDetail.Mem i) : maskScratch R s i = s i := by
  apply maskScratch_of_not_mem
  intro hmem
  exact hss ⟨i, hmem, hi⟩

/-- In every phase, the semantic observation of a C byte is that byte. -/
theorem semanticObservation_c {R : Regions} (hsc : R.scratch.Disjoint R.c)
    (phase : TerminalPhase) (s : Store) {i : Nat} (hi : R.c.Mem i) :
    semanticObservation R phase s i = s i := by
  cases phase <;> simp only [semanticObservation] <;> first
    | rfl
    | exact c_observable hsc s hi

/-- In every phase, the semantic observation of a status byte is that byte. -/
theorem semanticObservation_status {R : Regions}
    (hss : R.scratch.Disjoint R.statusDetail) (phase : TerminalPhase) (s : Store)
    {i : Nat} (hi : R.statusDetail.Mem i) : semanticObservation R phase s i = s i := by
  cases phase <;> simp only [semanticObservation] <;> first
    | rfl
    | exact status_observable hss s hi

/-- A well-formed descriptor makes scratch disjoint from both C and status, so
`c_observable` and `status_observable` always apply. -/
theorem scratch_disjoint_of_wellFormed {d : DescriptorBody} {M : MemoryWindow}
    (h : DescriptorWellFormed d M) :
    d.regions.scratch.Disjoint d.regions.c ∧
      d.regions.scratch.Disjoint d.regions.statusDetail := by
  obtain ⟨hc, _⟩ := h.aliasOk
  obtain ⟨_, _, hss, _, _, _, _, _, hsc, _⟩ := hc
  exact ⟨hsc, hss⟩

/-! ## Scratch is not a cross-invocation channel

"it is freshly initialized from the raw bytes for every invocation" — so the
entry contents of scratch are a function of the raw bytes alone, and two
invocations with the same raw bytes start from the same workspace. -/

/-- The entry store an invocation sees: exactly the raw bytes, zero elsewhere. -/
def entryStore {P : Wasm.Profile} (raw : RawInvocationBody P) : Store :=
  fun i => raw.bytes.toList.getD i 0

/-- Scratch is freshly initialized from the raw bytes: its entry contents depend
on nothing but those bytes, so no workspace state can cross invocations. -/
theorem scratch_fresh {P : Wasm.Profile} (raw raw' : RawInvocationBody P)
    (h : raw.bytes = raw'.bytes) (i : Nat) : entryStore raw i = entryStore raw' i := by
  simp [entryStore, h]

/-- Two invocations of the same raw bytes have identical entry observations. -/
theorem entryStore_determined {P : Wasm.Profile} (raw raw' : RawInvocationBody P)
    (h : raw.bytes = raw'.bytes) : entryStore raw = entryStore raw' := by
  funext i
  exact scratch_fresh raw raw' h i

/-! ## Anti-vacuity: the observable ranges are nonempty for a real descriptor -/

/-- For a descriptor with a nonempty C region, some byte really is observable and
sanctioned on success. -/
theorem success_c_write_sanctioned {R : Regions} (hne : R.c.isEmpty = false) :
    ∃ i, R.c.Mem i ∧ InRegions (sanctionedWriteRegions R .success) i := by
  obtain ⟨i, hi, _⟩ := ByteRange.overlaps_self R.c hne
  exact ⟨i, hi, ⟨R.c, by simp, hi⟩⟩

/-- The status range of a valid descriptor is exactly 32 bytes, hence nonempty
and observable. -/
theorem status_range_nonempty {d : DescriptorBody} {M : MemoryWindow}
    (h : DescriptorWellFormed d M) : d.regions.statusDetail.isEmpty = false := by
  have hs := h.statusIsRecord
  simp only [ByteRange.isEmpty, beq_eq_false_iff_ne, ne_eq, DescriptorBody.regions,
    hs, statusDetailBytes]
  decide

end WasmGemmGnaf.Gemm
