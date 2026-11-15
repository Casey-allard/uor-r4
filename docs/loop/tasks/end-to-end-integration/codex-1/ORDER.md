# ORDER: end-to-end-integration, codex-1

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: none (new workstream). Reviewed head: `f0aa7f767`.

This unit proposes `end-to-end-integration` as its own umbrella item in
[the roadmap](../../../../roadmap/items/end-to-end-integration.md), but it is filed under
`one-model-end-to-end` because the loop harness resolves an ORDER's item at the merge base, and an
item cannot be claimed in the pull request that creates it.

## Why this unit exists

The owner asked for the mechanisms to be connected into one end-to-end training and serving
process, with e-means attached where it fits and the CIRISKernel window/checkpoint imported. The
inventory of 2026-11-15 found no shared stage boundary: five selection rules for one read, four
aggregation rules, and every integration a bespoke port. This unit writes the architecture and the
unit sequence; it changes no code and promotes no mechanism.

## Claim record

`loop claim end-to-end-integration --lab-topic integration-architecture` is refused with exit 2
because the item file does not yet exist on `origin/main` (it is added by this unit), and the
existing item scopes still overlap `Cargo.lock`. This unit adds no dependency and does not touch
`Cargo.lock`; it continues under the owner's D85 direction.

## What this unit adds

- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`: seven stage interfaces,
  the `ReadOut` contract, the selection protocol (exactness → quality → cost, staged ablations), the
  28-mechanism map, and the four attachments:
  - e-means behind `Codebook::fit` at the four k-means sites, with the bit-identical export and the
    one-third-iteration gate, and the scaling laws explicitly not adopted at our scale;
  - CIRISKernel's checkpoint/purge discipline behind `Context` (ring + settled stub + mergeable
    fold + an overflow policy that replaces the silent drop/reset);
  - RoPE as a `Position` arm against SpiralCore transport spin, not a drop-in;
  - JEPA as a `Loss` arm judged on served quality;
  - the graph compiler's induction as a `Codebook` builder only.
- `docs/roadmap/items/end-to-end-integration.md`: the item with its done-when and out-of-scope.
- The unit sequence I1–I6, with I1 (`Read` interfaces and parity harness) and I2 (`Context`) on the
  critical path.

## Files in scope

- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`
- `docs/roadmap/items/end-to-end-integration.md`
- `docs/loop/tasks/end-to-end-integration/**`

Out of scope: any code, dependency, schema, GPU run or mechanism status change.

## Done when

1. ADR-0007 states the interfaces, the selection protocol, the mechanism map and the unit sequence.
2. Every one of the 28 board mechanisms has a stage and an integration action.
3. e-means, CIRISKernel, RoPE, JEPA and the graph compiler's induction each name the stage they fit,
   the gate they must pass, and what would reopen or drop them.
4. The docs link check, the REPORT check, `git diff --check` and the exact-head review pass.

## Cancel

Stop if the ADR needs a code or schema change to be coherent, or if a mechanism cannot be assigned a
stage without inventing one. Do not start I1 here.

STOP and report when done.
