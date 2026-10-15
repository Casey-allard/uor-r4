# #951 — G1: distill and qualify the one-layer R4 mixer on teacher and student prefixes

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T23:29:44Z
- **Updated:** 2026-08-26T05:24:24Z
- **Closed:** 2026-08-26T05:23:04Z
- **Labels:** enhancement
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/951
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #949

## Objective

Build only the training substrate needed for the single-layer R4 mixer and tokenizer-bound memory adapter from #950, then qualify them on teacher and student prefixes against disabled and permuted geometry/memory.

## Dependencies

Blocked by #950. Reuse the existing versioned teacher trace surfaces from #603 and freeze all source-model weights. Do not create another observation framework or generic training platform.

## Scope

- [ ] Implement a mixer-specific trainer/checkpoint only; dense forward and gradient products remain owned by `uor-matmul`.
- [ ] Pass a tiny synthetic overfit preflight before opening real trace data, and verify checkpoint save/reload identity.
- [ ] Freeze at most 4,096 train and 512 held-out positions plus the five-prompt, 32-token G0 rollout set.
- [ ] Fit the geometric query/key/value projections, bounded support policy, and memory-to-layer adapter using existing residual, Q/K/V, attention-output, and logit traces.
- [ ] Optimize a declared combination of operator-output alignment and next-token loss while keeping source weights frozen.
- [ ] Include deterministic student-prefix examples so the operator trains on states it creates.
- [ ] Compare real geometry/memory with disabled and permuted controls under identical data, parameters, support budget, and decode settings.
- [ ] Run at most three fitting rounds, each with a hard one-hour wall ceiling.
- [ ] Emit one compact report containing identities, losses, support diversity, memory contribution, logit deltas, rollout tokens, cycle status, and outcome.

## Acceptance criteria

- [ ] A fixed-seed synthetic batch of at most 64 examples reduces its loss by at least 50% within 500 steps before real training begins.
- [ ] One tiny parameter slice agrees with a finite-difference gradient check under a predeclared tolerance.
- [ ] Save/reload preserves the checkpoint digest and produces identical inference output for the focused fixture.
- [ ] The real-geometry/memory arm improves the frozen primary held-out loss by at least 5% relative to the permuted arm.
- [ ] Real geometry changes bounded prefix/memory support without collapsing to one position or one memory.
- [ ] Student-prefix results are reported separately and do not regress into a period-1 through period-4 cycle or identical output for every prompt.
- [ ] Permuting only memory worsens at least one predeclared memory-sensitive metric, demonstrating that the bound adapter is load-bearing before all-layer replacement.
- [ ] The five-prompt, 32-token treatment retains the G0 grammaticality/prompt-responsiveness gate.
- [ ] The result records `PROMOTE_TO_ALL_LAYERS` or `REDESIGN_REPRESENTATION`.

## Stop rules

- Failure of the tiny-overfit or checkpoint round-trip gate blocks real trace training.
- At most three one-hour fitting rounds under the frozen data and budget.
- If real geometry/memory does not clear the 5% held-out margin, stop this operator family and return to representation design.
- If teacher-prefix metrics improve while student-prefix rollouts regress, do not scale the layer count.
- Before closing with `REDESIGN_REPRESENTATION`, reblock #952 on the replacement-design issue or close #952-#955 as not triggered; closing this blocker alone is not promotion.

## Non-goals

- Generic autograd, optimizer framework, distributed training, optimizer sweeps, or full-model fine-tuning.
- Large-corpus training, full teacher parity, human-level quality, production serving, or release certification.
- P-4/table lowering, a new benchmark framework, or a new proof/conformance programme.

## Focused verification

- Touched-package compile checks.
- One focused gradient/tiny-overfit/checkpoint test.
- One bounded training invocation and its five-prompt free-running smoke.
- Training stays manual/local and never enters PR CI.

## Claim status

Bounded empirical qualification of one experimental layer and its memory adapter only.


## Comments (3)

### comment 5420253668 — 2026-08-26T03:40:26Z — Casey-allard

## G1 bounded-fit run contract

Claimed implementation lane: one-layer layer-29 geometric mixer only. Source embeddings, tokenizer, source attention/MLP weights, and the frozen G0 adapter remain unchanged. All learned dense products in the mixer forward/backward path use pinned `uor-matmul` revision `b13c98449948174f590e337c4dc25dfc394a07d0`.

