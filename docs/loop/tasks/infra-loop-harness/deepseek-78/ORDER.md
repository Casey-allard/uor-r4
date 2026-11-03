# ORDER: register the served-rounding cost item, deepseek-78

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance: a new item). Base: `a9d0461e2`.

## Goal

Register `served-rounding-cost` so the failed 6.83-6.96x train-as-served gate
has a claimable repair unit with a code scope, exact controls, and the existing
four-run comparison as its deciding measurement.

This unit only writes the new item and its ORDER/REPORT. It changes no serving
code, model behavior, acceptance value, or Claude-owned branch.

## Cancel probe

1. **Already covered by a claimable item?** No. `one-model-end-to-end` names
   `fbs_snapshot.rs` but is already claimed by open PR #3583, whose unit is the
   batched scorer. Adding the cost repair there would combine two independent
   units.
2. **Is the cost real?** Yes, at the registered run: 355 seconds served versus
   51/52 seconds plain at the same 256 steps, with the same source and data.
3. **Is this item already registered?** No.
   `docs/roadmap/items/served-rounding-cost.md` is absent at the base, and no
   other item requires the exact same four-run operational repair.

Not cancelled.

## Scope

- `docs/roadmap/items/served-rounding-cost.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-78/**`

No crate, model, serving artifact, notebook, plan, decision or owner file
changes in this registration unit.

## Done when

1. The item names the failed measurement, preserves its limits, and requires
   the exact four-run rerun rather than a longer or easier comparison.
2. The item's implementation scope is limited to the integer snapshot
   quantizer and its records; active float-training files remain outside this
   first repair claim.
3. Cancel wording protects determinism and refuses an inferred result.
4. Roadmap validation and the fresh exact-head review pass.

## Stop if

Registration would mark the item done, claim a result, widen an owner rule, or
overlap Claude's active copy-key branch.
