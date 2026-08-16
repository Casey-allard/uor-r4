# WASM-GEMM-GNAF — complete repository specification

Status: normative implementation contract
Repository baseline: `afflom/WASM-GEMM-GNAF` at `fdd58db98edf5b0a28c04bada3e78cef99adece4`
Implementation language: Lean 4
Product: a committed WebAssembly binary plus kernel-checked correctness and global-optimality proofs

This document specifies the complete target repository. It replaces the empty Rust-template behavior currently present at the baseline commit. A conforming repository implements every required definition, algorithm, proof, artifact, audit, and release gate below. A file that merely declares an interface or assumes its decisive theorem is not an implementation.

The words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, and **MAY** are normative.

## 1. Non-negotiable result

The repository SHALL produce one exact byte sequence, `artifacts/wasm-gemm-gnaf.wasm`, and close this proposition in Lean:

```lean
theorem released_wasm_gemm_gnaf_global_optimal :
  GlobalOptimal
    Release.wasmProfile
    Release.gemmProblem
    Release.costObjective
    Release.artifactBytes
```

`GlobalOptimal P G O bytes` SHALL mean all of the following, conjunctively:

1. `bytes` decodes and validates under the pinned WebAssembly profile `P`.
2. It is a closed, portable WebAssembly system under `P`; it does not inspect an ISA, processor family, engine, clock, or host feature.
3. On every raw invocation admitted by `G`, every maximal permitted WebAssembly execution terminates with the exact sanctioned result and observable memory effects.
4. On malformed, unrepresentable, or resource-invalid invocations it returns the exact typed status prescribed by `G`, with no unsanctioned trap or effect.
5. It stays within the bound resources used by the comparison.
6. Its complete charged score under `O` is attained.
7. For every finite byte sequence `competitor`, if `competitor` decodes and validates under the same profile, implements the same total GEMM problem, starts with the same information and state, and obeys the same resource contract, then:

   ```lean
   O.score releasedEvaluation.cost ≤ O.score competitorEvaluation.cost
   ```

8. Equal-score competitors are permitted; the released bytes are the canonical least identity among all minimizers.

The quantifier in item 7 ranges over **all** same-profile WebAssembly byte modules, not only GNAF plans, registered kernels, familiar algorithms, proof-carrying submissions, or candidates found by an attention index. A theorem scoped only to such a subset SHALL have a different name and SHALL NOT satisfy the release gate.

The theorem is about the exact abstract cost objective defined in this repository. It is not a statement that all conforming engines, processors, or physical machines execute the module fastest. Core WebAssembly specifies behavior, not host latency. The README SHALL state this distinction immediately beside every use of "globally optimal."

If the universal proof cannot be closed, the correct repository outcome is `incomplete`; weakening the universe, silently strengthening the machine, omitting cost, or changing the public wording is forbidden.

The decisive predicate SHALL have this definition shape; the implementation may refine field names but not the quantifiers or conjuncts:

```lean
def GlobalOptimal
    (P : Wasm.Profile)
    (G : Gemm.Problem P)
    (O : Cost.ProperObjective P G)
    (releasedBytes : ByteArray) : Prop :=
  Universal.ProfileValid P releasedBytes ∧
  Universal.SemanticCorrect P G releasedBytes ∧
  Universal.SemanticWithinResources P G releasedBytes ∧
  ∃ releasedEval : Universal.SystemEvaluation P G releasedBytes,
    Universal.SystemEvaluationRel P G releasedBytes releasedEval ∧
    Universal.Correct G releasedEval ∧
    Universal.Feasible G releasedEval ∧
    (∀ (competitorBytes : ByteArray),
       Universal.ProfileValid P competitorBytes →
       Universal.SemanticCorrect P G competitorBytes →
       Universal.SemanticWithinResources P G competitorBytes →
       ∀ competitorEval : Universal.SystemEvaluation P G competitorBytes,
         Universal.SystemEvaluationRel P G competitorBytes competitorEval ∧
         O.score releasedEval.cost ≤ O.score competitorEval.cost) ∧
    (∀ (competitorBytes : ByteArray),
       Universal.ProfileValid P competitorBytes →
       Universal.SemanticCorrect P G competitorBytes →
       Universal.SemanticWithinResources P G competitorBytes →
       ∀ competitorEval : Universal.SystemEvaluation P G competitorBytes,
         Universal.SystemEvaluationRel P G competitorBytes competitorEval →
         O.score releasedEval.cost = O.score competitorEval.cost →
         CanonicalBytesLE releasedBytes competitorBytes)
```

`SystemEvaluationRel` SHALL be sound, complete for every semantically correct resource-bounded module, and functional: two evaluations of the same bytes under the same profile/problem have identical observations and cost. `ProfileValid`, `SemanticCorrect`, `SemanticWithinResources`, `SystemEvaluationRel`, and `ProperObjective` SHALL be the frozen definitions specified below, never artifact-specific callbacks.

`authority/global-optimality-WGG-GO-1.json` SHALL contain the canonical unfolded proposition schema and identities of every scope-critical definition. The conformance gate SHALL compare the compiled, unfolded declaration bodies with those identities. Recording the implementation's own proposition without comparison to this frozen schema is insufficient.

## 2. Exact claim scopes

The repository SHALL keep four claims distinct.

### 2.1 Semantic portability

`PortableCorrect` proves identical sanctioned behavior under the bound WebAssembly semantics. It makes no performance claim.

### 2.2 Plan optimality

`PlanGlobalOptimal` compares checked GNAF plans. It is useful internally but SHALL NOT be presented as all-Wasm globality.

### 2.3 Artifact globality

`GlobalOptimal` compares the complete committed module with every same-profile correct and feasible WebAssembly module. This is the mandatory release theorem.

### 2.4 Lifecycle globality

`LifecycleGlobalOptimal` compares accumulation, indexing, proof checking, reoptimization, state storage, artifact construction, and execution across an exact update/request trace. It is a separate theorem. Until it is closed, the project MAY state exact maintenance bounds and rebuild equivalence, but SHALL NOT call the Atlas maintenance process itself globally optimal.

## 3. Quantifier order and meaning of arbitrary GEMM

The implementation is one uniform system. The final theorem SHALL fix the artifact before quantifying over invocations:

```lean
def releasedArtifact : ByteArray := Release.artifactBytes

theorem released_input_total :
  ∀ raw : Release.gemmProblem.RawInvocation,
    InvocationConforms Release.wasmProfile Release.gemmProblem
      releasedArtifact raw
```

The forbidden substitute is `∀ raw, ∃ module, InvocationConforms P G module raw`.

"Arbitrary GEMM" means every raw invocation representable by the released ABI and profile, including every valid combination of:

- `m`, `n`, `k`, and batch count;
- row-major, column-major, and explicitly affine-strided views admitted by the descriptor;
- independent transpose flags;
- all admitted operand, accumulator, and output scalar kinds;
- `alpha`, `beta`, and initial `C`;
- zero dimensions, skinny, rectangular, and maximal representable shapes;
- legal aliasing relationships;
- every operand bit pattern, including floating specials;
- every admitted arithmetic, rounding, overflow, and approximation contract;
- every memory placement satisfying alignment and bounds;
- invalid encodings and resource failures, which receive typed non-success results.

It does not mean an unbounded mathematical `Nat` that cannot be represented in WebAssembly memory. Representability limits are part of the exact problem, not an implicit escape clause.

## 4. Pinned authority and trust base

The following identities SHALL be recorded in `authority/manifest.json`, checked by content digest, and reproduced in `artifacts/proof-manifest.json`:

| Authority | Required pin |
|---|---|
| Repository baseline | `fdd58db98edf5b0a28c04bada3e78cef99adece4` |
| UOR-GNAF authority | `UOR-GNAF-v1-draft.2.md`, SHA-256 `5c342373b2ff809bfd607c413cafd0582d32bb097544c6597ff7d674fe99200a` |
| WebAssembly Core | official `wg-3.0` source, commit `9d36019973201a19f9c9ebb0f10828b2fe2374aa` |
| Lean | `leanprover/lean4:v4.30.0`, commit `d024af099ca4bf2c86f649261ebf59565dc8c622` |
| Lean packages | exact `lake-manifest.json` revisions; initial proof core uses only Lean and `Std` |

Changing an authority creates a new profile and invalidates dependent certificates. No floating tags, mutable action revisions, unpinned package revisions, or network-fetched proof inputs are allowed in release verification.

The mathematical theorem is relative to the Lean mechanization of the pinned WebAssembly semantics. The repository SHALL include a clause-by-clause conformance map from the pinned normative source to Lean declarations. Tests against an external reference interpreter are additional evidence, not a replacement for the formal semantics.

The disclosed logical trust base SHALL be exactly:

- the Lean kernel and the pinned Lean compiler used to check declarations;
- the pinned source tree recorded in the proof manifest;
- any Lean core logical axiom actually reported by transitive axiom collection, named individually.

The project SHALL NOT contain or depend on `sorryAx`, project-declared axioms, admitted theorems, an FFI truth oracle, `native_decide`, or an assumed compiler-correctness proposition. `Classical.choice` SHALL NOT produce executable witnesses. Any unexpected axiom makes the release gate fail.

Manifest identities SHALL be acyclic and SHALL use three ordered identity
stages followed by one non-self-bound external attestation:

1. `SourceManifestCore` covers immutable authority, handwritten Lean source,
   fixtures, and tool inputs while excluding every manifest and every generated
   output.
2. `GeneratedProofInputBody` binds
   `Identity SourceManifestCore.identitySchema sourceCore` plus the path,
   canonical bytes, and digest of every generated Lean source that is compiled
   on the final theorem path, including `Artifact/Bytes.lean`. Its own JSON
   encoding and every later output are excluded. `PreFinalEnvironmentBody`
   binds this identity, the exact Lean/toolchain/dependency identities, and the
   compiled declaration-environment digest used for the final proof check.
3. `OutputManifestBody` binds the source-core identity, generated-proof-input
   identity, pre-final-environment identity, artifact, Atlas seal, proof
   registry, generated documentation, and the frozen reproducibility plan,
   while excluding `MANIFEST.json` itself.
4. After two clean builds complete, CI emits a
   `ReproducibilityAttestationBody` that binds
   `Identity OutputManifestBody.identitySchema outputManifestBody`,
   both clean-tree input identities, and the two sets of compared output
   identities. It is a release-system attestation, is not an input to the Lean
   theorem or `OutputManifestBody`, is excluded from its own comparison set,
   and is not checked into the source tree.

`MANIFEST.json` is the canonical encoding of `OutputManifestBody`; its external
file digest is reported by CI but never included in its own preimage. The final
environment checker SHALL reject a generated source whose bytes differ from
`GeneratedProofInputBody`, and the artifact checker SHALL separately reject a
filesystem artifact whose bytes differ from the Lean term. No manifest or
generated file contains its own identity, and no two manifest stages hash each
other cyclically.

## 5. Final repository tree

The completed repository SHALL have this logical structure. Additional files are permitted only when owned by one of these layers and listed in `MANIFEST.json`.

```text
.
├── AGENTS.md
├── CONFORMANCE.md
├── LICENSE-APACHE
├── LICENSE-MIT
├── MANIFEST.json
├── README.md
├── SPEC.md
├── VERIFICATION.md
├── Justfile
├── lakefile.lean
├── lake-manifest.json
├── lean-toolchain
├── WasmGemmGnaf.lean
├── authority/
│   ├── manifest.json
│   ├── global-optimality-WGG-GO-1.json
│   ├── UOR-GNAF-v1-draft.2.md
│   ├── uor-gnaf.sha256
│   ├── wasm-core-wg-3.0.sha256
│   └── wasm-conformance-map.json
├── vendor/
│   └── wasm-spec/
│       └── <exact files from pinned commit required by the conformance map>
├── model/
│   ├── claims.json
│   ├── dependencies.json
│   ├── profiles.json
│   ├── reproducibility-plan.json
│   └── falsifiers.json
├── WasmGemmGnaf/
│   ├── Foundation/
│   │   ├── Result.lean
│   │   ├── Bytes.lean
│   │   ├── Canonical.lean
│   │   ├── Finite.lean
│   │   ├── Order.lean
│   │   ├── Identity.lean
│   │   └── Termination.lean
│   ├── Wasm/
│   │   ├── Revision.lean
│   │   ├── Feature.lean
│   │   ├── Types.lean
│   │   ├── Syntax.lean
│   │   ├── Binary.lean
│   │   ├── Validate.lean
│   │   ├── Numeric.lean
│   │   ├── Vector.lean
│   │   ├── Memory.lean
│   │   ├── Table.lean
│   │   ├── Store.lean
│   │   ├── Config.lean
│   │   ├── Step.lean
│   │   ├── Run.lean
│   │   ├── Fuel.lean
│   │   ├── Profile.lean
│   │   ├── Costed.lean
│   │   ├── Erasure.lean
│   │   └── Adequacy.lean
│   ├── Gemm/
│   │   ├── Scalar.lean
│   │   ├── FloatBits.lean
│   │   ├── Arithmetic.lean
│   │   ├── Shape.lean
│   │   ├── Layout.lean
│   │   ├── Aliasing.lean
│   │   ├── Descriptor.lean
│   │   ├── ABI.lean
│   │   ├── Classify.lean
│   │   ├── Reference.lean
│   │   ├── Observe.lean
│   │   └── Problem.lean
│   ├── Cost/
│   │   ├── Event.lean
│   │   ├── Vector.lean
│   │   ├── Trace.lean
│   │   ├── Aggregate.lean
│   │   ├── Lifecycle.lean
│   │   ├── Objective.lean
│   │   ├── Proper.lean
│   │   └── CanonicalObjective.lean
│   ├── GNAF/
│   │   ├── Object.lean
│   │   ├── Shape.lean
│   │   ├── Edge.lean
│   │   ├── Plan.lean
│   │   ├── Typing.lean
│   │   ├── Semantics.lean
│   │   ├── Resource.lean
│   │   ├── Normalize.lean
│   │   ├── Compile.lean
│   │   └── CompileCorrect.lean
│   ├── Universal/
│   │   ├── Competitor.lean
│   │   ├── Correct.lean
│   │   ├── Feasible.lean
│   │   ├── Sublevel.lean
│   │   ├── EnumerateBytes.lean
│   │   ├── EnumerateInputs.lean
│   │   ├── CheckExecution.lean
│   │   ├── Partition.lean
│   │   ├── Coverage.lean
│   │   ├── LowerBound.lean
│   │   └── Argmin.lean
│   ├── Atlas/
│   │   ├── State.lean
│   │   ├── SemanticClosure.lean
│   │   ├── Attention.lean
│   │   ├── Dependency.lean
│   │   ├── CostSurface.lean
│   │   ├── Envelope.lean
│   │   ├── Certificate.lean
│   │   ├── Delta.lean
│   │   ├── Update.lean
│   │   ├── Rebuild.lean
│   │   ├── Seal.lean
│   │   ├── Lifecycle.lean
│   │   └── Query.lean
│   ├── Artifact/
│   │   ├── Baseline.lean
│   │   ├── Select.lean
│   │   ├── Emit.lean
│   │   ├── Bytes.lean
│   │   ├── Manifest.lean
│   │   └── Execute.lean
│   ├── Theorems/
│   │   ├── WasmModel.lean
│   │   ├── GemmTotal.lean
│   │   ├── BaselineCorrect.lean
│   │   ├── CompilerCorrect.lean
│   │   ├── SublevelComplete.lean
│   │   ├── AttentionComplete.lean
│   │   ├── UpdateEqualsRebuild.lean
│   │   ├── UniversalLowerBound.lean
│   │   ├── Attainment.lean
│   │   ├── ArtifactCorrect.lean
│   │   ├── ArtifactGlobal.lean
│   │   ├── LifecycleBound.lean
│   │   └── Release.lean
│   └── Conformance/
│       ├── Claim.lean
│       ├── Registry.lean
│       ├── Manifest.lean
│       ├── AxiomAudit.lean
│       └── ReleaseGate.lean
├── Tools/
│   ├── Main.lean
│   ├── EmitArtifact.lean
│   ├── GenerateManifest.lean
│   ├── GenerateConformance.lean
│   ├── CheckClaims.lean
│   ├── CheckAxioms.lean
│   ├── CheckSources.lean
│   └── Reproduce.lean
├── Tests/
│   ├── Binary.lean
│   ├── Validate.lean
│   ├── Numeric.lean
│   ├── GemmReference.lean
│   ├── GnafCompiler.lean
│   ├── UniversalCoverage.lean
│   ├── AtlasUpdate.lean
│   ├── Lifecycle.lean
│   ├── Artifact.lean
│   └── Mutation.lean
├── fixtures/
│   ├── wasm-spec-tests/
│   ├── gemm-cases/
│   └── mutations/
├── artifacts/
│   ├── wasm-gemm-gnaf.wasm
│   ├── atlas-seal.bin
│   ├── generated-proof-input.json
│   ├── pre-final-environment.json
│   └── proof-manifest.json
└── .github/workflows/
    ├── verify.yml
    ├── reproducible.yml
    └── mutation.yml
```

The Rust workspace, `Cargo.toml`, `Cargo.lock`, `.cargo`, `crates`, `xtask`, Rust toolchain files, and Rust lint configuration from the template SHALL be removed after the Lean conformance replacement passes. No Rust code is part of the final product or proof path.

The vendored authority files are immutable inputs. `MANIFEST.json` SHALL enumerate every vendored path and digest, so offline verification recomputes the pins from content rather than trusting a checksum string.

## 6. Foundation layer

### 6.1 Result algebra

`Foundation/Result.lean` SHALL define closed, payload-bearing result types. At minimum:

```lean
inductive CheckResult (α ε : Type)
  | complete (value : α)
  | rejected (error : ε)
  | unresolved (missing : MissingSet)
  | unsupported (feature : FeatureId)
  | resourceExhausted (report : ResourceReport)
  | internalFailure (report : FailureReport)

inductive SolveResult (α : Type)
  | optimal (value : α) (certificate : GlobalCertificate value)
  | pareto (frontier : NonemptyCanonicalFrontier α)
  | infeasible (proof : InfeasibilityCertificate)
  | nonattained (proof : NonattainmentCertificate)
  | incomplete (coverage : CoverageGap)
  | unsealed (head : StateIdentity)
  | resourceExhausted (report : ResourceReport)
  | internalFailure (report : FailureReport)
```

No constructor named `optimal` may be created without an exact global certificate whose verifier reconstructs the final proposition.

