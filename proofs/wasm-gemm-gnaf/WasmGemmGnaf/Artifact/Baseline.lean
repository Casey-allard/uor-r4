/-
  Artifact/Baseline.lean --- the concrete baseline byte sequence, and exactly
  how far obligation GO-006 (nonemptiness) actually goes.

  Normative sources: SPEC.md sections 7.2/7.3 (the release profile and the
  release validator), 8.3/8.4 (the raw invocation carrier and the reference
  acceptance relation), 10.1 (`Universal.ProfileValid`,
  `Universal.SemanticCorrect`, `Universal.SemanticWithinResources`) and 13
  (Phase B / Phase D).

  ## GO-006, STATED

  `Universal.Admissible` is a conjunction of three extensional predicates, and
  the attainability theorem `Release.exists_globalOptimal_of_admissible_evaluation`
  needs one byte sequence satisfying all three *plus* an inhabitant of
  `Universal.SystemEvaluation`:

    (a) `Universal.ProfileValid Release.wasmProfile b`
    (b) `Universal.SemanticCorrect (Release.setting seam) b`
    (c) `Universal.SemanticWithinResources (Release.setting seam) b`
    (d) `Universal.SystemEvaluation (Release.setting seam) b`

  ## WHAT THIS FILE PROVES: (a), ON A CONCRETE LITERAL

  `Artifact.baselineBytes` is a closed term of type `ByteArray`: the canonical
  encoding of the module the GNAF compiler emits from `GNAF.gemmWitnessChecked`,
  the `1x1x1` modular-`u32` GEMM plan of `GNAF/CompileCorrect.lean`.  It is not
  a parameter, not a `Classical.choice`, and not an opaque constant.

  **`Artifact.baseline_profileValid`** proves conjunct (a) for it, outright and
  with no hypothesis:

  * `Wasm.decode baselineBytes = .ok baselineModule` — `Artifact.decode_emit`,
    which is `Wasm.encode_decode_roundtrip`;
  * `Universal.validateUnder Release.wasmProfile baselineModule = true` — the
    release validator from `GNAF.gemmWitness_compiles`, and the profile's
    first-order limit table by `decide` (every count in the emitted module is
    `0`, `1` or `2`, and every release limit is `2 ^ 32 - 1`);
  * `baselineModule.imports = []` — definitionally, `GNAF.moduleOf` declares no
    import;
  * `Universal.HasExactGemmExports` — the memory export and the pinned-ABI
    `gemm` export are *conjuncts of `Wasm.validate` itself*, so they are read
    straight out of `GNAF.gemmWitness_compiles`, and the emitted export list is
    literally the two-element list `[gemm, memory]`, so no third export exists.

  Nothing here is assumed and nothing is decided by `native_decide`.

  ## WHAT THIS FILE DOES NOT PROVE, AND WHY: (b), (c), (d)

  **GO-006 REMAINS OPEN.**  It is not discharged here, it is not discharged
  anywhere in this repository, and no theorem below may be cited as if it were.

  ### (b) `SemanticCorrect` — out of reach, and the obstruction is named

  `Universal.SemanticCorrect S b` unfolds to: for **every** raw invocation
  `raw : Gemm.RawInvocation Release.wasmProfile`, (i) some maximal execution of
  the costed initial configuration exists, and (ii) **every** maximal execution
  of it has a finite observation accepted by `Gemm.Reference.Accepts`.  And
  `Gemm.Reference.Accepts` is byte-exact: `Gemm.ReferenceSemanticAccepts`
  requires the outcome to be `.returned` with the reference status, with the
  entry store equal to `Gemm.entryObservable` at *every* index and the final
  store equal to `Gemm.referenceObservableStore` at *every* index, and the
  second conjunct additionally confines every candidate memory write to the
  sanctioned regions.

  Proving that for `baselineModule` is exactly SPEC §11.4's **`compile_refines`**,
  which `GNAF/CompileCorrect.lean` records as **omitted** under `BI-002`/`O-6`
  and explains at length: the GNAF machine of `GNAF/Semantics.lean` computes
  over unbounded `Nat` cells while the Wasm machine computes over `i32`, so the
  refinement statement is *false* as written without an explicit boundedness
  precondition and an explicit representation relation between a `GNAF.Machine`
  and a `Wasm.Config`; and the bounded version needs a simulation argument for
  every plan constructor, which exists nowhere here.  On top of that,
  `GNAF/CompileCorrect.lean` itself records that the witness plan is the
  `alpha = 1`, `beta = 0`, `1x1x1` instance and that its cell image is *not*
  proved to coincide with the byte image of `Gemm/ABI.lean` — so even a
  finished `compile_refines` would not immediately give `Accepts` over the whole
  raw-invocation domain.

  What it would take, concretely and in order: a representation relation between
  `GNAF.Machine` and `Wasm.Config`; a boundedness invariant discharging the
  `Nat`-versus-`i32` gap; a per-constructor simulation lemma for `GNAF.code`;
  the ABI bridge identifying `GNAF.gemmWitnessMem` with `Gemm.encodeHeader`'s
  byte image; and then, over *every* raw invocation rather than the classified
  ones, agreement with `Gemm.referenceObservableStore` — including the
  descriptor-invalid, unsupported and resource-exhausted branches.  None of the
  five exists.

  It is therefore **not** stated below, not as a theorem, not as a hypothesis of
  a theorem that concludes `Universal.Admissible`, and not inside any structure
  field.  The only place it appears is as an explicit, named premise of
  `Artifact.exists_globalOptimal_of_baseline_semantics`, whose doc comment says
  in terms that it is a *reduction*, not a discharge.

  ### (c) `SemanticWithinResources` — not even expressible yet

  It quantifies over `Release.Seam.machine`'s costed observations and compares
  against `Release.Seam.limit`.  Both are parameters of `Release.Seam`: SPEC
  §7.5's costed semantics and SPEC §10.1's costed machine do not exist in this
  repository (`Artifact/Release.lean`, header items 2 and 3), and SPEC §8.2's
  six-field resource contract determines only six of `Cost.DynamicVector`'s
  sixteen coordinates (item 4).  With `seam` universally quantified, (c) is
  simply false for some seams — e.g. any seam whose limit vector is zero — so
  no unconditional theorem about it can exist here, and manufacturing a
  favourable seam would be choosing the resource contract to fit the artifact.

  ### (d) `SystemEvaluation` — blocked on the same seam

  `Universal.InputEvaluation.treeComplete` needs
  `seam.machine.exploreAllCosted` to answer `.complete` with a
  `Foundation.NonemptyCanonicalFrontier`.  `Wasm/Fuel.lean` supplies only the
  uncosted, unordered, possibly empty `Wasm.exploreAll`.  It also needs
  `[Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]`, which is
  recorded **omitted** under `O-3`.

  ## WHAT ELSE IS PROVED: the honest partial results

  Everything about `baselineBytes` that is a *bytes-side* obligation is
  discharged, so that what remains is isolated on the machine side:

  * `Artifact.baseline_startsCostedInvocation_iff` — for the baseline bytes,
    `Universal.StartsCostedInvocation` is **equivalent** to a statement about
    `seam.machine` alone.  The decode conjunct and the `validateUnder` conjunct
    of `StartsCostedInvocation` are gone: they are theorems, not obligations.
  * `Artifact.systemEvaluation_module_eq` — every system evaluation of the
    baseline bytes evaluates exactly `baselineModule`; the decoder leaves no
    freedom.
  * `Artifact.baseline_admissible_iff` — `Universal.Admissible` at the baseline
    bytes is equivalent to conjuncts (b) and (c) alone.  This is the precise
    sense in which conjunct (a) of GO-006 is closed.

  `Classical.choice` is not used in this file except transitively through
  `Release.decider` in the single reduction theorem at the end, exactly as
  `Artifact/Release.lean` discloses.  There is no `sorry`, no `admit`, no
  project axiom, no `native_decide`, no `unsafe` and no `partial`.

  ## THE DISCLOSED PROFILE DEVIATION IS INHERITED

  `Release.wasmProfile` is `Wasm.unitWitnessProfile`, not SPEC §7.2's release
  literal: its cost table carries no rule rows.  Every statement below is at
  that profile and inherits the disclosure made in `Artifact/Release.lean`.
  The deviation is confined to the cost table, and `Universal.ProfileValid`
  reads only `P.body.limits`, so `Artifact.baseline_profileValid` would survive
  the substitution of the real release cost table unchanged.
