# #1104 — Implement and qualify the bounded native learned reference (#1102)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T15:37:50Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-09-03T16:15:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1104
- **Merged:** 2026-09-03T16:15:38Z
- **Merge commit:** 85d9eb8beca2a59ccda47e290afd483f7838982c
- **Base / Head:** main / UOR-Foundation:issue-1102-native-bridge
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The accepted raw-text reader/core/R4 path previously ran only through the Python research reference. This change implements the opt-in CPU Rust reference, immutable 21-component artifact, strict trusted loader and direct-byte exporter defined by #1086. The accepted reader/core, known vocabulary/query forms and four-fact context remain unchanged. Default R4G1 serving is unchanged.

The separately admitted comparison completed **NATIVE_REFERENCE_PRESERVED**. Both runtimes and both fresh-process replays produced 320/320 correct answers, 4,480/4,480 consumed roles and 16/16 typed refusals. All four complete f32 tensors met the frozen 1e-5 absolute limit; the largest difference was 4.76837158203125e-6. Within each runtime, the replay row JSONL and full tensor files were byte-identical. Two independent exports matched, and all eleven rejected loader cases plus the valid missing-qualification gate matched their frozen criteria.

Independent retained-evidence review checked all 6,727,680 output values, source/runtime/artifact/state identities, exact parser/discrete results, loader evidence and resources. It accepted **ACCEPTED_BOUNDED_NATIVE_REFERENCE_QUALIFICATION**. The full tensors, both artifacts and all mutation evidence remain retained locally; public metadata binds every original file. The 1,280-forward campaign took 8.809784 seconds, recorded 602,390,528-byte peak combined sampled RSS and retained a 75,039,076-byte complete ledger, within the unchanged limits. No fitting or withheld access occurred.