### 6.2 Canonical identity

Proof identity SHALL be structural:

```lean
inductive CanonicalDomainTag
  | wasmProfile | gemmProblem | costObjective | atlasState
  | atlasSealCheck | delta | request | queryResult
  | manifest | authority | artifact | generic

structure CanonicalSchema (α : Type) where
  version : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  encode : α → ByteArray
  encode_injective : Function.Injective encode
  typeTagNonempty : typeTag.size > 0

structure ObjectId (α : Type) where
  schemaVersion : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  canonicalBodyBytes : ByteArray

def Identity
    {α : Type} (schema : CanonicalSchema α) (body : α) : ObjectId α :=
  { schemaVersion := schema.version
    domain := schema.domain
    typeTag := schema.typeTag
    canonicalBodyBytes := schema.encode body }

theorem Identity_eq_iff
    {α : Type} (schema : CanonicalSchema α) {left right : α} :
  Identity schema left = Identity schema right ↔ left = right := by
  constructor
  · intro h
    exact schema.encode_injective
      (congrArg ObjectId.canonicalBodyBytes h)
  · intro h
    cases h
    rfl

structure CanonicalObjectId where
  schemaVersion : Nat
  domain : CanonicalDomainTag
  typeTag : ByteArray
  canonicalBodyBytes : ByteArray

def CanonicalObjectId.ofTyped
    {α : Type}
    (id : ObjectId α) : CanonicalObjectId :=
  { schemaVersion := id.schemaVersion
    domain := id.domain
    typeTag := id.typeTag
    canonicalBodyBytes := id.canonicalBodyBytes }

abbrev ProfileId := CanonicalObjectId
abbrev ProblemId := CanonicalObjectId
abbrev ObjectiveId := CanonicalObjectId
abbrev StateIdentity := CanonicalObjectId
abbrev FeatureId := CanonicalObjectId
abbrev DeltaId := CanonicalObjectId
abbrev RequestId := CanonicalObjectId
abbrev QueryResultId := CanonicalObjectId
```

A SHA-256 digest MAY index an object, but equality and proof reconstruction SHALL compare the canonical body. No proof may assume cryptographic collision resistance.

Canonical encoders SHALL be prefix-free, versioned, and injective. Map and set encodings SHALL use a proved total canonical order. Each identity equation SHALL be acyclic; no body may contain the identity derived from that same body.

Every identity call SHALL name one frozen `CanonicalSchema`; schemas are not
typeclass arguments. `Foundation/SchemaRegistry.lean` SHALL retain the finite
registry of every schema used by the release and prove that equal
`(schemaVersion, domain, typeTag)` triples identify the same registered body
type and encoder. Product/sum schemas derive their type tags structurally from
their component tags. Consequently, erasing a typed ID to
`CanonicalObjectId` cannot merge two registered types, and one body cannot be
assigned a second encoder by local instance selection. The `.generic` domain
still requires a unique nonempty structural `typeTag`.

Scope identities SHALL never encode functions or proof terms. The concrete carriers have this split:

```lean
structure Wasm.Profile where
  body : Wasm.ProfileBody       -- includes the first-order costTableBody
  lawful : Wasm.ProfileLawful body

def Wasm.Profile.costTableBody
    (profile : Wasm.Profile) : Wasm.CostTableBody :=
  profile.body.costTableBody

structure Gemm.Problem (profile : Wasm.Profile) where
  body : Gemm.ProblemBody profile.body  -- first-order kinds/modes/ABI/resources
  lawful : Gemm.ProblemLawful profile body

def Wasm.ProfileId (profile : Wasm.Profile) : ProfileId :=
  CanonicalObjectId.ofTyped
    (Identity Wasm.ProfileBody.identitySchema profile.body)

def Gemm.ProblemId
    {profile : Wasm.Profile}
    (problem : Gemm.Problem profile) : ProblemId :=
  CanonicalObjectId.ofTyped
    (Identity (Gemm.Problem.identitySchema profile)
      (Wasm.ProfileId profile, problem.body))

def Cost.ObjectiveId
    {profile : Wasm.Profile}
    {problem : Gemm.Problem profile}
    (objective : Cost.ProperObjective profile problem) : ObjectiveId :=
  CanonicalObjectId.ofTyped
    (Identity (Cost.Objective.identitySchema profile problem)
      (Wasm.ProfileId profile, Gemm.ProblemId problem, objective.body))
```

Semantic functions and law proofs are derived from these bodies and checked separately. This split applies to every state, partition, certificate, and manifest reference.

### 6.3 Finiteness and termination

All enumerators SHALL expose `Fintype` or finite-list coverage theorems. All recursive implementations SHALL be structural or use an explicit well-founded measure. Executable proof-producing functions SHALL not be partial or noncomputable.

## 7. Formal WebAssembly layer

### 7.1 Concrete semantics, not conclusion parameters

The generic interface MAY abstract raw semantic components:

```lean
structure Wasm.SpecMachine where
  Module Config Invocation Event Outcome Fault Trap ExceptionValue : Type
  decode   : ByteArray → Except Fault Module
  validate : Module → Bool
  initial  : Module → Invocation → Except Fault Config
  Step     : Config → Event → Config → Prop
  Halt     : Config → Outcome → Prop
  Trapped  : Config → Trap → Prop
  Thrown   : Config → ExceptionValue → Prop
  successors : Config → List (Event × Config)
  terminal : (config : Config) →
    Decidable (
      (∃ outcome, Halt config outcome) ∨
      (∃ trap, Trapped config trap) ∨
      (∃ exceptionValue, Thrown config exceptionValue))

def Wasm.SpecMachine.IsTerminal
    (M : Wasm.SpecMachine) (config : M.Config) : Prop :=
  (∃ outcome, M.Halt config outcome) ∨
  (∃ trap, M.Trapped config trap) ∨
  (∃ exceptionValue, M.Thrown config exceptionValue)

structure Wasm.InvocationBody (P : Wasm.Profile) where
  ptr : UInt32
  len : UInt32
  bytes : ByteArray

def Wasm.InvocationLawful
    (P : Wasm.Profile) (body : Wasm.InvocationBody P) : Prop :=
  body.bytes.size = body.len.toNat ∧
  body.ptr.toNat + body.len.toNat ≤ 2^P.addressBits ∧
  pagesFor (body.ptr.toNat + body.len.toNat) ≤ P.maxPages

structure Wasm.Invocation (P : Wasm.Profile) where
  body : Wasm.InvocationBody P
  lawful : Wasm.InvocationLawful P body

def Wasm.Invocation.gemmRaw
    {P : Wasm.Profile} (body : Wasm.InvocationBody P)
    (lawful : Wasm.InvocationLawful P body) : Wasm.Invocation P :=
  { body, lawful }
```

It SHALL NOT contain fields named or equivalent to `candidateCorrect`, `allProgramsCovered`, `lowerBound`, `globallyOptimal`, or a truth-producing oracle.

The concrete machine SHALL prove:

```lean
theorem mem_successors_iff_step :
  (event, next) ∈ M.successors config ↔ M.Step config event next

theorem successors_nodup (config : M.Config) :
  (M.successors config).Nodup

theorem terminal_iff_halt_trap_or_throw (config : M.Config) :
  M.IsTerminal config ↔
    (∃ outcome, M.Halt config outcome) ∨
    (∃ trap, M.Trapped config trap) ∨
    (∃ exceptionValue, M.Thrown config exceptionValue)
```

`successors` enumerates every permitted nondeterministic successor. A single evaluator path is not a replacement for this list.

`Wasm/Revision.lean` through `Wasm/Run.lean` SHALL construct the concrete pinned Core 3.0 model. Every enabled instruction, validation rule, store component, trap, and nondeterministic numeric result in the released profile SHALL have a Lean rule and a conformance-map entry.

Ownership is exhaustive: `Syntax` owns types, functions, globals, tables, memories, elements, data, imports, exports, and administrative instructions; `Validate` owns all context and declaration judgments; `Store` owns allocation and runtime instances; `Config` owns frames, labels, stacks, threads of control, instantiation, start invocation, and export calls; `Numeric` and `Vector` own set-valued numeric relations; `Step` owns every reduction rule; and `Run` owns observable terminal stores and maximal executions. `Adequacy` proves correspondence between these declarations and the vendored rule identifiers in the conformance map.

`profile_matches_pinned_revision` means that the concrete model and map are identity-bound to the vendored revision and that every enabled vendored rule has exactly one mapped Lean declaration. It does not claim that Lean can derive English prose from bytes. The reviewed transcription from normative rules to initial Lean definitions is an explicitly disclosed authority boundary; all subsequent GEMM, cost, coverage, and optimality reasoning is kernel checked.

### 7.2 Released portable profile

`Wasm.Profile` SHALL bind:

- the exact WebAssembly revision and enabled standard proposals;
- wasm32/wasm64 address width;
- module, memory, table, managed-GC-heap, stack, and invocation limits;
- import policy and initial store;
- exported ABI;
- permitted nondeterminism;
- validation and execution semantics identities;
- cost semantics identity.

The release profile SHALL be closed: no host functions, clocks, filesystem, network, accelerator, environment query, shared mutable host state, or unmodeled import. Standard WebAssembly SIMD may be enabled only if fully formalized. It is not AVX2 or NEON specialization. Relaxed SIMD, threads, exceptions, GC, memory64, and other proposals are either fully modeled and enabled or rejected by validation; documentation SHALL list the exact decision for each feature.

The decoder SHALL recognize the complete pinned Core 3.0 binary grammar. `validateUnder Release.wasmProfile` then applies this exact feature matrix; no feature defaults are permitted:

| Core 3.0 feature family | Release decision |
|---|---|
| scalar numeric, control, variables, functions | enabled |
| multi-value and extended constant expressions | enabled |
| bulk memory, multiple memories, tables | enabled |
| standard fixed 128-bit SIMD | enabled |
| reference types, typed function references, and GC | enabled |
| tail calls | enabled |
| exception handling | enabled |
| memory64 address types and instructions | rejected |
| shared memories, atomics, and threads | rejected |
| relaxed SIMD operations | rejected |
| component-model binaries and post-Core-3 proposals | rejected |

The first release SHALL use this concrete profile, with no implementation choice left implicit:

- module language: every pinned Core 3.0 form marked enabled in the matrix; disabled forms decode when grammatically valid and then fail profile validation;
- address model: wasm32;
- vector model: standard 128-bit vector instructions, excluding the relaxed family;
- imports: empty;
- shared memories and proposal features outside the pinned core grammar: rejected;
- memory: ordinary wasm32 memories subject to the normative 65,536-page limit; memory zero carries the ABI, and any additional core-valid memory is charged;
- tables, globals, data, element segments, start functions, and additional exports: permitted when core-valid and fully charged;
- required exports: one memory named `memory` and one function named `gemm` with ABI type `(i32, i32) → i32`;
- initial configuration: deterministic allocation, global/table/memory/data
  initialization, and construction of a harness control frame before executing
  a module start function; the costed harness then explores every permitted
  start-function branch, copies the raw invocation bytes only after a normal
  start return, and invokes `gemm`;
- permitted numeric nondeterminism: exactly the relation in the pinned semantics, with correctness and cost quantified over every permitted result and trace;
- host identity, engine tier, cache warmth, ISA, and wall-clock time: absent from the profile.

A wasm64, shared-memory, component-model, host-import, or later-feature claim requires a new profile and a new closed theorem. It cannot inherit the release theorem by name similarity.

The concrete release carrier is a checked literal, not a profile parameter
chosen by an implementation:

```lean
def Release.wasmProfileBody : Wasm.ProfileBody :=
  Wasm.canonicalCore3Wasm32ProfileBody
    (revisionCommit := "9d36019973201a19f9c9ebb0f10828b2fe2374aa")
    (costTableBody := Release.wasmCostTableBody)

def Release.wasmProfile : Wasm.Profile :=
  Wasm.Profile.checked Release.wasmProfileBody

theorem Release.wasmProfile_body :
  Release.wasmProfile.body = Release.wasmProfileBody := rfl
```

`canonicalCore3Wasm32ProfileBody` is the proof-free record whose projections
are exactly the feature decisions, limits, imports, exports, initial-store
protocol, nondeterminism, semantics identities, and cost identity stated in
this section. Its definition is unfolded and identity-checked by the release
gate; it has no default-feature or environment argument.

### 7.3 Decoder and validator

`Wasm/Binary.lean` SHALL implement canonical and permissive LEB128 handling exactly as pinned by the spec, section ordering, sizes, names, types, instructions, and malformed input behavior.

Required theorems include:

```lean
theorem decode_sound
    (h : Wasm.decode bytes = .ok module) :
  Wasm.DeclarativeBinaryRelation bytes module

theorem decode_complete
    (h : Wasm.DeclarativeBinaryRelation bytes module) :
  Wasm.decode bytes = .ok module

theorem encode_decode_roundtrip (module : Wasm.Module) :
  Wasm.decode (Wasm.encode module) = .ok module

theorem validate_bool_iff (module : Wasm.Module) :
  Wasm.validate module = true ↔ Wasm.DeclarativelyValid module

theorem validation_preservation
    (hvalid : Wasm.DeclarativelyValid module)
    (hstep : Wasm.Step config event next)
    (hconfig : Wasm.ConfigInstantiates module config) :
  Wasm.ConfigWellTyped next

theorem validation_progress
    (hvalid : Wasm.DeclarativelyValid module)
    (hconfig : Wasm.ConfigInstantiates module config)
    (hwelltyped : Wasm.ConfigWellTyped config) :
  (∃ outcome, Wasm.Halt config outcome) ∨
  (∃ trap, Wasm.Trapped config trap) ∨
  (∃ exceptionValue, Wasm.Thrown config exceptionValue) ∨
  (Wasm.successors config).Nonempty
```

### 7.4 Execution

`Wasm/Step.lean` SHALL define relational reduction. `Wasm/Run.lean` SHALL define:

```lean
inductive ExecutionObservation
  | returned
      (trace : List Wasm.Event)
      (gemmEntryObservableStore : ObservableStore)
      (value : Wasm.Value)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  | trappedBeforeEntry
      (trace : List Wasm.Event)
      (trap : Wasm.Trap)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  | trappedAfterEntry
      (trace : List Wasm.Event)
      (gemmEntryObservableStore : ObservableStore)
      (trap : Wasm.Trap)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  | thrownBeforeEntry
      (trace : List Wasm.Event)
      (exceptionValue : Wasm.ExceptionValue)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  | thrownAfterEntry
      (trace : List Wasm.Event)
      (gemmEntryObservableStore : ObservableStore)
      (exceptionValue : Wasm.ExceptionValue)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)

structure SemanticObservation where
  terminal : Wasm.TerminalStatus
  finalObservableStore : ObservableStore
  effects : ObservableEffects

def ExecutionObservation.unmaskedSemantic : ExecutionObservation → SemanticObservation
  | .returned _ _ value store effects => projectReturned value store effects
  | .trappedBeforeEntry _ trap store effects => projectTrap trap store effects
  | .trappedAfterEntry _ _ trap store effects => projectTrap trap store effects
  | .thrownBeforeEntry _ exceptionValue store effects =>
      projectThrown exceptionValue store effects
  | .thrownAfterEntry _ _ exceptionValue store effects =>
      projectThrown exceptionValue store effects

def ExecutionObservation.trace : ExecutionObservation → List Wasm.Event
  | .returned trace _ _ _ _ => trace
  | .trappedBeforeEntry trace _ _ _ => trace
  | .trappedAfterEntry trace _ _ _ _ => trace
  | .thrownBeforeEntry trace _ _ _ => trace
  | .thrownAfterEntry trace _ _ _ _ => trace

def ExecutionObservation.finalObservableStore :
    ExecutionObservation → ObservableStore
  | .returned _ _ _ store _ => store
  | .trappedBeforeEntry _ _ store _ => store
  | .trappedAfterEntry _ _ _ store _ => store
  | .thrownBeforeEntry _ _ store _ => store
  | .thrownAfterEntry _ _ _ store _ => store

def ExecutionObservation.gemmEntryObservableStore? :
    ExecutionObservation → Option ObservableStore
  | .returned _ entry _ _ _ => some entry
  | .trappedBeforeEntry _ _ _ _ => none
  | .trappedAfterEntry _ entry _ _ _ => some entry
  | .thrownBeforeEntry _ _ _ _ => none
  | .thrownAfterEntry _ entry _ _ _ => some entry

def Gemm.semanticFor
    {P : Wasm.Profile}
    (problem : Gemm.Problem P) (raw : problem.RawInvocation)
    (observation : ExecutionObservation) : SemanticObservation :=
  let unmasked := observation.unmaskedSemantic
  { unmasked with
    finalObservableStore :=
      problem.observableStoreProjection raw unmasked.finalObservableStore }

inductive MaximalExecution (initial : Wasm.Config)
  | finite
      (observation : ExecutionObservation)
      (run : FiniteExecution initial observation)
      (maximal : IsTerminalObservation observation)
  | diverges (events : Nat → Wasm.Event) (configs : Nat → Wasm.Config)
      (starts : configs 0 = initial)
      (step : ∀ i, Wasm.Step (configs i) (events i) (configs (i + 1)))
```

The exact observation includes return status, final ABI-visible memory, permitted effects, and trace-derived resources. Correctness SHALL quantify over every maximal permitted execution, not one favorable evaluator trace.

`gemmEntryObservableStore` is the exact store snapshot immediately after a
normally returned start function and the harness's raw-byte installation, and
immediately before entry to the exported `gemm`. The trace phase tags prove
that this boundary occurs exactly once on a normally started invocation.
`trappedBeforeEntry` and `thrownBeforeEntry` are the only constructors for a
terminal start/initialization branch and deliberately carry no entry store;
`returned`, `trappedAfterEntry`, and `thrownAfterEntry` carry exactly one.
An uncaught Core exception remains observably distinct from a trap. The
relational run rules prove those presence invariants, so no sentinel or
arbitrary store can enter a functional evaluation.
Harness installation writes occur before that boundary; all their bytes and
steps are charged but they are not candidate-call writes. `ObservableEffects`
contains only externally visible non-memory effects admitted by the closed
profile; ordinary instruction, load/store, and intermediate C-write order
remain in `trace` and the cost vector, not in semantic equality.
`Gemm.semanticFor problem raw` projects the returned/trapped/thrown status, the exact
raw-dependent ABI-visible store, and those external effects; it masks a
validated scratch range and nothing else. Therefore two correct algorithms
may have different traces, scratch bytes, and costs while sharing one
deterministic semantic observation.

