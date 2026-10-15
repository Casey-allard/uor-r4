# #1203 — Compose learned operations with formatted output

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T19:34:43Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-09-08T19:36:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1203
- **Merged:** 2026-09-08T19:36:21Z
- **Merge commit:** 66a4e1e01dab634e4890180290ecbeb965fb0e54
- **Base / Head:** main / UOR-Foundation:codex/composed-formatted-output
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The shared operation selector stopped after the first formatted result even when the equivalent plain response continued. This change continues the existing geometric Stop/Copy/Add router at those boundaries. After actual history 17 and an extra-3 “Again” request, the retained artifact generates `20 is 3 plus 17.\n23 is 3 plus 20.\n`, or the corresponding two Rust equalities, then stops. The second operation reads the first committed result.

Only operation-router parameters change. The previous router reconstructs the exact parent; the lexical emitter, dictionaries, other selectors, bounds and fixed geometry remain unchanged. Offline training checks real generated histories and first segments, executes supervised operators on actual records, and excludes boundaries that serving cannot reach. There is no new serving response template, parser, provider or matrix product.

Executed locally on retained `866cb92d`:

- Construction 42/42; open, reversed-context and separate post-selection fresh panels 28/28 each, checking complete text/EOS, ordered pending and committed writes, exact reads and per-byte checkpoint parity.
- Previous-router and intermediate-removal controls lose the intended continuations. All 663 retained answers, earlier panels and 254 construction-preservation cases pass. Fourteen older result/control reports match their saved actual outputs and decision identities, and all 72 earlier lexical outputs remain exact.
- Native API 50/50; 54 actual generated Rust equalities compile and evaluate; six read and six operator tests, source guard and formatting pass. Actual composed and prior lexical allocation checks pass with zero measured allocations and 72/136 checkpoint positions.

The first candidate regressed ten older Stop responses and is preserved as rejected. A subsequent checker error incorrectly required numeric writes for six valid abstentions; the receipt preserves those legacy results and the explicit baseline-label correction. All three candidates, source copies and charges are retained. The result and machine receipt document the 920.669 model seconds, 860.544 engineering-command seconds, prior resource charges and remaining limits.

These are finite repeated-Add and formatting results. Mixed operator sequences, general prose, full Rust program generation and energy advantage remain unqualified; new-artifact browser/WASM/HTTP is NOT_RUN. The next model step is mixed shared Copy/Add sequences and operand references. Queue compatibility acknowledgements execute no tests; the checks above are recorded local executions.

See `docs/native_geometric_composed_output_1140.md` and `docs/evidence/native_geometric_composed_output_1140.json`.

References #1140 and #820. The broader work in #1140 remains incomplete.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

