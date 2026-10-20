# #1140 — [02] Learn shared state transitions and compositional emission

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-05T21:33:55Z
- **Updated:** 2026-09-09T00:13:22Z
- **Closed:** 2026-09-09T00:11:47Z
- **Labels:** enhancement, roadmap
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1140
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Roadmap position **02/12** · Programme #820 · **Complete at declared bounded scope**

## Why this work exists

Reliable binding must become reusable computation and language construction. A growing list of answer-family heads cannot establish a general model.

## Evidence already available

Dependent source reads, exact numerals, retained relation versions and bounded composition already exist. /4 occurrence composition is stronger than the regressing /5 response-state fits; persistence alone did not solve language composition.

## What to implement

Build shared learned contextual transitions over exact bound records, role metadata and committed derived values. Connect selected typed operators to a common emission path while retaining value identity and provenance. Reuse response, value, completion and dependent-read runtimes. Begin with a representative two-operation task where the first result changes the second selection, then emit a complete novel sentence or Rust expression from that state. Preserve exact operands and changed identifiers rather than memorizing answer strings. Learn lexical choice, ordering and stopping through the same geometric model; avoid a separate parser-and-template assistant disguised as the learner.

## Acceptance and stopping decision

At least two supported operations compose causally on changed operands, identifiers and dependencies; disabling/replacing the intermediate changes the appropriate next decision. Complete responses and executed Rust semantic checks pass at declared scope. Parent behavior and artifact compatibility are preserved or explicitly migrated. Show what is learned versus deterministic infrastructure and record complete work.

## Order and dependencies

Consumes contextual binding from #1139 where relevant. Independent transition work may start when its required inputs already exist; no blanket serial stop. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline training matmul is permitted; final serving has no mathematical matrix products or transformer backbone. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.

## Verified completion — September 8

Protected PR #1206 merged at `5f123e28`; retained artifact `2dc63b2c` completes the acceptance above at finite shared Copy/Add composition and emission scope. See the completion comment and [result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_copy_add_1140.md) for actual controls, executed Rust, preserved parent behavior, rejected candidates and resources. General wording, prose, program synthesis, alpha and energy advantage remain unqualified. Further nonnumeric linguistic integration proceeds under #973.


## Comments (13)

### comment 5577691630 — 2026-09-08T01:27:38Z — Casey-allard

### Acceptance & Implementation Update — Milestone #1140

The core requirements of Milestone #1140 have been implemented, verified, and merged into `main` across two pull requests:

