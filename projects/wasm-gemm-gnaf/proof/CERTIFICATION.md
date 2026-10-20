# Certification outcome — WGG-GO-1

Answer class: **`WorkloadIncomplete`** (UOR-GNAF v1-draft.2 §10.9 total answer carrier)

This document is the repository's exact terminal answer for the requested claim.
It is not a status report, a roadmap, or a promise. Under the pinned authority it
is a *conforming terminal result*: UOR-GNAF §10.9 makes `WorkloadIncomplete` a
branch of the total answer type, and §13.3 requires it here rather than a label:

> Computational difficulty does not weaken exactness. A machine MAY return an
> honest weaker claim; it MUST NOT return an unproved global label.

SPEC.md §1 states the same rule for this repository:

> If the universal proof cannot be closed, the correct repository outcome is
> `incomplete`; weakening the universe, silently strengthening the machine,
> omitting cost, or changing the public wording is forbidden.

Accordingly **nothing below is weakened**. `GlobalOptimal` is defined at full
strength, quantified over all finite byte sequences, with the cost objective
charging every coordinate at weight one.

---

## 1. RequestedClaimScope

The requested claim is exactly `authority/global-optimality-WGG-GO-1.json`:

```
GlobalOptimal
  Release.wasmProfile
  Release.gemmProblem
  Release.costObjective
  Release.artifactBytes
```

with the quantifier in conjunct 7 ranging over **all** finite byte sequences that
decode and validate under the pinned profile — not GNAF plans, not registered
kernels, not attention-indexed candidates.

## 2. Obligation ledger

UOR-GNAF §19.3 fixes the required evidence for this claim class:

| Obligation | Required evidence | Status |
|---|---|---|
| Global scalar optimum | complete admission, **attainment**, and **exhaustive / proof-complete / no-improver coverage *or* a universal lower bound attained** | **OUTSTANDING** |

Decomposed against SPEC §10:

| # | Sub-obligation | Status |
|---|---|---|
| O-1 | Competitor universe defined extensionally over all byte strings | definable; stated at full strength |
| O-2 | Sublevel is finite and decidable | **closable** — see §4.1 |
| O-3 | Complete admission (`SystemEvaluationRel` sound/complete/functional) | outstanding, gated on O-6 |
| O-4 | Attainment: shipped bytes' exact score computed | outstanding, gated on O-6 |
| O-5 | **Universal lower bound `F`, attained** | **OUTSTANDING — no known technique** |
| O-6 | Mechanized Wasm Core 3.0 semantics (GC, EH, SIMD, tail calls) | outstanding — engineering; see §2.1 |
| O-7 | Non-degenerate release seam (`GO-008`) and a per-rule cost table (`CO-006`) | **discharged** — both closed; see §2.1 for the two disclosures that travel with `GO-008` |

Release gate step 9 fails. It is *supposed* to fail. See §6.

### 2.1 O-6 is larger than the module count suggests

A large module and theorem count (see `CONFORMANCE.md` for the live inventory) can
create a misleading impression of proximity. Three scope facts, all discovered by
audit rather than reported by construction, bound what has actually been built:

**The Wasm model is a declared subset, not Core 3.0.** `Wasm/Validate.lean` accepts
an `i32`-only executable subset: one memory, no imports, no tables, structured
control, `i32` loads/stores and locals. The release *profile* (`Wasm/Feature.lean`,
correctly transcribed) enables `i64`, fixed SIMD, reference types, GC, tail calls
and exception handling. The semantics for those enabled features are not modelled.

**The compiler cannot emit a correct release artifact.** Of the four arithmetic
modes SPEC §8.2 requires, only `modular` with a `u32` accumulator compiles.
`checked`, `strictFloat` and `exactDyadicRoundOnce` each need an accumulator wider
than `i32`, or a checked-overflow test `i32` cannot perform. `GNAF.compile` emits
`unreachable` for those nodes — it traps rather than computing something different,
which is the honest choice and is stated in the file's own doc comment — but the
consequence is that **SPEC §13 Phase B, "construct an input-total scalar GNAF GEMM
plan", is not achievable with the current compiler and target.**

Phase B is a prerequisite for everything downstream: no baseline means no attained
upper bound, hence no sublevel, hence no argmin. So O-6 blocks O-5 structurally,
independently of O-5's own obstruction. `BI-002` records this.

