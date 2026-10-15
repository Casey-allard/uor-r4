# #667 — model-source: GPT-2 #603 trace-capture + batched executor (item 2 of #657)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T17:48:36Z
- **Updated:** 2026-08-13T21:28:57Z
- **Closed:** 2026-08-13T21:28:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/667
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #657 (item 1, the Teacher dispatch enabler, landed). GPT-2's HuggingFaceGpt2Oracle returns the safe trait defaults for trace_capture_geometry (None) and step_with_trace_capture (false) — richer #603 trace profiles are refused, never zero-filled. GPT-2 needs a forward_capturing_trace equivalent (post-layer residual, q/k/v at pos, per-head attention weights). It also needs a batched executor: #657 item 1 kept observe_text_batched_command Llama-only because BatchedTeacher's per-sequence State is the Llama batched state and GPT-2 has none yet. Implement both WITH the compile pipeline so the taps are exercised end-to-end. See #657 for full scope. Filed unassigned per board rules.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