`Wasm/Fuel.lean` SHALL provide a verified executable runner:

```lean
theorem runFuel_sound
    (hmember : Wasm.TreeContains (Wasm.exploreAll bound initial) observation) :
  Wasm.FiniteExecution initial observation

theorem runFuel_complete_with_bound
    (hrun : Wasm.FiniteExecution initial observation)
    (hlen : observation.trace.length ≤ bound) :
  Wasm.TreeContains (Wasm.exploreAll bound initial) observation
```

The executable object is a bounded execution-tree explorer, not a one-path interpreter:

```lean
inductive ExecutionTreeResult (bound : Nat) (initial : Config)
  | complete (observations : NonemptyCanonicalList ExecutionObservation)
      (allBranchesMaximal : CoversEveryMaximalFiniteBranch initial observations)
  | nonterminalPrefix (trace : TraceOfLength (bound + 1))
  | initializationFailure (fault : Fault)

def exploreAll (bound : Nat) (initial : Config) : ExecutionTreeResult bound initial
```

Soundness SHALL map every returned observation to a relational execution. Completeness SHALL prove that every relational branch of length at most the bound occurs in the tree, and that `complete` contains every maximal branch. `nonterminalPrefix` proves the module cannot satisfy a smaller all-branch step sublevel. Completeness is relative to an explicit bound derived independently from the baseline score and objective properness. No general halting oracle is permitted.

### 7.5 Costed semantics

`Wasm/Costed.lean` SHALL label every semantic transition with exact abstract events. `Wasm/Erasure.lean` SHALL prove that removing cost labels yields exactly the ordinary execution:

```lean
theorem costed_erase_iff_plain_run :
  Wasm.CostedRun P module invocation costedTrace observation ↔
    Wasm.Run module invocation (eraseCosts costedTrace) observation

structure Wasm.InitializationObservation (P : Wasm.Profile) where
  initial : Wasm.Config
  costedEvents : List Wasm.CostedInitializationEvent
  configs : NonemptyCanonicalList Wasm.ConfigResourceSnapshot
  cost : Cost.DynamicVector
  costExact : cost =
    Cost.foldInitialization P.costTableBody configs costedEvents

def Wasm.initialGemmInvocationCosted
    (P : Wasm.Profile) (module : Wasm.Module)
    (invocation : Wasm.Invocation P) :
    Except Wasm.Fault (Wasm.InitializationObservation P)

theorem costed_initialization_sound
    (h : Wasm.initialGemmInvocationCosted P module invocation =
      .ok initialization) :
  Wasm.CostedInitialization P module invocation
    initialization.costedEvents initialization.configs
      initialization.initial

theorem costed_initialization_complete
    (h : Wasm.Initializes P module invocation initial) :
  ∃ initialization : Wasm.InitializationObservation P,
    Wasm.initialGemmInvocationCosted P module invocation =
      .ok initialization ∧
    initialization.initial = initial ∧
    Wasm.CostedInitialization P module invocation
      initialization.costedEvents initialization.configs initial

theorem costed_initialization_failure_iff
    (fault : Wasm.Fault) :
  Wasm.initialGemmInvocationCosted P module invocation = .error fault ↔
    Wasm.InitializationFailure P module invocation fault

theorem costed_initialization_erase
    (h : Wasm.initialGemmInvocationCosted P module invocation =
      .ok initialization) :
  Wasm.initialGemmInvocation P module invocation = .ok initialization.initial

theorem initialization_cost_exact
    (initialization : Wasm.InitializationObservation P) :
  initialization.cost =
    Cost.foldInitialization P.costTableBody
      initialization.configs initialization.costedEvents :=
  initialization.costExact

theorem costed_initialization_functional
    (left right : Wasm.InitializationObservation P)
    (hl : Wasm.initialGemmInvocationCosted P module invocation = .ok left)
    (hr : Wasm.initialGemmInvocationCosted P module invocation = .ok right) :
  left = right
```

`Wasm.CostedInitialization` is the relational allocation/static-initialization
judgment for the exact module and raw invocation. Its rules emit every event
and resource snapshot consumed by `foldInitialization`; its erasure is the
plain initializer. Soundness plus `costExact` therefore prevents a correct
plain configuration from being paired with a fabricated zero-cost trace.
`costed_initialization_complete` and `costed_initialization_failure_iff`
prevent the executable initializer from excluding a relationally valid
competitor by spuriously failing; neither theorem is stated in terms of
`Universal.evaluate` or the selected artifact.

`initialGemmInvocationCosted` stops at the pre-start harness configuration.
Allocation and static initialization are recorded in `initialization.cost`.
The module start function, raw-byte installation, exported call, and all of
their permitted nondeterministic choices are ordinary costed `Step` transitions
enumerated by `exploreAll`; a start trap is therefore visible rather than
collapsed by an initializer. Cost instrumentation SHALL not alter control,
values, traps, memory, or observable results.

Each `Wasm.ProfileBody` contains one first-order cost-table body, exposed as
`Wasm.Profile.costTableBody`; its canonical identity is the profile's
cost-semantics identity. The release labeling is fixed by
`Release.wasmCostTableBody = Release.wasmProfile.costTableBody`. That table
contains every pinned Core rule identifier and
the following total contribution law:

- `decodeCost bytes = bytes.size + 1` (one unit per consumed byte and one
  terminal accept/reject unit), independent of decoder implementation;
- `validationCost module` is one unit per node plus one unit per premise edge
  in the unique canonical declarative-validation derivation;
- every relational Core `Step` contributes one `wasmRuleSteps` unit;
- `dispatchSteps` additionally counts exactly branch, branch-table, direct or
  indirect call, return, exception transfer, tail-call transfer, and harness
  phase-transition rule identifiers;
- `scalarOps` is the number of scalar primitive numeric operations performed
  by the rule; `vectorLaneOps` is the exact number of active 128-bit lanes
  operated on, with a whole-vector shuffle contributing 16 byte lanes;
- each load/store/table/data/memory event contributes its exact accessed or
  allocated byte, page, or element count; failed or trapping accesses
  contribute the attempted rule step but no completed transfer;
- each GC struct/array or exception-object allocation contributes one
  `gcObjectsAllocated`, the canonical abstract byte width of every allocated
  field, element, exception tag, and payload to
  `gcBytesInitialized`, and every configuration contributes its total live
  managed-heap and live exception-object width to `peakGcLiveBytes`; reference
  width, header width, exception header width, payload layout, and packed field
  widths are fixed in `Release.wasmCostTableBody`, so a runtime-sized Core GC
  or exception allocation is never a unit-cost free store;
- raw installation contributes one `preparationSteps` and one `bytesWritten`
  unit per installed byte; module packing performed by candidate Wasm is
  ordinary Core work, never a free preparation callback;
- `outputBytes` is the number of sanctioned observable C/status bytes in the
  terminal relation; scratch is excluded; and
- `peakStackValues`, `peakPages`, and every other peak are maxima over the
  complete costed configuration sequence, while all nonpeak coordinates are
  sums.

For the released wasm32 GC profile, canonical cost widths are: references and
`i32`/`f32` fields 4 bytes, `i64`/`f64` fields 8, `v128` fields 16, packed
`i8`/`i16` fields 1/2, a struct header 8, and an array header 16. Struct fields
occur in declared order at the next multiple of `min(fieldWidth, 8)` and the
total rounds to a multiple of 8. Array stride is the element width rounded to
its `min(width, 8)` alignment and total size is header plus `length × stride`,
rounded to 8. An exception object has a 16-byte header; its payload fields use
the same declared-order field layout as a struct after that header, and its
total also rounds to 8. A caught exception remains live exactly while its
exception reference is reachable from the abstract store or control stack.
These are abstract cost bytes, not host-layout claims. They fix all GC and
exception allocation, initialization, and live-heap rows in
`Release.wasmCostTableBody`.

The body is constructed, not supplied by a caller:

```lean
def Release.wasmCostTableBody : Wasm.CostTableBody :=
  Wasm.buildCanonicalCostTable
    Release.core3RuleConformanceMap
    Release.canonicalRuleContribution
    Release.canonicalInitializationContribution
    Release.canonicalGcAndExceptionLayout

theorem Release.profile_cost_table_exact :
  Release.wasmProfile.costTableBody = Release.wasmCostTableBody := rfl
```

`PinnedCoreRuleId` is the finite inductive generated from the vendored
conformance map. `canonicalRuleContribution` and
`canonicalInitializationContribution` are exhaustive structural matches over
that inductive and the finite harness-event inductive, implementing the
formula above; neither has a default arm. `buildCanonicalCostTable` accepts
only a duplicate-free exact cover of every enabled rule identifier. The
compiled unfolded bodies and their canonical bytes are compared to the frozen
cost-profile projection of the `WGG-GO-1` authority proposition, rather than recorded from the
implementation after the fact.

`initialization.cost` is the exact fold over allocation and static
initialization events. `Cost.traceVector` is the exact mixed sum/maximum fold
over the event contributions and visited configurations. The repository SHALL
prove table coverage and exclusivity:

```lean
theorem wasm_cost_table_total (event : Wasm.Event) :
  ∃! contribution : Cost.DynamicVector,
    Cost.EventContribution Release.wasmCostTableBody event contribution

```

The evaluator retains the data needed for that equality; a terminal store and
an unannotated event list are not treated as a cost oracle:

```lean
structure Wasm.CostedExecutionObservation
    (P : Wasm.Profile) (initial : Wasm.Config) where
  observation : Wasm.ExecutionObservation
  costedTrace : List Wasm.CostedEvent
  configs : NonemptyCanonicalList Wasm.ConfigResourceSnapshot
  run : Wasm.CostedFiniteExecution P initial costedTrace configs observation
  maximal : Wasm.IsTerminalObservation observation
  cost : Cost.DynamicVector
  costExact : cost =
    Cost.foldTrace P.costTableBody configs costedTrace

def Wasm.CostedExecutionFor
    (P : Wasm.Profile) (module : Wasm.Module)
    (invocation : Wasm.Invocation P)
    (initialization : Wasm.InitializationObservation P)
    {initial : Wasm.Config}
    (costed : Wasm.CostedExecutionObservation P initial) : Prop :=
  Wasm.initialGemmInvocationCosted P module invocation =
      .ok initialization ∧
  initialization.initial = initial ∧
  Wasm.CostedInitialization P module invocation
    initialization.costedEvents initialization.configs initial ∧
  Wasm.CostedFiniteExecution P initial
    costed.costedTrace costed.configs costed.observation

def Wasm.MaximalExecution.CostedAs
    {P : Wasm.Profile}
    {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial)
    (costed : Wasm.CostedExecutionObservation P initial) : Prop :=
  execution = .finite costed.observation costed.run.erase costed.maximal

theorem costed_observation_cost_exact
    {P : Wasm.Profile}
    {initial : Wasm.Config}
    (costed : Wasm.CostedExecutionObservation P initial) :
  costed.cost =
    Cost.foldTrace P.costTableBody
      costed.configs costed.costedTrace :=
  costed.costExact

inductive Wasm.CostedExecutionTreeResult
    (P : Wasm.Profile) (bound : Nat) (initial : Wasm.Config)
  | complete
      (observations :
        NonemptyCanonicalList (Wasm.CostedExecutionObservation P initial))
      (coverage : CoversEveryMaximalCostedBranch P initial observations)
  | nonterminalPrefix (trace : TraceOfLength (bound + 1))
  | initializationFailure (fault : Wasm.Fault)

def Wasm.exploreAllCosted
    (P : Wasm.Profile) (bound : Nat) (initial : Wasm.Config) :
    Wasm.CostedExecutionTreeResult P bound initial

theorem exploreAllCosted_erases
    (P : Wasm.Profile) (bound : Nat) (initial : Wasm.Config)
    (observations :
      NonemptyCanonicalList (Wasm.CostedExecutionObservation P initial))
    (coverage : CoversEveryMaximalCostedBranch P initial observations)
    (h : Wasm.exploreAllCosted P bound initial =
      .complete observations coverage) :
  Wasm.exploreAll bound initial =
    .complete (observations.map (·.observation)) coverage.erase
```

`ConfigResourceSnapshot` contains total live value slots across every operand
stack, frame-local vector, label/handler payload, and exception payload;
ordinary-memory pages; table size; live managed-GC/exception bytes; and every
other peak input named by the cost vector. Function-entry local initialization
is an exact counted allocation event, including run-length-expanded local
declarations. `InitializationObservation` and `CostedExecutionObservation`
are proof-carrying evaluation evidence, not part of semantic equality. Every
system evaluation retains them and the plain configurations/observations they
erase to.

No decoder, validator, interpreter, compiler, or candidate may redefine these
charges. Changing any rule row or fold law changes the cost-profile identity
and the final proposition.

## 8. Exact GEMM problem

### 8.1 Descriptor

```lean
structure Gemm.Descriptor (P : Wasm.Profile) where
  m n k batch : Nat
  aKind bKind cKind accumulatorKind : ScalarKind
  aLayout : Layout aKind (batch, m, k)
  bLayout : Layout bKind (batch, k, n)
  cLayout : Layout cKind (batch, m, n)
  transposeA transposeB : Bool
  arithmetic : ArithmeticContract
    aKind bKind cKind accumulatorKind
  alpha beta : ScalarValue cKind
  aliasing : AliasingContract
  observation : ObservationContract
  resources : ResourceContract
  kindsCompatible : arithmetic.Compatible
  wellFormed : DescriptorWellFormed P m n k batch
    aLayout bLayout cLayout aliasing resources
```

`Layout` SHALL encode base, extent, signed or unsigned strides as admitted, alignment, element width, and accessible memory region. Well-formedness SHALL prove every address calculation stays within the bound memory using overflow-aware arithmetic.

The released ABI has no alignment field and imposes byte alignment only:
`Layout.alignment` is the derived constant one for every stored kind. Wasm
unaligned accesses remain legal and charged. A stronger alignment promise
requires a new ABI/problem identity; it cannot be inferred from an address or
used to reject an otherwise in-bounds raw invocation.

The first release SHALL admit exactly these stored scalar encodings: signed and unsigned 8-, 16-, 32-, and 64-bit integers; IEEE binary16, bfloat16, binary32, and binary64. Smaller integer and 16-bit floating operations may be implemented in software or standard vector lanes, but remain governed by the same Wasm trace semantics. Accumulators SHALL be selected explicitly from the compatible 32-bit, 64-bit, binary32, binary64, or exact-dyadic modes defined by `ArithmeticContract`.

### 8.2 Arithmetic contracts

No ring, semiring, associativity, distributivity, reassociation, FMA equivalence, or zero law is implicit. `ArithmeticContract` SHALL provide the exact operations and prove only the laws it advertises.

Required released modes SHALL be declared individually. Each mode SHALL specify:

- operand decoding and output encoding;
- multiplication, addition, `alpha`, and `beta` evaluation order;
- accumulator precision;
- modular, trapping, saturating, or unbounded intermediate behavior;
- rounding mode and rounding points;
- signed zero and subnormal handling;
- infinity and NaN outcome relation, including payload rules;
- whether reassociation or approximation is allowed;
- exact error metric and threshold when approximation is allowed.

The first release SHALL include modular integer GEMM, checked integer GEMM with a typed overflow result, strict round-at-each-declared-operation floating GEMM, and exact-dyadic accumulation followed by one declared output rounding. Approximate GEMM is outside the first release problem rather than silently accepted through a tolerance.

The mode equations are fixed. Modular integer mode converts operands to the declared accumulator width and performs every multiply, add, alpha scale, beta scale, and output conversion modulo that width. Checked integer mode performs those operations over mathematical integers and returns `checked-overflow` with C unchanged if any declared accumulator or output range is exceeded. Strict floating mode visits `k` in ascending order, rounds each product to the accumulator format, rounds every accumulator addition, separately rounds `alpha·sum` and `beta·C`, then rounds their addition to C; it uses the exact pinned IEEE/Wasm special-value relation. Exact-dyadic mode decodes finite values to signed dyadics, forms the complete mathematical `alpha·Σ(A·B)+beta·C`, and rounds once to C; nonfinite inputs follow its separately enumerated NaN/infinity table. Binary16 and bfloat16 use the same equations through proved software codecs. All released deterministic modes canonicalize NaN output to the quiet canonical payload specified in `FloatBits.lean`; no unobserved payload freedom remains.

The compatibility relation is the following closed table. A row not listed is
incompatible and classifies as `invalid`; there is no implicit mixed-kind or
mixed-signedness conversion.

| Mode | A, B, and C stored kinds | Required accumulator kind |
|---|---|---|
| modular | the same one of `i8`, `u8`, `i16`, `u16`, `i32`, `u32` | `u32` |
| modular | the same one of `i64`, `u64` | `u64` |
| checked | the same one of `i8`, `i16`, `i32`, `i64` | `i64` |
| checked | the same one of `u8`, `u16`, `u32`, `u64` | `u64` |
| strictFloat | the same one of `binary16`, `bfloat16` | `binary32` or `binary64` |
| strictFloat | `binary32` | `binary32` or `binary64` |
| strictFloat | `binary64` | `binary64` |
| exactDyadicRoundOnce | the same one of `binary16`, `bfloat16`, `binary32`, `binary64` | `exactDyadic` |

For modular mode, signed source bits are sign-extended and unsigned source
bits are zero-extended to the accumulator width, then interpreted in
`ZMod (2^w)`; output is the low C-width bit pattern. For checked mode, signed
and unsigned values embed into mathematical integers, every product, running
sum, alpha scale, beta scale, final addition, and output conversion is checked
against the declared accumulator or C interval, and the first overflow in the
fixed row-major `(batch,row,column,k)` order produces status 4 with C
unchanged. No saturating operation is released.

All floating modes use round-to-nearest, ties-to-even, gradual underflow, and
preserve the IEEE sign rules for nonzero results. Canonical quiet NaN bit
patterns are `0x7e00` for binary16, `0x7fc0` for bfloat16, `0x7fc00000` for
binary32, and `0x7ff8000000000000` for binary64. Strict mode uses the
declared accumulator format at every rounding point and rounds once more on C
conversion. Exact-dyadic mode has no finite intermediate rounding; an exact
zero is encoded as `+0`, except a negative exact result that underflows to zero
is `-0`. The closed special-value table is: any quiet NaN operand yields the
canonical quiet NaN; `0 × infinity` yields that NaN; nonzero finite times
infinity yields the xor-signed infinity; adding opposite infinities yields the
canonical NaN; adding equal-signed infinities or an infinity and a finite
value yields that infinity. Exact-dyadic reduction first applies this product
table, yields canonical NaN if both infinity signs occur among the scaled
terms, yields the sole infinity sign if exactly one occurs, and otherwise
rounds the finite exact dyadic. Alpha and beta participate in the same table.
Every codec, interval, bit extension, rounding function, special-value case,
and compatibility row is a first-order field of the canonical
`Gemm.ProblemBody`; no theorem may supply a different table through a typeclass
or callback.