-/
import WasmGemmGnaf.Artifact.Release
import WasmGemmGnaf.GNAF.CompileCorrect

set_option autoImplicit false

namespace WasmGemmGnaf.Artifact

open WasmGemmGnaf

/-! ## 1. The baseline module and the baseline bytes -/

/-- The baseline module: what SPEC §11.4's compiler emits from the anti-vacuity
GEMM witness plan of `GNAF/CompileCorrect.lean`.  A closed term. -/
def baselineModule : Wasm.Module := GNAF.compile GNAF.gemmWitnessChecked

/-- **The baseline byte sequence.**  The canonical encoding of `baselineModule`.
A closed term of type `ByteArray`: no parameter, no choice, no axiom. -/
def baselineBytes : ByteArray := emit baselineModule

/-- The baseline bytes decode back to the baseline module. -/
theorem baseline_decode : Wasm.decode baselineBytes = .ok baselineModule :=
  decode_emit baselineModule

/-- The baseline bytes are the *only* bytes that decode to the baseline module,
so "the baseline artifact" names one byte string and not a class of them. -/
theorem baseline_bytes_unique {b : ByteArray}
    (h : Wasm.decode b = .ok baselineModule) : b = baselineBytes :=
  emit_bytes_unique h baseline_decode

/--
  The baseline bytes begin with the pinned Core preamble `\0asm\1\0\0\0`.

  Stated because it is the one *concrete* thing about the byte string that is a
  theorem here.  For orientation only, and explicitly **not** a theorem:
  `#eval baselineBytes.size` reports `3798`, so the term is genuinely
  executable rather than a phantom — but the kernel does not reduce
  `Wasm.encode`, `by rfl` on that equation fails, and no declaration in this
  file depends on the number.
