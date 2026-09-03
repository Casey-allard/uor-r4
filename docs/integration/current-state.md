# Current programme map and correctness handoff

**Planning reconciliation: 2026-09-03.** Scientific evidence baseline:
`UOR-Foundation/uor-r4@e627252e525201815169ffd8364184953a46018d`.
This map supersedes earlier “current” and “next” sequencing prose, not its
measurements. Refresh native GitHub before selecting work. The
[integration plan](../uor_productization_integration_plan.md) and
[adoption record](workflow-adoption.md) describe the research, engineering and
paper lanes adopted through [#1081](https://github.com/UOR-Foundation/uor-r4/issues/1081); the
[claim ledger](claim-ledger.json) preserves their different evidence states.

## Current result and immediate research action

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

The immediate successor is **[#1085: specify removal of externally supplied
clause segmentation](https://github.com/UOR-Foundation/uor-r4/issues/1085)**.
Keep the accepted reader/core, known lexicon/query form and four-fact context.
Specify one text-to-clause adapter, its allowed inputs, typed failure behavior,
and comparison with the current oracle-segmented reference. The eventual
empirical child must freeze independent inputs, output/role preservation,
controls, decision branches and budget before observation. This is the next
product restriction selected by independent review; the diagnostic does not
prove that segmentation caused the control difference. No fit, new population,
downstream diagnosis, replacement control or geometry expansion is activated by
this specification handoff. #954 remains blocked.

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
| [#1085 language/context](https://github.com/UOR-Foundation/uor-r4/issues/1085), child of #973, eligible after #1082 delivery | Specify removal of supplied clause segmentation, then freeze a separate empirical child | Clause segmentation, role ambiguity, new semantic combinations, variable context and temporal updates need their own evidence; this issue plans them rather than automatically fitting. |
| [#1083 UOR integration](https://github.com/UOR-Foundation/uor-r4/issues/1083) | Typed identity/arithmetic ADR and review of one selected adapter boundary | Content hashes, structural identities, codec identities and derivation keys remain distinct; arithmetic needs a declared domain and error/cost contract. |
| [#1086 native reference bridge](https://github.com/UOR-Foundation/uor-r4/issues/1086), blocked by #1085 | Specify the accepted model's export/loader and matched native behavior contract before implementation | Preserve qualified behavior and identity; no canned answers or silent backend substitution. |
| [#1087 final serving representation](https://github.com/UOR-Foundation/uor-r4/issues/1087), blocked by #1086 and #1083 | Operator inventory and one concrete lowering decision with operation/error/resource obligations | Dense research-reference success does not establish the final integer/table kernel; a lowering candidate needs its own frozen contract. |
| [#1084 product interface](https://github.com/UOR-Foundation/uor-r4/issues/1084) | API/asset-ownership ADR and minimal same-origin shell with one honest supported operation | This can proceed in parallel without completing #962 or presenting bounded binding as general chat. |
| [#1088 coding/workspace](https://github.com/UOR-Foundation/uor-r4/issues/1088), blocked by #1084 and #955 | Minimal executable task/patch/result schema; separate harness and later model evaluations | UI actions and external teacher output are separate from the native model's coding competence. |
| [#1089 scientific paper/proofs](https://github.com/UOR-Foundation/uor-r4/issues/1089) | Claim/evidence table, named proof obligations, primary-source bibliography and reproducibility package | Guarantees require their own proof status; empirical capability stays empirical; submission is a separate action. |
| [#1090 capability/resource scorecard](https://github.com/UOR-Foundation/uor-r4/issues/1090) | Defined task axes, comparison populations and resource budgets | Freeze meaningful comparisons before a frontier/release claim; a tool install or a tiny entry probe cannot fill the scorecard. |
| [#1091 external-theory bridge](https://github.com/UOR-Foundation/uor-r4/issues/1091) | One precisely typed, independently reviewable NEMESIS/W33 hypothesis or finite construction | Require an explicit mapping to the actual R4 path and source/license provenance; terminology and outside claims are not inherited evidence. |

The [adopted issue record](adopted-issues.json) records native ownership and
dependencies at adoption time. #1081 delivered the planning workflow in
[PR #1092](https://github.com/UOR-Foundation/uor-r4/pull/1092), merge
`11e46611b82702e005165fb0034e1adf7d119a70`; #1082 owns the completed diagnostic
and #1085 the next specification. Planned lanes remain unassigned until active.
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
adapter. It consumes five supplied clauses (four facts and one question), a
known vocabulary and controlled query forms. The reader performs soft role
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
| `input` | Ordered lexical units, query, admissible evidence records with stable identities/provenance, and an explicit context snapshot. Declare segmentation, maximum support and supported shapes. | Five supplied clauses are qualified. No hidden canonical fields, target labels, future text or oracle answers may enter the model. |
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

After protected delivery, retain the detailed measurement record, update this
small pointer and its native issue links, and explicitly ingest accepted source
and claim records into local knowledge. Preserve superseded records with their
dates. Use [CONTINUE.md](CONTINUE.md) for the next task; refresh live GitHub
rather than treating this snapshot as permanent eligibility.
