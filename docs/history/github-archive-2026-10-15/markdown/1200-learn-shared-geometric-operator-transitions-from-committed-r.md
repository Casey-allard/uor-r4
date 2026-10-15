# #1200 — Learn shared geometric operator transitions from committed results

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T16:12:20Z
- **Updated:** 2026-10-09T13:17:38Z
- **Closed:** 2026-09-08T16:13:35Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1200
- **Merged:** 2026-09-08T16:13:34Z
- **Merge commit:** f502bc6e79854e90c891349888afc24c8f4ad646
- **Base / Head:** main / UOR-Foundation:codex/learned-operation-transition
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained model stopped after its first computed result. Add an artifact-bound shared signed-H4 Stop/Copy/Add selector that can use the actual committed result for another operation within the same response. In the measured extra-3 “Again” query after a prior total of 17, it now emits `20.\n23.\n`, records the first write ID as the second Add's dependency, then selects Stop below the three-operation safety cap.

This also repairs a completed-checkpoint input boundary exposed by actual API testing: EOS could make a session inactive while its captured query still needed closing. API and durable callers now distinguish that state from partial prefill. The failed API receipt and first Stop-only fit remain preserved.

Validation executed locally on M1:

- 36/36 construction, 24/24 open and 24/24 post-selection numeric transfers, with exact complete outputs, dependencies, EOS and checkpoint consistency.
- Both new controls preserve all 16 single-operation answers and lose all eight second operations. The intermediate intervention changes candidate availability, not only geometry.
- All 663 retained answers and prior preservation panels pass; prior current-query 48/48 and harder histories 16/16 pass. The old intermediate-removal control preserves all 32 independent responses and changes all 16 dependent ones.
- Fourteen actual-artifact API checks pass, including the composed response after checkpoint import, independent 17→18→17, chunked prefill, artifact roundtrip and identity isolation.
- Six focused mechanical tests, integer-kernel source guard, release compilation, formatting, claim wording and receipt/link/hash checks pass. Actual candidate allocation check records zero allocations in its stated ingestion/begin/predict/observe scope.

Retain `6f20982c` over frozen parent `ddc943f9`; preserve all negative artifacts. The evidence receipt records 312.644 model seconds, every failed command/retry, cumulative 7,754.223/9,450 seconds, prior reservation and storage margin. Queue statuses are compatibility acknowledgements, not tests.

Scope remains finite numeric query forms. General prose, meaningful Rust generation, generalized reasoning and energy advantage are unqualified. New-artifact browser/WASM is NOT_RUN and the new block requires the updated runtime. Next work is shared learned lexical choice and ordered sentence/Rust-expression emission from committed values.

References #1140


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