A signaling NaN in `alpha`, `beta`, A, B, or the initial C is the sole
released trigger for status 5. Detection precedes arithmetic and scans in that
listed carrier order, with each matrix in canonical `(batch,row,column)`
order. C remains unchanged. Quiet NaNs and invalid IEEE operations such as
`0 × infinity` produce the canonical quiet NaN under the table above and do
not produce status 5.

Status 5 is a normal returned GEMM result in the released ABI. It is not a
Core exception. Any uncaught WebAssembly exception terminal is rejected by
`Reference.Accepts`, regardless of its payload.

The release resource constants are Lean naturals: raw invocation extent at
most `2^32 - 1` bytes, ordinary memory at most 65,536 pages, tables at most
`2^32 - 1` elements, live Core-GC and exception heap at most `2^32 - 1`
canonical abstract bytes, peak total live value slots (operands, frame locals,
labels, handlers, and exception payloads) at most `2^64 - 1`, and at most
`2^320` costed reduction steps per raw invocation. The baseline proof SHALL
derive this step bound from the four `u64` loop extents and fixed per-iteration
code and SHALL show that every byte-level well-formed descriptor whose A, B,
and C regions fit wasm32 memory completes below it without scratch. Therefore
the resource predicate cannot be used to reject a memory-fitting GEMM merely
because it is large.

An exact-dyadic, round-once GEMM and a WebAssembly round-each-operation GEMM are different problem identities. A proof for one SHALL not be reused for the other without a proved bridge.

### 8.3 Raw classifier and ABI

The raw-input carrier is fixed before classification and cannot be narrowed by
the problem implementation:

```lean
structure Gemm.RawInvocationBody (P : Wasm.Profile) where
  ptr : UInt32
  len : UInt32
  bytes : ByteArray

structure Gemm.RawInvocation (P : Wasm.Profile) where
  body : Gemm.RawInvocationBody P
  lawful : Gemm.RawInvocationLawful P body

def Gemm.Problem.RawInvocation
    {P : Wasm.Profile} (problem : Gemm.Problem P) :=
  Gemm.RawInvocation P

theorem Gemm.RawInvocationLawful.toWasmInvocationLawful
    {P : Wasm.Profile} {body : Gemm.RawInvocationBody P}
    (h : Gemm.RawInvocationLawful P body) :
    Wasm.InvocationLawful P
      { ptr := body.ptr, len := body.len, bytes := body.bytes } := h

def Gemm.toWasmInvocation
    {P : Wasm.Profile} (problem : Gemm.Problem P)
    (raw : problem.RawInvocation) : Wasm.Invocation P :=
  Wasm.Invocation.gemmRaw
    { ptr := raw.body.ptr, len := raw.body.len, bytes := raw.body.bytes }
    raw.lawful.toWasmInvocationLawful
```

The harness places exactly `raw.body.bytes` at `raw.body.ptr`, passes
`(ptr,len)` to `gemm`, and does not synthesize any other descriptor data. All
byte strings of every representable length participate, including malformed
headers. `RawInvocationBody` is proof-free in its canonical encoding.
`RawInvocationLawful` is exactly `bytes.size = len.toNat ∧ ptr.toNat +
len.toNat ≤ 2^P.addressBits ∧ pagesFor (ptr.toNat + len.toNat) ≤ P.maxPages`;
its proof is not part of the identity preimage.

Required exact-domain theorems are:

```lean
theorem raw_invocation_roundtrip (raw : Gemm.RawInvocation P) :
  Gemm.decodeRawInvocation P (Gemm.encodeRawInvocation raw) = .ok raw

theorem raw_invocation_surjective
    (ptr len : UInt32) (bytes : ByteArray)
    (hlen : bytes.size = len.toNat)
    (hrange : ptr.toNat + len.toNat ≤ 2^P.addressBits)
    (hpages : pagesFor (ptr.toNat + len.toNat) ≤ P.maxPages) :
  ∃ raw : Gemm.RawInvocation P,
    raw.body.ptr = ptr ∧ raw.body.len = len ∧ raw.body.bytes = bytes

theorem problem_raw_invocation_definitional
    (problem : Gemm.Problem P) :
  problem.RawInvocation = Gemm.RawInvocation P
```

`Gemm/Classify.lean` SHALL implement a total classifier:

```lean
inductive Classification
  | valid (invocation : ValidInvocation)
  | invalid (report : InvalidInputReport)
  | unsupported (report : UnsupportedReport)
  | resourceExhausted (report : ResourceReport)
```

`Gemm/ABI.lean` SHALL fix exact byte offsets, integer endianness, descriptor encoding, matrix regions, output region, and status encoding. The classifier SHALL reject overlapping or out-of-bounds regions unless the exact alias contract permits them.

The release ABI header is exactly 256 bytes at `ptr`, all multibyte fields are little-endian, and all offsets are relative to `ptr`:

| Bytes | Field |
|---:|---|
| `0..3` | ASCII magic `WGNG` |
| `4..5` | ABI version `1` as `u16` |
| `6..7` | header size `256` as `u16` |
| `8,9,10,11` | A, B, C, and accumulator kind tags |
| `12` | arithmetic-mode tag |
| `13` | transpose bits: bit 0 A, bit 1 B; other bits zero |
| `14` | aliasing tag |
| `15` | zero |
| `16..23,24..31,32..39,40..47` | `m`, `n`, `k`, batch as `u64` |
| `48..87` | A view: offset, byte length, row stride, column stride, batch stride; five 64-bit fields, strides signed |
| `88..127` | B view in the same form |
| `128..167` | C view in the same form |
| `168..183` | alpha bits, value-width prefix followed by zero padding |
| `184..199` | beta bits, value-width prefix followed by zero padding |
| `200..207,208..215` | scratch offset and byte length |
| `216..223,224..231` | status-detail offset and byte length |
| `232..255` | zero |

Kind tags in order are `i8=0`, `u8=1`, `i16=2`, `u16=3`, `i32=4`, `u32=5`, `i64=6`, `u64=7`, `binary16=8`, `bfloat16=9`, `binary32=10`, `binary64=11`, and `exactDyadic=12` where permitted only as an accumulator. Arithmetic tags are `modular=0`, `checked=1`, `strictFloat=2`, and `exactDyadicRoundOnce=3`. Aliasing tags are `disjoint=0`, `cMayAliasAAfterRead=1`, and `cMayAliasBAfterRead=2`.

A and B are read-only and may overlap each other. `disjoint` requires C to be disjoint from both. `cMayAliasAAfterRead` permits overlap only with A and defines the result against an immutable logical snapshot of A taken before the first C write; `cMayAliasBAfterRead` is symmetric. A candidate need not physically copy when its schedule proves the same observation. No tag permits C to overlap both A and B.

Each view offset and byte length is `u64`; each row, column, and batch stride is
the two's-complement `i64` shown in the header. Address arithmetic is performed
over mathematical integers before conversion to wasm32. For a nontransposed A,
logical `(b,i,t)` addresses
`ptr + offset + b·batchStride + i·rowStride + t·columnStride`; transposed A
swaps `i` and `t`. B uses `(b,t,j)` and swaps `t` and `j` when transposed. C
uses `(b,i,j)`. Every addressed element byte must lie both in the view's
`[offset, offset + byteLength)` interval and in memory, without mathematical
or machine wraparound. Negative strides are valid under that test. Zero
strides and repeated addresses are valid for read-only A and B, including
broadcasts. Distinct logical C elements must have disjoint element-byte
ranges; zero C stride is valid only along an extent at most one. Empty logical
tensors perform no element address and still require the declared interval
endpoints to be representable. These rules, plus the alias tags below, are the
complete layout predicate.

`len` is the complete invocation extent. Every view, scratch range, and
status-detail range must lie inside it. Function results are `0=success`,
`1=invalid`, `2=unsupported`, `3=resource-exhausted`, `4=checked-overflow`,
and `5=arithmetic-exception`; no other return is sanctioned.

The header, scratch, and status-detail ranges are pairwise disjoint and each is
disjoint from A, B, and C. A and B may overlap only as stated above. C may
overlap exactly one of A or B only under its matching alias tag; otherwise it
is disjoint. Any other overlap is `invalid` at the alias-validation precedence
step. Scratch is candidate-private workspace: candidate-call writes may target
it, its final bytes are masked out of `SemanticObservation`, and it is freshly
initialized from the raw bytes for every invocation. C and status-detail are
fully observable. All nonscratch bytes outside the permitted C/status ranges
must equal their `gemmEntryObservableStore` values. Thus success may change C,
status, and scratch. A descriptor-level `invalid`, `unsupported`, or
`resource-exhausted` result may write only a separately validated status
range; if no trustworthy status range exists it writes nothing. Scratch is
not writable until the complete descriptor, scratch range, and disjointness
checks have succeeded. A runtime checked-overflow or arithmetic-exception may
write validated status and scratch but must leave C equal to its entry value.
Neither case can use workspace as a hidden cross-invocation channel.
`SanctionedWriteRegions raw terminal` is therefore the closed phase-dependent
function `∅`, `status`, `C ∪ status ∪ scratch`, or `status ∪ scratch` for those
four cases respectively. The reference equality separately masks scratch
only after complete descriptor validation.

The status-detail range SHALL be exactly 32 bytes for a valid descriptor. Its little-endian record is: status `u32` at `0..3`, field code `u32` at `4..7`, offending byte offset or logical index `u64` at `8..15`, required quantity `u64` at `16..23`, and available quantity `u64` at `24..31`. Field codes are `0=none`, `1=header`, `2=version`, `3=kind`, `4=arithmetic-mode`, `5=dimension`, `6=view`, `7=alias`, `8=resource`, `9=overflow`, and `10=arithmetic`. Success writes an all-zero record. If the detail range itself is invalid, the function returns the status code and performs no write.

`Gemm.StatusDetailOf` is a total canonical function, not a report callback.
Within the first failing precedence class it chooses the lowest header byte
offset, then the lexicographically least logical index. Its remaining fields
are fixed as follows: truncation uses `(required minimum end offset, len)`;
magic/version/header-size/reserved-bit failures use `(expected literal,
observed literal)`; kind and mode failures use `(allowed-tag bitset, observed
tag)`; incompatible tuples use `(compatible-row bitset, packed observed
kind/mode bytes)`; a dimension or view failure uses `(first required exclusive
byte endpoint, declared available endpoint)`, each saturated at `u64::MAX`;
alias failure uses `(0, overlapping byte count)`; resource failure uses
`(requested quantity, profile limit)`; checked overflow uses `(minimum signed
or unsigned bits required, accumulator/output bits available)`; signaling-NaN
arithmetic exception uses `(quiet-bit mask, offending raw bits)`. A logical
index too large for `u64` is saturated at `u64::MAX`. The exact expected
literals, bitsets, packing order, saturation, and scan order are finite tables
inside `Release.gemmProblemBody`. Status 5 occurs exactly for the signaling-NaN
case specified above. Therefore every classified raw input determines one
record byte sequence; two different record encoders cannot satisfy the same
release problem identity.

The finite encodings in that formula are numeric constants. The allowed-kind
bitset is `0x0fff` for A, B, or C and `0x1fff` for the accumulator; the
allowed-mode bitset is `0x0f`. Compatibility rows are numbered `0` through
`7` in the table's displayed order. The compatible-row bitset is the sum of
`2^r` for rows whose mode and stored-kind premise accepts the observed A/B/C
triple while ignoring only the accumulator field. The packed observed tuple
is `aTag | (bTag << 8) | (cTag << 16) | (accTag << 24) | (modeTag << 32)`.
All shifts are on `u64`; no host endianness or enumeration order participates.

Classification precedence is total and fixed: malformed magic/version/header size/reserved bits or truncated bytes produce `invalid`; otherwise unknown or profile-disabled kind/mode tags produce `unsupported`; otherwise incompatible tags, arithmetic overflow in descriptor calculations, illegal views, or alias violations produce `invalid`; otherwise failure of the fixed resource predicate produces `resource-exhausted`; all remaining descriptors are `valid`. Runtime checked overflow and arithmetic exception use statuses 4 and 5. The first matching class determines both return and status-detail record.

Every byte-level well-formed descriptor whose views fit memory and whose tag combination appears above SHALL classify as valid unless it violates the explicitly enumerated alias or resource predicate. Required anti-vacuity theorems SHALL provide a nonzero `1×1×1` witness for every mandatory scalar-kind × compatible arithmetic-mode × transpose × layout-class combination, and SHALL prove that the full C range and status are observable.

Tiling is an internal GNAF/Wasm schedule transformation, not a release ABI view. A future externally tiled layout requires a new versioned header with explicit tile dimensions and address map.

`Gemm/Problem.lean` SHALL construct one first-order literal
`Release.gemmProblemBody` containing exactly the feature, scalar,
compatibility, arithmetic, ABI, layout, alias, observation, status-precedence,
resource, and workload-repetition values in this section. The release value is
not implementation-selected:

```lean
def Release.gemmProblemBody :
    Gemm.ProblemBody Release.wasmProfile.body :=
  Gemm.canonicalWGNGv1ProblemBody
    Release.wasmProfile.body (workloadRepetitions := 1)

def Release.gemmProblem : Gemm.Problem Release.wasmProfile :=
  Gemm.Problem.checked Release.wasmProfile Release.gemmProblemBody

theorem Release.gemmProblem_body :
  Release.gemmProblem.body = Release.gemmProblemBody
```

`canonicalWGNGv1ProblemBody` is a literal record constructor defined in
`Gemm/ProblemBody.lean`; every projection reduces by `rfl` to the numeric ABI,
kind/mode table, arithmetic relation, status formula, resource value, and
observation rule printed above. It takes no callback, typeclass-selected
arithmetic, or artifact-derived argument.

The conformance registry SHALL identity-check the canonical encoding of this
body and each finite table projection. Replacing any row, rounding rule,
status, stride rule, resource constant, or workload multiplicity changes the
problem identity and cannot satisfy the recorded release proposition.

### 8.4 Reference relation

```lean
def Gemm.Reference.Accepts
    (problem : Gemm.Problem P)
    (raw : problem.RawInvocation)
    (observation : Wasm.ExecutionObservation) : Prop
```

`Accepts` is the conjunction of
`ReferenceSemanticAccepts problem raw (Gemm.semanticFor problem raw observation)` and
`Wasm.CandidateCallMemoryWritesWithin observation
(problem.SanctionedWriteRegions raw
  (Gemm.semanticFor problem raw observation))`. It does not prescribe a particular
internal trace; any trace satisfying the phase and write-safety invariant may
realize the same semantic observation. For valid input it SHALL express
`C ← alpha · op(A) · op(B) + beta · C` under the descriptor's exact
arithmetic, the exact returned status, the complete final C byte region,
exact status-detail bytes, and the absence of every forbidden effect. For
invalid input it SHALL express the exact typed status and permitted memory
effects. Trapped and uncaught-exception observations are rejected by every
released problem case. It SHALL be total over raw invocations.

Required proofs:

```lean
theorem classify_total (raw : problem.RawInvocation) :
  ∃ classification, Gemm.classify problem raw = classification

theorem valid_reference_nonempty
    (h : Gemm.classify problem raw = .valid invocation) :
  ∃ observation, Gemm.Reference.Accepts problem raw observation

theorem deterministic_mode_unique
    (hmode : problem.ModeDeterministic raw)
    (ha : Gemm.Reference.Accepts problem raw a)
    (hb : Gemm.Reference.Accepts problem raw b) :
  Gemm.semanticFor problem raw a = Gemm.semanticFor problem raw b

theorem reference_memory_safe
    (h : Gemm.Reference.Accepts problem raw observation) :
  Wasm.CandidateCallMemoryWritesWithin
    observation
      (problem.SanctionedWriteRegions raw
        (Gemm.semanticFor problem raw observation))

instance problem_input_fintype : Fintype problem.RawInvocation
```

`Wasm.CandidateCallMemoryWritesWithin observation regions` is defined from the
post-`gemm-entry` candidate write events and the bytewise difference between
`observation.gemmEntryObservableStore` and the final observable store. It
requires every candidate-call write event and every changed ABI-visible byte
to lie in `regions`; it is false for `trappedBeforeEntry`. Pre-entry
start/harness events are still costed and
semantically checked against the exact entry snapshot, but cannot be confused
with writes performed by the GEMM call. The predicate does not project memory
writes from `ObservableEffects`, whose domain is deliberately non-memory
effects.

## 9. Cost and objective

Core WebAssembly has no execution-time metric. This repository SHALL define and name an abstract UOR-Wasm cost model.

### 9.1 Complete vector

