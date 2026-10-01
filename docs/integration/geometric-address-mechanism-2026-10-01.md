# Geometry-primary attention address mechanism, October 1

References #1512, #973 and #820. Owner correction supersedes this lab's numerical-lowering next action. **Status: source-backed design; integrated address replacement and model results NOT_RUN.** Principal engineering, mathematics and independent architecture reviews agree on the separation below. Arithmetic approval of the unfinished Lorentz port is not architectural qualification.

## Problem and retained evidence

The current `StackModel::geometric_read` forms addresses with four dense learned maps of current contextual input and retained identity. Its Lorentz metric is geometric, and its recurrent trunk contains quaternion transport, but the maps still perform much of the semantic address construction. Four-bit execution changes numerical implementation, not that allocation of semantic work. The successful authored capture/hold/rebind models remain useful reference artifacts and optional offline compiler inputs. They are not geometry-primary attention qualification.

The integer Lorentz attempt is preserved on `codex/attention-integer-lorentz-20261001`, source `1cf9b83b99852a45fed68c9b3ddb4fa56bc0abe5`. Seventeen focused checks passed; the model evaluation is NOT_RUN. Do not continue its scalar/normalization sequence as the immediate research task.

#1438 changed magnitude, direction quantization and score function together while retaining dense upstream q/k. D6 measured disagreement with a frozen dense parent's rankings; it did not train the new representation from the start. These scoped negatives prevent an unchanged post-hoc replay, not admission of a newly learned geometric address mechanism. See [D6](d6-information-audit-result-2026-09-28.md) and the [toolbox](geometric-toolbox-2026-09-28.md).

## Selected integrated mechanism

Replace the four current/identity q/k maps at the native read boundary with learned placement in **separate ordered geometric channels**. For each four-coordinate lane, retain:

- Captured-content direction `c` in signed 2I, its radius and explicit zero/presence state.
- Current-context direction `r` in signed 2I, its radius and explicit zero/presence state.
- Exact source occurrence/version identity and payload reference outside the approximate geometric address.

The immediate placement candidate directly classifies the existing four-coordinate held/current lanes into signed roots; it does not introduce another learned dense query/key placement head under a different name. Its joint gradient, radial/presence encoding and exact served code producer remain implementation requirements. Before fitting, specify the actual causal encoder inputs and operations; the eventual deployed code producer cannot require the donor or a dense transformer. The surrounding existing recurrent computation remains an explicit later geometric-compilation obligation.

The current contextual state is not declared a semantic role. That interpretation requires learned role/context interventions. Its placement and capture interface must receive joint language-to-read credit in the same model graph; serving receives predicted state/operations, never authored role labels.

Form the query channels from prior held state `h[t-1]` and current contextual input `u[t]` before applying the capture step at `t`. Cache the source channels at their source occurrence; subsequent rebinding must not rewrite a historical address. For query occurrence `t` and causal source `j`, compute directed channels

`delta_c[l] = inverse(c[t,l]) * c[j,l]`

`delta_r[l] = inverse(r[t,l]) * r[j,l]`.

The learned compatibility has unary terms and an explicit content/context cross-factor:

`score(t,j) = sum_l (Uc[l,delta_c] + Ur[l,delta_r] + P[l,delta_c,delta_r]) + radial/zero contribution + age(t-j)`.

This is a new learned finite score, not a factorization or faithful numerical lowering of the old Lorentz score. Independent radial terms cannot reproduce its globally coupled lift. This is a design expression, not an implemented equation. The radial/zero contribution must be specified and compiled before model compute; merely retaining radius in metadata while the scorer ignores it is insufficient. Declare its resolution, units, rounding and factorization. Preserve NoRead and full causal support, including self, so a hidden candidate mask cannot supply the answer. Keep the current payload/value and output interfaces for the first attribution comparison and disclose their remaining dense/float work.

An empty identity is not the unit group element. Signed antipodes remain distinct. Multiple lanes retain a tuple; do not multiply all lanes into one root. A paired-H4 companion remains the fixed golden/Galois-coupled representation, never an independent second learned channel. This first signed-2I consumer does not assert a complete paired-H4 or zeta language mechanism. A Hopf observation is insufficient if the consumer needs the discarded fiber.

Why these choices matter:

1. A product `c*r` alone cannot recover arbitrary content/context pairs: many pairs share one product. Keep both channels for conditional compatibility.
2. Common left action cancels: `(a*cq)^-1*(a*ck) = cq^-1*ck`. Shared rotation is not a new semantic selector.
3. A full directed relative potential has 120 element slots. Restricting it to nine conjugacy classes adds conjugation invariance and loses distinctions; these are different function spaces.
4. If `L` counts individual signed-root channels, `120^L` is their tuple space; with `L` content/context lane pairs the joint directional tuple space is `120^(2L)`. Neither is a promise of injective learned placement or memory capacity. Exact occurrence identity is still required. One root cannot injectively represent more than 120 categories.
5. Unary content and context scores alone cannot express general conditional compatibility. The selected cross-factor is substantive, rather than more unrelated scalar features.

For a later relation-composition task, a predicted query-local action can produce `inverse(q*a)*k`, using the toolbox's declared right-action convention. Do not add that mechanism before a task distinguishes it. Geometric frame composition does not imply that all factual or linguistic relations form a group.

