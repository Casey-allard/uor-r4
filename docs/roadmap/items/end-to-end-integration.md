---
id: end-to-end-integration
title: Connect the mechanisms behind one stage interface, from training to integer serving
milestone: research
area: integration
status: ready
open_to: lab
needs_gpu: false
depends_on: [integer-hamming-read, fbs-serving-kernels, native-space-state-contract]
notebook: docs/research/notebook/m4-serving.md
scope: [docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md, docs/roadmap/items/end-to-end-integration.md, docs/loop/tasks/end-to-end-integration/**, crates/uor-r4-integer/src/stack/**, crates/uor-r4-integer/tests/**, crates/uor-r4-float-train/src/**, crates/uor-r4-training/src/**, crates/uor-r4-discrete/src/**, crates/uor-r4-core/src/native_geometric/**]
---

## Why it matters

The project has 28 mechanisms on [the board](../../mechanisms/README.md), but no shared stage
boundary. The inventory of 2026-11-15 found five selection rules and four aggregation rules for one
conceptual operation, and every integration is a bespoke port into
`crates/uor-r4-integer/src/stack/session.rs`. That is the gap this item closes: not another
mechanism, but the seams, the metric and the selection rule that let the mechanisms be compared and
composed.

"Global optimality" is only meaningful against a fixed corpus and a fixed measure. This item fixes
both before any mechanism is promoted.

## What is missing

- A common `Read` result and adapter so Hamming, flock, route and relation can be compared without
  rewriting the session.
- A `Context` seam: the CIRISKernel-inspired bounded ring with a settled stub, replacing today's
  no-compaction paths (the session refuses, the float decode slides, the discrete chat drops the
  oldest token, `StackChat` resets).
- A `Codebook::fit` seam so e-means can replace Lloyd at the four k-means sites already in the tree.
- A `Position` comparison (SpiralCore transport spin vs a RoPE phase arm) and a `Loss` arm for the
  JEPA latent objective, each judged on served quality, not on its own loss.
- An artifact that names each stage's mechanism and refuses an unknown composition.

## Done when

1. ADR-0007 records the seven interfaces, the selection protocol, the mechanism map and the unit
   sequence, with e-means, CIRISKernel, RoPE, JEPA and the graph compiler each attached to the stage
   it fits.
2. I1 lands the `Read` interfaces with bit-identical reference adapters and a parity harness.
3. I2 lands the `Context` ring and settled stub with full-history equivalence and witness-pair tests.
4. I3 lands `Codebook::fit` with an e-means arm and a bit-identical integer export at one site.
5. I4 and I5 land the `Position` and `Loss` arms with frozen-panel gates.
6. I6 lands the composition: artifact fields per stage, a portfolio config, and one end-to-end run
   with the cost ledger.

Each unit is its own PR; this item closes when I6 runs.

## Out of scope

Changing D85, new GPU qualification runs, promoting a PARKED mechanism without its own reopen
condition, and any claim of useful conversation.