```lean
structure Cost.StaticVector where
  moduleBytes        : Nat
  decodeSteps        : Nat
  validationSteps    : Nat
  staticDataBytes    : Nat

structure Cost.DynamicVector where
  instantiationSteps : Nat
  dispatchSteps      : Nat
  preparationSteps   : Nat
  wasmRuleSteps      : Nat
  scalarOps          : Nat
  vectorLaneOps      : Nat
  bytesRead          : Nat
  bytesWritten       : Nat
  memoryGrowPages    : Nat
  tableElementsAllocated : Nat
  gcObjectsAllocated : Nat
  gcBytesInitialized : Nat
  peakStackValues    : Nat
  peakPages          : Nat
  peakGcLiveBytes    : Nat
  outputBytes        : Nat

def Cost.sequentialCompose
    (first second : Cost.DynamicVector) : Cost.DynamicVector :=
  { instantiationSteps := first.instantiationSteps + second.instantiationSteps
    dispatchSteps := first.dispatchSteps + second.dispatchSteps
    preparationSteps := first.preparationSteps + second.preparationSteps
    wasmRuleSteps := first.wasmRuleSteps + second.wasmRuleSteps
    scalarOps := first.scalarOps + second.scalarOps
    vectorLaneOps := first.vectorLaneOps + second.vectorLaneOps
    bytesRead := first.bytesRead + second.bytesRead
    bytesWritten := first.bytesWritten + second.bytesWritten
    memoryGrowPages := first.memoryGrowPages + second.memoryGrowPages
    tableElementsAllocated :=
      first.tableElementsAllocated + second.tableElementsAllocated
    gcObjectsAllocated := first.gcObjectsAllocated + second.gcObjectsAllocated
    gcBytesInitialized := first.gcBytesInitialized + second.gcBytesInitialized
    peakStackValues := max first.peakStackValues second.peakStackValues
    peakPages := max first.peakPages second.peakPages
    peakGcLiveBytes := max first.peakGcLiveBytes second.peakGcLiveBytes
    outputBytes := first.outputBytes + second.outputBytes }

structure Cost.ArtifactVector where
  static : Cost.StaticVector
  dynamicSum : Cost.DynamicVector
  dynamicMax : Cost.DynamicVector

inductive Cost.ArtifactCoordinate
  | staticModuleBytes | staticDecodeSteps | staticValidationSteps
  | staticDataBytes
  | sumInstantiationSteps | sumDispatchSteps | sumPreparationSteps
  | sumWasmRuleSteps | sumScalarOps | sumVectorLaneOps
  | sumBytesRead | sumBytesWritten | sumMemoryGrowPages
  | sumTableElementsAllocated | sumGcObjectsAllocated | sumGcBytesInitialized
  | sumPeakStackValues | sumPeakPages | sumPeakGcLiveBytes
  | sumOutputBytes
  | maxInstantiationSteps | maxDispatchSteps | maxPreparationSteps
  | maxWasmRuleSteps | maxScalarOps | maxVectorLaneOps
  | maxBytesRead | maxBytesWritten | maxMemoryGrowPages
  | maxTableElementsAllocated | maxGcObjectsAllocated | maxGcBytesInitialized
  | maxPeakStackValues | maxPeakPages | maxPeakGcLiveBytes
  | maxOutputBytes

def Cost.ArtifactCoordinate.value :
    Cost.ArtifactCoordinate → Cost.ArtifactVector → Nat
  | .staticModuleBytes, c => c.static.moduleBytes
  | .staticDecodeSteps, c => c.static.decodeSteps
  | .staticValidationSteps, c => c.static.validationSteps
  | .staticDataBytes, c => c.static.staticDataBytes
  | .sumInstantiationSteps, c => c.dynamicSum.instantiationSteps
  | .sumDispatchSteps, c => c.dynamicSum.dispatchSteps
  | .sumPreparationSteps, c => c.dynamicSum.preparationSteps
  | .sumWasmRuleSteps, c => c.dynamicSum.wasmRuleSteps
  | .sumScalarOps, c => c.dynamicSum.scalarOps
  | .sumVectorLaneOps, c => c.dynamicSum.vectorLaneOps
  | .sumBytesRead, c => c.dynamicSum.bytesRead
  | .sumBytesWritten, c => c.dynamicSum.bytesWritten
  | .sumMemoryGrowPages, c => c.dynamicSum.memoryGrowPages
  | .sumTableElementsAllocated, c => c.dynamicSum.tableElementsAllocated
  | .sumGcObjectsAllocated, c => c.dynamicSum.gcObjectsAllocated
  | .sumGcBytesInitialized, c => c.dynamicSum.gcBytesInitialized
  | .sumPeakStackValues, c => c.dynamicSum.peakStackValues
  | .sumPeakPages, c => c.dynamicSum.peakPages
  | .sumPeakGcLiveBytes, c => c.dynamicSum.peakGcLiveBytes
  | .sumOutputBytes, c => c.dynamicSum.outputBytes
  | .maxInstantiationSteps, c => c.dynamicMax.instantiationSteps
  | .maxDispatchSteps, c => c.dynamicMax.dispatchSteps
  | .maxPreparationSteps, c => c.dynamicMax.preparationSteps
  | .maxWasmRuleSteps, c => c.dynamicMax.wasmRuleSteps
  | .maxScalarOps, c => c.dynamicMax.scalarOps
  | .maxVectorLaneOps, c => c.dynamicMax.vectorLaneOps
  | .maxBytesRead, c => c.dynamicMax.bytesRead
  | .maxBytesWritten, c => c.dynamicMax.bytesWritten
  | .maxMemoryGrowPages, c => c.dynamicMax.memoryGrowPages
  | .maxTableElementsAllocated, c => c.dynamicMax.tableElementsAllocated
  | .maxGcObjectsAllocated, c => c.dynamicMax.gcObjectsAllocated
  | .maxGcBytesInitialized, c => c.dynamicMax.gcBytesInitialized
  | .maxPeakStackValues, c => c.dynamicMax.peakStackValues
  | .maxPeakPages, c => c.dynamicMax.peakPages
  | .maxPeakGcLiveBytes, c => c.dynamicMax.peakGcLiveBytes
  | .maxOutputBytes, c => c.dynamicMax.outputBytes

inductive Cost.TieOrderTag
  | unsignedByteLexicographic

def UnsignedLexicographicLE : List UInt8 → List UInt8 → Prop
  | [], _ => True
  | _ :: _, [] => False
  | a :: as, b :: bs =>
      a.toNat < b.toNat ∨
      (a.toNat = b.toNat ∧ UnsignedLexicographicLE as bs)

def CanonicalBytesLE (left right : ByteArray) : Prop :=
  UnsignedLexicographicLE left.toList right.toList

structure Cost.LifecycleVector where
  authorityCheckSteps : Nat
  canonicalizationSteps : Nat
  canonicalNovelObjects : Nat
  canonicalNovelEdges  : Nat
  closureSteps        : Nat
  indexSteps          : Nat
  attentionBucketsTouched : Nat
  dependencyObjectsVisited : Nat
  partitionSteps      : Nat
  partitionCellsChanged : Nat
  verifierSteps       : Nat
  sealSteps           : Nat
  querySelectionSteps : Nat
  migrationSteps      : Nat
  retainedStateBytes  : Nat
  peakWorkingBytes    : Nat
  artifact             : Cost.ArtifactVector

abbrev Cost.CompleteSystemCost := Cost.ArtifactVector

def Cost.ExactAggregateCost
    {Raw : Type} [Fintype Raw]
    (P : Wasm.Profile)
    (bytes : ByteArray)
    (module : Wasm.Module)
    (repetitions : Nat)
    (dynamicFor : Raw → Cost.DynamicVector)
    (cost : Cost.CompleteSystemCost) : Prop :=
  1 ≤ repetitions ∧
  Wasm.decode bytes = .ok module ∧
  cost.static.moduleBytes = bytes.size ∧
  cost.static.decodeSteps = Wasm.decodeCost P.costTableBody bytes ∧
  cost.static.validationSteps =
    Wasm.validationCost P.costTableBody module ∧
  cost.static.staticDataBytes = Wasm.instantiatedStaticBytes P module ∧
  cost.dynamicSum =
    Cost.scale repetitions
      (Finite.fold ComponentwiseAdd Cost.DynamicVector.zero dynamicFor) ∧
  cost.dynamicMax =
    Finite.fold ComponentwiseMax Cost.DynamicVector.zero
      dynamicFor
```

Every artifact event is derived from a formal WebAssembly trace or canonical module byte object. Packing, layout conversion, dispatch, table lookup, any verification performed by the module, and output writes are charged. The implementation SHALL not compare its complete cost with a competitor's leaf multiply loop. Atlas proof construction and retained build state use `LifecycleVector`; they are not fields silently attached to one byte-only competitor.

### 9.2 Full-domain aggregation

The release workload is the canonical finite ordering of every raw invocation under the released profile. `Gemm.ProblemBody.workloadRepetitions` is a positive natural and equals one for the concrete release. The protocol decodes and validates the module once; in each repetition, for each raw value it creates a fresh pre-start instance, explores every permitted start-function execution, installs that raw value after each normally returned start, invokes `gemm`, records every maximal observation, and discards the instance. A trapped or diverging start is a trapped or diverging maximal invocation and cannot be omitted. Thus no invocation order can provide hidden persistent state. `Aggregate.lean` SHALL define worst-case and exact full-domain total costs over this protocol.

The canonical release score SHALL be a positive natural-weight sum of every unbounded charged coordinate. Every weight SHALL be at least one. Therefore:

```lean
theorem coordinate_le_score (c : Cost.ArtifactVector) :
  ∀ coordinate : Cost.ArtifactCoordinate,
    coordinate.value c ≤ CanonicalObjective.score c
```

This property is not cosmetic. It makes every score sublevel finite and supplies the all-Wasm coverage construction.

For the first release, every coordinate weight is exactly one. For each raw invocation, nondeterministic execution cost is the componentwise maximum over all permitted terminating traces. `dynamicSum` is `workloadRepetitions` times the componentwise full-domain sum; `dynamicMax` is the componentwise maximum over one raw invocation and is unchanged by repeating the same fresh-instance domain. The complete artifact vector adds module bytes and the one decode/validation pass, then every charged repetition of fresh instantiation, static-data initialization, dispatch, preparation, execution, and output. Any per-instance state must be encoded in the module/store and is thereby charged. The score is the natural-number sum of that complete vector. Canonical byte order breaks score ties but does not change the optimum value.

Atlas search, theorem construction, and release certification costs SHALL be measured and published in the lifecycle ledger. They are not runtime actions of the committed module and therefore are not included in `ArtifactGlobal`. If any generated table, proof checker, certificate, or Atlas state is loaded or consulted by the module, its bytes and accesses move into the artifact vector. Claims about build-plus-runtime or continual optimization SHALL use `LifecycleGlobalOptimal`, not `ArtifactGlobal`.

### 9.3 Proper objective

```lean
structure Cost.ObjectiveBody where
  version : Nat
  staticWeights : Cost.StaticCoordinateVector Nat
  dynamicSumWeights : Cost.DynamicCoordinateVector Nat
  dynamicMaxWeights : Cost.DynamicCoordinateVector Nat
  tieOrder : Cost.TieOrderTag

structure Cost.ProperObjective
    (P : Wasm.Profile) (G : Gemm.Problem P) where
  body : Cost.ObjectiveBody
  bodyValid : EveryCoordinateWeightPositive body
  boundOfScore : Nat → ResourceBounds
  sublevelBound : ∀ {c : CompleteSystemCost} {u : Nat},
    Cost.evaluate body c ≤ u → Within (boundOfScore u) c

def Cost.ProperObjective.score
    (objective : Cost.ProperObjective P G) (cost : CompleteSystemCost) : Nat :=
  Cost.evaluate objective.body cost

theorem Cost.ProperObjective.monotone
    (objective : Cost.ProperObjective P G)
    {a b : CompleteSystemCost} (h : ComponentwiseLE a b) :
  objective.score a ≤ objective.score b
```

An objective that leaves code size, execution, memory, advice, or another unbounded resource free SHALL not instantiate `ProperObjective` and SHALL not feed the global theorem.

`Release.costObjective.body` SHALL assign weight one to every constructor of
`Cost.ArtifactCoordinate` and set `tieOrder` to
`.unsignedByteLexicographic`. `UnsignedLexicographicLE` treats a proper prefix
as smaller and otherwise compares the first differing bytes as unsigned
naturals. These values are part of the canonical objective body; no locale,
hash order, module AST order, or implementation iteration order may break a
tie.

The implementation SHALL prove `evaluation.cost.static.moduleBytes = bytes.size`, positive accounting for every semantic transition and module/store byte, finiteness of every coordinate sublevel, and completeness of the byte/input/execution enumerators induced by `boundOfScore`. Positive weights without these bridge theorems do not establish finite coverage.

## 10. The universal competitor universe

### 10.1 Extensional definition

```lean
structure Universal.InputEvaluation
    (P : Wasm.Profile) (G : Gemm.Problem P)
    (module : Wasm.Module) (raw : G.RawInvocation) where
  initial : Wasm.Config
  initialization : Wasm.InitializationObservation P
  initialConfigEq : initialization.initial = initial
  initialEq : Wasm.initialGemmInvocationCosted P module
    (Gemm.toWasmInvocation G raw) =
    .ok initialization
  observations :
    NonemptyCanonicalList (Wasm.CostedExecutionObservation P initial)
  treeComplete : ∃ coverage,
    Wasm.exploreAllCosted P G.resources.maxSteps
      initial =
      .complete observations coverage
  resourceVector : Cost.DynamicVector
  resourceExact : resourceVector =
    Cost.sequentialCompose
      initialization.cost (Cost.maxOverCosts (observations.map (·.cost)))

structure Universal.SystemEvaluation
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray) where
  module : Wasm.Module
  decodeEq : Wasm.decode bytes = .ok module
  perInput : ∀ raw : G.RawInvocation, InputEvaluation P G module raw
  observationsComplete : ∀ raw, CoversEveryMaximalExecution (perInput raw)
  cost : Cost.CompleteSystemCost
  costExact : Cost.ExactAggregateCost P bytes module G.workloadRepetitions
    (fun raw => (perInput raw).resourceVector) cost

inductive Universal.EvaluationResult
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray)
  | complete (evaluation : SystemEvaluation P G bytes)
  | profileFailure (report : ProfileFailure)
  | initializationFailure (raw : G.RawInvocation) (report : FailureReport)
  | nonterminal (raw : G.RawInvocation) (prefix : NonterminalPrefix)
  | resourceExhausted (raw : G.RawInvocation) (report : ResourceReport)

def Universal.evaluate
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray) :
    EvaluationResult P G bytes :=
  -- the implemented finite decoder, validator, input enumerator, and all-branch explorer
  Universal.evaluateFinite P G bytes

def Universal.ProfileValid (P : Wasm.Profile) (bytes : ByteArray) : Prop :=
  ∃ module,
    Wasm.decode bytes = .ok module ∧
    Wasm.validateUnder P module = true ∧
    module.imports = [] ∧
    HasExactGemmExports P module

def Universal.StartsCostedInvocation
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray)
    (raw : G.RawInvocation)
    (initialization : Wasm.InitializationObservation P)
    (initial : Wasm.Config) : Prop :=
  ∃ module,
    Wasm.decode bytes = .ok module ∧
    Wasm.validateUnder P module = true ∧
    Wasm.initialGemmInvocationCosted P module
      (Gemm.toWasmInvocation G raw) =
      .ok initialization ∧
    initialization.initial = initial

def Universal.IsMaximalExecution
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray)
    (raw : G.RawInvocation)
    (initialization : Wasm.InitializationObservation P)
    {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial) : Prop :=
  StartsCostedInvocation P G bytes raw initialization initial ∧
  Wasm.IsMaximalExecutionFrom initial execution

def Universal.SemanticCorrectAt
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray)
    (raw : G.RawInvocation) : Prop :=
  (∃ initialization : Wasm.InitializationObservation P,
    ∃ initial : Wasm.Config,
    ∃ execution : Wasm.MaximalExecution initial,
      IsMaximalExecution P G bytes raw initialization execution) ∧
  ∀ (initialization : Wasm.InitializationObservation P)
    (initial : Wasm.Config) (execution : Wasm.MaximalExecution initial),
    IsMaximalExecution P G bytes raw initialization execution →
    ∃ observation,
      execution.HasFiniteObservation observation ∧
      Gemm.Reference.Accepts G raw observation

def Universal.SemanticCorrect
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray) : Prop :=
  ∀ raw, SemanticCorrectAt P G bytes raw

def Universal.SemanticWithinResourcesAt
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray)
    (raw : G.RawInvocation) : Prop :=
  (¬ ∃ initialization : Wasm.InitializationObservation P,
    ∃ initial : Wasm.Config,
    ∃ prefix : Wasm.RelationalPrefix initial (G.resources.maxSteps + 1),
      StartsCostedInvocation P G bytes raw initialization initial ∧ prefix.Valid) ∧
  (∀ (initialization : Wasm.InitializationObservation P)
    (initial : Wasm.Config) (execution : Wasm.MaximalExecution initial),
    IsMaximalExecution P G bytes raw initialization execution →
    ∃ costed : Wasm.CostedExecutionObservation P initial,
      execution.CostedAs costed ∧
      Cost.sequentialCompose initialization.cost costed.cost ≤
        G.resources.limit)

def Universal.SemanticWithinResources
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray) : Prop :=
  ∀ raw, SemanticWithinResourcesAt P G bytes raw

def InvocationConforms
    (P : Wasm.Profile) (G : Gemm.Problem P)
    (bytes : ByteArray) (raw : G.RawInvocation) : Prop :=
  Universal.ProfileValid P bytes ∧
  Universal.SemanticCorrectAt P G bytes raw ∧
  Universal.SemanticWithinResourcesAt P G bytes raw

theorem invocation_conforms_iff_extensional
    (P : Wasm.Profile) (G : Gemm.Problem P)
    (bytes : ByteArray) (raw : G.RawInvocation) :
  InvocationConforms P G bytes raw ↔
    Universal.ProfileValid P bytes ∧
    Universal.SemanticCorrectAt P G bytes raw ∧
    Universal.SemanticWithinResourcesAt P G bytes raw

def Universal.SystemEvaluationRel
    (P : Wasm.Profile) (G : Gemm.Problem P) (bytes : ByteArray)
    (evaluation : SystemEvaluation P G bytes) : Prop :=
  Universal.evaluate P G bytes = .complete evaluation

def Universal.Correct
    (G : Gemm.Problem P)
    (evaluation : SystemEvaluation P G bytes) : Prop :=
  ∀ raw,
    (evaluation.perInput raw).observations.Nonempty ∧
    ∀ observation ∈ (evaluation.perInput raw).observations,
      Gemm.Reference.Accepts G raw observation.observation

def Universal.Feasible
    (G : Gemm.Problem P)
    (evaluation : SystemEvaluation P G bytes) : Prop :=
  ∀ raw,
    (evaluation.perInput raw).resourceVector ≤ G.resources.limit
```

`InputEvaluation` is the exact result of the bounded all-successor tree. A
trap, uncaught exception, stuck state, initialization failure, or nonterminal
prefix makes `Correct ∧ Feasible` false. `SystemEvaluationRel` existence and
uniqueness SHALL be proved for every profile-valid module whose tree completes
within the problem resource limit. `Correct` includes execution existence
through `observations.Nonempty`; it cannot be vacuously true for an empty run
relation.

`MaximalExecution.HasFiniteObservation observation` holds for a returned,
trapped, or uncaught-exception finite maximal execution and is false for
`diverges`. Resource
feasibility therefore measures every bounded terminal execution independently
of semantic correctness. `SemanticCorrect` is the separate predicate that
rejects traps and uncaught exceptions and requires an accepted returned
observation. This separation is mandatory for the two independent reflection
equivalences below.