**The release seam is a parameter, not a construction.** `Release.Seam` bundles the
costed `Semantics`, the `CostedMachine`, and the resource `limit`, and **nothing in
the tree inhabits it**. Every release theorem — including
`Theorems.release_globalOptimal_of_nonempty` — is quantified over `(seam : Seam)`.

This is not vacuity in the strict sense: `Seam` *is* inhabitable, because
`Semantics` is three total functions and `CostedMachine` admits one that always
returns `.initializationFailure`. But such a seam leaves `SystemEvaluation`
uninhabited, so the nonemptiness antecedent is unsatisfiable and the implication
holds for want of any antecedent to test. The honest reading is therefore: **a true
implication whose antecedent may be unsatisfiable for every seam that can currently
be constructed.**

`GO-008` recorded this, and **`GO-008` is now closed.** `Release.seam` is a closed
term — `Release.semantics`, `Release.machine` (`Wasm.releaseCostedMachine`, the real
all-branch costed explorer of `Wasm/CostedExplore.lean`) and `Release.limit` — and
`Theorems.release_seam_nondegenerate` proves it is not the degenerate inhabitant:
no event is charged zero, no module validates for free, no resource coordinate is
budgeted at zero, and the machine both initializes and returns a **completed,
nonempty, canonically sorted** frontier at every raw invocation.
`Theorems.release_systemEvaluation_inhabited` then inhabits
`Universal.SystemEvaluation` at the constructed setting on the closed literal
`Release.witnessBytes`, which is also `Universal.ProfileValid`. So the release
implication is no longer one whose antecedent might be unsatisfiable at every
constructible seam.

Two disclosures travel with that closure, and neither is small.

* `Release.costEvent` is a **lower bound** on the SPEC §7.5 contribution law, not
  the law. `Universal.Semantics.costEvent` maps a *plain* `Wasm.Event`, which does
  not carry the transferred byte count, the `memory.grow` delta or the installed
  byte count — `Wasm.Event.step` alone is the erasure of fourteen rules.
  `Release.costEvent_le_eventContribution` proves the charge is componentwise `≤`
  `Wasm.eventContribution`; `costEvent_branch_charge` and `costEvent_trap_charge`
  prove it is *exact* on the branch and trap rules. Exactness elsewhere is O-6.
* The module the machine is discharged on is `Release.witnessModule` — the smallest
  module `GNAF.moduleOf` emits, `gemm` body `i32.const 0`, every branch terminating
  in at most three reduction steps — and **not** `Artifact.baselineModule`.
  Discharging the explorer on the compiled GEMM witness requires a termination proof
  for `GNAF.bodyCode`'s loops inside a `2 ^ 320` step budget, which is O-6.

`GO-006` is therefore **not** closed by `GO-008`: the literal that carries the
evaluation is not semantically correct (`witnessModule`'s `gemm` returns the
constant `0`), and the literal that is a GEMM implementation
(`Artifact.baselineBytes`) has neither a `SemanticCorrect` proof (`GNAF.compile_refines`,
`BI-002`) nor an evaluation.