-/
theorem baseline_magicPrefix :
    ∃ t : List UInt8, baselineBytes.toList = Wasm.magicBytes ++ t :=
  emit_prefix baselineModule

/-! ## 2. The baseline module passes release validation under the profile -/

/-- **SPEC §7.3.**  The baseline module passes the release validator. -/
theorem baseline_validate : Wasm.validate baselineModule = true :=
  GNAF.gemmWitness_compiles

/-- The declarative form of the same fact (SPEC §7.3). -/
theorem baseline_declarativelyValid : Wasm.DeclarativelyValid baselineModule :=
  GNAF.compile_declarativelyValid GNAF.gemmWitnessChecked

/-- The baseline module declares no import: it is closed. -/
theorem baseline_imports : baselineModule.imports = [] := rfl

/-- The baseline module's export list, in full. -/
theorem baseline_exports :
    baselineModule.exports =
      [{ name := Wasm.gemmExportName, desc := .func 0 },
       { name := Wasm.memoryExportName, desc := .mem 0 }] :=
  GNAF.compile_exports GNAF.gemmWitnessChecked

/--
  **SPEC §7.2.**  Every index-space population of the baseline module is inside
  the release profile's limit table.

  This is `decide`, and it is cheap for a reason worth recording: every count in
  `GNAF.moduleOf` is a literal `0`, `1` or `2` — one recursive type group, one
  function, one memory, two exports and nothing else — and every limit in
  `Wasm.canonicalResourceLimits` is `Wasm.wasm32Ceiling = 2 ^ 32 - 1`.  The
  function *body* is never forced.
-/
theorem baseline_withinProfileLimits :
    Universal.WithinProfileLimits Release.wasmProfile baselineModule = true := by
  decide

/-- **SPEC §10.1.**  The baseline module passes `Universal.validateUnder`: the
release validator together with the profile's limit table. -/
theorem baseline_validateUnder :
    Universal.validateUnder Release.wasmProfile baselineModule = true := by
  unfold Universal.validateUnder
  rw [baseline_validate, baseline_withinProfileLimits]
  rfl

/-! ## 3. The required exports

