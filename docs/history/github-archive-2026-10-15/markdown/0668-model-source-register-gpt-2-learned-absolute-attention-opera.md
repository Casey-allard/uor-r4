# #668 — model-source: register GPT-2 learned-absolute attention operator (#602, item 3 of #657)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T17:48:38Z
- **Updated:** 2026-08-13T19:19:47Z
- **Closed:** 2026-08-13T19:19:47Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/668
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #657. HuggingFaceGpt2Oracle::attention_operator_spec is deliberately None: the only registered operator standard-source-attention/1 declares positional_action=rope-rotation-of-q-and-k-before-scoring, which GPT-2 does NOT do (learned absolute positions). Register a new versioned operator (learned-absolute positional action) in attention.rs (registry still refusing unknown (id,version)) and wire attention_operator_spec to it; feeds the #606 certificate operator row. See #657. Filed unassigned per board rules.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