### Exact declared loss

`L = 0.55 * mean_squared(operator_target - operator_output) / (mean_square(operator_target) + 1e-6) + 0.25 * sampled_next_token_cross_entropy / ln(16) + 0.20 * support_cross_entropy / ln(candidate_count)`

The operator term targets the frozen source layer-29 attention output. The sampled token term uses the true next token plus 15 deterministic negatives through frozen source embedding rows. The support term targets 80% source attention mass plus 20% persistent-memory support. Only mixer q/k/v/output projections, mixer biases, output gain, and bounded support-logit parameters receive gradients.

### Gate-zero inheritance and reachability

- G0 source CID: `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- tokenizer CID: `blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073`
- base mixer checkpoint CID: `blake3:a63eed546dc6b88a6806f43a238f458f2ec276c7baf6034452ed21fc9baf1fc9`
- base adapter CID: `blake3:7fa11963f23f0e7f6b69a62129a2c01f8de71099d68a622b515a35e3ceffba44`
- G0 established that the injected layer-29 residual reached all 48,896 logits and passed 4/5 causal controls. That is the structural reachability prerequisite; G1 tests whether fitting yields metric-level causal value.

### Mandatory source-free preflight verdicts

All passed before any real source trace was opened (`source_trace_opened: false`):

- tiny overfit: 8 synthetic examples, 320 steps; normalized operator loss `0.008245710516348481 -> 0.00023744947884551948`, reduction `0.9712032724924388` (required at least 0.50)
- finite difference: analytic `0.003943587187677622`, numeric `0.003948807716369629`, absolute error `0.000005220528692007065` (allowed `0.002078976249322295`)
- checkpoint round-trip: bit-identical digest `blake3:cf899bef600e97cd8f03fc89da4eabf17e2aac172b7fa79817b367d86283b579`
- preflight report digest: `blake3:01c9510ca16d79f0fdc78d64f45b1bca2de5e3c4ab5a4f09eacbb1baef31001a`

### Frozen bounded dataset and controls

The source-bound run deterministically selects 27 fixed positions from the G0 P1/P2 teacher traces and P1 student trace: 18 train and 9 held out. This split is frozen and content-addressed before fitting. The evaluation compares real geometry with matched coordinate-permutation and memory-permutation controls on the same held-out positions, separately for teacher and student traces. Rollout evaluation is fixed at five 32-token real prompts plus matched disabled/coordinate-permuted/memory-permuted P1 controls.

### Decision rule

Metric to move: held-out normalized operator loss under real geometry relative to the matched coordinate-permutation control, plus positive persistent-memory use.

Exit rule: machine gate requires finite real/control losses, held-out real loss at least 5% below matched coordinate permutation on both frozen teacher and student partitions, and positive held-out memory support. The command then stops at `PENDING_OPERATOR_REVIEW`; five recorded 32-token outputs must each receive an explicit coherent yes/no review.

- if positive: all machine gates pass and all five operator reviews are coherent -> exact terminal verdict `PROMOTE_TO_ALL_LAYERS`, unblocking #952
- if negative: any machine gate fails, persistent-memory use is absent, or any of the five reviews is incoherent -> exact terminal verdict `REDESIGN_REPRESENTATION`; no broader layer-stack work launches

### Cost and ceilings

- maximum three independent fitting rounds
- maximum one wall-clock hour per round
- 80 deterministic full-batch updates per round by default
- 18 train / 9 held-out positions, 16 sampled token candidates, support width 4
- no broad BDD, Gate C, kappa, fuzz, Kani, or corpus-scale evaluation
- expected local wall time: roughly 1-3 hours including source tracing and bounded rollouts; hard fitting ceiling is 3 hours

Launching the real-source run only after this comment is posted.

### comment 5420809786 — 2026-08-26T04:58:42Z — Casey-allard

## Bounded-fit outcome: `REDESIGN_REPRESENTATION`

The single declared source-bound invocation completed all three permitted 80-step rounds and stopped with the exact terminal-negative verdict. No additional fit, expanded population, layer, or sweep was launched.

### Bound identities

- source weights: `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- tokenizer: `blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073`
- G0 base mixer: `blake3:a63eed546dc6b88a6806f43a238f458f2ec276c7baf6034452ed21fc9baf1fc9`
- frozen dataset: `blake3:b9ce10e7010724ffd8c42acd327e497f5f52559eefdf53f0de85aba289bdff97` (18 train, 9 held out; 12/6 teacher train/held-out and 6/3 student train/held-out)
- retained negative checkpoint digest: `blake3:8c84b8fe5b2d19be635fccf505b23a92e5daae1d3a93eda8d3465387ac75e139`
- retained negative mixer identity: `blake3:65f9f3a6e4db1690acf1d56ed5cbe0352a02f7ff36f1a992347e447652ecb18c`
- retained negative memory-adapter identity: `blake3:5ec83ccdd60b7fb7db985cef2b719fbe42e897285753fcdcc62e360c1d96897c`
- report digest: `blake3:98cae8e27808b7a5bd4e330bfd389a13f4720643da8315224e0510b89ab72469`