The evaluator reflection boundary SHALL close these exact theorems:

```lean
theorem system_evaluation_rel_sound
    (hrel : SystemEvaluationRel P G bytes evaluation) :
  (Correct G evaluation ↔ SemanticCorrect P G bytes) ∧
  (Feasible G evaluation ↔ SemanticWithinResources P G bytes)

theorem system_evaluation_rel_complete
    (hprofile : ProfileValid P bytes)
    (hcorrect : SemanticCorrect P G bytes)
    (hresources : SemanticWithinResources P G bytes) :
  ∃ evaluation : SystemEvaluation P G bytes,
    SystemEvaluationRel P G bytes evaluation

theorem system_evaluation_rel_functional
    (ha : SystemEvaluationRel P G bytes a)
    (hb : SystemEvaluationRel P G bytes b) :
  a = b
```

Thus evaluator failure cannot remove a semantically correct competitor from `GlobalOptimal`.

Fresh module instantiation occurs independently for every raw invocation. Start-function work, raw-byte installation, GEMM invocation, and final observation are charged on every input. No mutable state carries from one raw invocation to the next in `ArtifactGlobal`. An ordered, stateful horizon belongs exclusively to `LifecycleGlobalOptimal`.

The import graph SHALL enforce a dependency firewall: `Foundation`, `Wasm`, `Gemm`, `Cost`, and the extensional definitions in `Universal/Competitor.lean`, `Correct.lean`, and `Feasible.lean` SHALL NOT import `GNAF`, `Atlas`, `Artifact`, `Universal/LowerBound`, `Universal/Argmin`, or `Theorems`. A source-and-environment gate SHALL reject an artifact-, selector-, or conclusion-dependent scope predicate.

The universe is not stored as a finite route list. It is the extensional set of every finite byte sequence satisfying these predicates.

All competitors receive the identical raw invocation, fresh initial instance, and profile. A competitor may contain Strassen-like algorithms, data-dependent branches, generated tables, packing, a dispatcher, or any other valid WebAssembly construction. Those are included automatically. A host import, preloaded external answer table, extra initial information, or uncharged external process is excluded only because the complete-system profile excludes or charges it symmetrically.

### 10.2 No undecidable oracle

The repository SHALL not implement a total decider for unrestricted program equivalence or termination. Instead, it SHALL prove that every possible competitor capable of beating the known baseline lies in a finite, exactly checkable cost sublevel.

### 10.3 Sublevel construction

Given a proved correct baseline evaluation `bEval`, let `u = O.score bEval.cost`. Properness yields bounds on module bytes, execution steps, memory, instantiated state, and every other charged coordinate for any evaluated competitor with `O.score cEval.cost ≤ u`.

`Universal/Sublevel.lean` SHALL prove:

```lean
theorem possible_winner_within_sublevel
    (hc : ProfileValid P competitorBytes)
    (cEval : SystemEvaluation P G competitorBytes)
    (heval : SystemEvaluationRel P G competitorBytes cEval)
    (hcorrect : Correct G cEval)
    (hfeasible : Feasible G cEval)
    (hbetter : O.score cEval.cost ≤ O.score bEval.cost) :
  WithinSublevel (O.boundOfScore (O.score bEval.cost)) competitorBytes cEval
```

Then:

1. Byte strings within the module-size bound are finite and exactly enumerable.
2. Raw invocations within the ABI/memory profile are finite and exactly enumerable or covered by proved symbolic partitions.
3. Runs within the step and state bounds are finite.
4. Decoder, validator, reference acceptance, resource use, and exact trace cost are decidable on that finite carrier.
5. Every sublevel module is checked, or a proof certificate covers its exact canonical partition.
6. Every module outside the sublevel is proved unable to beat the baseline.

No assumption equivalent to "the enumeration contains every winner" is permitted; that statement is the conclusion of `SublevelComplete.lean`.

### 10.4 Search partitions

For feasibility, the implementation MAY partition the finite space by canonical module-byte prefix, validation type, control-flow summary, semantic summary, or proved lower bound. A partition has a canonical body and exact denotation:

```lean
structure PartitionScopeBody where
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  sublevel : ResourceBounds
  baselineScore : Nat

structure PartitionScope where
  profile : Wasm.Profile
  problem : Gemm.Problem profile
  objective : Cost.ProperObjective profile problem
  body : PartitionScopeBody
  profileIdEq : body.profileId = Wasm.ProfileId profile
  problemIdEq : body.problemId = Gemm.ProblemId problem
  objectiveIdEq : body.objectiveId = Cost.ObjectiveId objective

def PartitionScope.sublevel (scope : PartitionScope) : ResourceBounds :=
  scope.body.sublevel

def PartitionScope.baselineScore (scope : PartitionScope) : Nat :=
  scope.body.baselineScore

structure PartitionBody (scope : PartitionScope) where
  prefix : ByteArray
  suffixLengthBound : Nat
  semanticConstraints : CanonicalConstraintSet
  rank : Nat

def PartitionBody.Denotes (p : PartitionBody scope) (bytes : ByteArray) : Prop :=
  ∃ suffix : ByteArray,
    bytes = p.prefix ++ suffix ∧
    suffix.size ≤ p.suffixLengthBound ∧
    ∀ constraint ∈ p.semanticConstraints, constraint.Holds bytes

inductive PartitionResult (scope : PartitionScope) (parent : PartitionBody scope)
  | exhausted
      (members : CanonicalList CheckedByteResult)
      (exact : ∀ bytes, parent.Denotes bytes ↔ ∃ member ∈ members, member.bytes = bytes)
  | dominated
      (lowerBound : Nat)
      (memberLowerBound : ∀ bytes,
        parent.Denotes bytes →
        ProfileValid scope.profile bytes →
        SemanticCorrect scope.profile scope.problem bytes →
        SemanticWithinResources scope.profile scope.problem bytes →
        ∀ evaluation : SystemEvaluation scope.profile scope.problem bytes,
          SystemEvaluationRel scope.profile scope.problem bytes evaluation ∧
          lowerBound ≤ scope.objective.score evaluation.cost)
      (strict : scope.baselineScore < lowerBound)
  | empty
      (proof : ∀ bytes, parent.Denotes bytes → ¬ ProfileValid scope.profile bytes)
  | split
      (children : NonemptyCanonicalList (PartitionBody scope))
      (cover : ∀ bytes, parent.Denotes bytes → ∃ child ∈ children, child.Denotes bytes)
      (disjoint : PairwiseDisjointDenotations children)
      (decreases : ∀ child ∈ children, child.rank < parent.rank)
  | incomplete (gap : CoverageGap)
```

Every result identity SHALL bind the proof-free `scope.body`, the complete
`parent` body, and all child/member preimages. The resolved `PartitionScope`
and its equality witnesses are verifier inputs, never identity preimages. Only
the first four constructors may occur in a sealed global certificate.
`incomplete` propagates to `SolveResult.incomplete` and blocks release. The
strict rank decrease makes recursive coverage well-founded.

### 10.5 Universal coverage theorem

```lean
theorem universal_sublevel_coverage :
  ∀ (competitorBytes : ByteArray),
    Universal.ProfileValid P competitorBytes →
    Universal.SemanticCorrect P G competitorBytes →
    Universal.SemanticWithinResources P G competitorBytes →
    ∀ evaluation : Universal.SystemEvaluation P G competitorBytes,
      Universal.SystemEvaluationRel P G competitorBytes evaluation →
      O.score evaluation.cost ≤ baselineScore →
      CoveredBySeal releaseSeal competitorBytes evaluation
```

This theorem SHALL be constructed from canonical encodings, properness, partition coverage, and checker soundness. It SHALL have no coverage hypothesis.

## 11. GNAF plan and compiler

The GNAF grammar is the constructive proposal language and the native representation of known semantics. It is not the definition of the universal competitor universe.

### 11.1 Plan syntax

`GNAF/Plan.lean` SHALL represent complete systems, including:

- raw-input classification;
- shape/layout dispatch;
- packing and unpacking;
- loop nests and index maps;
- blocking, tiling, and traversal order;
- reductions and arithmetic contracts;
- scratch allocation;
- scalar and standardized Wasm vector operations;
- code/data tables;
- status and output construction.

An embedded validated Wasm process may be represented as a canonical opaque process node only when its exact semantics, resource behavior, and cost certificate are retained. "Opaque" SHALL not mean "trusted."

### 11.2 Typing and semantics

Every plan SHALL carry exact input/output types and resource transitions. `GNAF/Semantics.lean` SHALL define behavior independently of compilation. `GNAF/Resource.lean` SHALL prove termination and bounds.

### 11.3 Normalization

Normalization SHALL be terminating, semantics preserving, cost nonincreasing for the bound objective, canonical, and idempotent:

```lean
theorem normalize_semantics (plan : GNAF.CheckedPlan P G) :
  GNAF.Eval (GNAF.normalize plan) = GNAF.Eval plan

theorem normalize_cost_le (plan : GNAF.CheckedPlan P G) :
  GNAF.certifiedCost (GNAF.normalize plan) ≤ GNAF.certifiedCost plan

theorem normalize_idempotent (plan : GNAF.CheckedPlan P G) :
  GNAF.normalize (GNAF.normalize plan) = GNAF.normalize plan

theorem normalize_canonical
    (ha : GNAF.SemanticallyEquivalent a b)
    (hca : GNAF.IsCanonical a) (hcb : GNAF.IsCanonical b) :
  a = b
```

### 11.4 Verified compiler and emitter

```lean
def GNAF.compile
    {P : Wasm.Profile} {G : Gemm.Problem P} :
    CheckedPlan P G → Wasm.Module
def Artifact.emit : Wasm.Module → ByteArray

theorem compile_refines
    (plan : GNAF.CheckedPlan P G) (raw : G.RawInvocation)
    (trace : List Wasm.Event)
    (observation : Wasm.ExecutionObservation)
    (hrun : Wasm.Run (GNAF.compile plan)
      (Gemm.toWasmInvocation G raw) trace observation) :
  GNAF.Accepts plan raw observation

theorem compile_resources
    (plan : GNAF.CheckedPlan P G) :
  Wasm.ModuleWithin P (GNAF.compile plan) plan.resourceBound

theorem compile_cost_exact
    (plan : GNAF.CheckedPlan P G) (raw : G.RawInvocation)
    (initialization : Wasm.InitializationObservation P)
    {initial : Wasm.Config}
    (costed : Wasm.CostedExecutionObservation P initial)
    (hrun : Wasm.CostedExecutionFor P
      (GNAF.compile plan) (Gemm.toWasmInvocation G raw)
        initialization costed) :
  Cost.sequentialCompose initialization.cost costed.cost =
    GNAF.certifiedCost plan raw
theorem decode_emit : Wasm.decode (Artifact.emit m) = .ok m
```

The emitter constructs byte candidates produced from GNAF plans. Universal selection, however, ranges over raw byte strings and SHALL retain the exact winning bytes. A permissively valid winner is not silently decoded and re-encoded. For every selected candidate the state retains both its exact bytes and decoded AST. If canonical re-encoding is used, it first requires a theorem that re-encoding preserves all behavior/resources, does not increase score, and does not worsen the raw-byte tie order.

`Artifact/Bytes.lean` SHALL contain the selected bytes as a checked literal generated from the pure selection result:

```lean
def Release.selectionResult : SolveResult SelectedArtifact :=
  Artifact.select
    Release.wasmProfile Release.gemmProblem Release.costObjective Release.seal

theorem Release.selectionResult_optimal :
  ∃ selected certificate,
    Release.selectionResult = .optimal selected certificate

def Release.selectedArtifactOrBaseline : SelectedArtifact :=
  match Release.selectionResult with
  | .optimal selected _ => selected
  | _ => Release.provedCorrectBaseline

theorem Release.selectedArtifactOrBaseline_eq
    (h : Release.selectionResult = .optimal selected certificate) :
  Release.selectedArtifactOrBaseline = selected

def Release.artifactBytes : ByteArray :=
  Release.selectedArtifactOrBaseline.bytes

def Release.committedArtifactBytes : ByteArray :=
  -- a generated literal whose complete contents are checked in
  Release.generatedByteLiteral

theorem Release.committed_eq_selected :
  Release.committedArtifactBytes = Release.artifactBytes
```

The filesystem file is outside the Lean kernel. `artifact-check` SHALL compare every file byte with `Release.committedArtifactBytes`; the proof manifest records both the Lean declaration and file digest. An `IO` writer may write the literal but may not transform it. If any external compiler is introduced, byte-for-byte translation validation against the proved AST is mandatory; testing source and binary separately is insufficient.

## 12. UOR Atlas state

### 12.1 State

```lean
structure Atlas.StateBody where
  declarationBase : CanonicalDeclarationSet
  accumulatedDeltaRoot : DeltaRoot
  semanticObjects : CanonicalObjectMap
  shapeEdges : CanonicalHypergraph
  semanticClosure : SemanticClosureBody
  attentionIndex : AttentionIndexBody
  dependencyGraph : DependencyGraphBody
  candidateFacts : CandidateFactMap
  costSurfaces : CostSurfaceMap
  searchPartitions : PartitionMap
  lowerEnvelope : LowerEnvelopeBody
  certificates : CertificateStoreBody
  objectiveId : ObjectiveId
  profileId : ProfileId
  problemId : ProblemId

def Atlas.StateId (body : StateBody) : StateIdentity :=
  CanonicalObjectId.ofTyped
    (Identity Atlas.StateBody.identitySchema body)

structure Atlas.UnsealedState where
  body : StateBody
  bodyId : StateIdentity
  bodyIdEq : bodyId = Atlas.StateId body
  retainedObjects : CompleteObjectGraph body

structure Atlas.SealCore where
  stateId : StateIdentity
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  closureRoot : ClosureRoot
  attentionRoot : AttentionRoot
  dependencyRoot : DependencyRoot
  partitionCoverRoot : PartitionCoverRoot
  envelopeRoot : EnvelopeRoot
  certificateRoot : CertificateRoot
  retentionRoot : RetentionRoot
  baselineScore : Nat

abbrev Atlas.SealCoreIdentity := ObjectId Atlas.SealCore

def Atlas.SealCoreId (core : SealCore) : SealCoreIdentity :=
  Identity Atlas.SealCore.identitySchema core

inductive Atlas.SealCheckTag
  | closureLeast
  | attentionComplete
  | dependenciesComplete
  | universalCoverComplete
  | envelopeExact
  | certificatesSound
  | retentionComplete

structure Atlas.SealCertificateBody where
  version : Nat
  coreId : SealCoreIdentity
  stateId : StateIdentity
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  closureRoot : ClosureRoot
  attentionRoot : AttentionRoot
  dependencyRoot : DependencyRoot
  partitionCoverRoot : PartitionCoverRoot
  envelopeRoot : EnvelopeRoot
  certificateRoot : CertificateRoot
  retentionRoot : RetentionRoot
  closureCheckResultId : CanonicalObjectId
  attentionCheckResultId : CanonicalObjectId
  dependencyCheckResultId : CanonicalObjectId
  partitionCheckResultId : CanonicalObjectId
  envelopeCheckResultId : CanonicalObjectId
  certificateStoreCheckResultId : CanonicalObjectId
  retentionCheckResultId : CanonicalObjectId

abbrev Atlas.SealIdentity :=
  ObjectId (Atlas.SealCore × Atlas.SealCertificateBody)

def Atlas.VerifiesSealCertificateBody
    (state : UnsealedState) (core : SealCore)
    (body : SealCertificateBody) : Prop :=
  body.version = 1 ∧
  body.coreId = Atlas.SealCoreId core ∧
  body.stateId = state.bodyId ∧
  body.stateId = core.stateId ∧
  body.profileId = core.profileId ∧
  body.problemId = core.problemId ∧
  body.objectiveId = core.objectiveId ∧
  body.closureRoot = core.closureRoot ∧
  body.attentionRoot = core.attentionRoot ∧
  body.dependencyRoot = core.dependencyRoot ∧
  body.partitionCoverRoot = core.partitionCoverRoot ∧
  body.envelopeRoot = core.envelopeRoot ∧
  body.certificateRoot = core.certificateRoot ∧
  body.retentionRoot = core.retentionRoot ∧
  body.closureCheckResultId =
    Atlas.canonicalSealCheckResultId state core .closureLeast ∧
  body.attentionCheckResultId =
    Atlas.canonicalSealCheckResultId state core .attentionComplete ∧
  body.dependencyCheckResultId =
    Atlas.canonicalSealCheckResultId state core .dependenciesComplete ∧
  body.partitionCheckResultId =
    Atlas.canonicalSealCheckResultId state core .universalCoverComplete ∧
  body.envelopeCheckResultId =
    Atlas.canonicalSealCheckResultId state core .envelopeExact ∧
  body.certificateStoreCheckResultId =
    Atlas.canonicalSealCheckResultId state core .certificatesSound ∧
  body.retentionCheckResultId =
    Atlas.canonicalSealCheckResultId state core .retentionComplete

structure Atlas.SealCertificate (state : UnsealedState) (core : SealCore) where
  body : SealCertificateBody
  bodyValid : VerifiesSealCertificateBody state core body
  coreBindsState : core.stateId = state.bodyId ∧
    core.profileId = state.body.profileId ∧
    core.problemId = state.body.problemId ∧
    core.objectiveId = state.body.objectiveId
  closureLeast : VerifiesLeastClosure state core
  attentionComplete : VerifiesAttentionCoverage state core
  dependenciesComplete : VerifiesDependencyCoverage state core
  universalCoverComplete : VerifiesRootPartitionCover state core
  envelopeExact : VerifiesLowerEnvelope state core
  certificatesSound : VerifiesCertificateStore state core
  retentionComplete : ResolvesEveryReferencedPreimage state core

structure Atlas.SealedState where
  state : UnsealedState
  core : SealCore
  certificate : SealCertificate state core
  sealId : SealIdentity
  sealIdEq : sealId =
    Identity Atlas.SealedState.identitySchema (core, certificate.body)
```

The deterministic checkers define `canonicalSealCheckResultId` by storing the
complete checker input, result, and retained preimages in canonical form. The
repository SHALL prove:

```lean
theorem seal_certificate_body_unique
    (state : Atlas.UnsealedState)
    (core : Atlas.SealCore)
    (a b : Atlas.SealCertificateBody)
    (ha : Atlas.VerifiesSealCertificateBody state core a)
    (hb : Atlas.VerifiesSealCertificateBody state core b) :
  a = b
```

