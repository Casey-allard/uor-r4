# Conformance

**Inventory:** 97 Lean modules, 55,504 lines, 3,146 proved theorems.
Generated live; prose documents cite this table rather than repeating counts.

Generated from `model/claims.json` by `just docs`. Do not edit by hand.

Claim levels are load-bearing (SPEC 17.1). Only `formalProof` supports the
words "proved", "theorem", or "globally optimal".

## Claims

| ID | Level | Status | Statement | Lean declaration |
| --- | --- | --- | --- | --- |
| `CO-001` | formalProof | discharged | Every one of the 36 charged artifact coordinates is bounded by the canonical score. | `WasmGemmGnaf.Cost.coordinate_le_score` |
| `CO-002` | formalProof | discharged | Module bytes are bounded by the canonical score (size half of properness). | `WasmGemmGnaf.Cost.moduleBytes_le_score` |
| `CO-003` | formalProof | discharged | The canonical objective is monotone under componentwise cost order. | `WasmGemmGnaf.Cost.CanonicalObjective.monotone` |
| `CO-004` | formalProof | discharged | Sequential cost composition is associative; sums on cumulative coordinates, maxima on the three peaks. | `WasmGemmGnaf.Cost.sequentialCompose_assoc` |
| `CO-005` | formalProof | discharged | The 36-element artifact coordinate index is completely covered by its enumeration. | `WasmGemmGnaf.Cost.ArtifactCoordinate.mem_all` |
| `LF-001` | authority | verified | Lean toolchain is leanprover/lean4:v4.30.0 at commit d024af099ca4bf2c86f649261ebf59565dc8c622. | `—` |
| `LF-002` | authority | verified | UOR-GNAF authority is UOR-GNAF-v1-draft.2.md with SHA-256 5c342373b2ff809bfd607c413cafd0582d32bb097544c6597... | `—` |
| `WS-001` | open | outstanding | Mechanized WebAssembly Core 3.0 semantics (GC, exception handling, fixed SIMD, tail calls) with decode_soun... | `—` |
| `UV-001` | open | outstanding | Universal sublevel coverage over all same-profile byte modules, with no coverage hypothesis. | `—` |
| `LB-001` | open | outstanding | A universal lower bound F on total charged cost over every correct feasible same-profile module, attained b... | `—` |
| `GO-001` | open | outstanding | released_wasm_gemm_gnaf_global_optimal : GlobalOptimal Release.wasmProfile Release.gemmProblem Release.cost... | `—` |
| `AT-001` | formalProof | discharged | The Atlas seal's universal-cover check is blind to the byte universe: it is determined by the recorded sear... | `WasmGemmGnaf.Atlas.universalCoverCompleteCheck_scope_blind` |
| `AT-002` | open | outstanding | Atlas.seal_implies_universal_coverage: a sealed state implies universal sublevel coverage over all profile-... | `—` |
| `LB-002` | formalProof | discharged | Charged-operation lower bound for bilinear 3x3 matrix-multiplication schemes: T >= r + 18, uniformly in the... | `WasmGemmGnaf.Universal.BilinearScheme.chargedOps_lower_bound` |
| `GO-002` | formalProof | discharged | GlobalOptimal is stated at full strength: its competitor quantifier ranges over all of ByteArray, and it eq... | `WasmGemmGnaf.Universal.globalOptimal_iff_over_all` |
| `GO-003` | formalProof | discharged | The competitor universe of GlobalOptimal is inhabited and both universal clauses fire: the released bytes t... | `WasmGemmGnaf.Universal.globalOptimal_lower_bound_fires` |
| `UV-002` | formalProof | discharged | SPEC 10.3(1): byte strings within a module-size bound are finitely enumerated, and every competitor whose s... | `WasmGemmGnaf.Universal.sublevel_bytes_enumerated` |
| `CM-002` | formalProof | discharged | Decode faults and instantiation faults are disjoint: no machine fault is both. | `WasmGemmGnaf.Wasm.Fault.decoding_ne_instantiation` |
| `LB-003` | formalProof | discharged | Full-domain aggregation lower bound: if every raw invocation costs at least k on a charged coordinate, the ... | `WasmGemmGnaf.Universal.sum_ge_card_mul` |
| `LB-004` | formalProof | discharged | The attainment gap: a lower bound L strictly below the released score S does NOT establish minimality; and ... | `WasmGemmGnaf.Universal.lower_bound_below_released_is_not_optimality` |
| `BI-001` | formalProof | discharged | GNAF.compile emits a module that passes the released profile validator for every checked plan. | `WasmGemmGnaf.GNAF.compile_validates` |
| `BI-002` | open | outstanding | The GNAF compiler can emit a module implementing the FULL released GEMM problem (all four arithmetic modes ... | `—` |
| `BI-003` | formalProof | discharged | Plan.inReleasedSubset is inhabited by an actual GEMM plan: gemmWitness classifies the raw ABI header, dispa... | `WasmGemmGnaf.GNAF.gemmWitness_inReleasedSubset` |
| `AT-003` | formalProof | discharged | The Atlas semantic closure is the LEAST closed set containing its input, equals the derivation closure, and... | `WasmGemmGnaf.Atlas.semantic_closure_least` |
| `AT-004` | formalProof | discharged | Incremental Atlas accumulation equals full rebuild; empty delta is a fixed point; compatible deltas commute. | `WasmGemmGnaf.Atlas.incremental_eq_full_rebuild` |
| `CM-003` | buildEvidence | verified | MANIFEST.json realises SPEC 4's three ordered identity stages, acyclically: no stage contains its own ident... | `—` |
| `GO-004` | formalProof | discharged | GlobalOptimal is ATTAINABLE: if the admissible set is nonempty and the decider answers on admissible bytes,... | `WasmGemmGnaf.Universal.exists_globalOptimal_of_nonempty` |
| `GO-005` | formalProof | discharged | SystemEvaluation is a subsingleton: every field is pinned by decodeEq, initialEq, treeComplete, resourceExa... | `WasmGemmGnaf.Universal.systemEvaluation_subsingleton` |
| `GO-006` | open | outstanding | NONEMPTINESS: one concrete byte sequence with proofs of ProfileValid, SemanticCorrect and SemanticWithinRes... | `—` |
| `GO-007` | open | outstanding | EXHIBITION: the committed release literal equals the byte sequence selected by the argmin. | `—` |
| `GM-002` | formalProof | discharged | SPEC 8.4 reference obligations: reference_total, valid_reference_nonempty, deterministic_mode_unique, refer... | `WasmGemmGnaf.Gemm.reference_total` |
| `BI-004` | formalProof | discharged | The GNAF plan language can express writing a computed value into memory. | `WasmGemmGnaf.GNAF.storeReg_reads_back` |
| `BI-005` | formalProof | discharged | The anti-vacuity GEMM witness WRITES its result: after evaluation the declared C region of gemmWitness hold... | `WasmGemmGnaf.GNAF.gemmWitness_writes_C` |
| `UV-003` | open | outstanding | The concrete release Decider satisfies DeciderAnswersAdmissible: it returns a completed evaluation on every... | `—` |
| `GO-008` | formalProof | discharged | AMENDED: Release.Seam is inhabited by a NON-DEGENERATE seam -- its CostedMachine is the actual all-branch c... | `WasmGemmGnaf.Release.systemEvaluation_inhabited` |
| `CO-006` | formalProof | discharged | The release cost table contains a row for every pinned Core rule identifier, with exact scalarOps, vectorLa... | `WasmGemmGnaf.Wasm.canonicalCostTable_exact_cover` |
| `WS-002` | formalProof | discharged | The costed all-branch explorer completes whenever the plain explorer does with a nonempty observation list,... | `WasmGemmGnaf.Wasm.exploreAllCosted_complete_of_nonempty` |
| `BI-006` | open | outstanding | Every branch of the compiled GEMM baseline terminates within the released 2^320 step bound, so exploreAllCo... | `—` |
| `GO-009` | open | outstanding | A module that is BOTH evaluable (SystemEvaluation inhabited) AND SemanticCorrect for the release GEMM problem. | `—` |
| `BI-007` | formalProof | discharged | The GNAF plan language can express input-dependent computation: reading a descriptor field from memory into... | `WasmGemmGnaf.GNAF.gemmKernel_writes_C` |
| `BI-008` | formalProof | discharged | The GNAF plan language expresses an INPUT-DEPENDENT GEMM: one CheckedPlan, GNAF.gemmKernel, loads m, n and ... | `WasmGemmGnaf.GNAF.gemmKernel_writes_C` |
| `CM-004` | buildEvidence | verified | CI verifies the exact-SHA build: the authority digest step runs from the directory the checksum file names. | `—` |
| `CM-005` | buildEvidence | verified | The SPEC 15 inventory is derived from SPEC.md and checked against the compiled environment, not from a hand... | `—` |
| `WS-003` | open | outstanding | Release.wasmProfile is backed by the completed pinned Core 3.0 semantics rather than the i32 witness profil... | `—` |
| `UV-004` | formalProof | discharged | Constructive duplicate-free enumeration of every lawful raw invocation, with a global choice-free Fintype i... | `WasmGemmGnaf.Gemm.raw_input_finite` |

## Axiom closure

Every `formalProof` claim's transitive axioms, from `#print axioms`:

- `CO-001` — `propext`, `Quot.sound`
- `CO-002` — `propext`, `Quot.sound`
- `CO-003` — none
- `CO-004` — `propext`
- `CO-005` — `propext`
- `AT-001` — `propext`, `Classical.choice`, `Quot.sound`
- `LB-002` — `propext`, `Quot.sound`
- `GO-002` — `propext`, `Classical.choice`, `Quot.sound`
- `GO-003` — `propext`, `Classical.choice`, `Quot.sound`
- `UV-002` — `propext`, `Classical.choice`, `Quot.sound`
- `CM-002` — none
- `LB-003` — `propext`, `Quot.sound`
- `LB-004` — none
- `BI-001` — `propext`, `Quot.sound`
- `BI-003` — none
- `AT-003` — `propext`, `Quot.sound`
- `AT-004` — `propext`, `Classical.choice`, `Quot.sound`
- `GO-004` — `propext`, `Classical.choice`, `Quot.sound`
- `GO-005` — `propext`, `Classical.choice`, `Quot.sound`
- `GM-002` — `propext`, `Classical.choice`, `Quot.sound`
- `BI-004` — `propext`, `Quot.sound`
- `BI-005` — `propext`, `Quot.sound`
- `GO-008` — `propext`, `Classical.choice`, `Quot.sound`
- `CO-006` — none
- `WS-002` — `propext`, `Classical.choice`, `Quot.sound`
- `BI-007` — `propext`, `Quot.sound`
- `BI-008` — `propext`, `Quot.sound`
- `UV-004` — `propext`, `Quot.sound`

Permitted: `propext`, `Quot.sound`, `Classical.choice` (Lean core logical
axioms, SPEC 4). Any `sorryAx` or project-declared axiom fails the gate.

## Refuted framings

Recorded so they are not silently re-asserted:

- **REFUTED** (high) — Closure requires the exact tensor rank of <3,3,3> (open, 19<=R<=23).
  - Multiplications and additions are both charged at weight one, so rank is not the binding quantity. Laderman's 23-mult 3x3 scheme costs 23+98=121 charged ops vs naive 45. Truth value invariant across [19,23].
- **REFUTED** (medium) — Closure requires exact minimal-program-size lower bounds, an open circuit-complexity problem.
  - Weight-one module bytes creates a minimal-total-cost obligation, not a minimal-program-size one; asymptotic circuit lower bounds are the wrong authority.
- **REFUTED** (high) — SPEC 10.3's 'finite and exactly enumerable' is false at release scale.
  - As literally written it is provable with no computation: a subset of a finite type is finite. The magnitudes are irrelevant to a finiteness claim.

## Outstanding obligations

12 outstanding. Terminal answer for `GO-001`: `WorkloadIncomplete`
(UOR-GNAF v1-draft.2 section 10.9). See `CERTIFICATION.md`.

- `WS-001` (O-6) — Mechanized WebAssembly Core 3.0 semantics (GC, exception handling, fixed SIMD, tail calls) with deco
- `UV-001` (O-5) — Universal sublevel coverage over all same-profile byte modules, with no coverage hypothesis.
- `LB-001` (O-5) — A universal lower bound F on total charged cost over every correct feasible same-profile module, att
- `GO-001` (—) — released_wasm_gemm_gnaf_global_optimal : GlobalOptimal Release.wasmProfile Release.gemmProblem Relea
- `AT-002` (O-5) — Atlas.seal_implies_universal_coverage: a sealed state implies universal sublevel coverage over all p
- `BI-002` (O-6) — The GNAF compiler can emit a module implementing the FULL released GEMM problem (all four arithmetic
- `GO-006` (O-6) — NONEMPTINESS: one concrete byte sequence with proofs of ProfileValid, SemanticCorrect and SemanticWi
- `GO-007` (O-5) — EXHIBITION: the committed release literal equals the byte sequence selected by the argmin.
- `UV-003` (O-6) — The concrete release Decider satisfies DeciderAnswersAdmissible: it returns a completed evaluation o
- `BI-006` (O-6) — Every branch of the compiled GEMM baseline terminates within the released 2^320 step bound, so explo
- `GO-009` (O-6) — A module that is BOTH evaluable (SystemEvaluation inhabited) AND SemanticCorrect for the release GEM
- `WS-003` (O-6) — Release.wasmProfile is backed by the completed pinned Core 3.0 semantics rather than the i32 witness
