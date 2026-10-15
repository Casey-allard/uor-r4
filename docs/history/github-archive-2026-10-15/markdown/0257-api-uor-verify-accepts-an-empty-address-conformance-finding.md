# #257 — /api/uor/verify accepts an empty address (conformance finding: canonical negative vector)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T11:50:26Z
- **Updated:** 2026-07-28T13:20:57Z
- **Closed:** 2026-07-28T13:20:57Z
- **Labels:** bug, conformance
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/257
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
`/api/uor/verify` (src/server.rs, added with the #195 envelope) accepts an **empty address as valid**. "Empty subject must reject" is about as canonical as a negative verification vector gets.

### Classification
Conformance finding: a textbook negative vector for a UOR-VERIFICATION suite. A verifier that passes the empty case provides negative assurance — it converts the attestation feature from evidence into decoration.

### DoD
- Empty/malformed/wrong-axis addresses rejected with typed errors; only canonical-pipeline-computed addresses verify.
- Test enumerating the negative cases (empty, truncated hex, wrong prefix, valid-syntax-wrong-content) and one positive case.
- Cases contributed as seed vectors for the verification conformance suite.

*Pairs with the attestation-envelope conformance issue; both from Maura's Gate-4 vector-category mapping.*


## Comments (1)

### comment 5104519751 — 2026-07-28T13:06:08Z — Casey-allard

Being fixed atomically with the envelope side — plan and rationale on the sibling issue (see the comment there); branch `issue-256-uor-conformance` will close both.


---
_Comment/review ids in this file: 5104519751_