Construction is acyclic: proof-free `StateBody` is identified first, proof-free `SealCore` refers to that identity, proof-free `SealCertificateBody` records the checked certificate-object identities, `SealCertificate` proves that body/core, and `sealId` identifies only `(core, certificate.body)`. No identity hashes a Lean proof or function, and no component contains its enclosing identity.

Optimizer conclusions such as "best," "dominated," or "selected" SHALL NOT be premises in semantic closure. Semantic facts are computed first; snapshot identity is fixed; optimization certificates are constructed afterward. This prevents a selected result from warranting itself.

### 12.2 Attention

Every shape and edge receives a canonical applicability signature. Attention is an exact work-routing mechanism:

```lean
def Atlas.attend
    (s : StateBody) (request : RequestSignature) : CanonicalSet CanonicalObjectId
```

For artifact selection, an omitted candidate or partition is legal only when a retained proof shows that it is empty, cannot beat the current bound, or is exactly reconstructible elsewhere. Required theorem:

```lean
theorem attention_no_optimum_relevant_false_negative
    {profile : Wasm.Profile}
    {problem : Gemm.Problem profile}
    {objective : Cost.ProperObjective profile problem}
    {state : Atlas.UnsealedState}
    {core : Atlas.SealCore}
    {candidateBytes : ByteArray}
    {evaluation : Universal.SystemEvaluation profile problem candidateBytes}
    (hsealed : Atlas.SealCertificate state core)
    (hprofile : Wasm.ProfileId profile = core.profileId)
    (hproblem : Gemm.ProblemId problem = core.problemId)
    (hobjective : Cost.ObjectiveId objective = core.objectiveId)
    (hc : Universal.ProfileValid profile candidateBytes)
    (heval : Universal.SystemEvaluationRel profile problem candidateBytes evaluation)
    (hscore : objective.score evaluation.cost ≤ core.baselineScore) :
  Atlas.AttentionContains core candidateBytes ∨
  Atlas.HasSoundExclusionCertificate core candidateBytes evaluation
```

An attention score alone is never an optimality proof.

### 12.3 Dependency and invalidation

Every certificate SHALL list its exact object, edge, profile, problem, objective, and partition dependencies. Adding or changing an edge invalidates the complete transitive impact cone before any new seal. Unaffected certificates may be reused only with a verified transition warrant.

### 12.4 Lower envelope

The envelope maps exact descriptor/workload/objective regions to attained candidates or complete Pareto frontiers. It SHALL distinguish:

- attained minimum;
- infeasible region;
- nonattained infimum;
- incomplete coverage;
- unsupported profile;
- invalidated or unsealed state.

### 12.5 Update operator

```lean
structure Atlas.BuildBudget where
  remainingSteps : Nat
  remainingBytes : Nat

structure BudgetedResult (initial : BuildBudget) (α ε : Type) where
  result : CheckResult α ε
  consumedSteps : Nat
  consumedBytes : Nat
  remainder : BuildBudget
  stepsConservation : consumedSteps + remainder.remainingSteps = initial.remainingSteps
  bytesConservation : consumedBytes + remainder.remainingBytes = initial.remainingBytes

def Atlas.accumulate :
  (budget : BuildBudget) → UnsealedState → Delta →
    BudgetedResult budget UnsealedState UpdateError
def Atlas.rebuild :
  (budget : BuildBudget) → CanonicalDeclarationSet →
    BudgetedResult budget UnsealedState UpdateError
def Atlas.seal :
  (budget : BuildBudget) → UnsealedState →
    BudgetedResult budget SealedState SealError

def Atlas.semanticApplyBody : StateBody → Delta → StateBody
def Atlas.semanticRebuildBody : CanonicalDeclarationSet → StateBody
def Atlas.replayRecognitionCost : UnsealedState → Delta → BuildBudget
def Atlas.rebuildSufficientBudget : CanonicalDeclarationSet → BuildBudget
```

`semanticApplyBody` and `semanticRebuildBody` are total, structurally recursive
mathematical functions over finite canonical inputs. They do not allocate an
unbounded operational capability and do not bypass a result algebra. The
budgeted functions are executable implementations that refine these bodies.
The sufficient-budget functions are computed from exact input sizes and
verified termination measures rather than postulated.

The implementation sequence is:

1. canonicalize and accumulate genuinely new objects and edges;
2. compute the least new semantic closure;
3. update exact attention buckets;
4. compute the dependency impact cone;
5. invalidate every affected optimization fact;
6. refine or exhaust affected universal search partitions;
7. recompute exact cost surfaces and envelope cells;
8. verify all certificates from canonical preimages;
9. construct the immutable seal only when coverage is complete.

Required laws:

```lean
theorem update_empty_fixed_point :
  Atlas.accumulate budget state Atlas.Delta.empty =
    Atlas.noChangeResult budget state

theorem semantic_update_idempotent :
  Atlas.semanticApplyBody
      (Atlas.semanticApplyBody state.body delta) delta =
    Atlas.semanticApplyBody state.body delta

theorem update_idempotent
    (hfirst : Atlas.accumulate budget state delta = first)
    (hcomplete : first.result = .complete successor)
    (hreplay : Atlas.BudgetCovers first.remainder
      (Atlas.replayRecognitionCost successor delta)) :
  (Atlas.accumulate first.remainder successor delta).result = .complete successor

theorem accumulate_complete_refines_semantic
    (hupdate : (Atlas.accumulate budget state delta).result = .complete successor) :
  successor.body = Atlas.semanticApplyBody state.body delta

theorem rebuild_succeeds_at_computed_bound
    (declarations : CanonicalDeclarationSet) :
  (Atlas.rebuild (Atlas.rebuildSufficientBudget declarations) declarations).result =
    .complete (Atlas.unsealedFromBody (Atlas.semanticRebuildBody declarations))

theorem update_batch_confluent
    (hcompatible : Atlas.Compatible left right) :
  Atlas.canonicalize (Atlas.applyBatch state [left, right]) =
    Atlas.canonicalize (Atlas.applyBatch state [right, left])

theorem incremental_eq_full_rebuild
    (hupdate : (Atlas.accumulate budget state delta).result = .complete successor) :
  Atlas.canonicalize successor.body =
    Atlas.canonicalize
      (Atlas.semanticRebuildBody
        (state.body.declarationBase ∪ delta.declarations))

theorem candidate_expansion_value_monotone
    (honlyAdds : Atlas.OnlyAddsCandidates oldState newState) :
  newState.body.lowerEnvelope.value ≤ oldState.body.lowerEnvelope.value

theorem sealed_query_sound
    (sealed : Atlas.SealedState)
    (hquery : Atlas.query sealed request = .optimal selected certificate) :
  certificate.Verifies selected request sealed
```

`contents` is exactly `state.body.declarationBase`, so rebuild equivalence has no ambient input. The pure formal update model is single-threaded and immutable. Parallel search workers may propose canonical deltas, but they are merged through this pure function before sealing. Filesystem publication uses a temporary file plus atomic rename as an operational safeguard; no theorem relies on a concurrent CAS, hidden affine token, or ambient mutable head. Query and artifact emission SHALL use only a `SealedState`, so stale or partially merged evidence is never promoted.

## 13. Complete implementation algorithm

The release builder in `Artifact/Select.lean` SHALL implement this total pipeline.

### Phase A — authority and profile

1. Read no network state.
2. Recompute all authority content identities.
3. Construct the concrete Wasm profile, GEMM problem, and canonical proper objective.
4. Prove their internal validity and identity equalities.

### Phase B — baseline

1. Construct an input-total scalar GNAF GEMM plan.
2. Prove classifier, reference, termination, memory-safety, and resource correctness.
3. Compile and emit it with the verified compiler.
4. Compute its exact full-domain cost.
5. Use that attained cost only as an upper bound, never as a globality premise.

### Phase C — universal finite sublevel

1. Derive coordinate bounds from the baseline score and objective properness.
2. Construct the canonical root partition for every byte string capable of tying or beating the baseline.
3. Recursively decode, validate, summarize, split, exhaust, or dominate partitions.
4. For each potentially correct module, verify every raw input and every permitted bounded execution, either explicitly or via a sound and complete symbolic certificate.
5. Compute exact complete-system cost surfaces.
6. Store counterexamples for incorrect candidates and proof objects for valid candidates.
7. Prove the partition family disjoint, complete, and root-covering.

### Phase D — attained argmin

1. Fold a decidable total order over the exact finite survivor carrier.
2. Include the baseline as a nonempty witness.
3. Select minimum score and then canonical byte identity.
4. Prove selected membership, correctness, feasibility, lower bound, and attainment.

### Phase E — Atlas seal

1. Store candidate, cost, partition, lower-bound, and dependency objects.
2. Recompute the attention and envelope roots.
3. Verify that every root partition is exhausted, empty, dominated, or recursively complete.
4. Construct one `Atlas.SealedState` value.

### Phase F — artifact and manifest

1. Take the exact selected raw bytes; do not normalize or rebuild them through an unverified path.
2. Decode and validate them again inside Lean.
3. Prove full input-total refinement.
4. Generate `Artifact/Bytes.lean`, record its exact canonical bytes in
   `GeneratedProofInputBody`, and compile only after that identity is fixed.
5. Record the checked final declaration-environment digest and its exact
   source, toolchain, and dependency inputs in `PreFinalEnvironmentBody`.
6. Prove inside Lean that the generated literal equals
   `Release.artifactBytes`; have the external release gate compare the
   committed file byte for byte with that literal.
7. Write the artifact, seal, proof manifest, and acyclic output manifest
   deterministically, then reproduce every generated proof input and output in
   two clean trees. Only after both comparisons finish, emit the external
   `ReproducibilityAttestationBody`; never feed that attestation back into an
   identity it reports.

### Phase G — final theorem

1. Lift carrier minimality through universal sublevel coverage.
2. Prove every outside-sublevel competitor has a strictly larger score.
3. Join correctness, feasibility, lower bound, and attainment.
4. Close `released_wasm_gemm_gnaf_global_optimal` without hypotheses.

## 14. Core proof chain

The proof dependency graph SHALL be acyclic and match this order:

```text
pinned authority identities
        |
        v
Wasm syntax/binary/validation/execution ---- costed semantics --erasure--> plain semantics
        |                                      |
        v                                      v
GEMM ABI/classifier/reference            exact trace costs
        |                                      |
        +------------------+-------------------+
                           v
                 baseline plan correctness
                           |
                    attained upper bound
                           |
                  proper-objective bounds
                           |
          all-byte/all-input sublevel coverage
                           |
                 exact survivor costs
                           |
                constructive finite argmin
                     /             \
                    v               v
          universal lower bound   attainment
                    \               /
                     v             v
                exact artifact correctness
                           |
                           v
          released_wasm_gemm_gnaf_global_optimal
```

No lower node may be included as a field of an upper node whose purpose is to prove it.

## 15. Required Lean declarations

The following public declarations SHALL exist with fully implemented bodies and proofs. Names may gain namespaces but not weaker propositions.

```lean
Wasm.decode_sound
Wasm.decode_complete
Wasm.validate_iff_declarative
Wasm.validation_progress
Wasm.mem_successors_iff_step
Wasm.bounded_tree_covers_every_branch
Wasm.runFuel_sound
Wasm.runFuel_complete_with_bound
Wasm.costed_erase_iff_plain_run
Wasm.costed_initialization_erase
Wasm.profile_matches_pinned_revision

Gemm.classify_total
Gemm.reference_total
Gemm.valid_input_finite
Gemm.raw_input_finite
Gemm.raw_invocation_roundtrip
Gemm.raw_invocation_surjective
Gemm.abi_roundtrip
Gemm.classifier_exact_domain
Gemm.mandatory_family_nonzero_witnesses
Gemm.observation_covers_status_and_full_c

Cost.module_bytes_exact
Cost.transition_accounting_positive
Cost.objective_sublevel_finite

GNAF.normalize_semantics
GNAF.normalize_cost_le
GNAF.compile_refines
GNAF.compile_cost_exact
Artifact.decode_emit

Universal.possible_winner_within_sublevel
Universal.byte_enumerator_complete
Universal.input_enumerator_complete
Universal.execution_checker_sound
Universal.execution_checker_complete_within_sublevel
Universal.system_evaluation_rel_sound
Universal.system_evaluation_rel_complete
Universal.system_evaluation_rel_functional
Universal.partition_cover_complete
Universal.universal_sublevel_coverage
Universal.selected_le_every_sublevel_member
Universal.all_competitors_lower_bound

Atlas.semantic_closure_least
Atlas.attention_no_optimum_relevant_false_negative
Atlas.invalidation_complete
Atlas.incremental_eq_full_rebuild
Atlas.seal_verifier_reconstructs_every_preimage
Atlas.seal_implies_universal_coverage
Atlas.lifecycle_prefix_conservation
Atlas.lifecycle_native_bound
Atlas.lifecycle_incremental_semantics_eq_full_rebuild
Atlas.lifecycle_full_rebuild_comparator_exact

Artifact.released_bytes_equal_selection
Artifact.committed_literal_equal_selection
Artifact.released_bytes_decode
Artifact.released_bytes_validate
Artifact.released_input_total
Artifact.released_attains_lower_bound
Artifact.released_wasm_gemm_gnaf_global_optimal
```

The final declaration SHALL not accept `Coverage`, `LowerBound`, `Correct`, `FaithfulWasm`, `CompilerCorrect`, or `GlobalOptimal` as parameters. Those facts must be concrete upstream theorems.

## 16. Asymptotic family

The concrete release theorem is finite because WebAssembly profiles and resource sublevels are finite. The repository SHALL also define a uniform family indexed by an explicit scale profile:

```lean
structure ScaleProfile where
  addressBits : Nat
  maxPages : Nat
  maxInvocationBytes : Nat
  workloadRepetitions : Nat

def ScaleProfile.profile (s : ScaleProfile) : Wasm.Profile :=
  Release.wasmProfile.withLimits s.addressBits s.maxPages s.maxInvocationBytes

def ScaleProfile.problem (s : ScaleProfile) : Gemm.Problem s.profile :=
  Release.gemmProblemFor s.profile s.workloadRepetitions

def ScaleProfile.objective (s : ScaleProfile) :
    Cost.ProperObjective s.profile s.problem :=
  Release.canonicalObjectiveFor s.profile s.problem

structure SuccessfulBuild (s : ScaleProfile) where
  bytes : ByteArray
  decodedModule : Wasm.Module
  sealedState : Atlas.SealedState
  evaluation : Universal.SystemEvaluation s.profile s.problem bytes
  localChecks : BuildLocalChecks s bytes decodedModule sealedState evaluation

inductive BuildResult (s : ScaleProfile)
  | success (result : SuccessfulBuild s)
  | rejected (report : InvalidScaleReport)
  | resourceExhausted (report : ResourceReport)
  | internalFailure (report : FailureReport)

def buildFamily (s : ScaleProfile) : BuildResult s
```

`SuccessfulBuild s` SHALL contain only concrete bytes, the canonical state and seal produced by `buildFamily`, checker outputs, and their local validity equalities. Its seal may carry coverage certificates constructed and verified by the builder. It SHALL NOT contain `GlobalOptimal`, a free lower bound, or a coverage premise independent of that checked seal, and callers cannot construct it without the builder-result equality used below.

The family theorem SHALL use one terminating builder:

```lean
theorem buildFamily_global
    (s : ScaleProfile)
    (hs : s.Admissible)
    (result : SuccessfulBuild s)
    (hresult : buildFamily s = .success result) :
  GlobalOptimal s.profile s.problem s.objective result.bytes
```

The family SHALL also prove constructive success rather than leaving `SuccessfulBuild s` empty:

```lean
theorem buildFamily_succeeds
    (s : ScaleProfile) (hs : s.Admissible) :
  ∃ result : SuccessfulBuild s, buildFamily s = .success result
```

For the first family, admissibility is the concrete nonempty predicate:

```lean
def ScaleProfile.Admissible (s : ScaleProfile) : Prop :=
  s.addressBits = 32 ∧
  s.maxPages = 65536 ∧
  s.maxInvocationBytes = 2^32 - 1 ∧
  1 ≤ s.workloadRepetitions

def ScaleProfile.WorkloadLE (a b : ScaleProfile) : Prop :=
  a.addressBits = b.addressBits ∧
  a.maxPages = b.maxPages ∧
  a.maxInvocationBytes = b.maxInvocationBytes ∧
  a.workloadRepetitions ≤ b.workloadRepetitions

def canonicalScale (n : Nat) : ScaleProfile :=
  { addressBits := 32
    maxPages := 65536
    maxInvocationBytes := 2^32 - 1
    workloadRepetitions := n + 1 }

theorem canonicalScale_admissible (n : Nat) :
  (canonicalScale n).Admissible

theorem canonicalScale_directed (a b : Nat) :
  ∃ c,
    (canonicalScale a).WorkloadLE (canonicalScale c) ∧
    (canonicalScale b).WorkloadLE (canonicalScale c)

theorem canonicalScale_cofinal (s : ScaleProfile) (hs : s.Admissible) :
  ∃ n, s.WorkloadLE (canonicalScale n)

theorem canonicalScale_unbounded :
  ∀ horizon, ∃ n, horizon < (canonicalScale n).workloadRepetitions

theorem canonicalScale_problem_identity_injective
    (a b : Nat)
    (ha : a ≠ b) :
  Gemm.ProblemId ((canonicalScale a).problem) ≠
    Gemm.ProblemId ((canonicalScale b).problem)
```

`workloadRepetitions` is a theorem-relevant field of `Gemm.ProblemBody`. At
scale `n`, the canonical protocol executes every raw invocation exactly
`n + 1` times, always with a fresh instance, and aggregates one static
decode/validation vector plus `(n + 1)` copies of the complete dynamic
full-domain vector. Thus the family is not a renamed search bound: problem
identity, total charged work, and potentially the globally optimal balance
between module bytes and dynamic work all vary with `n`. The implementation
SHALL prove the exact affine cost formula and the identity-injectivity theorem
above. A different address model requires a new mechanized profile and a new
explicitly inhabited directed family.

