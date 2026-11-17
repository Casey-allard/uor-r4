---
id: end-to-end-integration
title: Connect training and integer serving behind shared stage interfaces
milestone: research
area: architecture
status: ready
open_to: lab
needs_gpu: false
depends_on: [integer-hamming-read, fbs-serving-kernels, native-space-state-contract]
notebook: docs/research/notebook/m4-serving.md
scope: [docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md, docs/roadmap/items/end-to-end-integration.md, docs/loop/tasks/end-to-end-integration/**, crates/uor-r4-integer/src/stack/**, crates/uor-r4-integer/tests/**, crates/uor-r4-float-train/src/**, crates/uor-r4-training/src/**, crates/uor-r4-discrete/src/**, crates/uor-r4-native-vector/src/**, crates/uor-r4-core/src/native_geometric/**]
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

**November 17, 2026.** The November 16 decision changes the model route that the original D86 sequence below supported. The interfaces and finite fixtures remain available. Use the written-state S0–S6 contract to identify which are actually consumed by P1–P4. Record a new decision before treating a different model run as fulfillment of I6. [Written-state decision](../../loop/tasks/written-state-count-model/claude-409/DECISION.md)

- One common `Read` result and interface, whose reference implementation is the native Q31
  first-pick read that D86 step 6 trains and serves (ADR-0007 Amendment 1). The Hamming copy and
  ask heads come second. Flock, route and relation adapters wait for a second consumer.
- A `Context` seam: the CIRISKernel-inspired bounded ring with a settled stub, replacing today's
  no-compaction paths (the session refuses, the float decode slides, the discrete chat drops the
  oldest token, `StackChat` resets). It must pass the codex-3 witness pair.
- After D86 step 6: a `Codebook::fit` seam, so e-means can replace Lloyd at one of the four k-means
  sites.
- An artifact that names each stage's mechanism and refuses an unknown composition.

Withdrawn or parked by ADR-0007 Amendment 1:
- the RoPE `Position` arm, because position is transport spin, "no RoPE";
- the JEPA `Loss` arm, because its trainer uses cross-entropy and an f64 Adam (CHARTER §3.2, §3.7).

## Done when

1. ADR-0007 records the seven interfaces, the selection protocol, the mechanism map and the unit
   sequence, as amended.
2. I1 lands the one `Read` interface with `Q31FirstPickRead` as its bit-identical reference, shared
   by training and `IntegerStackSession`.
3. I2 lands the `Context` ring and settled stub with full-history equivalence and the witness-pair
   gate.
4. I6 is the same run as D86 step 6: one model trained and served natively, with artifact fields per
   stage and the cost ledger, judged by its replies.

Each unit is its own PR; this item closes when I6 runs. "D86 step 6" is step 6 of the D86
review's path in PLAN.md's owner direction of 2026-11-15.

Follow-ups after it closes, as their own items: I3 (e-means at one site) and any CE-free native
`Loss` arm.

## Out of scope

Changing D85, new GPU qualification runs, promoting a PARKED mechanism without its own reopen
condition, and any claim of useful conversation.


## How to start

Read ADR-0007 and its amendments alongside the registered phase plan. Compare the interface required by the next phase with the merged finite replay records before changing a backend.

## Rules that apply

D85 requires shared typed operators, lossless state and raw-word train/serve replay. Promotion and GPU evaluation follow the charter’s evidence and pre-registration rules.

## Sources

[ADR-0007](../../reference/adr/0007-stage-interfaces-for-end-to-end-integration.md) · [Native-state contract](native-space-state-contract.md) · [Written-state experiment](written-state-count-model.md)
