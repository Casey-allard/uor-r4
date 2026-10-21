# ORDER: infra-loop-harness, claude-3 (re-run P-1 and P-2 for Claude Code)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `222591cb5`.

## Goal

Re-run probes P-1 and P-2 for Claude Code now that the owner has signed in the `claude` CLI, and record the results.

## Premises and cancel lines

These are unchanged from [claude-2's ORDER](../claude-2/ORDER.md), which committed them before the first run.
- **P-1:** if Claude Code's hooks do not fire inside its subagents, its subagents get read-only tools.
- **P-2:** if any part fails, Claude Code stays interactive-only.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/claude-3/**`
- `docs/loop/README.md`: only the Claude Code parts of the P-1 and P-2 results cells.

The scratch runs stay in `local/claude/loop-probes/claude-p1/` and are not committed.

## Done when

Both cells state the Claude Code result with evidence anyone can check, and the REPORT lists what did not work first.

## Stop and ask if

- a probe needs a change to any client's global settings;
- a probe would read or print a secret;
- any file outside scope needs to change.