- [Execution and limits](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_execution.md)
- [Independent result review](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_result_review.md)
- [Machine audit](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_evidence/result-review.json)
- [Qualification handoff](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_evidence/qualification-handoff.json)
- [Full retained-evidence index](https://github.com/UOR-Foundation/uor-r4/blob/5af0ae38df122853815947aafefb6a88b3f519ea/docs/r4_native_bridge_1102_evidence/retained-evidence-index.json)

Qualification is limited to this exact artifact, binary/runtime and known authoring stratum. It establishes no independent semantic holdout, general parsing/context/generation/coding, mathematical equivalence, portability, geometry advantage or final integer/table kernel. The successful ordinary qualify()/serving path remains NOT_RUN; the measured binary has research modes, and a newly linked service needs its own truthful binary binding. #1084 receives the separate service/API ownership ADR handoff; it is not implemented here. Prior Python preservation, weak-control, descriptive and unavailable results remain. No unique evidence or user material was deleted.

Verification was the named one-shot offline release build (175.932499 seconds; 682,391,353 observed output bytes), admitted loader/comparison/replay and independent evidence analysis, plus claim wording and diff/metadata checks. The earlier Cargo --offline failure was a local cache miss, resolved by restoring 25 exact lockfile-matching archives; both failed metadata observations remain recorded. Broad QA was not run. The five required CI acknowledgements are queue transport only and are not test or behavior evidence.

Closes #1102.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5528186814 — 2026-09-03T15:39:53Z — Casey-allard

Draft-delivery verification: PR head `1c6be403a9de732dcbb1d9aad8fda66c2b18579c` is open/draft, not queued, not configured for auto-merge and not merged; #1102 remains open and parked. Independent final review found the source/claim/closure records consistent with that limited state.

The local public knowledge index now includes the current issue snapshot, draft roadmap/claim entries and source-review/prebuild records, explicitly marked unbuilt/unmerged. Retrieval of the new #1102 prebuild evidence and checkpoint comment was verified.

The five green compatibility statuses are transport acknowledgements under the current dormant-QA policy. The build job is skipped; those statuses do not establish that this candidate compiles or passes the frozen native comparison. Both execution envelopes remain unconsumed. The next action remains exact pinned dependency-cache restoration before offline build and independent release admission.


### comment 5528617148 — 2026-09-03T16:13:47Z — Casey-allard

Independent agent review: **APPROVED_FOR_PROTECTED_DELIVERY** for final head `0388f7e4cc6cfc3d2041307683564eda8e473065`. No remaining findings. This is an agent source/evidence review, not a GitHub-account approval or CI behavioral result.

The independently decoded retained result is `NATIVE_REFERENCE_PRESERVED`. Evidence commit `5af0ae38df122853815947aafefb6a88b3f519ea`, all nine final roadmap/claim files and the PR description were reviewed. The exact 12-field qualification receipt binds accepted result audit `5e75c5a3407d91bf14a6ec8d57981a72cfe110cdaf1ddfd0c71f6ea0a07974e8`; no build, model load, forward, export or replay was added during review.

Final delivery review SHA256: `7f2cbc77317791f1a1678b02caba198169bc4752596cb2c7cc75960a46a6e0bb`.

Ordinary qualify()/serving execution remains NOT_RUN; the actual host binary must be bound truthfully in a separate #1084 handoff. Historical weak, descriptive and unavailable results remain preserved. Both admitted envelopes are consumed. Protected queue and actual merge verification are required before closure.

<details>
<summary>Exact final review receipt</summary>

```json
{"accepted_result_audit_sha256":"5e75c5a3407d91bf14a6ec8d57981a72cfe110cdaf1ddfd0c71f6ea0a07974e8","additional_work_during_final_review":{"QA_suites":0,"builds":0,"exports":0,"fits":0,"forwards":0,"loader_calls":0,"model_loads":0,"model_replays":0,"sealed_run_root_edits":0,"source_edits":0},"approval_scope":"Evidence commit 5af0ae38 plus exactly the nine final documentation/roadmap/claim files hashed above and the reviewed PR description. Complete through the protected PR and merge queue; no direct main push, admin bypass, invented external status, or dormant QA activation. Verify the actual merged commit and issue state before reporting closure.","claim_limits":"NATIVE_REFERENCE_PRESERVED is empirical preservation on the exact original authoring stratum, binary/runtime/profile and artifact. No general language, new semantic holdout, portability, universal proof, geometry-superiority, performance, ordinary serving, or final integer-kernel result is asserted.","closure_state_at_review":"Scientific decision and independent result review complete; protected delivery and GitHub issue closure still pending.","evidence_commit":"5af0ae38df122853815947aafefb6a88b3f519ea","findings":[],"issue":1102,"measured_source_revision":"1c6be403a9de732dcbb1d9aad8fda66c2b18579c","next_action_after_closure":"Separately activate #1084 to freeze service/API, artifact ownership and truthful actual-host qualification. No successor work is part of this approval.","pr":{"closes_issue":1102,"description_path":"/Users/casey.allard/.codex/uor/issue-1102-cache-recovery/final-pr-body.md","description_sha256":"ea2bd3aa31904cc365c5d51b0b5b10cc0870631e36f00b54b91eec9e8e2e8d28","planned_title":"Implement and qualify the bounded native learned reference (#1102)","url":"https://github.com/UOR-Foundation/uor-r4/pull/1104"},"qualification_sha256":"61d29aa80e6bcd3d163b2ff2a6da4faab04414ea9f4284d80b798c4e46cf5369","remaining_diff_sha256":"ebb16b18a6b78efee11d31226a351de6f02fe94099b0bf0a982c2ea78cdb4360","reviewed_uncommitted_files":[{"bytes":92005,"path":"AGENTS.md","sha256":"3febd8e6542c41f49833febffefc072c45010b40aafdb649002181b4f395b11f"},{"bytes":101807,"path":"README.md","sha256":"36562577021b3cec5b74d9bb6d4cf67752e34eced6c6c4f62b6ecfd38ee83997"},{"bytes":96997,"path":"ROADMAP.md","sha256":"8c634260335efd6442b6c74bd0a0cb47436422b82c80c689bc6f47c4bfa49f3e"},{"bytes":183303,"path":"docs/RESEARCH.md","sha256":"24999d06f21b17ca7e9c6af4d585d7c1032f0773efcb8ab805ac1c7808647a82"},{"bytes":6755,"path":"docs/integration/CONTINUE.md","sha256":"d0d971148a47ffc17db80060b0608da7f7cff7587a1c05d62fb8a202ce2fdc2a"},{"bytes":90723,"path":"docs/integration/claim-ledger.json","sha256":"53262755bfe14ae3903c01ba7736d0579fa77f8f154c4db61abf178f2e584f32"},{"bytes":24290,"path":"docs/integration/current-state.md","sha256":"baa37e1adc784716dca5abf390800be547dca2777c654da44a99b95b65236195"},{"bytes":88493,"path":"docs/r4_intelligence_completion_plan.md","sha256":"e0b8db80c1deddcba38e813713b7c684e7b5376539d9f95186d96985da586ded"},{"bytes":39091,"path":"docs/uor_productization_integration_plan.md","sha256":"392ba378d3a877ee3ac5a957cdc888bb6053ebffc3095ff7838b9c516c782ac8"}],"reviewer":"independent_review","schema":"uor-r4.native-bridge-final-delivery-review/1","status":"APPROVED_FOR_PROTECTED_DELIVERY","utc_reviewed":"2026-09-03T16:12:36.663329+00:00","verified":{"all_12_new_ledger_evidence_sources_match_evidence_commit":true,"historical_assembly_issue_state_explicitly_labelled":true,"implementation_sources_unchanged_since_build":true,"new_host_cannot_reuse_measured_binary_identity":true,"no_frozen_evidence_or_qualification_changed":true,"ordinary_qualify_success_path_NOT_RUN":true,"ordinary_service_endpoint_NOT_IMPLEMENTED":true,"original_prebuild_state_preserved":true,"prior_native_claim_scope_status_observations_and_limits_preserved_in_history":true,"qualification_audit_result_artifact_state_binary_runtime_bindings_match":true,"qualification_canonical_ascii_without_LF":true,"qualification_exact_12_field_schema":true,"queue_acknowledgements_described_as_transport_only":true,"within_runtime_byte_equality_correctly_scoped_to_rows_and_tensors":true}}
```

</details>



---
_Comment/review ids in this file: 5528186814, 5528617148_

