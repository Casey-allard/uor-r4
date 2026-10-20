# #1100 — Implement retained-evidence assembly and guarded comparison release (#1094)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T12:56:23Z
- **Updated:** 2026-10-09T14:16:53Z
- **Closed:** 2026-09-03T12:57:34Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1100
- **Merged:** 2026-09-03T12:57:34Z
- **Merge commit:** 1ff8b81cb060ca2e8ced409ec78ae566b5b98891
- **Base / Head:** main / UOR-Foundation:issue-1094-retained-assembly
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The original #1094 preparation stopped before Python startup. Re-running preparation would repeat retained work and discard its resource history. This change assembles the preserved authoring and accepted #1096 runtime evidence into a distinct immutable envelope and implements `run-retained`, which requires an independent exact-envelope release and carries the original 120-second policy debit and 3,465,401 bytes into execution and replay.

The source was independently reviewed and committed at `07ec3f0d39d08ac5bf9c2ba7a6b864229e007867`. Metadata assembly completed with SHA256 `48fae2d391e347e89a290b12a8af97cf8266c5913a21e71f21c1bef74ef54c62`. Independent review then issued the separate `ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON` receipt, SHA256 `5787e4a64113800c5fc82cd1d32d564d9c6e3a344e74ca102a754fe82dccee23`. Original files and byte-identical public copies are indexed in the evidence record.

Admission binds the executing source, exact profile/environment, unchanged worker and original population. A durable admission marker precedes fresh checks; an execution marker precedes withheld payload access. Runtime/source/asset checks consume the existing budgets, and a stopped or previously admitted envelope cannot be retried. Adapter, reader/core, worker, curator, policy, populations and empirical criteria are unchanged.

Validation: all 7 retained-metadata checks and 11 campaign admission/accounting checks passed using temporary synthetic fixtures and worker stubs; claim wording and diff whitespace passed. Independent source and envelope review verified 170 coordinator files, 169 worker files and 19 historical references. The scoped code index was refreshed. Current roadmap, continuation and claim/evidence records describe the separate next comparison. Broad QA remains dormant; protected acknowledgements are transport only.

No new preparation, readiness worker, runtime/asset verification, model work, fit, withheld read/hash, comparison or replay ran. The withheld directory remains mode 000. Original UNAVAILABLE_REFERENCE_REPLAY, #1096 runtime-only readiness, #1079 weak-control and #1082 descriptive limits are preserved. No mathematical proof or raw-text model-preservation claim is added. Storage was reviewed; nothing was deleted.

References #1094. This delivers implementation, metadata assembly and conditional release only. #1094 and #973 remain open; #954 remains blocked. The next action is the separately activated frozen comparison and fresh-process replay through this exact release.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

