# ORDER: native-space-state-contract, codex-12

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Reviewed head: `e5a32f458`.

## Why this unit exists

Two token-objective configurations are now measured negatives (m4 codex-6 and codex-7): the raw
writer target and the normalized unit target both lower the independent alignment dot and leave
the served tokens unchanged. The shared state is **one** 4-word Hamilton gate, while the records
ask it to align 112 lane carries toward 112 different next-token directions. Before writing more
code, this unit records the decision: which configuration comes next, with a decision review, so
the next unit is one bounded implementation rather than another guess.

## Claim record

`loop claim native-space-state-contract --lab-topic native-lane-gates` was refused because this
item's `docs/model/**` scope overlaps the still-open order m4-export-uorlut01-cli/codex-7. This
unit therefore lives entirely in its task folder (always allowed), adds no `docs/model/` file and
no code, and continues under the owner's open D85/D86 direction.

## Decision

Adopt **per-lane gates, shared across layers** (option A) as the next bounded configuration, with
per-layer-lane gates (option B) and a different objective/update (option C) as the explicit
fallbacks. The design, evidence, schema, operator path, bar and cancel are in
[DESIGN.md](DESIGN.md).

## What changes in this unit

- `DESIGN.md`: the evidence, the three options, the chosen one, the state/artifact/serving/schema
  changes, the independent bar, the cancel conditions, the cost and the implementation steps.
- A fresh-context **decision review** reading the whole project, the two negative reports, this
  ORDER and the design. The review is the unit's gate.

No production code, artifact, serving, kernel or test change. The implementation is the next
unit.

## Files in scope

- `docs/loop/tasks/native-space-state-contract/codex-12/**`

Out of scope: `docs/model/**` (blocked by the open M4 order), every crate, the donor pilot (it is
already running on wick under another session), any GPU run.

## Size

About 120 to 180 lines of design and records.

## Done when

1. The design names the state size and schema, the operator path, the exact independent measure,
   the bar, the cancel and the cost.
2. A fresh-context decision review approves it (or returns CHANGES and the design is revised).
3. The ORDER and REPORT are committed with the design.

## Cancel

Stop if the decision review finds a more decisive configuration, if option A cannot keep the
one-state contract (same typed operator, bit-identical replay, lossless serialization), or if the
donor pilot's data makes the token objective moot. Do not start the implementation or a GPU run in
this unit.

STOP and report when done.