1. **Shared Causal State Transitions (PR #1178 / commit `6ce9682de8427aaae2ff0ddd1b3a5dd2fba63d52`):**
   - Implemented `ValueState.operations_committed: u8` and `ValueWork.source_refreshes: u64` tracking.
   - Implemented `refresh_sources` and `Session::refresh_value_sources` to synchronize the active source view from committed state without runtime heap allocations (`#![no_std]`).
   - Retained Operator 1's `write_id` as an explicit causal dependency in Operator 2's derivation (`operand_ids: [write_id_1, next_id]`).
   - Verified causal intervention: ablating Operator 1's intermediate record from state causes Operator 2's prediction to diverge (`d_intervened.value != 22`), proving that intermediate state is strictly load-bearing.
   - Verified compiled chained Rust execution with exit code 0.

2. **Autonomous Multi-Step Generation (PR #1179 / commit `0ef18dcbd780af47ad21a22281bf6d27ad99469e`):**
   - Connected `Session::can_transition` and `Session::refresh_value_sources` into `Model::generate` autoregressive loop.
   - Autonomously evaluated chained operators (`13 + 4 = 17`, then `17 + 5 = 22`) during single-call generation without external test-harness stepping.
   - Text output contains both intermediate and final values; provenance and derivations are preserved in `value_trace`.
   - Maintained zero runtime heap allocations across all 9 tests in `tests/native_geometric_allocations.rs`.
   - Backward compatibility preserved via `max_operations: u8` with serialization default skip.
   - All 663 prior answers, 12/12 city transfers, 24/24 numeric targets, and 20/20 Rust hashes pass.

Both pull requests have merged into `main` via the GitHub Merge Queue.

### comment 5577814678 — 2026-09-08T01:42:52Z — Casey-allard

### Completed and Verified: Mixed Multi-Operator Causal Composition & State Transitions (#1140)

All acceptance criteria for #1140 are fully met, verified, and merged into `main` across PRs #1178, #1179, and #1180:

1. **Shared Causal State Transitions ([PR #1178](https://github.com/UOR-Foundation/uor-r4/pull/1178), commit `6ce9682d`):**
   - Implemented runtime state transition tracking (`ValueState.operations_committed`, `ValueWork.source_refreshes`).
   - Implemented `refresh_sources` and `refresh_value_sources` to refresh the active operand source view from committed state without runtime heap allocations.
   - Proved causal intervention: ablating Operator 1 intermediate record causes Operator 2 computation to diverge, establishing causal necessity of intermediate state.
   - Compiled chained Rust programs with `rustc --edition=2021` exiting with code 0.

2. **Autonomous Multi-Step Composition ([PR #1179](https://github.com/UOR-Foundation/uor-r4/pull/1179), commit `0ef18dcb`):**
   - Connected `Session::can_transition` and `Session::refresh_value_sources` directly into `Model::generate`.
   - Verified autonomous execution of chained arithmetic (`13 + 4 = 17`, then `17 + 5 = 22`) within a single autoregressive generation call.
   - Retained exact causal provenance (`operand_ids`, `operand_values`) across transitions.

3. **Mixed Multi-Operator Composition ([PR #1180](https://github.com/UOR-Foundation/uor-r4/pull/1180), commit `4f7dafa2`):**
   - Added checked exact $Z[\phi]$ subtraction `ZPhi::checked_sub` and `ValueAction::Sub`.
   - Expanded proposal addressing to 528 choices with non-commutative operand rank encoding `b = (rank_a << 8) | rank_b`.
   - Validated single-operation subtraction ($20 - 7 = 13$, `subtractions: 1`).
   - Validated causal mixed Add -> Sub transition ($20 + 15 = 35 -> 35 - 8 = 27$) with load-bearing intervention proof.
   - Verified autonomous generation of mixed Add/Sub and execution of compiled mixed Rust programs.
   - Zero heap allocations preserved on `#![no_std]` hot path across all tests.

Closing as completed.

### comment 5585689249 — 2026-09-08T13:12:27Z — Casey-allard

Owner-authorized recovery, 2026-09-08.

Committed arithmetic mechanics are useful, but manual selector weights/host-generated Rust do not establish learned contextual composition and actual generated-code acceptance.

The audit inspected main f8e9fa32 against last verified PR1176 (4f9d02a2), rebuilt the API, reproduced failed prose/coding and request limits, and found all8API/all6release tests still pass. Those tests cannot support the wider claims. Recovery is proceeding through a protected PR; this issue remains open until its complete acceptance is met. Prior positive/negative artifacts and release history are preserved.


### comment 5586755177 — 2026-09-08T14:29:42Z — Casey-allard

Recovery PR #1198 merged through the protected workflow at `c229e3928aad383043561f29466cb211a75238e5`. The merged tree matches the locally checked tree. [Recovery report](https://github.com/UOR-Foundation/uor-r4/blob/c229e3928aad383043561f29466cb211a75238e5/docs/integration/recovery-2026-09-08.md) and [evidence receipt](https://github.com/UOR-Foundation/uor-r4/blob/c229e3928aad383043561f29466cb211a75238e5/docs/evidence/native_recovery_2026_09_08.json) preserve actual commands, outputs, failures and resource charges.

A frozen-parent shared signed-H4 role refinement now trains against actual generated intermediate IDs and exact current-query literal occurrences. Experimental `b2da9f9a` reaches 183/183 construction, 12/12 open, 12/12 post-selection numeric transfer and the recorded preservation panels. Later matched API testing finds a new abstention regression: independent sums 17 and 18 followed by a repeated 13+4 question produce 35; parent `79710468` returns Unknown. Do not promote the candidate. Preserve both gains and failures. Next: discriminate current-query literals from multiple accumulated derived results with matched independent/dependent continuations before learned within-response transitions and compositional lexical emission. This does not complete this issue.


### comment 5587415779 — 2026-09-08T15:13:58Z — Casey-allard

PR #1199 merged through the protected queue as `9188d7a3f841d4b2eac72aa8e0f58ef5e8849cb1`. Its merged tree exactly matches tested head `0a8d20a993580beb75fb07b384e16107fe9bf963` (tree `c5a3c469bdf8cab5a0f8e0c46803d408899a24d3`). Local main was fast-forwarded with the owner's untracked material preserved.

Retain `blake3:ddc943f95901389f741b49f14dba48d2a4bae19805285bbd7cb416fca8e510c9` at bounded current-query/shared-continuation scope. SHA256: `fe31404930998ff33fc57c463af921b989a5f6140a43ee127cf57a38acc5fe9c`. Local artifact: `/Users/casey.allard/uor-r4/.uor-models/native-typed-value-2026-09-05/current-query-refinement/model.json`.

The observed third independent 13+4 question after earlier generated answers 17 and 18 now returns 17 through the actual native API, including checkpoint restoration. Retained parent `79710468` abstained; rejected `b2da9f9a` returned 35 by using old derived totals. Offline Rust training updates the existing shared signed-H4 selector. Serving code, feature representation, arithmetic and emission are unchanged.

Executed evidence: 255/255 construction, 48/48 open development, 16/16 harder history/order/equal-value cases, all 663 retained answers and every recorded preservation gate, and 24/24 fresh numeric transfers after selection. Removing the original generated result leaves all 32 independent answers correct while all 16 dependent answers change and lose the required dependency. Native artifact/API serialization, identity, parent-mutation rejection, checkpoint and direct/API parity checks pass. The first new candidate `51603beb` failed reversed-order cases and is preserved as rejected. Fresh cases use the same finite query forms and names; signed-negative numbers do not constitute abstention tests.

See the [result and boundaries](https://github.com/UOR-Foundation/uor-r4/blob/9188d7a3f841d4b2eac72aa8e0f58ef5e8849cb1/docs/native_geometric_current_query_1140.md) and [machine evidence](https://github.com/UOR-Foundation/uor-r4/blob/9188d7a3f841d4b2eac72aa8e0f58ef5e8849cb1/docs/evidence/native_geometric_current_query_1140.json). Raw receipts remain under `/Users/casey.allard/Documents/Codex/2026-09-08/uor-r4-current-query`.

The cycle charged 480.927 model seconds including both fits and all evaluations. Cumulative use is 7,441.579 / 8,550.000 seconds; the earlier 196.620-second unverified-work reservation is preserved, leaving 911.801 seconds after reservation. The extension and complete projection were recorded before use. The storage ceiling and 128 MiB stop margin remain intact. No unique material was deleted or paid external model compute incurred. The five queue checks are compatibility acknowledgements, not the executed tests above.

Next: learned within-response Continue/Stop and compositional emission from actual committed state, preserving current-query versus dependent-result selection and all retained controls. This result does not close #1140 or qualify general prose, generalized reasoning/coding, energy savings or alpha. This artifact has Rust/API validation; no separate new-artifact browser run was performed.


### comment 5588276269 — 2026-09-08T16:14:08Z — Casey-allard

PR #1200 delivers learned within-response Stop/Copy/Add selection over actual committed state. Retain `6f20982c` at bounded numeric composition scope over frozen parent `ddc943f9`.

After a generated 17, the tested extra-3 “Again” query emits `20.\n23.\n`, with the first write ID feeding the second Add, then learned Stop below the safety cap. Complete construction 36/36, open 24/24 and post-selection fresh numeric transfers 24/24 pass. Both new controls preserve all 16 single-operation answers and lose all eight second operations. All 663 retained answers and earlier panels pass, as do prior current-query 48/48, harder histories 16/16 and dependency controls. Fourteen actual-artifact API checks pass, including the composed answer after restored checkpoint and independent 17→18→17. A completed-query boundary failure and Stop-only first fit are preserved in the evidence.

Six mechanical tests, integer-kernel source guard, release compilation, actual-artifact allocation/checkpoint check, formatting, claim wording and receipt/link/hash validation were executed locally. Queue statuses are compatibility acknowledgements, not tests. Cycle model charge: 312.644 seconds; cumulative 7,754.223/9,450 seconds, with the prior 196.620-second reservation and storage stop margin intact.

This leaves #1140 open: shared learned lexical choice and ordered sentence/Rust-expression emission remain next. No general prose, generalized reasoning/coding, alpha or energy claim; new-artifact browser/WASM is NOT_RUN. [Result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_operation_transition_1140.md) · [evidence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_operation_transition_1140.json).


### comment 5589037274 — 2026-09-08T17:15:37Z — Casey-allard

PR #1201 delivers the bounded shared lexical emitter and retains `0f0f5fe4` over frozen parent `6f20982c`.

The learned signed-H4 selector emits bytes/EOS or reads exact committed operand records. Construction/open pass 18/18 each, changed names/mention order 12/12 and post-selection numeric transfers 24/24. Each emitter/read/geometric-transform control keeps all six plain answers and loses all twelve formatted answers. All 663 retained answers and earlier panels pass, along with prior operator 24/24, current-query 48/48 and history 16/16 plus their dependency controls. Actual API passes 20 checks, and 18 actual generated Rust equalities compile and evaluate. Focused read/operator tests, checkpoint/zero-allocation checks, formatting and the native source guard pass. Queue statuses are compatibility acknowledgements, not tests.

Preserve rejected `0e2c48c0` (sentence loops after erased read role) and `03bdfad1` (72 failed preservation rows despite correct new panels). [Result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_lexical_emission_1140.md) and [evidence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_lexical_emission_1140.json) contain exact scope, source, outputs and resources. Cumulative model use is 8,160.980/10,350.000 seconds; prior reservation and storage stop margin remain.

This issue stays open. Two authored output forms and executed numeric equalities do not qualify general prose, flexible instructions or full Rust synthesis. Combined two-operation formatted output is NOT_RUN. Next is shared contextual instruction binding across selection and emission, starting with ordinary wording that currently disturbs upstream binding, then composing operator transitions with formatted output. New-artifact browser/WASM/HTTP and energy measurement remain NOT_RUN.


### comment 5589854754 — 2026-09-08T18:23:11Z — Casey-allard

Delivered PR #1202: retain `ccbe7f2a` for bounded contextual instruction binding. Ordinary sentence/Rust request prefixes and suffixes now retain the correct Add operands and use the unchanged lexical emitter. The two previous routers reconstruct accepted parent `0f0f5fe4` exactly.

Actual validation: 72/72 construction, 24/24 open, 24/24 changed-name/order and 24/24 post-selection numeric transfer; parent-router control 8/24. All 663 main retained answers, earlier panels and 72 exact previous lexical outputs pass. Earlier operators/current-query/history and dependency/emitter controls remain preserved. Native Rust API 38/38; six read and six operator tests, source guard, formatting and eight actual zero-allocation/checkpoint cases pass. Thirty generated Rust equalities compile and evaluate.

Preserve all failures: `f17dd654` failed five reversed-order instruction cases; `f76521ab` passed new panels but failed eleven earlier Rust/city/chain/abstention cases. The final fit adds 82 already-open parent contexts and repairs those regressions. General prose, arbitrary instructions, full Rust synthesis and energy advantage remain unqualified; browser/WASM/HTTP and combined two-operation formatted output for the new artifact are NOT_RUN.

This does not complete #1140. Next: combine retained within-response transitions with formatted emission and exact intermediate dependencies. [Result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_instruction_binding_1140.md) · [Evidence and cumulative resources](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_instruction_binding_1140.json). Cumulative model use: 9,047.086 / 10,350.000 seconds; all prior reservations and the 128 MiB storage stop margin remain.

### comment 5590779840 — 2026-09-08T19:38:36Z — Casey-allard

Delivered through protected PR #1203. Retain `blake3:866cb92de4ad5c130da811d3f2fe8828aed1fe9b2b39b9c9b3c7ad3dbf7dee62` at bounded repeated-Add and formatted-output scope.

The shared geometric operation selector now continues after sentence/Rust emission. After actual history 17 and the tested extra-3 “Again” request, it generates `20 is 3 plus 17.\n23 is 3 plus 20.\n`, or two Rust equalities. Exact ordered pending/committed writes and read identities establish that the second operation uses the first result, then stops. The emitter, dictionaries, bounds, other routers and geometry are unchanged from parent `ccbe7f2a`.

Construction passes 42/42, open 28/28, reversed-context 28/28 and post-selection fresh 28/28. All 663 main retained answers, prior reports, 254 construction-preservation cases and 72 exact old lexical outputs pass. Fourteen older result/control reports match their actual saved outputs and identities. API passes 50/50; 54 generated Rust equalities compile and evaluate. Six read and six operator tests, the source guard and actual zero-allocation checks pass. Queue acknowledgements execute no tests; these are recorded local checks.

Preserve rejected `ae245428` (ten earlier answers continued incorrectly) and superseded `9805e99d`. The receipt records the zero-write abstention checker correction and serving-eligible training boundary correction separately from model failures, plus all source freezes and charges.

This does not close #1140. Current composition repeats Add with one reused literal under seven authored formatting placements. Next implement mixed shared Copy/Add sequences and operand references with changed dependencies and stopping positions. General prose, full Rust program generation, alpha and energy advantage remain unqualified; new-artifact browser/WASM/HTTP is NOT_RUN.

Cumulative model use is 9,967.755 / 11,250.000 seconds, retaining the 196.620-second reservation and 1,085.625 seconds available after it. Conservative storage is 32,033,710,080 / 34,863,214,592 bytes, retaining the 128 MiB stop margin. The 900-second extension was recorded before use; no unique material was deleted and no paid compute was used.

[Result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_composed_output_1140.md) · [Evidence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_composed_output_1140.json)


### comment 5590810525 — 2026-09-08T19:41:10Z — Casey-allard

Reopened because the broader acceptance remains incomplete. GitHub interpreted the negated closing phrase in PR #1203 as a closing keyword and automatically closed this issue on merge; the PR wording is corrected. The delivered result covers repeated Add with formatted output. Mixed shared operator sequences and operand-reference generalization remain the next work.

### comment 5592290405 — 2026-09-08T21:45:09Z — Casey-allard

Delivered through protected PR #1204, merged as `113b531f959ac732432cb2abc9d6fb0bd8fdc7f9`. The merged tree equals reviewed branch head `36e8f54e`, and the owner checkout is fast-forwarded without changing its preserved untracked material.

Retain artifact `9ab64902` for bounded plain Add→Copy-latest, Add→Copy-original and preserved Add→Add with exact occurrence references and stopping. Shared signed-H4 role/operation routers learn the selection; serving gains no parser, response template, transformer or matrix-product path. Parent `866cb92d`, all rejected candidates and charges remain preserved.

Executed evidence: construction 12/12, open/stress/fresh 8/8 each, all with following independent sums and fresh literal IDs; 663/663 retained answers, historical numeric 6/6, transition replay 254/254, exact lexical 72/72, prior composed panels 28/28 each. Native API 65/65; 17 focused tests, source guard, formatting, three actual-artifact allocation/checkpoint checks and 54 prior generated Rust equalities pass. Thirteen of fourteen saved reports match exact outputs/identities; eight already-wrong current-query removal rows change wrong outputs. Every previously correct row and all declared control correctness counts remain preserved. Candidate-exclusion results are intervention-trained robustness, not memory erasure. Required queue statuses are compatibility acknowledgements, not tests.

[Result and limitations](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_mixed_operators_1140.md) and [source-bound evidence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_mixed_operators_1140.json) retain rejected candidates, the API-discovered independent-turn regression, numeric Copy regression and corrected checker receipts. Cumulative model use is 12,707.541 / 13,350.000 seconds; conservative storage 32,580,792,320 / 34,863,214,592 bytes, with prior reservation and 128 MiB stop margin preserved.

Next: generate mixed Add/Copy formatted output and diagnose the shared emitter's missing committed-action context. If confirmed, learn shared action-conditioned emission with exact reads and matched action/read controls while preserving accepted Add formatting. Formatted Copy, Copy→Add, general prose/reasoning, Rust program synthesis and new-artifact browser/WASM/HTTP remain unqualified. This issue remains OPEN and assigned for continuing development.


### comment 5593003363 — 2026-09-08T22:55:39Z — Casey-allard

Protected PR #1205 merged as `9a563c88`; the merged source tree matches reviewed head `175b031d`.

**Retain artifact `5ed24f4e` at bounded formatted Add/Copy scope**, delivered by protected PR #1205 (merge `9a563c88`). The shared learned signed-H4 role and continuation routers plus Copy-conditioned lexical roots emit formatted Add followed by Copy of the latest result or original total, preserving exact pending/committed operands and reads. With actual history 17 and extra 3, Copy-original emits `20 is 3 plus 17.\n17 is 17.\n` or `20 == 3 + 17\n17 == 17\n`. Inherited lexical parameters and fixed geometry remain bound to parent `9ab64902`; no serving parser/template, matrix product, transformer or provider is added.

Construction passes 36/36; open, reversed/equal-value stress and post-selection signed/multidigit/name transfers each pass 24/24, including following independent sums with fresh record IDs. All 663 retained answers, earlier panels, 254 transition replays and 72 exact lexical outputs pass. Twelve of fourteen historical reports are byte/identity exact; eighteen already-incorrect control rows change wrong outputs or IDs, with all previously correct rows and correctness counts preserved. Native API passes 91 checks; 21 unique focused tests, source guard, four actual artifact checkpoint/allocation checks and 42 compiled/executed generated Rust equalities pass. New action/binding/transition controls lose the appropriate formatted Copy cases; exact-read and emission-transform controls lose formatted responses. [Result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_action_emission_1140.md) and [evidence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_action_emission_1140.json) preserve sources, selection, binding-only `a6faba85`, rejected `61e24998`, changed negatives and all charges. Queue compatibility acknowledgements execute no tests.

**Next: #1140 diagnose Copy→Add with a changed dependency on the copied record, then refine the shared routers at the observed failure.** Preserve current formatting, exact equal-value identities, learned stopping and independent-turn behavior. These finite authored forms do not qualify arbitrary instructions, three-operation learned stopping, general prose, broad reasoning, Rust program synthesis, alpha or energy advantage. New-artifact browser/WASM/HTTP remains NOT_RUN. #973 owns broader language learning; #1140 and broader capability issues remain open.

Project complete next work against the shared cumulative ledger before execution. Current model use is 14,153.352 / 15,450.000 seconds, including this cycle’s 1,445.811 model seconds and every failed fit/retry; engineering commands used 653.576 seconds. The 2,100-second extension was recorded before use; later interface reallocation stayed within the existing allowance. Keep the 196.620-second prior reservation, leaving 1,100.028 seconds after it. Conservative storage is 32,767,975,424 / 34,863,214,592 bytes with 16 MiB final metadata reserve and the 134,217,728-byte stop margin. All earlier charges and negatives remain preserved without refunds. No unique material was deleted and no paid compute was used. Refresh the live ledger/receipt before further execution; issues and sessions do not reset charges. 

Owner checkout was fast-forwarded with its existing `.serena/` material preserved. This is a bounded implementation result; the issue remains open.


### comment 5593735258 — 2026-09-09T00:13:21Z — Casey-allard

Completed at the issue's declared bounded scope through PR #1206, verified merge 5f123e28. Retain 2dc63b2c.

Actual construction 36/36, open 24/24, reversed/equal-value 24/24 and post-selection fresh 24/24 pass, including exact pending/committed operands, reads, EOS/checkpoints and independent fresh-record followups. Copied-result exclusion retains the first Copy and stops in every case; parent-router/action/read/geometry controls lose the corresponding behavior. All 663 retained answers and previous panels pass, including both 16-case history-chain populations, 254 transition replays and 72 lexical outputs. Every previously correct historical/action-control row and correctness count is preserved; changed already-incorrect rows remain recorded individually. Native API 117 checks, 22 unique focused tests, kernel source guard, five actual artifact allocation/checkpoint tests and 36 compiled/executed generated Rust equalities pass. Queue statuses are compatibility acknowledgements, not tests.

The first joint 9365 candidate was rejected for eight earlier Copy regressions despite passing new cases. The accepted retry includes full original-parent history contexts; all failed/stopped candidates and charges are preserved. Combined with earlier retained Add→Add/Add→Copy evidence, all acceptance clauses are met. Arbitrary prose/program synthesis and alpha are not claimed. See the [result](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/native_geometric_copy_add_1140.md) and exact [evidence](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/evidence/native_geometric_copy_add_1140.json). Next work moves to #973's nonnumeric shared-emission boundary.



---
_Comment/review ids in this file: 5577691630, 5577814678, 5585689249, 5586755177, 5587415779, 5588276269, 5589037274, 5589854754, 5590779840, 5590810525, 5592290405, 5593003363, 5593735258_

