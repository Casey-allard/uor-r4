# Shared geometric language core — first implementation

The owner adopted this direction on September 12, 2026. The [canonical plan](project-track.md#current-implementation-sequence--owner-adopted-september-12) owns sequence and acceptance; [current state](current-state.md) owns execution results, artifact identities and the active restart point. This document specifies the first mechanism and its decision boundary. References [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) and [#964](https://github.com/UOR-Foundation/uor-r4/issues/964); it does not complete either issue.

## Decision to make

**Hypothesis:** a shared finite geometric recurrence, contextual read and byte emitter can learn contextual prediction and generate useful unseen continuations when trained together through the actual discrete forward path.

The first implementation tests whether that complete path is trainable. It does not assume that a small finite state or hard reads have sufficient capacity for general language. A failed run calls for a bounded diagnosis of state distinctions, candidate access, choice and credit assignment. It does not authorize an unchanged retry or an accumulating series of request-specific correction heads.

## Complete experimental path

| Stage | Initial implementation | Required boundary |
| --- | --- | --- |
| Ingress | Exact ordered bytes, including separators, with an explicit EOS symbol | Preserve byte and position identity; do not reinterpret hash bits as semantic similarity. |
| Recurrent state | Shared learned choices of canonical H4 finite compositions and state updates | Retain signed orientation; use reusable finite operations rather than a table over the complete state Cartesian product. |
| Context access | Bounded stored geometric context, a learned query and geometric read selection | Make available candidates and chosen context observable separately; use causal earlier state only. |
| Emission | Binary branch decisions constructing a byte or EOS from shared geometric state | No inherited additive feature-to-token predictor, dense vocabulary projection, teacher response, answer template or transformer fallback. |
| Offline learning | Coordinate updates to interacting parameters against a common predictive objective evaluated on the hard forward path | Rust throughout; disclose objective, update order, seed and dose. Offline floating point or matrix multiplication is permitted by policy, but is not necessary to describe this initial coordinate method. |

The implementation lives in `crates/uor-r4-core/src/native_geometric/shared_core/`. A byte-level emitter avoids requiring a pretrained vocabulary projection. That is an implementation choice, not evidence of linguistic competence or efficient scaling. Output must include actual continuation and termination behavior; teacher-forced scores do not substitute for generation.

The subsequent [schema-2 calibrated emitter](../native_geometric_calibrated_emission_973.md) keeps the ordered H4 components separate through a learned union/intersection of two angular caps. It versions the original emission decision without changing recurrence or contextual access. Its fixed calibration failed admission, so no joint-fit or new-generation result is claimed; current state owns the resulting development action.

This first path uses H4 finite composition as its geometric foundation. The existing geometry compiler provides exact signed `Z[phi]` anchors, group composition/inverse tables and the paired-H4/icosian witness. Exact byte identities address canonical prime-token entries; the first four fixed zeta channels advance phase counters and address learned phase-root tables. Four recurrent H4 slots are independent computational state, not independently learned Galois companions. The retained prime/ordered-n-let memory records and typed operators keep their scoped contracts. Their complete integration and the predictive contributions of these geometric mechanisms are not established by this prototype. Exact durable memory, typed arithmetic/copy operations and existing API/session/checkpoint serving are not yet connected to the experiment.

## Experiment and acceptance

Before fitting, pin source, parameter initialization, byte corpus, development/held-out split, generation prompts, controls and resource projection in the exclusive local attempt. Use short text, fact/update patterns, dialogue and elementary Rust fragments at a modest dose. Hold out content and structural combinations; do not silently turn a failed final draw into another claimed independent result.

First verify that the runtime exercises recurrence, causal bounded access and every byte/EOS decision without disallowed fallback. Focused checks should cover finite-table invariants, causality and bounds, state reset, byte/EOS validity and meaningful parameter influence. These establish implementation properties, not language quality.

The learning report must contain the initialized and learned hard-forward predictive results, actual generated continuations and termination, and matched controls that remove or alter relevant context, recurrence or geometric contribution. State the exact comparative gate before execution. Advancing the architecture requires held-out predictive improvement together with improvement in generated behavior and evidence that useful context contributes. Report mixed outcomes explicitly. A tiny authored corpus can establish only its bounded result; training improvement alone is not a selected language model.

The intervention names have deliberately narrow meanings. `ContextDisabled` suppresses stored-context selection. `StateDisabled` substitutes the identity only for the prior-root input to the first recurrent composition; phase accumulation, context entries, reads and later state updates remain. `TransportDisabled` substitutes the byte code for that first prior-root/byte product; phase, cross-lane, read and output compositions remain. `ZetaDisabled` suppresses the phase-code composition while still advancing phase counters. A difference under one of these controls measures the named intervention, not removal of all context, all state or all geometric computation.

A language-quality positive would justify the next bounded scale/integration decision. It would not promote this artifact into the retained serving path. Replacement later requires applicable retained behavioral controls, exact memory/operator and session interfaces, independent transfer, executed generated-code semantics where claimed, and a complete operation and laptop-cost assessment. No matrix-free or energy claim transfers from an isolated kernel to the inherited decoder.

## Resource and preservation boundary

Local execution is bounded by the existing `shared-core-first-step/projection.json` in the established project handoff. It covers preparation, build, fit, controls, evaluation, diagnosis and closure, charges the shared cumulative ledger and preserves the 128 MiB storage stop margin. No additional paid compute, destructive cleanup or resource-limit reset is authorized. The current-state record links the host-specific projection and receipts; this page does not mirror their changing totals.

Keep `15baec48` as the retained model. Preserve the earlier `f0901dad` comparison, V7 `aede9c18`, subsequent negative repair candidates, intentional dirty source, sealed reports and original paths. The accumulated repair is parked; this experiment does not restart it, repeat V3–V7 or import its unqualified assembly into serving.

The source recommendation and process reevaluation remain in `.uor-handoff/2026-09-12-codex-v7/process-reevaluation/`. Active local navigation remains in that established handoff, linked from the isolated shared-core worktree. Record one decision and one next action after the bounded experiment. Deliver source and warranted findings through the protected PR workflow without claiming unfinished issue acceptance.
