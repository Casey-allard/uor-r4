# #1099 — Freeze retained-evidence preparation and release boundary for #1094

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T12:21:12Z
- **Updated:** 2026-10-09T14:16:53Z
- **Closed:** 2026-09-03T12:22:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1099
- **Merged:** 2026-09-03T12:22:43Z
- **Merge commit:** 6008e3527cb119d12af4abd99fe86a3d4ebe5a53
- **Base / Head:** main / UOR-Foundation:issue-1094-preparation-freeze
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The original #1094 preparation stopped before Python startup, and its final elapsed snapshot did not measure the last receipt write/exit interval. A fresh `prepare` currently resets its budget, while `run` accepts only a newly completed preparation and drops earlier accounting. This freezes the separate resumption contract after #1096's accepted runtime startup.

The contract quarantines the full original 120-second preparation allocation as a policy debit, preserves the recorded elapsed values and 3,465,401-byte history, and defines a distinct retained-evidence assembly and independent release receipt. Execution and replay retain 120 seconds each under the unchanged 360-second cumulative cap. The required coordinator path is explicitly unimplemented, and release remains **NOT_ADMITTED**. Adapter, worker, curator, model computation, populations and comparison criteria are unchanged.

Independent budget, source and contract reviews verify the original receipts, current executable blockers and 19 pinned source/evidence references. The roadmap, continuation prompt and claim/evidence ledger point to implementation of the assembly and a launch gate that preserves history. The original unavailable result, #1096's runtime-only readiness, #1079's weak-control result and #1082's descriptive limits remain intact. Relevant pinned NEMESIS/W33/UOR originals were re-inspected without importing capability or proof claims.

Validation: JSON/source/evidence hash and link integrity, preservation of prior claim records, claim wording, scoped diff whitespace and independent reviews. No package import, preparation, runtime invocation, withheld access, model work, fit, comparison, replay or broad QA ran. Protected checks are transport acknowledgements. Storage was reviewed; no unique evidence or user material was deleted.

References #1094. This completes the preparation contract and review step only; the model-comparison issue remains open. Next action: implement the frozen retained-evidence assembly and budget-preserving launch gate, then obtain independent approval of the exact implemented envelope before withheld execution.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5525674632 — 2026-09-03T12:22:04Z — Casey-allard

Independent final transport-consistency review of head `85c05b7234c85dac73f16ed21149b1b0230c6530` (tree `206c9b0b19ad199b31756d13026ebc29b1e84072`) reported no actionable findings. The reviewer independently confirmed all 9 prior claims retain their substantive content, all 35 prior evidence references are unchanged, all 5 new references match committed SHA256 values, and the original #1094 historical body is preserved byte-for-byte after its current-handoff header. All 16 changed paths are documentation/metadata.

The [independent contract review](https://github.com/UOR-Foundation/uor-r4/blob/df4d2f9b6d38500109c80583fed7e02d3a157b1a/docs/r4_text_clause_preparation_1094_review.md) binds the four core document hashes and all 19 pinned source/receipt identities. Current pointers and the ledger agree: CONTRACT_ACCEPTED / PREPARATION_CONTRACT_FROZEN, execution release NOT_ADMITTED, #1094 open, model comparison/replay NOT_RUN. The next action is implementation of retained-evidence assembly and a budget-preserving launch gate, then independent exact-envelope release review.

No runtime, model or withheld-payload check was performed in this final review. Protected check acknowledgements concern transport only.



---
_Comment/review ids in this file: 5525674632_

