# Client adapter contract

All clients read the same GitHub-backed [protocol](protocol.md), decisions,
claims, issues and artifact manifests. A client-specific prompt translates the
workflow into available tools; it does not create another project policy.

At join, declare whether the client can: read/write the repository; use
authenticated `gh`; make a full worktree; launch subagents; submit and inspect
runner jobs; update a heartbeat without interrupting a tool call; push a draft
checkpoint; resume after quota/compaction; and receive cross-lab messages.
Report each as `AVAILABLE`, `MANUAL`, `UNAVAILABLE` or `UNVERIFIED`, with a small
observed smoke where appropriate. Do not claim a background scheduler merely
because a prompt says “continue”. Keep provider credentials, session files and
personal memory out of Git.

| Client | Initial packet | Supported fallback until its smoke is recorded |
|---|---|---|
| Codex | [Codex goal](prompts/codex.md) | Use local shell/gh and native subagents; a fresh pasted packet resumes the lab |
| Claude | [Claude goal](prompts/claude.md) | Use its existing tool/subagent route; checkpoints and runner survive quota loss |
| Anti-Gravity | [Anti-Gravity goal](prompts/antigravity.md) | Use the configured terminal/tools; record UI-only or manual reattachment steps |
| OpenCode/DeepSeek | [DeepSeek goal](prompts/opencode-deepseek.md) | Fast submit/status calls avoid the known long-command harness boundary |
| Any additional client | [Joining goal](prompts/join.md) | Declare capabilities, create/reuse a lab board and claim ready work |

No adapter has been certified by this document. The execution receipt names
what actually ran. The host runner can outlive the submitting client; the
client's reasoning cannot be assumed to continue after its tokens expire.

Subagent packets must carry the current goal and source SHA; question and
expected decision; exact owned files/read-only scope; input artifacts and
history; allowed resources/tools; success/failure/uncertainty criteria; and a
required concise source-linked return. Use cheap capable agents for mechanical
inventory, formatting and routine code checks; reserve deeper research/review
for unresolved mathematical, learning or integration questions. Independently
verify consequential outputs. A subagent's final message is a report, not a
measurement or an approved merge.
