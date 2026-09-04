# Current programme map and correctness handoff

**Planning reconciliation: 2026-09-04, #1107 records
`WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`; independent static review accepts
the source-level contract mapping and its explicit unverified boundary.**
Protected delivery closes #1107 only. Original audit baseline:
`UOR-Foundation/uor-r4@e627252e525201815169ffd8364184953a46018d`.
This map supersedes earlier “current” and “next” sequencing prose, not its
measurements. Refresh native GitHub before selecting work. The
[integration plan](../uor_productization_integration_plan.md) and
[adoption record](workflow-adoption.md) describe the research, engineering and
paper lanes adopted through [#1081](https://github.com/UOR-Foundation/uor-r4/issues/1081); the
[claim ledger](claim-ledger.json) preserves their different evidence states.

## Current workbench source candidate and owner decision

[#1105](https://github.com/UOR-Foundation/uor-r4/issues/1105) delivered the
[native four-fact workbench ADR](../adr/0006-native-four-fact-workbench-service.md)
and [machine contract](../r4_service_contract_1105.json). They specify one
dedicated, opt-in `r4-workbench` Rust host, one private same-executable worker
and the first four-fact research-reference shell. They are independently
accepted definitions. [#1107](../r4_workbench_candidate_1107.md) adds the
dedicated crate and candidate source for that host, private worker, private
comparison entry and shell. Source review freezes the result as
`WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`.

Compilation, tests, model loads, qualification calls, forwards, service/HTTP
execution, browser acceptance, numerical behavior and platform behavior are
all `NOT_RUN_BY_POLICY`. Source presence therefore establishes none of those
behaviors. #1084 remains open and unassigned. Its one concrete next action is
for the owner to decide whether to authorize a separate manual qualification
workflow; automated agents neither dispatch nor perform it. Any future private
comparison must still validate fresh host execution permission separately from
the artifact's immutable original export provenance. Neither the consumed
#1102 CLI/coordinator nor its qualification receipt supplies permission or
identity for this source candidate.

## Retained native result

[#1102 native reference](../r4_native_bridge_1102_execution.md) records
**`NATIVE_REFERENCE_PRESERVED`**. The one offline build passed; the separately
admitted comparison passed all twelve loader gates, 320/320 answers,
4,480/4,480 consumed roles and 16/16 refusals in both runtimes and phases.
All four full f32 tensors met the frozen absolute limit `1e-5`; the largest
error was `4.768372e-6`. Both fresh-process replays were exact. The comparison
used 1,280 forwards, zero fitting, 8.809784 seconds and 75,039,076 retained
ledger bytes; 26,910,720 bytes of full tensors remain available for review.

Independent result review accepts the bounded result; protected delivery is tracked in
[PR #1104](https://github.com/UOR-Foundation/uor-r4/pull/1104). Its earlier offline
cache failures remain recorded; exact locked cache restoration resolved that
preparation issue without changing dependencies or the frozen contract.
Both build and comparison envelopes are now consumed. Do not rerun either.

The measured binary provides research comparison modes, not a service endpoint;
its identity cannot be assigned to a newly linked host. Successful ordinary
`qualify()`/`answer()` operation and HTTP/lifecycle acceptance remain separate
integration decisions. #1083 typed integration and #1087 final lowering remain
separate. #973 stays open and #954 blocked.

The [#1086 contract](../r4_native_reference_1086_contract.json), delivered at
`93613bf82782ca78406fe2739dcc8d9e1d0f2b9e`, is unchanged. The observed scope is the
original 320 authoring rows and 16 refusals, B=1 both arms, known vocabulary and
query forms, four facts and the pinned reader/core/R4. It is empirical finite
preservation, with no mathematical proof, semantic novelty, general language,
longer context, generation, reasoning/coding or final integer-kernel claim.

### Retained empirical baseline

The [sole #1094 comparison](../r4_retained_comparison_1094.md) completed
**`CLAUSE_ADAPTER_PRESERVED`**, with exact full and oracle fresh-process replay;
independent review accepted the result. Its frozen population comprises 1,600 valid
rows (320 authoring and 1,280 withheld), 80 refusal rows and 16 boundary controls
across 20 already-observed groups. The same reader/core, known vocabulary/query
forms and four-fact context remain bound. All 1,600 valid inputs, complete compared
tensors and answers matched; all 96 refusal/boundary cases matched with zero model
forwards. These valid rows are related renderings, not independent semantic trials.
Execution plus replay used 6,400 logical forwards; operator wall time was
15.821630625 seconds and the final cumulative resource snapshot was 135.697821334
seconds including the conservative 120-second preparation debit.
The historical 3,465,401-byte ledger remains charged. Withheld permissions have
returned to mode 000; the consumed envelope cannot be rerun.

#1094's bounded scientific DoD is complete and the issue is closed through
[protected PR #1101](https://github.com/UOR-Foundation/uor-r4/pull/1101), merge
`eade29f4b78435e9857936786426bb34e596b301`. Its then-next native contract is now
specified under #1086 above. Native export, historical R4G1 interchange and final
integer/table serving remain separate boundaries; no native model work ran while
specifying that contract.

This positive branch removes externally supplied clause segmentation only within
the frozen controlled-language population. It establishes no new semantic worlds,
general English, generation, reasoning, coding, mathematical proof or final-kernel
qualification. #1079's weak-control verdict and #1082's descriptive limits remain
unchanged; #973 stays open and #954 remains blocked.

**Preserved earlier evidence and handoffs.** The preparation/release checkpoints
below describe their original outcomes and then-current next actions; the result
above supersedes their scheduling and `NOT_RUN` status for this sole comparison.

[#1079](https://github.com/UOR-Foundation/uor-r4/issues/1079), delivered by
[#1080](https://github.com/UOR-Foundation/uor-r4/pull/1080), is complete at
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK`. All 156 primary criteria and all 25,600
answer comparisons pass. The fact-frame control meets the frozen strong-drop
criterion in six of six views; the valid token-frame control meets it in three
of six. Keep those results separate. These are six already-observed,
controlled-language views, not independent general-English or coding evidence.
The [measurement record](../r4_zoology_language_r4_1079.md) and its immutable
JSON envelopes remain authoritative.

The [#1082 diagnostic](../r4_token_exposure_1082.md) is complete at
`TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`, with exact fresh-process replay of all
286,720 used-role measurements and complete evidence. The frozen reader,
core, frames, two renderings and control were unchanged. Averaged over the four
fact slots and 8,192 supported rows per view, view 0 gives fact locations about
0.0035% changed-frame attention; view 1 gives fact objects about 0.0039%. Other
used roles are near full exposure. Highly displaced roles retain
almost all their weighted individual displacement after pooling. Changed and
retained supported-answer strata have very similar role means. Role-selective
exposure and retained answers despite displacement are observed; no downstream
cause is established and #1079's weak-control verdict remains unchanged.

[#1085's specification](clause-segmentation-1085.md) is complete at
`CLAUSE_SEGMENTATION_SPECIFIED`. It defines one deterministic text-to-clause
adapter, exact raw-only input/output/refusal schemas and a separate empirical
comparison while preserving the reader/core, lexicon/query and four-fact context.
The [source audit](clause-segmentation-1085-sources.md) links original
NEMESIS/W33/UOR material without importing capability or proof claims.
#1085 itself performed no implementation, population preparation, fit or evaluation.

The [#1094 implementation/preparation](../r4_text_clause_adapter_1094.md) returned
`UNAVAILABLE_REFERENCE_REPLAY`. The committed adapter recovered all 320
authoring inputs exactly and matched all 16 refusal cases, but the OS denied
execution of the pinned interpreter before Python startup. In that stopped
preparation, model loads/forwards were zero; effective worker isolation, model
preservation, withheld comparison and replay were `NOT_RUN`. The independently
curated 1280 withheld valid, 64 refusal and 16 boundary-control rows remain sealed.
The original stop and its receipts are preserved. Supplied segmentation stays qualified.

The separate [#1096 readiness decision](../r4_isolated_runtime_readiness_1096.md)
recorded **`ISOLATED_RUNTIME_READY`** in its sole attempt: all four harmless
corpus/reference/history/results probes were denied, with null model states and
zero model loads/forwards/updates. The attempt took 2.058100333 seconds and had a
704,806,912-byte combined peak-RSS bound. Independent result review passed, and
#1096 was delivered at `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`. This
qualifies the named runtime/probe contract; it neither proves the precise cause
of the original denial nor establishes model behavior or universal isolation.

The [frozen #1094 preparation contract](../r4_text_clause_preparation_1094.md)
now has an [implemented retained-evidence assembly and launch gate](../r4_retained_assembly_1094.md).
Committed source `07ec3f0d` produced the distinct metadata status
`PREPARATION_ASSEMBLED_FROM_RETAINED_EVIDENCE`, bound by assembly SHA256
`48fae2d391e347e89a290b12a8af97cf8266c5913a21e71f21c1bef74ef54c62`.
Independent exact-envelope release is
**`ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON`**. The assembly's embedded
`NOT_ADMITTED` is immutable; the separate exact release receipt governs execution. This step
implemented admission/accounting/launch plumbing and assembled retained evidence
without a new preparation worker, model, fit, withheld read, comparison or replay.

The original preparation's final write/exit tail was unmeasured, so its full
120-second allocation remains quarantined as a conservative debit, not a
120-second observed runtime. The original 3,465,401 bytes remain counted; the
corpus is counted once and new receipts/spools add to that ledger. The next
separately activated task under **[#1094](https://github.com/UOR-Foundation/uor-r4/issues/1094)**
is its frozen comparison and fresh-process replay through `run-retained` from
the bound coordinator with the verified exact release. Fresh source,
runtime and release checks consume the 120-second execution allocation; replay
has its own 120-second allocation, with 120 + execution + replay at most 360
seconds. No new preparation or automatic retry is admitted. A durable admission
marker precedes fresh identity checks, and the execution-start receipt precedes
the first withheld hash/read; interrupted or stopped envelopes cannot be reused.
#1094 remains open, parked and unassigned after this delivery. Its original
`UNAVAILABLE_REFERENCE_REPLAY` is unchanged; comparison/replay remain `NOT_RUN`.
Neither assembly nor readiness revises #1079's weak token control, establishes
new mathematical proof or raw-text capability, or unblocks #954. #973 stays open.

The user-requested [afflom ecosystem review](afflom-ecosystem-followup.md)
inspects Prism, both Atlas sources, LexLean, lean4-prod, GNAF and both matmul
repositories. Typed arithmetic, identity and correspondence boundaries guide
#1083/#1087/#1089. No dependency repin, upstream execution or measured speed
improvement follows from the source audit.

## Sequencing and ownership

The capability dependency chain remains
[#973](https://github.com/UOR-Foundation/uor-r4/issues/973) →
[#954](https://github.com/UOR-Foundation/uor-r4/issues/954) →
[#955](https://github.com/UOR-Foundation/uor-r4/issues/955) →
[#962](https://github.com/UOR-Foundation/uor-r4/issues/962) →
[#963](https://github.com/UOR-Foundation/uor-r4/issues/963) →
[#964](https://github.com/UOR-Foundation/uor-r4/issues/964) →
[#965](https://github.com/UOR-Foundation/uor-r4/issues/965).
[#940](https://github.com/UOR-Foundation/uor-r4/issues/940) remains dormant
administrator work that separately blocks release. [#1090](https://github.com/UOR-Foundation/uor-r4/issues/1090) also blocks #965 until the capability/resource scorecard is adopted. Closed children and green
transport acknowledgements do not complete their parent capability.

| Lane / native owner | Next bounded deliverable | Promotion boundary |
|---|---|---|
| [#1082 attention diagnostic](https://github.com/UOR-Foundation/uor-r4/issues/1082), child of #973 | Completed; [exact report/replay](../r4_token_exposure_1082.md) | Role-selective exposure observed; preservation and the weak-control verdict retained. |
| [#1085 language/context specification](https://github.com/UOR-Foundation/uor-r4/issues/1085), child of #973 | Completed; [adapter/schema/comparison contract](clause-segmentation-1085.md) | Specification only; later transfer stages remain separately staged. |
| [#1094 adapter comparison](https://github.com/UOR-Foundation/uor-r4/issues/1094), child of #973 | [Sole comparison and exact fresh-process replay](../r4_retained_comparison_1094.md) completed `CLAUSE_ADAPTER_PRESERVED`; independent result accepted, scientific DoD complete and #1094 closed through protected PR #1101 at `eade29f4b78435e9857936786426bb34e596b301` | Only bounded raw-text entry: unchanged reader/core, known vocabulary/query forms, four facts and 20 already-observed groups. Original preparation stop preserved; no semantic novelty, generation, native export or final-kernel claim. |
| [#1096 runtime readiness](https://github.com/UOR-Foundation/uor-r4/issues/1096), child of #1094 | Delivered `ISOLATED_RUNTIME_READY` at `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`; sole zero-forward attempt and independent review complete | Four harmless probes denied; no model load/output/replay, raw-text qualification or universal isolation claim. |
| [#1083 UOR integration](https://github.com/UOR-Foundation/uor-r4/issues/1083) | Typed identity/arithmetic ADR and review of one selected adapter boundary | Content hashes, structural identities, codec identities and derivation keys remain distinct; arithmetic needs a declared domain and error/cost contract. |
| [#1086 native reference specification](https://github.com/UOR-Foundation/uor-r4/issues/1086) | [Contract specified](../r4_native_reference_1086.md) as `NATIVE_REFERENCE_CONTRACT_SPECIFIED`; export, native implementation, loads, forwards, evaluation and replay `NOT_RUN` | Exact artifact/state/operator identities and a separate empirical comparison are defined; no native capability or execution release follows from the specification. |
| [#1102 native implementation/export/comparison](https://github.com/UOR-Foundation/uor-r4/issues/1102), child of #1086 | [Measured `NATIVE_REFERENCE_PRESERVED`](../r4_native_bridge_1102_execution.md); independent result accepted and delivered through protected PR #1104; both admitted envelopes consumed | Exact native artifact/profile preserves the bounded reader/core, vocabulary/query and four-fact operation. New host-binary qualification and final integer/table serving remain separate. |
| [#1087 final serving representation](https://github.com/UOR-Foundation/uor-r4/issues/1087), with #1083 typed UOR prerequisite | Separate lowering contract with operation/error/resource obligations after the native-reference boundary is qualified | Specification or dense research-reference success does not establish the final integer/table kernel; a lowering candidate needs its own frozen contract. |
| [#1105 service/API specification](https://github.com/UOR-Foundation/uor-r4/issues/1105), child of #1084 | Accepted [ADR](../adr/0006-native-four-fact-workbench-service.md) and [machine contract](../r4_service_contract_1105.json); `SERVICE_API_CONTRACT_SPECIFIED`; protected delivery closes only #1105 | Contract only; no host, worker, shell, build, model operation, HTTP or browser behavior exists. |
| [#1107 workbench source candidate](https://github.com/UOR-Foundation/uor-r4/issues/1107), child of #1084 | [Dedicated crate, private worker/comparison entries and first shell](../r4_workbench_candidate_1107.md) frozen as `WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`; protected delivery closes only #1107 | Static source evidence only. Build, tests, model work, HTTP/service/browser behavior, portability and numerical qualification are `NOT_RUN_BY_POLICY`. |
| [#1084 product interface](https://github.com/UOR-Foundation/uor-r4/issues/1084) | Remains open and unassigned; owner decides whether to authorize a separate manual qualification workflow | Automated agents do not dispatch or run qualification. Bounded Four-fact source is not general chat or verified runtime behavior. |
| [#1088 coding/workspace](https://github.com/UOR-Foundation/uor-r4/issues/1088), blocked by #1084 and #955 | Minimal executable task/patch/result schema; separate harness and later model evaluations | UI actions and external teacher output are separate from the native model's coding competence. |
| [#1089 scientific paper/proofs](https://github.com/UOR-Foundation/uor-r4/issues/1089) | Claim/evidence table, named proof obligations, primary-source bibliography and reproducibility package | Guarantees require their own proof status; empirical capability stays empirical; submission is a separate action. |
| [#1090 capability/resource scorecard](https://github.com/UOR-Foundation/uor-r4/issues/1090) | Defined task axes, comparison populations and resource budgets | Freeze meaningful comparisons before a frontier/release claim; a tool install or a tiny entry probe cannot fill the scorecard. |
| [#1091 external-theory bridge](https://github.com/UOR-Foundation/uor-r4/issues/1091) | One precisely typed, independently reviewable NEMESIS/W33 hypothesis or finite construction | Require an explicit mapping to the actual R4 path and source/license provenance; terminology and outside claims are not inherited evidence. |

The [adopted issue record](adopted-issues.json) records native ownership and
dependencies at adoption time. #1081 delivered the planning workflow in
[PR #1092](https://github.com/UOR-Foundation/uor-r4/pull/1092), merge
`11e46611b82702e005165fb0034e1adf7d119a70`; #1082 owns the completed diagnostic
and #1085 the completed specification. #1094's sole comparison completed bounded
raw-text preservation and exact replay, with independent result acceptance.
Its earlier unavailable preparation, separate #1096 runtime-only readiness and
retained-evidence release remain preserved. #1094 is closed through protected
PR #1101. #1086 now specifies the separate native export/loader/reference-behavior
contract; its [#1102 implementation/export/comparison successor](https://github.com/UOR-Foundation/uor-r4/issues/1102)
now records `NATIVE_REFERENCE_PRESERVED` on the original authoring stratum.
The specification and Python result remain separate from that later measured
native evidence; no final-kernel or service qualification transfers automatically.
[#1105](https://github.com/UOR-Foundation/uor-r4/issues/1105) retains the
accepted service/API contract; [#1107](https://github.com/UOR-Foundation/uor-r4/issues/1107)
owns only the unbuilt source candidate. After protected #1107 delivery, #1084
remains open and unassigned for the owner's separate manual-qualification
decision. Planned lanes remain unassigned until active.
The pre-adoption snapshot had nine open issues and all 24 #973 children closed;
those counts are historical now that the new children exist.
The S0–S7/F0 compiler-era graph is retained as history, not a second active queue.

## #973 → #954 consumer contract

This is the explicit intake specification for the current mechanism family.
It names the interfaces that must be qualified; it does **not** declare that
#1079 already satisfies #973's higher-context terminal or #954's final
source-free serving boundary. #954 remains blocked.

### Qualified reference available now

The present reference is the learned #1077 role reader plus the frozen #1073
compound-binding core, executed ordinarily and through the #1079 two-stage R4
adapter. Its reader consumes five clauses (four facts and one question), a known
vocabulary and controlled query forms. The independently accepted #1094 comparison
qualifies matching raw-text entry before that unchanged reader within its fixed
four-fact/known-query population. The reader performs soft role
pooling; the core attends over four facts plus the learned null and projects
through the full 4096-token vocabulary. The model receives no gold role or
answer labels at inference. It is not the older #953 decoded route loop and
does not inherit that loop's state schema or higher-context qualification.

The reference has a frozen reader artifact, core artifact, tokenizer/data
binding, model policy, native-frame bundle, implementation closure and result
envelopes, identified in [claim-ledger.json](claim-ledger.json). Its single-token
`UNKNOWN` task label is not yet a general typed abstention policy. Paragraph,
conversation and bounded-global state, contradiction handling, free generation,
and a production API for this artifact remain outside its measured scope.

### Required handoff schema

| Field | Required semantics | Current boundary |
|---|---|---|
| `artifact` | Versioned manifest with model/artifact bytes identity, implementation revision, lexical codec, model/input policy, geometry/frame identities, data lineage and qualified runtime plan. | #1079 binds these for its research execution; a native loader must preserve them. |
| `input` | Ordered lexical units, query, admissible evidence records with stable identities/provenance, and an explicit context snapshot. Declare segmentation, maximum support and supported shapes. | Five supplied clauses remain qualified; #1094 also qualifies matching raw-text entry on its fixed four-fact/known-query population. No hidden canonical fields, target labels, future text or oracle answers may enter the model. |
| `state` | Versioned prior-state identity; ordered hierarchy records and implemented scope; causal append/update rules; bounded-global snapshot identity and size. Unsupported scope is typed unavailable, not fabricated state. | The current fact-binding reference does not implement the required paragraph/conversation/global state handoff. |
| `output` | Tagged `ANSWER`, `ABSTAIN`, `CONFLICT`, `CLARIFY` or `UNSUPPORTED_SCOPE`; lexical output and selected IDs where applicable; no substitution of provider text. | The reference emits a task answer token. The remaining tags and their behavioral policies require qualification. |
| `evidence_trace` | Consumed record/snapshot IDs, causal positions/support, declared geometric contributions, selected output, state-before/state-after identities and complete work accounting. | A trace is provenance. Only matched interventions establish whether its qualified state affects the answer. |
| `replay` | Pinned artifact/input/runtime, exact decision and permitted numerical comparisons, immutable result, independent-process replay and resource report. | Reuse the existing evidence when bindings are unchanged; freeze any new comparison envelope before seeing outcomes. |

The serialized field names above define the handoff to implement; they are not
claims that an existing public API already emits that schema. Keep actual
manifest/file CIDs distinct from model-state CIDs and from derived trace keys.

### Admission and decision criteria

1. **Freeze and reproduce the accepted reference.** #973 must name the exact
   artifact and input/state/output schema. Preserve the current ordinary/R4
   result and its weak token-control finding. A successor may not obtain a pass
   by revising the revealed #1079 threshold or replacing its control.
2. **Qualify the required context before correctness intake.** On a declared
   independent population, the accepted paragraph/conversation/bounded-global
   state must change the actual decoded decision under matched disabled or
   permuted-state controls, with natural support/work and causal access bound.
   The owning child freezes populations, numeric criteria, runtime and divergent
   actions before evaluation. Generalization from a supplied four-fact task
   cannot be assumed, and no new numeric floor is invented by this map.
3. **Meet the consumer's execution boundary.** #954's final terminal requires
   its native source-free/forbidden-operation contract. Current dense/softmax
   research code is a reference, not evidence that this serving condition is
   met. A separately scoped reference probe must retain that distinction and
   cannot close #954's final terminal.
4. **Keep correctness labels out of mechanism selection.** The accepted
   artifact and state rules are frozen before #954's independent answer or
   constraint oracle is consulted. Do not tune admission, roles, geometry,
   support, conflict policy or candidate costs against the correctness reveal.
5. **Apply #954's existing four-case entry decision only after admission.** A
   global-only fact must fail when global state is disabled; a conversation/global
   conflict must be surfaced and handled by the frozen policy; a local supported
   fact must remain correct; an unsupported question must abstain. Report the
   denominator four, answered-conditional and overall correctness, and separate
   conflict/abstention outcomes. This entry probe does not establish broad
   correctness or frontier capability.

If provenance is unavailable or the accepted context is inert, #954 remains
blocked and the owning #973 mechanism is revised or retired according to its
frozen decision. If the admitted four-case correctness probe fails, stop and
revise C1 without expanding or tuning the revealed population. Only the actual
qualified C1 artifact proceeds to #955's reasoning contract; product fixtures
and imported project memories do not substitute for it.

## Keeping this map current

After protected delivery, retain the detailed record and update this small
pointer and its native issue links. Preserve superseded records with their
dates. Local knowledge ingestion is `NOT_RUN_BY_POLICY` for automated agents;
the tracked claim/source records remain the reviewable index update. Use
[CONTINUE.md](CONTINUE.md) for the next task and refresh live GitHub rather than
treating this snapshot as permanent eligibility.