`Wasm.validate` already carries the two export conjuncts, so they are read out
of `baseline_validate` rather than reproved.  `&&` associates to the left, so
the eight conjuncts of `Wasm.validate` are a left-nested tower and the two
export conjuncts sit at `h.1.1.2` and `h.1.2`. -/

/-- The memory export, extracted from release validation itself. -/
theorem baseline_exportsMemory : Wasm.Module.exportsMemory baselineModule = true := by
  have h := baseline_validate
  unfold Wasm.validate at h
  simp only [Bool.and_eq_true] at h
  exact h.1.1.2

/-- The pinned-ABI `gemm` export, extracted from release validation itself. -/
theorem baseline_checkGemmExport :
    Wasm.Module.checkGemmExport baselineModule = true := by
  have h := baseline_validate
  unfold Wasm.validate at h
  simp only [Bool.and_eq_true] at h
  exact h.1.2

/-- The baseline module exports the `gemm` function and the memory, and nothing
else at all. -/
theorem baseline_hasExactGemmExports :
    Universal.HasExactGemmExports Release.wasmProfile baselineModule := by
  refine ⟨baseline_exportsMemory, baseline_checkGemmExport, ?_⟩
  intro e he
  rw [baseline_exports] at he
  simp only [List.mem_cons, List.not_mem_nil, or_false] at he
  rcases he with rfl | rfl
  · exact Or.inl rfl
  · exact Or.inr rfl

/-! ## 4. Conjunct (a) of GO-006, discharged -/

/--
  **Conjunct (a) of GO-006, proved.**

  `Universal.ProfileValid Release.wasmProfile Artifact.baselineBytes` — for a
  concrete, closed byte sequence, with no hypothesis of any kind.

  This is the whole of what GO-006 asks for on the profile side, and it is the
  only one of GO-006's four requirements that this repository can currently
  meet.  It does **not** discharge GO-006: see `baseline_admissible_iff` for
  exactly what is left, and the file header for why each remaining piece is out
  of reach.
-/
theorem baseline_profileValid :
    Universal.ProfileValid Release.wasmProfile baselineBytes :=
  ⟨baselineModule, baseline_decode, baseline_validateUnder, baseline_imports,
    baseline_hasExactGemmExports⟩

/-- The same fact in the unpacked form `Universal.profileValid_module` returns. -/
theorem baseline_profileValid_module :
    ∃ module : Wasm.Module,
      Wasm.decode baselineBytes = .ok module ∧
      Universal.validateUnder Release.wasmProfile module = true ∧
      module.imports = [] ∧
      Universal.HasExactGemmExports Release.wasmProfile module :=
  Universal.profileValid_module baseline_profileValid

/-- Nonemptiness of the *profile-valid* set — the (a)-only fragment of GO-006. -/
theorem exists_profileValid :
    ∃ b : ByteArray, Universal.ProfileValid Release.wasmProfile b :=
  ⟨baselineBytes, baseline_profileValid⟩

/-! ## 5. What the baseline bytes settle about the seam-dependent predicates

Everything in this section is a *bytes-side* discharge: the decode and validate
obligations that the seam-dependent predicates carry are removed, leaving the
machine-side obligations exposed and unaltered. -/

section Seam

variable (seam : Release.Seam)

/--
  Every system evaluation of the baseline bytes evaluates exactly
  `baselineModule`.  The decoder leaves no freedom, so no evaluation of these
  bytes can be an evaluation of some other module.
-/
theorem systemEvaluation_module_eq
    [Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]
    (e : Universal.SystemEvaluation (Release.setting seam) baselineBytes) :
    e.module = baselineModule :=
  (Universal.profileValid_module_unique baseline_decode e.decodeEq).symm

/--
  **The bytes-side of `StartsCostedInvocation`, discharged.**

  For the baseline bytes, SPEC §10.1's `Universal.StartsCostedInvocation` is
  *equivalent* to a statement about `seam.machine` alone: its decode conjunct
  and its `validateUnder` conjunct are theorems (`baseline_decode`,
  `baseline_validateUnder`), and its module is forced to be `baselineModule`.

  This is the sharpest thing that can be said about conjuncts (b) and (c)
  without the costed machine: it does not prove that the machine initializes,
  and it does not prove anything whatever about what happens after it does.
