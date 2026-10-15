# #1106 — Specify the native four-fact service API and ownership contract

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T17:13:50Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-09-03T17:17:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1106
- **Merged:** 2026-09-03T17:17:54Z
- **Merge commit:** 8ba5887e067e4cc33a1a3c9a6459fcc4b4077a11
- **Base / Head:** main / UOR-Foundation:issue-1105-service-contract
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The accepted #1102 native comparison supplies a bounded four-fact operation, but no request server or qualification for a newly linked host. This change freezes the service API and ownership boundary needed to implement that operation truthfully.

ADR-0006 and its machine-readable companion specify a dedicated opt-in `r4-workbench` host, one private worker using the same final executable, and the first **Four-fact research reference** shell. They define seven routes, exact raw-byte/result identity, typed capability/error/progress states, job/model generations, cancellation and confirmed-reap behavior, bounded artifact/evidence/static-asset intake, and private IPC sequencing.

The final binary must contain its private comparison mode before its build is frozen. Fresh host execution permission remains distinct from the artifact's original export provenance. A newly linked host requires its own independently accepted numerical qualification; ordinary API/lifecycle/browser acceptance is a separate delivery gate. The old #1102 qualification or consumed #1102/#1094 envelopes cannot be substituted.

Independent review accepted `SERVICE_API_CONTRACT_SPECIFIED` after resolving the recorded lifecycle, identity and trust-chain findings. The source audit follows pinned donor, NEMESIS, W33 and UOR originals and preserves their actual support and license boundaries. Current roadmap/continuation pointers and the claim ledger distinguish this specification from the unchanged prior empirical findings.

Validation is limited to the named decision:

- Source/metadata checker: 28 rows covering 27 unique pinned originals, 29 type definitions, seven routes, five illustrative wire objects and 16 scenario IDs; exact native/refusal tags and original result fields; 0.277 seconds.
- Independent source/specification review and a separate non-writing metadata reproduction.
- `python3 scripts/check_claim_wording.py` and `git diff --check`.
- Ledger metadata comparison preserves all 13 prior claims' statements, assumptions, observations, limits and outcomes; only current handoff pointers advance.
- Storage inventory records zero deletion; all research/model/sealed-input and unique evidence remains retained.

No service/worker implementation, build, test suite, server/browser run, model load, qualification API call, fit, forward, export or replay ran. The examples are illustrative specification objects, not service outcomes. Implementation guarantees remain Unproven; #1102's bounded empirical result and all earlier negative/unavailable findings remain unchanged. Required GitHub jobs are the repository's transport acknowledgements, not QA or scientific evidence.

Evidence is pinned at commit `f830b1a774fca7945f340ab05391f28c2fe6c4fc`; the following pointer/ledger commit cites those immutable bytes. Public knowledge records and the native parent/child handoff are refreshed after the protected merge.

Closes #1105.

References #1084. The parent remains open and unassigned. Its next action is a separately activated implementation child for this host, worker and first shell, with concrete source/build/environment admission before a build and a fresh actual-host qualification release before model work.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5529415435 — 2026-09-03T17:17:36Z — Casey-allard

Final independent agent delivery review accepts head `b991dff95a9abf5ec4edbe3dddc512f787764f93` as **ACCEPTED_FOR_PROTECTED_SPECIFICATION_DELIVERY**.

It verified all nine evidence-reference hashes against commit `f830b1a774fca7945f340ab05391f28c2fe6c4fc`, the corrected acceptance-state pointers, the exact PR body, and preservation of all 13 prior claims' substantive fields. Only nine current next-action pointers advance; the sole new claim is the #1105 service definition. The complete diff is clean. The closure directive is #1105 only; #1084 remains open and unassigned.

Final delivery review receipt SHA256: `88572d9c59a39d14393de351a1bf3599ec56f8096d7e750f80e80e4bbce3402a`. The committed independent specification review is `docs/r4_service_contract_1105_review.md`, SHA256 `99fdb6fb8986520bb02457c65cc0dacc7117ddecaa2cfca74a8011d802c6e480`.

This acceptance covers specification delivery only. No service/worker implementation, build, model load, qualification call, forward, replay or browser/HTTP run was performed. Implementation guarantees remain Unproven. The five required GitHub jobs are transport acknowledgements, not test or model results.



---
_Comment/review ids in this file: 5529415435_

