# #1102 — Implement and qualify the frozen native learned-reference bridge

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-03T14:11:37Z
- **Updated:** 2026-09-03T16:16:18Z
- **Closed:** 2026-09-03T16:15:40Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1102
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Current result — 2026-09-03

**NATIVE_REFERENCE_PRESERVED; independent result review accepted.** [PR #1104](https://github.com/UOR-Foundation/uor-r4/pull/1104) delivers the opt-in native reference, immutable artifact/exporter, trusted loader and retained empirical evidence. Merged through the protected queue at `85d9eb8beca2a59ccda47e290afd483f7838982c` on 2026-09-03; #1102 is **closed**. The merged tree exactly matches the independently reviewed delivery head `0388f7e4cc6cfc3d2041307683564eda8e473065` and is present in origin/main. The accepted reader/core, vocabulary/query forms and four-fact context remain fixed.

Both initial runtimes and their fresh-process replays returned **320/320 correct answers, 4,480/4,480 correct consumed roles and 16/16 exact typed refusals**. All four complete f32 tensors satisfy the frozen absolute 1e-5 criterion; maximum difference `4.76837158203125e-6`. Within each runtime the row JSONL and full tensor files replay byte-for-byte. Two independent exports match, and all 12 zero-forward loader gate cases meet their frozen criteria.

Independent review decoded all 6,727,680 retained f32 values and checked the exact discrete/parser/refusal results, artifact/source/runtime identities, loader outcomes and resource accounting. Disposition: `ACCEPTED_BOUNDED_NATIVE_REFERENCE_QUALIFICATION`. Result audit SHA256 `5e75c5a3407d91bf14a6ec8d57981a72cfe110cdaf1ddfd0c71f6ea0a07974e8`; qualification receipt SHA256 `61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369`.

The sole offline build succeeded in 175.932499 seconds with 682,391,353 observed output bytes. The sole comparison used 1,280 forwards, five successful engine loads, ten successful model-state loads and zero updates/withheld reads. Its final recorded wall observation was 8.809784 seconds, peak combined sampled RSS 602,390,528 bytes, and complete retained ledger 75,039,076 bytes. **Both one-attempt envelopes are consumed; do not rerun them.** The earlier two offline metadata failures remain recorded. Their local cache miss was resolved by restoring 25 exact Cargo.lock-matching archives without changing pins or runtime assumptions.

- [Execution, resources and limits](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_execution.md)
- [Independent result review](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_result_review.md)
- [Machine audit](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_evidence/result-review.json)
- [Qualification and later service handoff](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_evidence/qualification-handoff.json)
- [Complete retained-file identities](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_evidence/retained-evidence-index.json)

This is measured preservation for one exact native artifact/binary/runtime and already-observed authoring stratum. It establishes no independent semantic holdout, general parsing, larger context, generation, reasoning/coding, mathematical equivalence, cross-platform portability, geometry advantage or final integer/table runtime. Successful ordinary `qualify()`/serving execution remains **NOT_RUN**. The measured binary offers research modes; a newly linked service needs its own truthful binary/runtime qualification. #1094's Python preservation, #1079's weak control and #1082's descriptive limits remain; #973 stays open and #954 blocked. No unique evidence or user material was deleted. Broad QA remains dormant; required CI acknowledgements are transport only.

**One next action after protected closure:** #1084 freezes the Rust service/API, artifact ownership and capability/lifecycle ADR for `answer_four_fact_raw_text/v1`. It is a separate task and is not implemented here. #1083 and #1087 retain their separate integration/lowering obligations.

## Historical checkpoints and frozen scope

The records below preserve what was known before dependency restoration and the admitted outcome. Their parked/NOT_RUN/next-action statements describe those earlier checkpoints, not the current result above. The original scope, criteria and caps are preserved.

### Prebuild checkpoint — 2026-09-03

**PREBUILD_DEPENDENCY_UNAVAILABLE.** [Draft PR #1104](https://github.com/UOR-Foundation/uor-r4/pull/1104) preserves the opt-in native implementation, exporter/loader/comparison harness and independent source reviews at head `1c6be403a9de732dcbb1d9aad8fda66c2b18579c`. [Decision record](https://github.com/UOR-Foundation/uor-r4/blob/ccabce9f/docs/r4_native_bridge_1102.md), [exact prebuild accounting](https://github.com/UOR-Foundation/uor-r4/blob/ccabce9f/docs/r4_native_bridge_1102_prebuild.json).

Two offline dependency metadata queries failed on missing `aho-corasick 1.1.4`; the extracted source tree does not verify the required archive SHA256 `ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301`. No archive, dependency pin or runtime contract was substituted. Independent review permits draft preservation only, with no merge or execution approval.

Build, exports, loader calls, model loads/forwards, fitting, comparison and replay remain NOT_RUN / zero. Both execution envelopes are unconsumed; no concrete release is admitted. #1102 remains **open, parked and unassigned**; the draft says References #1102. Prior #1094 Python qualification, #1079 weak control and #1082 descriptive limits remain unchanged; #973 open and #954 blocked.

**One next action:** restore exact Cargo.lock-matching dependency archives locally, then resume this existing draft's unconsumed offline build and independent concrete source/binary/runtime/input/output release review. No comparison before that release is accepted. Do not queue the draft or create a second implementation.

The prior handoff and frozen scope below remain historical authority; this checkpoint supersedes their activation wording only.

## Accepted protected contract handoff — 2026-09-03

Parent #1086 is closed through protected PR #1103 at `93613bf82782ca78406fe2739dcc8d9e1d0f2b9e`. [Normative specification](https://github.com/UOR-Foundation/uor-r4/blob/93613bf82782ca78406fe2739dcc8d9e1d0f2b9e/docs/r4_native_reference_1086.md), [machine contract](https://github.com/UOR-Foundation/uor-r4/blob/93613bf82782ca78406fe2739dcc8d9e1d0f2b9e/docs/r4_native_reference_1086_contract.json), [independent review](https://github.com/UOR-Foundation/uor-r4/blob/93613bf82782ca78406fe2739dcc8d9e1d0f2b9e/docs/r4_native_reference_1086_review.md). Contract SHA256 `e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`; evidence commit `d56d952d360ec167343903c7ff3f6149218410f1`. Native behavior is **NOT_RUN / Unproven**. This issue remains parked and unassigned until separately activated. This handoff is not an export or execution release.

## Parked successor to #1086

This issue owns implementation/export and the separately released empirical bridge for `R4LearnedReferenceV1`. It remains parked and unassigned. Creation is a handoff, not an execution release. #1086 must be protected-merged before activation.

Accepted design: the #1086 native-reference prose, machine contract, operator/source audits and independent specification review. The specification preserves the accepted #1094 reader/core/codec/R4 identities and raw-byte four-fact operation; it provides no native artifact or native behavior evidence. The exact protected revision and contract digest will be attached at #1086 closure.

## Deliverable and admission order

1. From current origin/main and the live graph, implement only the opt-in `uor-r4-core::learned_reference` and `uor-r4-api::learned_reference` boundary defined by #1086. Preserve historical R4G1/default serving/final integer runtime. Implement the exact 21-component container, constructor-free Safetensors export, trusted loader, tied head, input/refusal/result/capability schemas and fixed scalar f32/f64 numerical profile.
2. Freeze an independent, concrete implementation/export/comparison release before asset deserialization or outcomes. Bind exact source closure, revisions, rustc/lockfile/flags/binary, libm0.2.16 profile, runtime/hardware, accepted assets/state identities, the original 320 authoring and 16 refusal files with independent references/owner, all output paths and the durable one-attempt budget supervisor. Do not reuse the consumed #1094 envelope or access its resealed withheld population.
3. After independent release approval, perform two identical bounded exports and the separately frozen zero-forward loader mutation/identity gate. Preserve all mutation copies and rejected evidence. No model scoring after a failed gate.
4. Only if admitted, compare pinned Python and native at B=1 on those known authoring fixtures, then replay each once in a fresh process. Use the exact discrete criteria, all four full tensors, per-row/per-tensor maximum absolute error <=1e-5 with relative tolerance zero, full reference errors, 320 answers/4480 roles per arm and exact within-arm replay from #1086. Persist full tensors for both arms and phases. A bad/non-reproducing Python reference is unavailable, not a native mismatch.
5. Apply the frozen outcomes and caps: 1280 logical forwards, four scoring/replay engine loads plus twelve zero-forward loader-gate attempts (eleven rejected, one successful engine; five successful engine loads total), zero fit/refusal forwards; 120s export/integrity +120s comparison +120s replay, 360s cumulative, 3GiB combined RSS, 128MiB complete bytes; separate offline native build <=15min and <=2GiB new output. One consumed envelope, stop overrides completion, no adaptive tolerance/backend/batch changes or retries.
6. Independently review actual outcome/evidence and deliver through protected PR/queue. Update native issues, roadmap, claim/evidence records and index, then stop. Positive qualifies only this exact native reference for a later #1084 handoff; mismatch preserves Python qualification and parks a separately scoped repair. Unavailability or budget stop preserves partial evidence without promotion.

## Limits

No new training, population, general parser, larger context, generation, reasoning/coding claim, provider fallback or UI/HTTP wiring. #1079 weak-control and #1082 descriptive findings remain; #973 open and #954 blocked. #1083 typed UOR and #1087 final lowering remain separate. This issue is not blanket authorization to expand the design or run dormant QA.

Concrete next action when explicitly activated: bind the reviewed #1086 contract to a small native implementation and an independently reviewed export/input/runtime release, before any new model work.


## Comments (5)

### comment 5527385937 — 2026-09-03T14:36:32Z — Casey-allard

## Active native implementation and bridge contract — 2026-09-03

The user's next-step instruction activates #1102 only from current protected origin/main `93613bf82782ca78406fe2739dcc8d9e1d0f2b9e`; #1086 is closed. Casey-allard owns isolated branch `issue-1102-native-bridge`. The accepted #1086 contract SHA256 is `e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115` and is not revised by implementation convenience.

Decision: does the exact constructor-free export, native loader and scalar Rust reference preserve the pinned Python path on the original 320 authoring rows and sixteen refusals? The implementation first binds the full source, input/reference, toolchain/binary/runtime and output/release identities for independent review. No asset deserialization or outcome access before that release is accepted. The original #1094 envelope is consumed and is not reused; its withheld population remains sealed.

Reachability: all 320 admitted raw requests reach the same accepted learned reader/core/two-stage R4 operator path. The cheap hard gate is duplicate export identity, exact decoded-state/codec/frame reconciliation and the twelve-call zero-forward loader gate in #1086. A failed gate prevents scoring. The four full tensors contribute 26,910,720 retained bytes over both arms and replay; the declared fixed artifact payload is 2,160,742 bytes. Actual complete pre-admission storage must fit 128 MiB.

The numerical/discrete/floor/refusal/replay criteria and distinct positive, mismatch, unavailable and budget-stop consequences are exactly #1086's. B=1 both arms; 1,280 total logical forwards; zero fitting/refusal forwards. Four scoring/replay engines plus one successful loader-gate engine; twelve gate attempts, eleven rejected, bounded partial model-state work. Native build is one preparation envelope <=15 minutes / <=2 GiB new build output, offline release; export/integrity, execution and replay each <=120 seconds, <=360 cumulative, combined RSS <=3 GiB. No adaptive tolerance, batch, backend, population, time reset or run retry.

Named checks: source/contract reconciliation, independent implementation and exact-release review, one budgeted native release build (compiler errors may be repaired only within that same unreset preparation budget), static schema/hash/accounting checks, the declared loader/identity and matched behavior/replay checks after admission, claim wording and diff whitespace, and independent final outcome/delivery review. Other QA remains dormant. No UI/service wiring, final lowering, new architecture or unrelated science is activated.

Positive delivers only the bounded native-reference qualification and hands the exact artifact/operation to #1084. A mismatch retains Python qualification and parks a separately scoped repair; unavailability or a budget stop retains all partial evidence without promotion. #973 remains open and #954 blocked. Storage was reviewed before worktree creation (45,752,488 KiB free); preserve user changes, source/evidence, models, sealed inputs and referenced worktrees. No cleanup is planned.


### comment 5528168708 — 2026-09-03T15:38:25Z — Casey-allard

## Current checkpoint — 2026-09-03

**PREBUILD_DEPENDENCY_UNAVAILABLE.** [Draft PR #1104](https://github.com/UOR-Foundation/uor-r4/pull/1104) preserves the opt-in native implementation, exporter/loader/comparison harness and independent source reviews at head `1c6be403a9de732dcbb1d9aad8fda66c2b18579c`. [Decision record](https://github.com/UOR-Foundation/uor-r4/blob/ccabce9f/docs/r4_native_bridge_1102.md), [exact prebuild accounting](https://github.com/UOR-Foundation/uor-r4/blob/ccabce9f/docs/r4_native_bridge_1102_prebuild.json).

Two offline dependency metadata queries failed on missing `aho-corasick 1.1.4`; the extracted source tree does not verify the required archive SHA256 `ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301`. No archive, dependency pin or runtime contract was substituted. Independent review permits draft preservation only, with no merge or execution approval.

Build, exports, loader calls, model loads/forwards, fitting, comparison and replay remain NOT_RUN / zero. Both execution envelopes are unconsumed; no concrete release is admitted. #1102 remains **open, parked and unassigned**; the draft says References #1102. Prior #1094 Python qualification, #1079 weak control and #1082 descriptive limits remain unchanged; #973 open and #954 blocked.

**One next action:** restore exact Cargo.lock-matching dependency archives locally, then resume this existing draft's unconsumed offline build and independent concrete source/binary/runtime/input/output release review. No comparison before that release is accepted. Do not queue the draft or create a second implementation.

The prior handoff and frozen scope below remain historical authority; this checkpoint supersedes their activation wording only.



### comment 5528258147 — 2026-09-03T15:45:39Z — Casey-allard

## Preparation resumed — cache clarification

The earlier Cargo error was a local dependency cache miss under `--offline`, not an offline client service. The frozen contract requires the native build to remain offline; it does not prohibit restoring exact locked prerequisites beforehand. Independent review confirmed this interpretation.

The selected dependency query downloaded 25 archives (2,749,985 bytes) in 0.775 seconds. Every new archive matched its Cargo.lock checksum; Cargo.lock is unchanged. The same query with `--offline --locked` now succeeds. Resolved libm remains 0.2.16 with only `default` and `arch`, matching the frozen archive SHA256. Both metadata outcomes are preserved.

#1102 is active and assigned again. Next is the same unconsumed 900-second/2-GiB release/offline build, then independent concrete release review before any export or model work. The existing draft remains unqualified and unmerged. No second task, new experiment, or changed dependency/input/tolerance is introduced.


### comment 5528418573 — 2026-09-03T15:58:03Z — Casey-allard

## Exact one-attempt comparison admission — before outcomes

The one offline release build passed in 175.932499 seconds with 682,391,353 bytes in its final observed-generation ledger, inside the unchanged 900-second/2-GiB limit. Binary SHA256 `d423d8d3c3acd2d1c6215c21206e1bec7583e4dd37e84f30f70f79e77c40d53f`; source revision `1c6be403a9de732dcbb1d9aad8fda66c2b18579c`. Metadata-only runtime check reports the required FPCR settings. This is build/runtime preparation evidence, not model parity.

Independent review has now accepted exactly one attempt under frozen #1086 contract SHA256 `e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115`:

- Concrete release SHA256 `2c3c2f73eb6cf804eb69b2afb0f979ae623a512ca0492e47df2af70d6cbaca8b`.
- Independent acceptance SHA256 `88c8f8b4223ab83cca072b263c6a4b2febe542173c040fbf9f73bbc6143f4647`; status `ACCEPTED_FOR_ONE_NATIVE_BRIDGE_ATTEMPT`.
- Original 169-file Python reference closure is retained byte-for-byte. All 236 launch-file identities and 494 build-source identities were checked; no source/input pin changed.
- Original authoring raw/reference SHA256s remain `58ecc23bf6a587508355266e039bb0cb3bdbf7597204fec40aee2c34d7f89503` / `d6740ea56caa6e7df9dea7be4b9636880056629549c5e72783ba988a8ca3c660` (672,846 combined bytes). No withheld population is opened.
- Exact minimal launch environment, binary, runtime, paths, source closure, immutable output root and both OS access profiles are bound. Actual byte checks and four denied-path probes remain inside admission.

**Frozen decision:** two independently constructed identical exports; eleven rejected and one valid zero-forward loader gates; then 320 original valid rows and 16 original refusals at B=1 in both pinned Python/native arms, each with one fresh-process replay. Compare all four full f32 tensors using per-row/per-tensor max absolute error <=1e-5 and relative tolerance zero, exact discrete/parser/answer/14-consumed-role criteria, full retained tensors and exact within-arm replay.

**Caps:** 120 seconds each for export/integrity, execution and replay; 360 seconds cumulative; 3-GiB combined RSS; complete 128-MiB ledger; at most 1,280 logical forwards, five successful engine loads / ten successful model-state loads, zero fitting or refusal forwards. Conservative planned bytes are 123,767,303, leaving 10,450,425 under the cap. One admission consumes this envelope; stop overrides completion, with no retries, tolerance/backend/batch/input changes or clock reset.

**Consequences:** preserved qualifies only this exact bounded native reference for a later #1084 handoff. Mismatch preserves the Python qualification and failed native evidence for a separately scoped repair. Missing/invalid reference is unavailable, with no native behavior verdict. A resource/interruption stop preserves partial evidence and provides no renewed allowance. No outcome or merge acceptance is asserted here; all earlier weak/negative/descriptive results remain.

The next action is the single supervised invocation. Source, comparison release and reviews will be attached to the existing protected PR with its actual outcome. #1102 is the only active task; no successor is activated.


### comment 5528649640 — 2026-09-03T16:16:18Z — Casey-allard

Delivered and closed through protected [PR #1104](https://github.com/UOR-Foundation/uor-r4/pull/1104), merge [`85d9eb8beca2a59ccda47e290afd483f7838982c`](https://github.com/UOR-Foundation/uor-r4/commit/85d9eb8beca2a59ccda47e290afd483f7838982c). The merged tree exactly equals reviewed head `0388f7e4cc6cfc3d2041307683564eda8e473065`; origin/main and GitHub CLOSED state were independently read after merge.

The frozen DoD is complete:

- Opt-in CPU Rust learned reference, exact 21-component container, direct-byte exporter, trusted loader and raw-text operation implemented without changing the accepted reader/core, known vocabulary/query forms or four-fact context.
- The separately admitted comparison returned **NATIVE_REFERENCE_PRESERVED**: each initial/replay arm produced 320/320 correct answers, 4,480/4,480 correct consumed roles and 16/16 exact typed refusals. All four full tensors met 1e-5 absolute tolerance; maximum observed difference was 4.76837158203125e-6. Within each runtime, row JSONL and complete tensor files replayed byte-for-byte. Both exports matched and all twelve loader gates satisfied their criteria.
- Independent retained-evidence review accepted **ACCEPTED_BOUNDED_NATIVE_REFERENCE_QUALIFICATION** after decoding all 6,727,680 f32 output values and checking source/artifact/runtime identities, discrete results, loader evidence and resource accounting. [Execution and limits](https://github.com/UOR-Foundation/uor-r4/blob/85d9eb8beca2a59ccda47e290afd483f7838982c/docs/r4_native_bridge_1102_execution.md), [independent result review](https://github.com/UOR-Foundation/uor-r4/blob/85d9eb8beca2a59ccda47e290afd483f7838982c/docs/r4_native_bridge_1102_result_review.md), [final delivery approval](https://github.com/UOR-Foundation/uor-r4/pull/1104#issuecomment-5528617148).
- The canonical qualification receipt SHA256 is `61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369`. [Qualification handoff](https://github.com/UOR-Foundation/uor-r4/blob/85d9eb8beca2a59ccda47e290afd483f7838982c/docs/r4_native_bridge_1102_evidence/qualification-handoff.json) binds accepted audit `5e75c5a3407d91bf14a6ec8d57981a72cfe110cdaf1ddfd0c71f6ea0a07974e8`.
- One offline build and one comparison envelope were consumed within their frozen caps. Full tensors, both artifacts, loader mutation evidence, historical failed metadata observations and user/research material remain retained. No fit or withheld access occurred. Current roadmap and claim/evidence records are delivered. The five protected queue acknowledgements are transport only, not model or QA evidence.

Qualification is confined to this exact artifact/binary/runtime and already-observed authoring stratum. Successful ordinary `qualify()` and serving execution remain **NOT_RUN**; no endpoint is delivered. No general language, new semantic holdout, context expansion, generation, reasoning/coding, mathematical equivalence, portability, geometry superiority or final integer/table kernel is established. The earlier Python preservation, weak control and descriptive results remain; #973 stays open and #954 blocked. Neither consumed envelope may be rerun.

**One concrete next action:** #1084 freezes the Rust service/API, artifact ownership and capability/lifecycle ADR for `answer_four_fact_raw_text/v1`. A newly linked service must obtain its own truthful binary/runtime binding. #1084 implementation or new measurement is not part of this completed task.



---
_Comment/review ids in this file: 5527385937, 5528168708, 5528258147, 5528418573, 5528649640_