-/
theorem baseline_startsCostedInvocation_iff
    (raw : Gemm.RawInvocation Release.wasmProfile)
    (initialization : Universal.InitializationObservation Release.wasmProfile)
    (initial : Wasm.Config) :
    Universal.StartsCostedInvocation (Release.setting seam) baselineBytes raw
        initialization initial ↔
      (seam.machine.initialGemmInvocationCosted baselineModule
          (Universal.toWasmInvocation raw) = .ok initialization ∧
        initialization.initial = initial) := by
  constructor
  · rintro ⟨module, hdec, _, hinit, hcfg⟩
    have hmod : module = baselineModule :=
      (Universal.profileValid_module_unique baseline_decode hdec).symm
    subst hmod
    exact ⟨hinit, hcfg⟩
  · rintro ⟨hinit, hcfg⟩
    exact ⟨baselineModule, baseline_decode, baseline_validateUnder, hinit, hcfg⟩

/-- The `←` direction on its own: given only that the costed machine
initializes the baseline module, the baseline bytes really do start that costed
invocation. -/
theorem baseline_startsCostedInvocation
    (raw : Gemm.RawInvocation Release.wasmProfile)
    (initialization : Universal.InitializationObservation Release.wasmProfile)
    (hinit : seam.machine.initialGemmInvocationCosted baselineModule
      (Universal.toWasmInvocation raw) = .ok initialization) :
    Universal.StartsCostedInvocation (Release.setting seam) baselineBytes raw
      initialization initialization.initial :=
  (baseline_startsCostedInvocation_iff seam raw initialization _).mpr ⟨hinit, rfl⟩

/--
  **Exactly what is left of GO-006 at the baseline bytes.**

  `Universal.Admissible` at `baselineBytes` is equivalent to conjuncts (b) and
  (c) alone.  Conjunct (a) has been removed from the obligation, not assumed
  into it.

  Read this as the scoreboard: one of the three extensional predicates is a
  theorem; the other two are open, for the reasons the file header gives.
-/
theorem baseline_admissible_iff :
    Universal.Admissible (Release.setting seam) baselineBytes ↔
      (Universal.SemanticCorrect (Release.setting seam) baselineBytes ∧
        Universal.SemanticWithinResources (Release.setting seam) baselineBytes) := by
  constructor
  · rintro ⟨_, hb, hc⟩
    exact ⟨hb, hc⟩
  · rintro ⟨hb, hc⟩
    exact ⟨baseline_profileValid, hb, hc⟩

/--
  **A reduction, and explicitly NOT a discharge of GO-006.**

  Read the hypotheses before reading the conclusion.  `hcorrect`, `hresources`
  and `e` are conjuncts (b), (c) and (d): they are *premises supplied by the
  caller*, and this repository cannot supply any of the three.  Nothing in this
  file, and nothing in `Artifact/Release.lean`, discharges them.

  What the theorem adds over `Release.exists_globalOptimal_of_admissible_evaluation`
  is precisely one thing: the byte sequence is no longer existentially
  quantified.  It is the closed term `Artifact.baselineBytes`, and its
  profile-validity conjunct is discharged inside the proof rather than assumed.

  Citing this theorem as evidence that a globally optimal artifact exists would
  be citing its hypotheses as facts.  They are not facts.  **GO-006 is open.**
-/
theorem exists_globalOptimal_of_baseline_semantics
    [Foundation.Fintype (Gemm.RawInvocation Release.wasmProfile)]
    (hcorrect : Universal.SemanticCorrect (Release.setting seam) baselineBytes)
    (hresources :
      Universal.SemanticWithinResources (Release.setting seam) baselineBytes)
    (e : Universal.SystemEvaluation (Release.setting seam) baselineBytes) :
    ∃ bytes : ByteArray,
      Universal.GlobalOptimal (Release.setting seam) (Release.decider seam)
        (Release.costObjective seam) bytes :=
  Release.exists_globalOptimal_of_admissible_evaluation seam
    ⟨baselineBytes,
      (baseline_admissible_iff seam).mpr ⟨hcorrect, hresources⟩, ⟨e⟩⟩

end Seam

end WasmGemmGnaf.Artifact
