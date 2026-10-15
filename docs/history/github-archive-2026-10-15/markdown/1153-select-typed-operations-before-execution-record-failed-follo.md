# #1153 — Select typed operations before execution; record failed follow-up composition

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T18:50:37Z
- **Updated:** 2026-10-09T13:18:01Z
- **Closed:** 2026-09-06T19:19:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1153
- **Merged:** 2026-09-06T19:19:16Z
- **Merge commit:** 14a4c85e360f1c3738b1a4364048e83a1c622783
- **Base / Head:** main / UOR-Foundation:codex/typed-operator-admission
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The numeric selector executes every candidate before scoring even though its features do not depend on the result. This change scores first, executes the selected Copy/Add candidate, and preserves stable overflow fallback with explicit execution/rescoring counters. The learned numeric scorer and retained artifact 2600b95b remain unchanged.

The actual-artifact composition check gives **3/3 correct initial sums but 0/6 follow-ups**. All intermediate values and new operands are retained. Repeat instructions choose Add over the sum plus an old operand; removing the intermediate changes those outputs. Add-new-value instructions abstain. This localizes the next learning work to query-conditioned operator/operand choice; it is not missing storage or general reasoning qualification. Twelve Full/control observations and the failed acceptance assertion are preserved.

Validation:

- Same-artifact pre-change/rebuilt executions preserve complete output/state/traces: 48/48 dependent, 62/62 earlier, 24/24 transfer, 28/28 exposed-name and 5/5 persistent-session cases. The 28 longer-context Generation objects also match historical evidence except work, with exact writes.
- Four selected-execution/causal tests, response-cache and nested-counter tests pass. Numeric and actual-artifact dependent allocation checks pass with zero allocations. Native CLI returns the revised Zurich value.
- Four newly generated Rust function texts match the retained check source; recompilation and all 12 semantic assertions pass. This is preservation of familiar functions, not new composition evidence.
- Offline runtime/probe/CLI builds, formatting, architecture policy and claim wording pass. Required protected CI remains separately visible; compatibility acknowledgements do not certify audit/fuzz/WASM/Gate C.

On the 62-case set, additions fall 236 to 16 with 24 selected operations; 272,116 feature comparisons remain. No whole-model speedup or new angular typed selector is claimed. The ignored learned diagnostic was explicitly executed and failed its behavioral acceptance; its negative does not invalidate the separately verified execution-preservation change.

The owner authorized necessary allowance extensions. Recorded +60 and +30 seconds cover 74.262 seconds of actual model work; cumulative use is 1871.006/1890 seconds. The unoptimized diagnostic exceeded its initial time projection and was not repeated. Build work is 382.586 seconds, storage fits, paid external compute is zero and all material is preserved.

See docs/native_geometric_typed_admission_1139.md and its compact evidence. Next: adapt the existing signed-H4 source learner to joint typed operator/operand selection on causally generated intermediate states, with complete output and matched controls. That learning change is not implemented here. Refs #1139, #1140; neither full handoff is closed.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

