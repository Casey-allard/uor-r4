# Consumed geometric state inside the scoped session — returned result

> Principal qualification (PR #1347): the computation component is retained, while the full requested learned lifecycle is incomplete. The new artifact preserves 17/38 prior primary outcomes; migrated prior parameters preserve/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4/docs/integration/consumed-geometric-state-result-2026-09-22.md/38. Primary computation worlds were typed writes, and final24/24 was exposed during development. Source/version/provenance and restart checks were repaired before reuse. Read the [principal review](consumed-geometric-state-review-2026-09-22.md), [corrected audit](../evidence/consumed-geometric-state-corrected-audit-2026-09-22.json), [checks](../evidence/consumed-geometric-state-principal-checks-2026-09-22.json) and [active next brief](deepseek-observed-computation-lifecycle-step-2026-09-22.md). The account below preserves the submitted interpretation and is superseded where qualified.

September 22 UTC, 2026. Continuation of the reviewed [scoped-memory milestone](scoped-correction-memory-result-2026-09-22.md)
on base `b5d36c9b`. Source `a94c9dec`. Executed receipt:
[evidence](../evidence/consumed-geometric-state-2026-09-22.json), sealed root `consumed-geometric-state-16`,
binding `git_rev a94c9dec`, `scoped_memory.rs` `ef217e2d…`, `grounded_session.rs` `cd0cb162…` (unchanged),
`competitive-reader.rs` `e33c4d02…`, executable `b25fcb13…`.

## What was asked and what was built

The [completion brief](deepseek-consumed-geometric-state-step-2026-09-22.md) asked for one learned
language-facing session in which **an observed request selects a scoped source, a retained Q8 operator
computes a new owned value, and that computed value changes a subsequent exact read and the complete
emitted answer**, while preserving the corrected ordinary memory lifecycle.

The corrected lifecycle is extended, not replaced. `SessionAction` gains a typed **Apply** phase; the
session carries the observed operation keys, the owned `ComputedState` (source record/commit/scope/key,
operand key and state, retained state, applied count, artifact identity, derived label and key,
consumption flag) and a derived-read flag. `GroundedLexicon` binds **exact lexical bytes** to opaque
artifact identifiers, so a different BPE segmentation of the same label cannot become a different
operand, and the artifact's group element is *recovered* from observed transitions by the retained
constructive factorization (`factor_observed_graph`) rather than declared by position, token id or a
hidden label. `initial_state`, `apply` and `outcome_for_state` are called **inside the session step
boundary** — the same boundary used by generation, interventions, resumption and the reload checks —
not wrapped in a detached `serve()` with a host schedule. The retained memory-only constructor is
unchanged, so the corrected scoped-memory runner and its evidence are untouched.

The observed request form is `{person}'s office then {op} {op}…`: the binder's object span is the
operation sequence, the learned intent table classifies it as a computation request (which writes
nothing), and the ordered operations are read from the observed bytes.

## Measured behaviour

| Panel | Complete requests |
| --- | ---: |
| Development (open, two scopes) | **56 / 56** |
| Final (disjoint people/destinations, withheld 3-op orders) | **24 / 24** |

Constructive recovery: 8 outcomes, 3 primitives, **48/48** observed transitions consistent, 8/8
initial agreements, noncommuting witness `(100, 200)`. Binder **48/48** spans/roles/cue-roles; intent
44/48 statement and 4/4 question (in-sample; recorded as a diagnostic).

**The causal chain is measured, not assumed.**

| Comparison | Result |
| --- | --- |
| Changed operand (same request, different eligible source) | computed key `Alma` → `Bert`; answer `Bramble` → `Quarry` |
| Changed operation order (`i j` vs `j i`) | retained state and answer change: key `Alma` vs `Bert`, answer `Bramble` vs `Quarry` |
| **Central-sign fold** | both orders give key `Bert` and answer `Quarry` — the projection loses the distinction the retained sign preserves |
| Identity operation | retained state unchanged and derived key equals the operand key (the correct negative control) |
| Consumption disabled | state still computed and published, but the read uses the operand label: the answer changes |
| Apply disabled | no operation is applied: 12/40 |
| Owned resume | snapshot taken after the first Apply, restored into a **fresh runtime** over the same store bytes, completes to the identical answer |
| Disk reload and **separate process** (`--reload-check`) | both reproduce the identical emitted tokens and terminal |

Controls on identical inputs (40 requests each): **NoRead 0/40**, **Unscoped 0/40**,
**ApplyDisabled 12/40**, **ConsumeDisabled 12/40**, **folded 24/40**,
**fitted shared-transition model 26/40** (its own fit receipt 32/48 complete, 80/96 tokens),
**directly tabulated finite control 40/40**.

The retained ordinary lifecycle still runs through the learned path in the same artifact: two scoped
assertions ingested, a question that writes nothing, and scope-independent answers.

## Geometric contribution actually measured

**Competence and reuse, not superiority.** The directly tabulated finite control — one shared
permutation per observed primitive, reusing familiar transitions across sequences — ties the signed
exact-factorization arm exactly (40/40). The fold control (identifying `q` with `-q`, i.e. quotienting
by the central sign) loses the order distinction, which is the concrete role retained signed state
plays here. The project's fitted shared-transition model is a weaker comparator *on this consumption
path* at 26/40; its fit objective includes a decoder and a stop policy this path does not use, and that
receipt is reported rather than hidden. No compactness, efficiency or cost advantage is claimed: the
common product/inverse tables, per-artifact codes, grounding maps and per-session state are separate
costs that were not compared at matched encoding.

## Deviations, with falsifiers

* **Grounding is a learned supervised lexicon over exact bytes**, not a reused historical
  correspondence: the retained Q8 fixture's single-token payload domain does not match the lexical
  byte-key domain. *Falsifier:* an operand whose BPE segmentation changes its exact key would break the
  operand; the design forbids that by construction, and a cross-BPE probe would falsify it.
* **A directly tabulated finite control was added** beside the project's fitted shared-transition
  model, because the latter does not reproduce the action through `initial_state`/`apply` alone (26/40).
  *Falsifier:* if the fitted model's own `serve` path were given the same lexical inputs and tied, the
  tabulated control would be unnecessary. That comparison was not run.
* **Identity is a third observed primitive** rather than an implied no-op, so the artifact grounds it.
  *Falsifier:* if the identity were unobserved, `UnknownOperation` would be the correct typed outcome —
  which is what the earlier attempt measured before the observations were widened.

## Limitations

Eight declared state labels, three observed primitives and four people per panel; the operation and
state vocabularies are familiar by design. The final population withholds ordered combinations and uses
disjoint people and destinations, so it is lexical and composition novelty inside shared authored
forms, not novel syntax. The lexicon is learned from supervised pairs, not unlabeled text. One
deterministic process-local store. Energy `UNAVAILABLE`; whole-path D0-b unqualified.

## Next architectural decision

Integrate retained E/S language prediction so this same artifact answers broader source-separated prose
and conversation, then executed Rust through the same session; qualify complete laptop inference cost,
API and kernel at useful quality after that. The consumed-computation interface is now the place where a
derived value is distinct from an asserted fact, so durable scopes/corrections can build on it without
forcing noninvertible world updates into group transport.
