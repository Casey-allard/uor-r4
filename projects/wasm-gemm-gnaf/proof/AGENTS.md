# AGENTS

The standing brief for anyone — human or otherwise — changing this repository.

Read `SPEC.md` first: it is the normative contract, and it wins over this file.
Read `CERTIFICATION.md` second: it is the repository's exact terminal answer for the
release claim, and it is what most changes will be trying to move.

## The one rule that matters

**If you cannot prove it, omit it.**

Not `sorry`. Not `axiom`. Not a `Prop` field on a structure that quietly asserts the
conclusion. Not a redefined `GlobalOptimal`. Not a narrowed quantifier. Omission is
the *conforming* representation of an undischarged obligation — the claim registry
carries it at level `open`, and the gate fails on it honestly.

This is not conservatism. A repository whose decisive theorem rests on a hidden
assumption is worth less than one that admits the gap, because the first one lies to
everybody downstream and the second one tells you exactly what to go work on.

## Prohibited on the proof path (SPEC §19)

`sorry`, `admit`, project-declared `axiom`, `native_decide`, `unsafe`, `partial`,
`noncomputable`, unchecked FFI, a cryptographic collision assumption, an assumed
all-program coverage or lower-bound proposition, and any theorem conclusion stored as
an unproved structure field.

Every file starts with `set_option autoImplicit false`, placed **after** its imports.

The source scan is defence in depth. The decisive audit is `Tools/axioms.py`, which
reads the compiled environment via `#print axioms`. Permitted closure: `propext`,
`Quot.sound`, `Classical.choice` — named individually, never as a category.

## Claim levels are load-bearing (SPEC §17.1)

| Level | Means |
| --- | --- |
| `authority` | reproduced from a pinned authority. Not established here. |
| `buildEvidence` | constructed and checked here. Evidence, not proof. |
| `formalProof` | a Lean declaration with a kernel-checked proof. |
| `measurement` | measured and reported. Never asserted. |
| `open` | an obligation with no discharge. Never asserted. |

Only `formalProof` supports the words "proved", "theorem", or "globally optimal".
No test, benchmark, cross-engine run, or candidate search promotes a claim across
that line — SPEC §17.1 and §18 both say so, and `M4` in the mutation suite enforces it.

## Changing a claim

In this order:

1. A row in `model/claims.json` with its level and its `specSection`.
2. A falsifier in `model/falsifiers.json`, wired into `Tools/mutation.py`.
3. The Lean declaration, proved.
4. `just docs` to regenerate `CONFORMANCE.md` (it is generated; editing it by hand is
   a mistake the gate catches).
5. `just vv`.

Steps 1–2 before step 3 is the discipline. A claim that arrives with its proof but no
falsifier cannot be shown to be capable of failing.

## Prose discipline (SPEC §21)

Never use "universal", "global", "arbitrary", "complete", or "optimal" without linking
the exact profile/problem/objective/universe proposition. Every use of "globally
optimal" refers to the abstract UOR-Wasm cost objective of SPEC §9 — **not** physical
latency on any engine. That distinction must appear beside every such use.

While `WGG-GO-1` is outstanding, `README.md` carries the SPEC §21 pre-closure wording
verbatim. Do not edit it to sound more finished.

## The gate is supposed to be red

`just vv` fails at step 9. That is the conforming outcome under UOR-GNAF §13.3:
*a machine may return an honest weaker claim; it must not return an unproved global
label.* Do not add `continue-on-error`, do not lower a condition, do not mark an
outstanding claim `discharged` to get a green run. Making the gate pass by changing
the gate is the one failure mode this repository is built to prevent.

## What would actually close it

Recorded in `CERTIFICATION.md` §5, so the ledger stays falsifiable: an attained
universal lower bound on total charged cost (UOR-GNAF §19.3), or a proof-complete
symbolic partition covering the sublevel, or a `no-improver` coverage argument —
plus the mechanized Core 3.0 semantics. Anything less is an intermediate research
state, not proven global optimality (SPEC §23).
