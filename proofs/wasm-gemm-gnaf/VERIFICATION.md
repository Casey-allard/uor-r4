# Verification

What each gate discharges, and the defect planted to prove it can fail.

Run everything with `just vv`. It is **expected to fail** at step 9 while `WGG-GO-1`
is outstanding; see [CERTIFICATION.md](CERTIFICATION.md). A green gate in this state
would mean the gate had been weakened.

## Gate map

| Gate | Discharges | Falsifier |
| --- | --- | --- |
| `just build` | the Lean library compiles under the pinned toolchain | — |
| `just axioms` | every `formalProof` claim's transitive axiom closure is inside the SPEC §4 trust base | `M5` |
| `just claims` | registry is nonempty, ids unique, no orphan dependencies | `M2`, `M3` |
| `just mutation` | each decisive checker rejects a planted fault | self-testing |
| `just docs` | `CONFORMANCE.md` is generated, deterministic, byte-clean | `reproducible.yml` |
| `just vv` | all 13 conditions of SPEC §20.2 | `M6` |

## Why the source scan is not the decisive audit

SPEC §19 is explicit: source scanning is defence in depth. The decisive audit inspects
the **compiled environment** and the transitive dependencies of every public theorem,
via `#print axioms` in `Tools/axioms.py`. A `sorry` reaches the environment as
`sorryAx`, so it is caught there even if the text scan is evaded.

Current closure over every `formalProof` claim: `propext`, `Quot.sound`. Both are Lean
core logical axioms and are named individually as SPEC §4 requires. No `sorryAx`, no
project-declared axiom, no `Classical.choice`.

## Planted falsifiers

`Tools/mutation.py`, registered in `model/falsifiers.json`. Each applies its mutation
to a **copy**, never to the repository.

| ID | Family | Plants | Rejected by |
| --- | --- | --- | --- |
| M1 | CM | mutated authority bytes | content digest recomputation |
| M2 | CM | duplicate claim id | registry uniqueness |
| M3 | CM | orphan claim dependency | registry dependency check |
| M4 | GO | `formalProof` level with no Lean declaration | claim-level rule (SPEC §17.1) |
| M5 | LF | `sorry` on the proof path | forbidden-construct scan |
| M6 | GO | green gate while `GO-001` is outstanding | release gate step 9 |
| M7 | AT | citing the seal's cover check as universal coverage | `AT-001` blindness lemma + `AT-002` absence |
| M8 | CM | a stale `.olean` masking a non-elaborating root | direct `lean WasmGemmGnaf.lean` |

M7 and M8 were added after real defects, not hypothetically. M7 answers the audit
finding that `universalCoverCompleteCheck` verifies bookkeeping only and is
satisfiable by an empty cover. M8 answers a worse one: `lakefile.lean` used
`globs := #[.submodules ...]`, which never builds the root module, so `lake build`
reported green for an entire cycle while two modules declared clashing `Fault`
types and the root did not elaborate at all. **`lake build` success is not evidence
that the code elaborates**; the gate now checks the root directly.

M4 and M6 are the ones that matter. SPEC §18 warns that a mutation suite which merely
expects runtime output differences does not test claim integrity; M4 and M6 attack the
claim machinery itself — they check that the repository cannot be made to *say* it
proved global optimality without a Lean declaration behind it.

## What is not yet falsifiable

Universal-coverage integrity (partition gaps, overlaps, forged lower bounds, stale
seals) cannot be falsification-tested until the checkers exist. Those falsifiers are
registered as outstanding rather than passing vacuously — a suite that passes because
its target does not exist is worse than no suite.