### Preflight

All gates passed before source access (`source_trace_opened: false`):

- synthetic operator loss `0.008245710516348481 -> 0.00023744947884551948` (97.1203% reduction, 8 examples, 320 steps)
- gradient analytic `0.003943587187677622` vs finite difference `0.003948807716369629`; absolute error `0.000005220528692007065` within `0.002078976249322295`
- checkpoint round trip bit-identical with digest `blake3:cf899bef600e97cd8f03fc89da4eabf17e2aac172b7fa79817b367d86283b579`

### Fitting and primary gate

Round wall times were 72.205s, 72.683s, and 72.681s, all below the one-hour ceiling. Total train loss moved `1.0046638350 -> 0.9274201393`, but improvement was support-only in substance: support `1.0233210822 -> 0.6371028324`, while operator alignment remained `0.9999995861` and sampled-token loss remained `0.9999990894`.

- all held-out: real `0.9414138396581014`, coordinate-permuted `0.9851714902453952`, advantage `0.04441627779585279` -> **FAIL** versus required `0.05`
- teacher held-out: real `0.9433101018269857`, permuted `0.9867582321166992`, advantage `0.044031180967715605`
- student held-out: real `0.9376213153203329`, permuted `0.9819980065027872`, advantage `0.04519020495825049`

Memory was causally load-bearing but insufficient for promotion: real target-memory probability `0.10343818697664472` versus memory-permuted `0.04907133450938596`, delta `0.05436685246725877`; memory-permuted loss was `0.9467250638537936`.

### Product smoke and stop action

Five real 32-token sequences were distinct, UTF-8 decodable, support-bounded, and had no period-1 through period-4 cycle. The frozen human 4/5 rubric is `NOT EXERCISED`: the primary machine gate had already failed, so operator review could not override the terminal-negative branch.

Concrete replacement issue #958 was created, parented natively to #949, and blocked by #951 until this result lands. #952 now has native blockers #951 and #958. After #951 closes, #958 becomes the next eligible design issue; #952 remains blocked unless #958 later records explicit `PROMOTE_TO_ALL_LAYERS`.

Broad BDD, Gate C, kappa, fuzz, Kani, corpus-scale work, full teacher parity, and exhaustive certification are `NOT_RUN` by design.

### comment 5420989122 — 2026-08-26T05:24:24Z — Casey-allard

Closure evidence

- PR #959 merged through the protected merge queue at effceeda7d89218a7d09f01d2975b1ab8ea9e131.
- Binding merge-group CI run 32933399889 completed successfully on the exact merged tree.
- The bounded G1 qualification terminated REDESIGN_REPRESENTATION after 3 rounds: held-out real loss 0.9414138396581014 versus coordinate-permutation loss 0.9851714902453952, advantage 0.04441627779585279 below the declared 0.05 gate. Teacher advantage was 0.044031180967715605; student advantage was 0.04519020495825049.
- Replacement issue #958 is an open, unassigned child of #949 and is now the next eligible work item; #952 remains blocked by open #958. No #958 work was started.
- Focused local format, claim-wording, compile, geometric-training, qualification, CLI, wasm, exact-source-closure, and targeted clippy checks passed. BDD, Gate C, kappa reproduction, fuzz/Kani, and corpus-scale runs were NOT_RUN locally as required by the issue boundary.


---
_Comment/review ids in this file: 5420253668, 5420809786, 5420989122_