The Atlas lifecycle analysis SHALL separately use an unbounded finite update
trace `(Delta 0, …, Delta (horizon - 1))`. Each scale binds the exact initial
state, delta sequence, query sequence, and seal points. Its complete lifecycle
cost includes accumulation, least closure, attention updates, invalidation,
partition refinement, proof checking, retained bytes, sealing, query, and
execution. The repository SHALL prove incremental/full-rebuild equality at
every prefix and state its competitive or regret comparator before attaching
an asymptotic bound. The artifact-family theorem and lifecycle theorem are
distinct; neither may borrow a free preparation phase from the other.

The lifecycle carrier and accounting are normative rather than prose-only:

```lean
structure Atlas.LifecycleTraceBody where
  version : Nat
  profileId : ProfileId
  problemId : ProblemId
  objectiveId : ObjectiveId
  initialStateId : StateIdentity
  deltaIds : CanonicalList DeltaId
  requestIds : CanonicalList RequestId
  requestAfterPrefixes : CanonicalList Nat
  sealAfterPrefixes : CanonicalFinset Nat
  horizon : Nat

abbrev Atlas.LifecycleTraceIdentity := ObjectId Atlas.LifecycleTraceBody

def Atlas.LifecycleTraceId (body : LifecycleTraceBody) : LifecycleTraceIdentity :=
  Identity Atlas.LifecycleTraceBody.identitySchema body

structure Atlas.ResolvedLifecycleTrace (body : LifecycleTraceBody) where
  initialState : UnsealedState
  deltas : CanonicalList Delta
  requests : CanonicalList Request
  resolvesInitial : initialState.bodyId = body.initialStateId
  resolvesDeltas : CanonicalIds deltas = body.deltaIds
  resolvesRequests : CanonicalIds requests = body.requestIds
  horizonEq : body.horizon = body.deltaIds.length
  requestScheduleLength :
    body.requestAfterPrefixes.length = body.requestIds.length
  requestOrdinalsValid : ∀ ordinal ∈ body.requestAfterPrefixes,
    ordinal ≤ body.horizon
  sealOrdinalsValid : ∀ ordinal ∈ body.sealAfterPrefixes,
    ordinal ≤ body.horizon
  completePreimages : CompleteLifecycleObjectGraph body

structure Atlas.LifecycleSizeVector where
  horizon : Nat
  authorityBytesChecked : Nat
  deltaBytes : Nat
  requestBytesChecked : Nat
  canonicalNovelObjects : Nat
  canonicalNovelEdges : Nat
  closureDerivations : Nat
  attentionBucketsTouched : Nat
  dependencyImpactObjects : Nat
  partitionCellsChanged : Nat
  certificateBytesChecked : Nat
  retainedStateBytes : Nat
  sealCount : Nat
  sealInputBytesScanned : Nat
  queryCount : Nat
  attentionCandidatesVisited : Nat
  migrationObjects : Nat
  peakWorkingBytes : Nat
  artifactWork : Cost.ArtifactVector

inductive Atlas.LifecycleAlgorithmTag
  | nativeIncremental
  | canonicalFullRebuildAtEverySeal

structure Atlas.LifecyclePrefixResult
    {body : LifecycleTraceBody}
    (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag) where
  prefixOrdinal : Fin (body.horizon + 1)
  beforeStateId : StateIdentity
  deltaId : Option DeltaId
  afterStateId : StateIdentity
  sealId : Option SealIdentity
  queryResultIds : CanonicalList QueryResultId
  cost : Cost.LifecycleVector
  exactTransition : VerifiesExactLifecyclePrefix
    algorithm trace prefixOrdinal beforeStateId deltaId afterStateId
      sealId queryResultIds
  exactCost : Cost.VerifiesLifecyclePrefixCost
    algorithm trace prefixOrdinal beforeStateId deltaId afterStateId
      sealId queryResultIds cost

structure Atlas.LifecycleEvaluation
    {body : LifecycleTraceBody}
    (trace : ResolvedLifecycleTrace body)
    (algorithm : LifecycleAlgorithmTag) where
  prefixes : NonemptyCanonicalList (LifecyclePrefixResult trace algorithm)
  exactPrefixCover : CoversExactlyEveryPrefix body prefixes
  size : Atlas.LifecycleSizeVector
  sizeExact : VerifiesLifecycleSize trace prefixes size
  total : Cost.LifecycleVector
  totalExact : total = Cost.sumLifecycle (prefixes.map (·.cost))

inductive Atlas.LifecycleComparatorTag
  | canonicalFullRebuildAtEverySeal

def Atlas.canonicalFullRebuildEvaluation
    {body : LifecycleTraceBody}
    (trace : ResolvedLifecycleTrace body) :
    LifecycleEvaluation trace .canonicalFullRebuildAtEverySeal

def Atlas.nativeLifecycleBound
    (table : Cost.PrimitiveCostTable)
    (size : LifecycleSizeVector) : Cost.LifecycleVector
```

`LifecycleTraceBody`, every referenced delta/request body, and
`Cost.PrimitiveCostTable` are first-order, canonically encoded release inputs;
no function or proof is hashed. A lifecycle evaluation is strategy-indexed;
there is no untagged evaluation whose native and full-rebuild costs can be
interchanged. Each prefix verifier binds its strategy, before state, selected
delta, after state, optional seal, exact query-result identities, and charged
cost. `nativeLifecycleBound` is the fixed
componentwise polynomial encoded in `Cost/Lifecycle.lean`: authority checking
is bounded by `cAuthority · authorityBytesChecked`; canonicalization is
bounded by `cCanonicalize · deltaBytes · (log2 (retainedStateBytes + 2) + 1)`;
closure by `cClosure · closureDerivations`; indexing by
`cIndex · (canonicalNovelObjects + canonicalNovelEdges +
attentionBucketsTouched)`; invalidation by
`cDependency · dependencyImpactObjects`; partition work by
`cPartition · partitionCellsChanged`; verification and sealing by their fixed
coefficients times `certificateBytesChecked`, `sealCount`, and the cumulative
`sealInputBytesScanned`; query work by `cQuery · (queryCount +
requestBytesChecked + attentionCandidatesVisited)`. Artifact work is
exactly `size.artifactWork`; migration is bounded by
`cMigration · migrationObjects`; retained-state and peak-working-byte output
coordinates are respectively `size.retainedStateBytes` and
`size.peakWorkingBytes`, not absorbed into constants. Every
coefficient is a positive natural in the canonical primitive-cost table.

`Cost.sumLifecycle` is also frozen: every step/count coordinate is natural
addition in prefix order; `retainedStateBytes` and `peakWorkingBytes` are
componentwise maxima; artifact static coordinates are charged at each actual
artifact construction, artifact dynamic sums are added, and artifact dynamic
peaks are maxima. The empty fold is the all-zero vector. This mixed fold, the
prefix order, and the exact final/max retained-state interpretation are part
of the lifecycle cost-profile identity.

The implementation SHALL close these theorems:

```lean
theorem lifecycle_prefix_conservation
    {body : Atlas.LifecycleTraceBody}
    {trace : Atlas.ResolvedLifecycleTrace body}
    {algorithm : Atlas.LifecycleAlgorithmTag}
    (evaluation : Atlas.LifecycleEvaluation trace algorithm) :
  evaluation.total =
    Cost.sumLifecycle (evaluation.prefixes.map (·.cost))

theorem lifecycle_native_bound
    {body : Atlas.LifecycleTraceBody}
    {trace : Atlas.ResolvedLifecycleTrace body}
    (evaluation : Atlas.LifecycleEvaluation trace .nativeIncremental) :
  evaluation.total ≤
    Atlas.nativeLifecycleBound Release.primitiveCostTable evaluation.size

theorem lifecycle_incremental_semantics_eq_full_rebuild
    {body : Atlas.LifecycleTraceBody}
    {trace : Atlas.ResolvedLifecycleTrace body}
    (evaluation : Atlas.LifecycleEvaluation trace .nativeIncremental) :
  SameSealedQueryAndExecutionObservations
    evaluation
    (Atlas.canonicalFullRebuildEvaluation trace)

theorem lifecycle_full_rebuild_comparator_exact
    {body : Atlas.LifecycleTraceBody}
    {trace : Atlas.ResolvedLifecycleTrace body}
    (evaluation : Atlas.LifecycleEvaluation trace .nativeIncremental) :
  Atlas.regretAgainst .canonicalFullRebuildAtEverySeal evaluation =
    Cost.truncatedDifference
      evaluation.total
      (Atlas.canonicalFullRebuildEvaluation trace).total
```

An amortized statement SHALL divide only the exact summed numerator by the
explicit positive `body.horizon`; it SHALL retain the nonamortized total and
every prefix result. A competitive or regret claim against any comparator
other than the canonical full rebuild requires a new first-order comparator
body, a complete causal-information contract, and a separately proved theorem.

Asymptotic bounds SHALL name a directed limit regime and count separately:

- canonical novelty;
- touched attention buckets;
- dependency impact cone;
- partitions split or discharged;
- semantic and verifier steps;
- retained proof/state bytes;
- query selection work;
- emitted execution work and output.

Persistent work MAY be amortized only over an exact declared trace and horizon. The family theorem establishes a UOR-native stateful complexity model. Calling it a new classical complexity class additionally requires proved simulations, containments, and a separation theorem; the repository SHALL not infer those from the cost vocabulary alone.

## 17. Conformance model

### 17.1 Claim levels

```lean
inductive ClaimLevel
  | authority
  | buildEvidence
  | formalProof
  | measurement
  | open
```

Only `formalProof` supports words such as "proved," "theorem," or "globally optimal." Tests, benchmarks, external citations, and measurements cannot promote a claim to that level.

### 17.2 Formal claim row

Each row in `model/claims.json` SHALL contain:

- unique claim ID and exact statement;
- claim level;
- proposition canonical bytes and identity;
- exact Lean declaration and source module;
- Wasm profile, GEMM problem, cost objective, universe, and artifact identities;
- direct proof-dependency IDs;
- collected transitive axiom names;
- checker command;
- falsifier fixture and expected rejection;
- status and release applicability.

The conformance tool SHALL load the compiled Lean environment, confirm the declaration exists with the recorded proposition, recompute every identity, collect its transitive axioms, and reject duplicate or orphan rows.

For `GO-*`, `UV-*`, `CO-*`, `WS-*`, and `GM-*`, the tool SHALL unfold every scope-critical definition, compare it to the independently checked `WGG-GO-1` authority schema, and enforce the dependency firewall. A self-recorded proposition such as `GlobalOptimal := True`, an artifact-specific `ProfileValid`, or a selector-defined feasibility predicate must fail even if its recorded hash matches itself.

### 17.3 Required claim families

- `LF-*`: Lean/toolchain/dependency identity and axiom closure.
- `WS-*`: WebAssembly binary, validation, execution, feature, and cost-erasure semantics.
- `GM-*`: GEMM classifier, arithmetic, ABI, and reference semantics.
- `BI-*`: emitted byte identity, decoding, validation, imports/exports, and execution refinement.
- `UV-*`: universal byte/input/run sublevel and partition coverage.
- `CO-*`: complete accounting, objective laws, properness, and aggregation.
- `LB-*`: universal lower bound and falsification mutations.
- `AT-*`: GNAF/Atlas closure, attention, invalidation, update, and seal.
- `GO-*`: attainment and final global theorem.
- `RT-*`: executable differential evidence, never proof promotion.
- `ME-*`: pinned engine measurements, always measurement or open.
- `CM-*`: registry, manifest, source, dependency, and release-gate integrity.

`CONFORMANCE.md` SHALL be generated deterministically from this registry and SHALL be byte-clean after generation.

## 18. Test and falsification requirements

Tests supplement proofs and SHALL exercise at least:

- malformed binaries, noncanonical encodings, invalid section order, invalid
  types, traps, caught exceptions, and uncaught exception terminals;
- every enabled WebAssembly instruction rule and proposal boundary;
- zero dimensions, `k = 0`, maximum dimensions, stride overflow, alignment, and aliasing;
- integer overflow modes and all floating special classes;
- cancellation, signed zero, subnormals, infinities, NaNs, and rounding ties;
- every status result and forbidden memory effect;
- compiler round-trip and cost-event correspondence;
- attention false positives and planted false negatives;
- dependency invalidation after new edges;
- full-rebuild/update equality across delta orderings;
- partition gaps, overlaps, duplicate identities, forged lower bounds, and stale seals;
- artifact byte mutation, manifest mutation, profile mutation, and objective mutation.

Mutation gates SHALL plant at least one defect per proof family and confirm the corresponding checker fails. A mutation suite that merely expects runtime output differences does not test universal-coverage integrity.

Cross-engine runs MAY include pinned Wasmtime, Wasmer, V8, and reference-interpreter versions. They demonstrate portability behavior only; throughput rankings remain measurements and cannot discharge `GO-*`.

## 19. Source and theorem discipline

Every proof/product module SHALL start with:

```lean
set_option autoImplicit false
```

The proof dependency graph SHALL exclude:

- project-declared `axiom` declarations;
- `sorry`, `admit`, and generated `sorryAx`;
- `unsafe`, `partial`, and `noncomputable` definitions on the product/proof path;
- unchecked FFI results;
- `native_decide` and unrecorded code-generation trust;
- theorem conclusions stored as unproved structure fields;
- a cryptographic collision assumption;
- an assumed all-program coverage or lower-bound proposition;
- an assumed equality between source, generated AST, and committed bytes.

Source scanning is defense in depth. The decisive audit SHALL inspect the compiled environment and the transitive dependencies of every public theorem.

## 20. Build, verification, and release

### 20.1 Commands

The `Justfile` SHALL expose:

```text
just bootstrap       verify exact local tool identities without network mutation
just build           lake build with warnings treated as errors
just test            compile and run all executable tests
just prove           build every public theorem and proof manifest
just emit            deterministically emit artifact and Atlas seal
just artifact-check  decode, validate, execute-check, and identity-check committed bytes
just claims          validate the complete claim/dependency registry
just axioms          collect and enforce exact theorem axiom closures
just mutation        execute planted falsifiers
just reproduce       emit twice from clean temporary trees and compare every byte
just docs            regenerate conformance documentation
just vv              run the entire normative release gate
```

`just vv` SHALL include every command above as applicable. No required dependency or license gate may live only in CI.

### 20.2 Release gate

From a clean checkout with network disabled, `just vv` SHALL fail unless all of the following hold:

1. Toolchain, authority, handwritten source, generated proof-input, pre-final
   declaration-environment, and dependency identities match their ordered
   acyclic manifests.
2. The required claim graph is nonempty, acyclic, and complete.
3. Lean builds with no placeholder or unexpected axiom.
4. The concrete WebAssembly and GEMM semantics are built.
5. Universal sublevel and outside-sublevel coverage are proved.
6. The committed artifact exists and its bytes match the proved value.
7. Artifact decode, validation, ABI, correctness, resource, and cost theorems hold.
8. Universal lower bound and attainment theorems hold.
9. `released_wasm_gemm_gnaf_global_optimal` exists with the exact recorded proposition and accepted axiom set.
10. The Atlas seal reconstructs from retained canonical objects.
11. Mutation suites demonstrate that each decisive gate rejects a planted fault.
12. Two clean emissions produce byte-identical artifact, seal, manifest, and generated documentation.
13. The worktree remains clean after verification.

### 20.3 CI

Actions SHALL be pinned to commit hashes. CI SHALL use the exact checked-in Lean toolchain, no mutable `stable` selector, and no undeclared cache input. Separate jobs MAY improve diagnosis, but one final job SHALL run the same `just vv` command used locally.

## 21. README and publication language

After the final theorem closes, the primary wording SHALL be:

> The committed `wasm-gemm-gnaf.wasm` binary is Lean-proved input-total for every invocation in the released arbitrary-GEMM ABI. Under the pinned WebAssembly semantics, closed portable profile, exact GEMM relation, and fully charged UOR-Wasm objective, it attains the global minimum among every correct and feasible WebAssembly byte module in the same profile. The proof covers the complete all-byte cost sublevel and proves that no module outside it can tie or improve the score. This is an abstract WebAssembly-cost theorem, not a claim about physical latency on every engine or processor.

Before closure, the README SHALL instead say:

> Global optimality is the release theorem required by `SPEC.md`; it has not been established. No current test, benchmark, candidate search, or partial proof may be cited as that theorem.

The project SHALL never use "universal," "global," "arbitrary," "complete," or "optimal" without linking the exact profile/problem/objective/universe proposition.

## 22. Implementation sequence

Each stage below SHALL leave `just vv` truthful. A theorem claim is registered only when its complete dependency chain exists.

1. Replace template metadata and bootstrap the pinned Lean project.
2. Implement canonical foundation, result algebra, identity, and environment-level axiom audit.
3. Implement and conformance-map the pinned WebAssembly semantics.
4. Implement costed semantics and prove erasure.
5. Implement GEMM ABI, classifier, arithmetic contracts, and reference relation.
6. Implement and prove the complete scalar baseline GNAF plan.
7. Implement the verified GNAF-to-Wasm compiler and pure emitter.
8. Emit and prove the first correct artifact; make no optimality claim.
9. Implement proper objectives and derive finite winner sublevels.
10. Implement universal byte, input, and bounded-run enumerators.
11. Implement partition certificates, checkers, and exact Atlas retention.
12. Prove sublevel completeness and outside-sublevel exclusion.
13. Implement constructive argmin and prove the universal lower bound.
14. Prove attainment by the exact emitted bytes.
15. Implement Atlas update, attention, invalidation, rebuild equivalence, and sealing.
16. Close the concrete release theorem and register `GO-*` claims.
17. Add the uniform scale-family theorem and native asymptotic bounds.
18. Publish only after clean, offline, reproducible verification succeeds.

Adding a scalar kind, arithmetic mode, ABI form, WebAssembly feature, objective, or resource regime is a semantic-universe change. It SHALL extend the classifier, reference semantics, cost model, universal coverage, compiler, artifact, proof manifest, and release theorem together.

## 23. Definition of complete

The repository is complete only when:

- every required file and declaration above is implemented;
- the exact committed artifact is generated by the proved path;
- correctness covers every raw invocation and every permitted execution;
- comparison covers every same-profile WebAssembly byte module;
- the cost objective charges every unbounded advantage symmetrically;
- universal coverage is derived, not assumed;
- lower bound and attainment are both proved;
- Atlas attention and incremental maintenance cannot omit an optimum-relevant object;
- final theorem dependencies pass the compiled axiom audit;
- all claims, artifacts, and authorities are identity-bound and reproducible;
- `just vv` succeeds from a clean offline checkout;
- public prose uses exactly the theorem's scope.

Anything less is an intermediate research state, not proven global optimality.
