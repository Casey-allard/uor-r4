# #922 — feat(#843): typed transition observations and deterministic induction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T15:01:49Z
- **Updated:** 2026-08-22T17:39:30Z
- **Closed:** 2026-08-22T15:25:16Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/922
- **Merged:** 2026-08-22T15:25:16Z
- **Merge commit:** 85f3d03a7e38eb5936bcd92d60f8bd5681986f81
- **Base / Head:** main / UOR-Foundation:issue-843-observation-induction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 3 of 6 for issue #843 (item B of S4 tracker #826). Builds the offline compiler half: the typed transition observation record and the deterministic inducer that turns observations into the bounded operator rule set the packed sections will carry and the deployed planner will execute.

Thirteen guarantees from `docs/bounded_semantic_transitions_spec_843.md` §3 are asserted and pass, in about 8 s. No planning capability is claimed — that is the measured question of increments 5 and 6.

## Scope and non-goals

**In scope**

- `crates/uor-r4-graph-format/src/plan.rs` (**new**) — the deployed-form primitives and the §4.2 frozen capacities. `core`-only and allocation-free: fixed arrays, no owned collections, no strings, no floating point. `SlotVec` (bounded typed valuation), `EffectDelta` (applied by *saturating* integer add, so a slot at its bound stays there rather than wrapping into a valid-looking state), `PreconditionMask` + `CompareOp` (read mask plus per-slot comparison). Every operation is P-4 legal: mask `AND`, integer compare, saturating add/sub. No multiply, no divide, no float.
- `crates/uor-r4-graph-compiler/src/semantic_transitions.rs` (**new**) — the observation record with all thirteen fields of §3.1 including negative and conflicting examples; `observe()`, which attempts every operator from every state of a bounded canonical breadth-first prefix and records the outcome whether or not it applied; and the frozen induction pipeline: content-addressed partitioning, an ordered reduction, deduplication on the canonical rule key, declared-conflict detection, ordinal band assignment, split enforcement and a leakage scan.

**Key design choice, and why.** The induced object is a *schematic* operator rule set — `(precondition, effect, support, band)` — not a grounded `(state, action) → state` table. Grounding happens at plan time by saturating integer addition on the packed slot vector. A grounded table scales with the reachable state space; a rule set scales with the operator vocabulary, and that is what makes the frozen capacities and the P-4 hot path achievable at all.

**Non-goals:** no packed section, no planner, no witness, no conformance id — increments 4–6. Compiler / off-serving-path throughout; nothing here is deployed-serving evidence.

## Acceptance-criteria status

Advances *"compiler outputs are deterministic and contain no evaluation leakage"* — asserted, not asserted-by-comment. The remaining four criteria belong to the format, runtime and measurement increments.

## Tests and verification

`crates/uor-r4-graph-compiler/tests/semantic_transitions_843.rs` (13 tests) and `plan.rs`'s own module tests (11). Full local ladder before push: fmt, clippy `-D warnings`, `cargo test --workspace --no-fail-fast`, the `no_std` ladder both ways, the `wasm32-unknown-unknown` lib build, `xtask validate` (R1/R4/R5/gnaf), `repo-conformance` R2/R3, `cargo deny check bans`, and the claim-wording gate.

`plan.rs` is deliberately in the `no_std` crate and carries no `alloc` dependency, so the no_std ladder covers it from the first increment it exists rather than after the runtime needs it.

## Evidence artifacts

Fitting on the low half of the topology axis (cells 0–3, 256 seeds per family, horizon 8):

| Family | Rules | Conflicts | Observations | Negatives |
|---|---|---|---|---|
| graph-navigation | 8 | 0 | 32 256 | 440 (13‰) |
| symbolic-transformation | 6 | 0 | 23 520 | 0 (0‰) |
| constraint-satisfaction | 8 | 0 | 32 512 | 1 516 (46‰) |
| multi-hop-evidence | 8 | 0 | 32 256 | 440 (13‰) |
| counterfactual-intervention | 8 | 0 | 27 840 | 0 (0‰) |

**The induced rule count equals the size of the whole shared effect pool in every family** — eight for the grid families, six for symbolic transformation. That is precisely the property the Amendment A1 pool design exists to produce: an inducer fitted on the low half of the topology axis recovers every primitive it will need on the high half, so a held-out cell is a novel *composition* and never a novel *primitive*. A held-out cell is therefore hard rather than unsolvable, which is the only way the equal-budget comparison in increment 6 can mean anything.

Determinism is asserted the way the contract states it: byte-identical rule sets across repeated runs, across reversed input order, and across shard counts 1, 3, 16 and 97.

## Negative and unavailable results

**The negative-evidence floor was scoped, and the reasoning is recorded.** Written unconditionally, it refused two of five families — symbolic transformation and counterfactual intervention carry their whole topology in the operator effect set and declare no forbidden regions, so their dynamics really are total and there is no negative evidence to demand. Refusing them would have been the instrument reporting a property of the task as a sampling failure. The floor now applies only where the observed tasks *declared* forbidden regions, which is the case it is actually for: a boundary that existed and was never probed, from which any induced precondition would be unfalsifiable. A test asserts both halves — it fires on a constrained family whose boundary observations are stripped, and does not fire on an unconstrained one.

**One assumption is recorded rather than papered over.** The reference model's operators carry no declared precondition, so every induced rule is unconditional and no `PreconditionFailed` observation arises on this benchmark. The precondition machinery is exercised instead by the forbidden-region observations, and a test asserts the boundary does **not** become an operator precondition: the induced effect vocabulary equals the declared operator effects exactly, with the boundary adding none and removing none. That is what keeps a query-side constraint out of the artifact-side dynamics — the specific way a naive inducer would bake a constraint into `T`.

## Compatibility and migration

Purely additive. One new public module in each of two crates; no existing type, signature, section, artifact or wire format changes. `plan.rs` defines capacities and shapes but writes no section, so no artifact produced before this change reads differently after it. No `Result` is introduced in shipped code: capacity boundaries return `Option`, and induction returns a typed `InductionOutcome::Refused(...)` as *data* rather than as an error, so a refusal is as inspectable as a success and cannot be discarded by a `?`.

## Conformance effects

None. `CONFORMANCE.md` is byte-identical at 32 ids and no RF id is added or altered. The reserved RF-33 stays unregistered unless an arm is actually lowered in increment 5/6.

## Claim-boundary changes

None. Induction is compiler / off-serving-path and establishes no planning or reasoning capability. The ordinal band is derived from a support count and is explicitly not a calibrated probability (the S2 boundary); everything observed is teacher-forced (the S3 boundary).

## Intentionally excluded

No generated file, model, cache, credential or user-owned file is staged. The maintainer's day-branch checkout is untouched; all work is in a dedicated worktree.

References #843


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