`GNAF.compile_validates` is *not* vacuous — it holds for every `CheckedPlan` — and
the in-subset results are no longer unwitnessed either: `GNAF.gemmWitness` is a
concrete `1×1×1` modular-`u32` GEMM plan that classifies the raw ABI header,
dispatches on the layout class, runs the blocked traversal and the `i`/`j` loop
nest, accumulates the `k` reduction under the modular contract, sets the status
word, stores the product into the declared `C` region and publishes it. It is
proved in `Plan.inReleasedSubset`, type checked, compiled, validated and free of
`unreachable`; its accumulator is proved to hold exactly `(A·B) mod 2^32`; and
`GNAF.gemmWitness_writes_C` reads the `C` region back little-endian at the stored
width and gets `alpha · A · B + beta · C mod 2^32` for the `alpha` and `beta`
this descriptor declares (`BI-003` and `BI-005`, both discharged; `BI-004` is
the store constructor's own load-after-store law). The witness therefore *writes*
its result: the earlier limitation — that SPEC §11.1's plan language had no
constructor moving a register into memory — no longer holds, `Plan.storeReg` is
that constructor and `GNAF.storeReg_reads_back` is its load-after-store law.

Three things this still does not reach, recorded on `BI-003`: the descriptor
declares `alpha = 1` and `beta = 0` and the plan carries no scaling node, so it
is that instance of `C ← alpha · op(A) · op(B) + beta · C` and not the general
form; the statement is about `Plan.eval`, not about the emitted Wasm, which is
`compile_refines` and is omitted; and it is one `1×1×1` modular-`u32` shape, not
the scalar-kind × mode × transpose × layout-class family `BI-002` still records
as outstanding. Phase B is thus begun, not achieved.

## 2.2 External audit findings (accepted)

An external audit at commit `c44a06d` found the following. All were verified and
all are accepted; several were missed by this repository's own auditing.

**The inventory materially understated the proof surface.** `model/claims.json`
showed 10 outstanding rows. SPEC §15 requires 58 declarations, of which **22 are
discharged and 36 outstanding**. `Tools/required.py` now derives that inventory
from SPEC.md and queries the *compiled environment*; gate steps 4, 5, 8 and 9 test
declaration presence instead of reading a status field. A hand-maintained JSON
status is not evidence.

**CI had never established the exact-SHA build.** `verify.yml` ran
`sha256sum -c authority/uor-gnaf.sha256` from the repository root, but the
checksum file names the authority *without* a directory prefix, so the step failed
and the workflow stopped **before `lake build`**. Neither the build nor the axiom
closure had been established by CI at any commit. Fixed.

**The decider is noncomputable and its completeness is circular.**
`Release.evaluateClassically` assumes `Nonempty (SystemEvaluation …)` and extracts
a witness by `Classical.choice`; `Release.decider` wraps it. It decodes,
validates, enumerates and executes nothing, and the completeness theorem takes an
existing evaluation as an *argument* rather than proving one exists. SPEC §19
excludes noncomputable definitions from the product/proof path. `UV-003` is
**reopened**, and `Tools/releasepath.py` now fails the gate on it.

**The release profile is the i32 witness profile.** `Release.wasmProfile` is
`Wasm.unitWitnessProfile`, and its cost table is `canonicalCostTableUnits` rather
than SPEC §7.5's `Release.wasmCostTableBody`. The syntactic `∀ ByteArray` in
`GlobalOptimal` therefore does not yet range over the authority-required Core 3.0
competitor universe. Recorded as `WS-003`.

**The emitter cannot refine the plan.** `Compile.lean`'s `storeReg`/`loadReg`
clauses bind width as `_` and always emit full-word `storeW`/`loadW`, so the
emitted code does not respect the declared width `storeReg_reads_back` states.
The ABI installs raw bytes at the supplied pointer while compilation treats cell
`i` as an `i32` at absolute address `4*i` with no repacking. Abstract `loopReg`
snapshots its bound once; emitted code re-reads the extent each test. The shipped
baseline still compiles `gemmWitnessChecked`, not `gemmKernel`. So every
`gemmKernel_*` theorem is about `Plan.eval` only, and `compile_refines`,
`compile_cost_exact` and the termination bound are legitimately open. Recorded as
`BI-008`.

**Crux delta of the previous commit: zero.** `BI-007` was real progress on the
abstract plan language, but it closed no executable evaluator, no compiled
baseline, no universal coverage, no lower bound, no selected artifact and no final
theorem. That assessment is correct and is recorded here rather than argued with.

## 3. What is discharged

Kernel-checked under Lean 4.30.0; the live inventory is the first line of
`CONFORMANCE.md`. Zero `sorry`, zero `admit`, zero project-declared `axiom`, zero
`native_decide`, zero `partial`/`unsafe`/`noncomputable` on the proof path.

Layers built: `Foundation`, `Wasm`, `Gemm`, `Cost`, `GNAF`, `Universal`, `Atlas`,
`Artifact`, `Theorems`, `Conformance`.

Load-bearing results:

| Declaration | Content |
|---|---|
| `Universal.exists_globalOptimal_of_nonempty` | **the optimality half** — see §3.2 |
| `Universal.systemEvaluation_subsingleton` | makes `GlobalOptimal` satisfiable at all — §3.2 |
| `Cost.coordinate_le_score` | every one of the 36 charged coordinates is ≤ the canonical score |
| `Cost.moduleBytes_le_score` | module size is bounded by the score (size half of properness) |
| `Cost.CanonicalObjective.monotone` | componentwise ≤ implies score ≤ |
| `Wasm.mem_successors_iff_step` | the successor enumerator is exactly the `Step` relation |
| `Wasm.encode_decode_roundtrip` | binary round-trip for the modelled subset |
| `Wasm.costed_run_iff_plain_run` | cost instrumentation is transparent (SPEC §7.5) |
| `Atlas.semantic_closure_least` | least closure, and it *equals* the derivation closure |
| `Atlas.incremental_eq_full_rebuild` | incremental accumulation = full rebuild |
| `Atlas.universalCoverCompleteCheck_scope_blind` | **hardening** — see §3.1 |

`coordinate_le_score` is the hinge: it is what makes every score sublevel finite,
and it is what SPEC §10.3's coverage construction rests on.

### 3.2 The optimality half is discharged; the residue is exhibition

`Universal.exists_globalOptimal_of_nonempty` (SPEC §13 Phase D) proves that **some**
byte sequence satisfies `GlobalOptimal` in full — all three extensional conjuncts,
the evaluation existential with `Correct` and `Feasible`, the universal lower-bound
clause over *all* of `ByteArray`, and the canonical tie-break clause — given
(i) a nonempty admissible set and (ii) a decider that answers on admissible bytes.

The lower bound never required an analytic certificate. Because the artifact is
*defined* as the selection (SPEC §11.4), it falls out of the argmin construction:
scores are naturals, so a nonempty admissible set has a least score; every minimizer
satisfies `moduleBytes ≤ score` by `coordinate_le_score`, hence is bounded-length and
there are finitely many; a finite nonempty set under the total order
`CanonicalBytesLE` has a canonical-least element.

`systemEvaluation_subsingleton` is what makes this possible at all: `GlobalOptimal`'s
lower-bound clause is a *conjunction* over every inhabitant of the evaluation type,
so it is satisfiable only because that type has a unique inhabitant, pinned by
`decodeEq`, `initialEq`, `treeComplete`, `resourceExact` and `costExact`.

**This relocates O-5.** The residue is no longer "prove a lower bound"; it is two
exhibition problems:

- `GO-006` — nonemptiness: one concrete module with proofs of the three extensional
  predicates *and* an evaluation. This is SPEC §13 Phase B, blocked by O-6. Two of
  the four conjuncts now hold on a closed literal (`Release.witness_profileValid`
  and `Theorems.release_systemEvaluation_inhabited`); the missing one is
  `SemanticCorrect`, and it is missing on *every* literal this repository has.
- `GO-007` — identification: the committed literal equals the selected bytes.
  `exists_globalOptimal_of_nonempty` yields an *existential* produced by classical
  reasoning and names no literal. Naming one means running the selection over the
  admissible set — the step that is physically infeasible. Defining
  `Release.artifactBytes` noncomputably would make the theorem provable and leave no
  committed artifact, which SPEC §11.4 forbids.

Axiom closure by `#print axioms`: `propext`, `Quot.sound`, and `Classical.choice`
on some Prop-level results. All three are Lean core logical axioms, named
individually as SPEC §4 requires. SPEC §4's restriction on `Classical.choice` is
that it must not produce *executable witnesses*; the results using it are
Prop-level, so the restriction is not triggered. No `sorryAx`.

### 3.1 Hardening: the seal cannot stand in for coverage

An adversarial audit found the most attractive route to a false global-optimality
claim in this repository, and it was closed by **strengthening**, not by softening
any wording.

`Atlas.universalCoverCompleteCheck` — the seal's universal-cover checker — verifies
*bookkeeping consistency*: that the recorded cover matches the recorded candidate
facts. It never quantifies over the byte universe, and it is satisfiable by a cover
that records nothing at all. Left implicit, a reader could take a passing seal as
evidence of universal coverage.

`Atlas.universalCoverCompleteCheck_scope_blind` now makes this kernel-checked: the
check is a function of exactly the recorded `searchPartitions`, `candidateFacts`
identities, and `partitionCoverRoot`. Two states agreeing on those agree on the
check, whatever byte strings exist. **Therefore no proposition quantified over
`ByteArray` follows from the seal check.**

This is why `Atlas.seal_implies_universal_coverage` (a SPEC §15 required
declaration) is **absent** rather than derived: deriving it from this check would
be unsound. Claim `AT-002` records the absence; falsifier `M7` fails if anyone
later supplies it without a real proof, or removes the blindness lemma that makes
the absence principled. This is exactly the UOR-GNAF §18 non-claim that *index
presence or provenance alone proves eligibility, correctness, or optimality*.

### 3.2 Hardening: the forbidden-construct scan

The source scan initially matched any identifier containing `sorry`, so it
false-positived on `Conformance/AxiomAudit.lean`'s `sorryAx` constructor — which
exists precisely in order to *reject* that axiom. A gate that cries wolf gets
disabled, so the pattern was tightened to token boundaries. It is now clean on the
real tree and still catches a planted `theorem p : True := by sorry`.

The decisive audit remains `#print axioms` over the compiled environment
(`Tools/axioms.py`), per SPEC §19; the text scan is defence in depth.

Authority pins verified by content, not by trusting a string:

| Authority | Status |
|---|---|
| UOR-GNAF v1-draft.2 | SHA-256 `5c34…200a` — **matches the SPEC §4 pin exactly** |
| Lean toolchain | `v4.30.0`, commit `d024af09…c622` — **matches the SPEC §4 pin exactly** |
| WebAssembly Core wg-3.0 `9d360199…74aa` | **not vendored** — gate step 1 fails |

## 4. Why O-5 is outstanding

This section was adversarially reviewed. Two framings I initially advanced were
**refuted** and are recorded here as refuted, not quietly dropped.

### 4.1 What is *not* the obstruction

**Not tensor rank — and this is now proved, not merely argued.** It is tempting to
say closure requires the exact tensor rank of ⟨3,3,3⟩ (open; 19 ≤ R ≤ 23). It does
not. `LB-002` (`Universal.BilinearScheme.chargedOps_lower_bound`) formalizes the
accounting: for a bilinear scheme with `r` products, A-side supports `p_k ≥ 1`,
B-side supports `w_k ≥ 1`, and reconstruction supports summing to at least 27
(each of the nine outputs is a rank-three bilinear form, so ≥3 products feed it),
total charged operations satisfy

```
T ≥ r + 18
```

**uniformly in `r`**, and the bound is attained exactly by the naive algorithm
(`r = 27`, 18 additions, 45 charged operations — `naive_attains_bound`). Because
the bound holds for every `r`, no resolution of `R(⟨3,3,3⟩)` anywhere in [19,23]
changes the release theorem's truth value. The causal link the framing asserts is
false.

The mechanism is that the objective charges multiplications *and* additions at
weight one, so a low-rank scheme pays for every combining addition it introduces:
dropping to `r` multiplications buys at most `27 − r` operations, while Laderman's
23-multiplication scheme spends 98 additions for 121 charged operations against
naive's 45. The saving available from low rank is a few operations; the price is
dozens. **Refuted, high confidence.**

**Not asymptotic circuit lower bounds.** Weight-one module bytes creates a
minimal-*total-cost* obligation, not a minimal-program-size one, so the open
problem of superlinear circuit lower bounds for explicit functions is the wrong
authority to invoke. **Refuted, medium confidence.**

**Not the finiteness claim.** SPEC §10.3's literal words — the sublevel is "finite
and exactly enumerable" — are provable with no computation at all: a subset of a
finite type is finite, every quantifier ranges over a finite carrier, and filtering
is the exact enumeration. §10.3 as written is closable today.

### 4.2 What *is* the obstruction

The binding obligation is the UOR-GNAF §19.3 disjunct: **a universal lower bound,
attained.** Concretely, closure requires both of:

1. a proved floor `F` on the *total charged cost* of every correct, feasible,
   same-profile WebAssembly module over the complete raw-invocation domain; and
2. shipped bytes whose exact score **equals** `F`.

Neither half has a known route.

**The exhaustive route is physically excluded.** The alternative to an analytic
floor is enumeration. The candidate space is `256^u` where `u` is the module-byte
bound. `256^u` exceeds the holographic bit bound of the observable universe
(~10^123) at `u > 51` bytes. A conforming GEMM module is orders of magnitude
larger. Enumeration is not slow; it is excluded by physics, and no increase in
compute, parallelism, or patience changes this.

**The analytic route has no known technique.** An adversary lower bound would have
to be tight to within a few dozen units across the entire summed domain, over a
space expressive enough to include memory, branches, SIMD, and GC. Worse, the
optimum is plausibly *not* the naive kernel — data-dependent shortcuts (skipping
multiplications by zero, exploiting structure shared across the enumerated domain)
mean there is no obvious candidate whose cost one could match with an adversary
argument. Existing exhaustive-optimality precedents are far smaller: Knuth's
results cover 5-variable Boolean functions; BB(5) took decades with heavy
machine-specific normalization. Neither scales to multi-hundred-byte Wasm modules
over a `~10^(1.29×10^10)`-element input domain.

**It is decidable, and that matters.** Item 7 is not undecidable, and no appeal to
undecidability is made here. Because `staticModuleBytes` carries weight one, any
competitor scoring below `S` has module length below `S`; because
`sumWasmRuleSteps` carries weight one, execution can be capped at `S` steps and
anything exceeding it rejected — so the halting problem never arises. Item 7 is
therefore a bounded, mechanically computable statement: enumerate the byte strings
shorter than `S`, run each for at most `S` steps on each domain input, compare.
Its truth value is mathematically determinate.

**The obstruction is exactly infeasibility plus the absence of a certificate.**
`256^u` passes the observable universe's holographic bit bound at `u > 51` bytes,
so the decision procedure exists and can never be run. Closure therefore needs an
analytic certificate instead, and none is known — the bound would have to be tight
to within a few dozen units across the whole summed domain, over a space including
memory, branches, SIMD, and GC.

**A further complication, recorded because it is easy to miss.** Under
charge-everything accounting over a *finite* input domain, the optimum may not be
an arithmetic algorithm at all. Dispatch is charged, so shape-specialized kernels
are penalized; and for narrow scalar kinds a lookup table can undercut arithmetic
on `staticDataBytes` against `sumScalarOps`. The argmin is a discrete trade-off
across 36 coordinates, not an algebraic invariant. This is a second, independent
reason tensor rank is the wrong place to look — and it means there is no obvious
candidate whose cost an adversary argument could be matched against.

**Residual uncertainty, stated honestly.** The reviewer who refuted the
circuit-complexity framing did so at *medium* confidence and conceded that the
alternative routes relocate the difficulty rather than eliminating it. The `LB-002`
floors are proved for *bilinear* schemes; a non-bilinear straight-line program over
`ZMod (2^w)` is outside that accounting, though it too pays for every operation it
performs. The correct characterization is therefore: O-5 is **not proved
impossible**, and is decidable in principle — it has **no known route**, and the
only mechanical route is physically excluded. Under UOR-GNAF §13.1 a generic
hardness result must not be used to dismiss a bounded profile; that is why this is
filed as an *outstanding obligation with no known discharge*, never as an
impossibility theorem.

## 5. What would discharge O-5

Recorded so the ledger is falsifiable rather than rhetorical:

- an analytic lower-bound theorem on total charged cost over the full domain,
  tight enough that a constructible module attains it; **or**
- a proof-complete symbolic partition (SPEC §10.4 `dominated`) covering the sublevel
  without per-module enumeration; **or**
- a `no-improver` coverage argument in the UOR-GNAF §19.3 sense.

Any of these, plus O-6, closes the claim. Absent all of them the answer stays
`WorkloadIncomplete`.

## 6. What must not happen

Per UOR-GNAF §18, these remain explicit **non-claims**, and none may be presented
as discharging WGG-GO-1:

- that incomplete saturation, search, autotuning, benchmarking, or learned
  selection proves an optimum;
- that an internal plan optimum proves the containing complete system globally
  optimal;
- that a restricted-universe optimum applies to an omitted parent universe;
- that **correctness may be weakened because complete optimization is expensive**.

Specifically forbidden here: narrowing the competitor universe to GNAF plans and
renaming the result; dropping a cost coordinate to make a floor reachable;
restricting the input domain to make the sum tractable; or asserting
`released_wasm_gemm_gnaf_global_optimal` with any hypothesis attached. Each would
produce a different, weaker proposition wearing the release theorem's name.

## 7. Reproducing this outcome

```
just claims     # registry: WGG-GO-1 present, status incomplete
just axioms     # axiom closure of every proved declaration
just vv         # full gate; FAILS at step 9 by design
```

`just vv` failing is the correct observable behavior of a conforming repository in
this state. A green `just vv` here would mean the gate had been weakened.