## Reusable source and the missing integration

- `crates/uor-r4-core/src/native_geometric/learner/integrated_attention/geometry.rs`: `FiniteAlgebra`, signed relative actions, `Cyclic120`, `EnergyTables`, selected unary/pair four-bit coefficients and read counts. Its maximum 32 channels can accommodate separate content/context channels. Packed pair tables use 128-stride padding. With eight content/context lane pairs, 16 unary tables and eight pair tables occupy 66,560 parameter bytes before radial/zero, model metadata or indexes. This is a layout calculation, not measured model cost.
- `crates/uor-r4-training/src/geometric_read.rs`: exact signed hard forward and smooth Hamilton straight-through scaffolding. Its old normalized dense-q/k consumer and per-lane MLP are not the selected architecture.
- `crates/uor-r4-training/src/geometric_stack.rs`: existing contextual states, learned identity latch, shared causal read and binding observer. The new address mode must remove/bypass the four q/k maps explicitly, share ordinary/source-observer scoring and bind its actual representation in save/load.
- `crates/uor-r4-core/src/native_geometric/source_routing.rs`: learned ordered H4 metadata and exact payload references. Its bounded frontend does not establish general language.
- `crates/uor-r4-core/src/native_geometric/learner/shared_transition.rs`: shared primitive action interface, reusable without its finite-domain capacity assumptions.

The old `integrated_attention/training.rs` explicitly uses truncated local credit. Reuse its finite runtime arithmetic; **do not resume its A1–A4 driver** as the default learning method. The new Rust autodiff bridge must expose and verify actual language/context gradients, hard-forward behavior and independently reloaded output before an integrated fit. A surrogate's gradient is not automatically the derivative of the hard discrete function; state its definition and limitation. Do not fabricate a hard-path backward by reusing the inference-only numerical replay seam.

## Both learning and weight recompilation remain supported

Native joint learning and offline weight-informed geometric compilation/distillation share this runtime target. Retain donor embeddings, dense q/k weights, contextual activations and operator behavior as possible offline inputs. They may inform geometric code placement, relative potentials, shared actions and output operators. Token embeddings supply useful structure but do not by themselves preserve all reasoning dynamics. Source tokenization must also be bound; matching numerical token IDs is insufficient.

A compiler report must identify which donor components were converted, which were learned or distilled, retained capability and degradation, artifact size and actual runtime operations. Donor access ends before independent serving. No lossless, cheap compilation of arbitrary weights is assumed, and an exponentially large lookup of transformer states is not an acceptable hidden backbone. Direct conversion, operator fitting and behavior distillation are distinct claims. Keeping this route does not restart the parked unchanged #1518 parity chase or authorize paid compute.

Primary research supports these distinctions, not this proposed implementation: [ROME](https://arxiv.org/abs/2202.05262) studies factual associations in model feed-forward computation; [distillation](https://arxiv.org/abs/1503.02531) transfers learned predictive behavior between models; [VSA capacity analysis](https://arxiv.org/abs/2301.10352) treats representational capacity as a concrete constraint. None proves frontier capability, efficient exact weight recompilation or this geometric attention design.

## Decisive comparison and admissibility

First implement the address mode and joint-learning bridge in the existing native graph; do not create a separate fixture engine. Then register a complete bounded work card before compute. Keep the capture interface, exact occurrence log, payload/output interfaces and candidate access fixed; train the representation from the outset. Preserve all successful and negative parents.

Compare the signed-2I mechanism with a separately trained ordinary finite/categorical control at declared equal state cardinality, channel capacity, coefficient budget and candidate access. `Cyclic120` is a reusable equal-cardinality finite-action control; it is not automatically an optimal ordinary model or a nongeometric control. Declare its encoder/gradient and cost differences. If claiming a noncommutative benefit, include order/action composition that distinguishes it. Relabeling the same group is an isomorphic implementation, not removal of its algebra.

Evaluate changed query/value/order, role reversals, new content/context combinations, gap/rebinding, same-valued distinct occurrences, absence and self competitors. Distinguish selection of the correct occurrence from answering the right value by coincidence. Separate interventions on the relative algebra from interventions on the cross-factor, with exact IDs/content available; execute actual outputs. Disabling the cross-factor establishes interaction dependence, not geometric composition by itself. The trained algebra comparison and ordered-action task are needed for claims about the particular group structure. Do not use oracle roles, source masks or a hand-authored semantic/source-selection frontend at serving to solve the experiment. The declared learned encoder/typed compiler and ordinary tokenization remain permitted.

Outcomes change the next action:

- Useful selection and causal geometric-path dependence: admit this component at the measured scope; retain its artifact and advance typed span/commit and serving integration. Matching the control can admit the component without claiming geometric superiority.
- Ordinary control succeeds but geometry fails: diagnose placement, radial/zero handling, interaction capacity or learning bridge using saved errors. Preserve the candidate; do not retire geometry as a family.
- Both fail: investigate the shared interface/credit problem. Do not conduct a blind metric/quantizer sweep.
- Execution unavailable: no quality conclusion.

Natural language, complete integer serving, bounded candidate indexing, energy and frontier intelligence remain separate unfinished obligations. The first geometric addressing result is an intermediate mechanism, not a redefinition of the programme goal.
