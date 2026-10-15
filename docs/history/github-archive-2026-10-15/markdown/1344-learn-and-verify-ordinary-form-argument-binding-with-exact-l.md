# #1344 — Learn and verify ordinary-form argument binding with exact lexical joins

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-22T03:01:34Z
- **Updated:** 2026-10-09T13:17:59Z
- **Closed:** 2026-09-22T05:40:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1344
- **Merged:** 2026-09-22T05:40:31Z
- **Merge commit:** b218058f7445f7e815c7aca505d9e0d7f913ae26
- **Base / Head:** main / UOR-Foundation:codex/structured-argument-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973, #1139, #962, #963, #964; programme #820.

The preceding reader could find spans on fixed-layout clauses but could not consistently interpret reordered arguments or separate trailing material. This change learns a joint span/role hypothesis across the tested ordinary forms, then uses exact lexical identity for dependent answers across different BPE tokenizations.

The joint semi-Markov decoder has learned singleton background scores, cue-sided argument roles and precomputed interval potentials. Both categorical and H4-hybrid arms receive ordered observations. The same loaded causal session owns read/continue/emit and saved continuation; serving receives no gold spans, roles, expected answers or source pointer.

## Principal review and corrections

Independent source, evidence and architecture reviews retain the result. Source `940a9076` additionally enforces16-bit feature-token domains, fixes premature i32 clipping in the joint cue score, checks complete feature-difference updates against independent enumeration, preserves non-object text and later extents in source interventions, and adds a separately named exact lexical-membership control. Original submitted roots/artifacts remain unchanged.

| Population/control | Complete answers |
| --- | ---: |
| Categorical and H4, development |48/48 each|
| Categorical and H4, exposed regression |40/40 each|
| Categorical and H4, final-composition panel |24/24 each|
| Matched lexical membership, exposed |32/40|
| Historical token membership, exposed |38/40|
| Maximum two reads, exposed |30/40|
| Reads disabled, exposed |0/40|

The correction retains byte-identical learned artifacts and all344 original row behaviors. Its additional control yields384 rows; including six interventions,2,820 checkpoints independently verify with zero frame, provenance, oracle or continuation errors. The isolated source edit now keeps `downtown`: `Office Park`→`Cedar Annex` changes the complete answer on the same three-read path.

## Scope

The first final composition panel appeared in root7 after selection; root8 and the principal run replay it. These are authored combinations with shared vocabulary/forms and all questions fitted. They do not establish unseen-entity, novel-syntax or broad-language generalization. The hybrid's descriptor map remains initialized: a strict exact-hit search starting at96/96 cannot accept any improvement. Equal answers are a scoped quality tie, not learned-code evidence or equal inference cost. Whole-path D0-b/energy remain unqualified.

## Executed validation

- 23 focused module tests pass, including nonzero semantic/background exhaustive comparison, wide-score ordering, full update parity and matched control restoration.
- 23 runner tests pass; one legacy artifact-dependent test explicitly ignored.
- Offline release build and actual corrected exposed execution pass.
- Independent six-file seal/source/executable verification; both model bytes and prior behaviors unchanged.
- Format, claim-wording and diff checks pass;1,038 relative document links and3 new JSON receipts validated. Existing unused/dead-code compiler warnings remain; no warning-free claim.

The next complete milestone is **correction-aware conversation memory over unfamiliar entities**, with learned ingestion/temporal intent, exact scope/entity/relation/version authority and a pinned dependent read view. Q8 result consumption follows that milestone, then E/S language and executed Rust.

[Principal review](https://github.com/UOR-Foundation/uor-r4/blob/a2e099fcb8ff92e980af40d44378bb44ed825551/docs/integration/ordinary-form-binding-review-2026-09-22.md) · [Full next brief](https://github.com/UOR-Foundation/uor-r4/blob/a2e099fcb8ff92e980af40d44378bb44ed825551/docs/integration/deepseek-scoped-correction-memory-step-2026-09-22.md) · [Checks/resources](https://github.com/UOR-Foundation/uor-r4/blob/a2e099fcb8ff92e980af40d44378bb44ed825551/docs/evidence/ordinary-form-binding-principal-checks-2026-09-22.json)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5771752371 — 2026-09-22T05:41:59Z — Casey-allard

Principal review and protected delivery complete: PR #1344 merged as `b218058f7445f7e815c7aca505d9e0d7f913ae26`. Full merged tree equals reviewed `a2e099fcb8ff92e980af40d44378bb44ed825551`; all23 changed paths verified.23 focused module and23 runner tests pass (1legacyignored), releasebuild and corrected exposed generation pass. Both learned modelbytes/all344originalbehaviors retained; matched lexicalmembership32/40 vs primary40/40, isolated objectedit keepssuffix;2,820checkpoint continuations independently verify.

The ordinary-form interface is retained at bounded authored scope. Final-panel replays are not new fresh draws; fixed-code H4 quality tie and saturated descriptor search do not establish learned geometric advantage or equal cost.

Next is one correction-aware scoped conversation-memory lifecycle for unfamiliar entities, then consumed Q8 computation and E/S language/executedRust. [Full principal review](https://github.com/UOR-Foundation/uor-r4/blob/b218058f7445f7e815c7aca505d9e0d7f913ae26/docs/integration/ordinary-form-binding-review-2026-09-22.md) and [comprehensive next brief](https://github.com/UOR-Foundation/uor-r4/blob/b218058f7445f7e815c7aca505d9e0d7f913ae26/docs/integration/deepseek-scoped-correction-memory-step-2026-09-22.md).

Six owning issues are current/open; knowledge and memory include the verified merge. Free space39,132,946,432B;4,576,190,464allocatedB of reviewed caches/authorizedinstallers cleared, unique research andotherDownloads preserved. Cumulativeledger322,677,886/333,700,000ms; no paidcompute or newallowance extension. Owner checkout unchanged andclean.



---
_Comment/review ids in this file: 5771752371_

