# #923 — feat(#843): packed bounded-planning sections PSCH, PTRN, PGOL and PWIT

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T15:27:53Z
- **Updated:** 2026-08-22T17:39:31Z
- **Closed:** 2026-08-22T15:47:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/923
- **Merged:** 2026-08-22T15:47:24Z
- **Merge commit:** f5fccc511e48379559ad5586af71a220aa2ff617
- **Base / Head:** main / UOR-Foundation:issue-843-packed-plan-sections
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 4 of 6 for issue #843 (item B of S4 tracker #826). Adds the four optional R4G1 sections `docs/bounded_semantic_transitions_spec_843.md` §5 freezes, their two-stage validation, and the self-contained plan-witness encoding of §7. Sixteen tests, including the §9 planted-mutation table row by row.

## Scope and non-goals

| Section | Id | Carries |
|---|---|---|
| `PSCH` | `0x13` | slot shape, the operator **effect** vocabulary (strictly ascending), the eight frozen capacities as *recorded* values, the three ordinal band thresholds |
| `PTRN` | `0x14` | 52-byte rule rows canonically ordered by `(operator, precondition, effect)`, plus a 4-byte-per-operator index that tiles the row array in order |
| `PGOL` | `0x15` | the goal and forbidden-region predicates one planning query carries, each group strictly ascending |
| `PWIT` | `0x16` | a versioned, self-contained plan witness |

All four carry the optional bit, so an artifact without them — or a reader that does not consume them — behaves exactly as before, and every artifact produced before they existed stays valid. Parsers are `core`-only and allocation-free; builders are `alloc`-gated, because a section is written offline and only ever read on a hot path. Nine typed `Plan*` `FormatError` variants and four `ObjectKind`s are added, so every rejection names what was not a product of the bytes.

**Non-goals:** no planner, no `R4Engine` wiring, no measurement, no conformance id — increments 5 and 6.

## Acceptance-criteria status

Advances *"the selected packed planner is bit-equivalent to reference semantics on valid fixtures and fails closed on corrupt or incompatible data"* on the fails-closed half, and *"witness replay independently validates each transition and terminal goal"* in full: `PlanWitnessBytes::replay` is written and asserted here; increment 5 supplies its producer.

## Two design points worth review

**The capacity header is checked, not honoured.** A recorded capacity larger than the one this build enforces is refused with a typed error rather than accepted. A capacity header is a promise about bounded work, and reading one larger than the scratch that will be provided is exactly the silent overflow the bound exists to prevent. A test mutates each of the eight capacity fields independently and asserts every one is enforced — not just the first.

**The operator index tiles the rows.** `PTRN` rows are grouped by operator and the index gives each operator's contiguous slice, so resolving `operator -> rules` is a fixed-offset table read whose cost is not a function of how many rules the table holds. Parse rejects an index that leaves a row unreachable or reachable twice.

## Tests and verification

`crates/uor-r4-graph-format/tests/plan_sections_843.rs` (16 tests). Full local ladder before push: fmt, clippy `-D warnings`, `cargo test --workspace --no-fail-fast` (181 suites), the `no_std` ladder both ways, the `wasm32-unknown-unknown` lib build, `xtask validate` (R1/R4/R5/gnaf, 32 ids), `repo-conformance` R2/R3, `cargo deny check bans`, and the claim-wording gate.

## Evidence artifacts

**The planted-mutation table is asserted row by row**, and the two kinds of mutation are distinguished rather than lumped together. *Structural* — truncation, trailing bytes, bad magic, an unsupported version, non-zero reserved bytes, an unenforced capacity header, an out-of-range operator id, a read mask disagreeing with its per-slot operations, an out-of-range ordinal band, non-canonical rows, an index that does not tile the rows, an unknown comparison code, a duplicate key — is refused at parse with a typed error. *Semantic* — a corrupted step, a corrupted terminal state — parses and then replays `Invalid`.

**The #846 rule is asserted directly.** A plan whose terminal state satisfies the goal, but whose second step walks through a forbidden region, replays as `Invalid { step: 1, EntersForbiddenRegion }` — not `Valid`. A right answer via an invalid intermediate step is not a valid plan.

**The witness is self-contained by construction.** It carries its own initial state, its goal and forbidden predicates *inline*, and per step both the applied effect and the resulting state, so `replay()` re-verifies it from those bytes alone — no model output, no planner, and no other section. The considered-candidate records are informational and replay does not read them, exactly as the #844 reference witness documents.

## Negative and unavailable results

**The witness envelope is measured, and the worst case is over.** This benchmark's realistic worst case — five operators, at most seven forbidden regions, horizon 16 — encodes to roughly 2.1 KiB against the frozen `PLAN_WITNESS_MAX_BYTES` = 4096, about 2× headroom. The theoretical worst case (64 forbidden regions, 64 candidates recorded per step, 16 steps) **exceeds** the frozen cap. That is handled the way §4.3 mandates rather than by quietly raising the cap or dropping records: the encoder returns `None` and the producer must emit `Decline(capacity)`. A test asserts the encoder refuses rather than truncating. **No silent cap.**

## Compatibility and migration

Purely additive. Four new optional section ids, one new public module, nine new error variants, four new `ObjectKind`s, and four `GraphView` accessors. No existing type, signature, section or wire layout changes. Because the sections are optional and absent from every existing artifact, `plan_schema()` and friends return `Ok(None)` on every artifact produced before this change, and serving is identical to the pre-#843 baseline. `plan_rule_table()` and `plan_predicates()` resolve their schema together with their own section, because a rule table without its schema is not a product of the bytes.

## Conformance effects

None. `CONFORMANCE.md` is byte-identical at 32 ids and no RF id is added or altered. The reserved RF-33 stays unregistered unless an arm is actually lowered in increment 5/6.

## Claim-boundary changes

None. These are byte formats and their validation; nothing here establishes a planning capability or reads on a serving path yet.

## Intentionally excluded

No generated file, model, cache, credential or user-owned file is staged. The maintainer's day-branch checkout is untouched; all work is in a dedicated worktree.

References #843


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

